// SPDX-License-Identifier: AGPL-3.0-only
//! Pure layout solver for MuConsole.
//!
//! `solve(viewport, safe_area, area, font) -> Frames` places every top-level
//! region of the console so that **nothing overflows and the page never
//! scrolls**: the frames exactly tile the usable canvas, and each CRT gets a
//! metric-aware column grid with at least one row and one column. Overflow
//! inside a panel is handled by panel-local paging, never by scrollbars.
//!
//! The crate has no browser dependencies; it is unit tested natively for
//! every target viewport and both CRT fonts.
#![forbid(unsafe_code)]

pub mod grid;
pub mod trainway;

pub use grid::{solve_grid, truncate, CrtGrid, GridSpec};
pub use muconsole_protocol::Area;
pub use muconsole_theme::instruments::Rect;
pub use trainway::{allocate_lanes, AllocatedLane, TrainwayLayout};

use muconsole_theme::metrics::{CrtFontMetrics, CRT_FLOOR_DESKTOP_PX, CRT_FLOOR_PHONE_PX};

/// CSS-px viewport (from `visualViewport`, never hard-coded).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Viewport {
    pub width: f64,
    pub height: f64,
}

impl Viewport {
    #[must_use]
    pub const fn new(width: f64, height: f64) -> Self {
        Self { width, height }
    }

    /// Map `window.visualViewport` (`width`, `height`, `scale`) to the
    /// layout canvas. The visual viewport shrinks while pinch-zoomed, so the
    /// size is multiplied back by `scale`; this also tracks the iOS Safari
    /// toolbar collapsing without hard-coded heights. Non-finite or
    /// non-positive scales count as 1.
    #[must_use]
    pub fn from_visual_viewport(width: f64, height: f64, scale: f64) -> Self {
        let s = if scale.is_finite() && scale > 0.0 {
            scale
        } else {
            1.0
        };
        Self::new((width * s).max(0.0), (height * s).max(0.0))
    }
}

/// `env(safe-area-inset-*)` values in CSS px.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct SafeArea {
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
    pub left: f64,
}

impl SafeArea {
    pub const ZERO: Self = Self {
        top: 0.0,
        right: 0.0,
        bottom: 0.0,
        left: 0.0,
    };
}

/// Breakpoint class (decided here, not in CSS).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Breakpoint {
    /// w ≥ 1280 and h ≥ 760: 80 px switch rail, 3 cards + carousel switch +
    /// rotary, Trainway and Delivery side by side.
    Wide,
    /// Everything between: 80 px rail, up to 2 cards + rotary, Delivery
    /// collapsed to the gate summary.
    Medium,
    /// h < 480: 72 px rail, no carousel, Trainway / Delivery as pages.
    CompactLandscape,
    /// w < 520: bottom selector strip, 1 card, swipe pages.
    Phone,
}

/// Classify a usable canvas size.
#[must_use]
pub fn classify(width: f64, height: f64) -> Breakpoint {
    if width < 520.0 {
        Breakpoint::Phone
    } else if height < 480.0 {
        Breakpoint::CompactLandscape
    } else if width >= 1280.0 && height >= 760.0 {
        Breakpoint::Wide
    } else {
        Breakpoint::Medium
    }
}

/// Dashboard panel kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PanelKind {
    Trainway,
    Delivery,
    Fleet,
}

/// One dashboard panel: tungsten plate, CRT glass and its text grid.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Panel {
    pub kind: PanelKind,
    /// Panels with the same page are visible together.
    pub page: u32,
    pub plate: Rect,
    pub crt: Rect,
    pub grid: CrtGrid,
}

/// One twin-CRT scorecard module.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Card {
    pub plate: Rect,
    pub ci_crt: Rect,
    pub cd_crt: Rect,
    pub ci_grid: CrtGrid,
    pub cd_grid: CrtGrid,
}

/// Carousel row.
#[derive(Debug, Clone, PartialEq)]
pub struct Carousel {
    pub cards: Vec<Card>,
    /// 3-position spring-return CAROUSEL switch plate (Wide only).
    pub switch: Option<Rect>,
    /// Lamp page dots.
    pub dots: Rect,
}

impl Carousel {
    /// Bounding box of the cards, switch and dots.
    #[must_use]
    pub fn bounds(&self) -> Rect {
        let mut rects: Vec<Rect> = self.cards.iter().map(|c| c.plate).collect();
        rects.extend(self.switch);
        rects.push(self.dots);
        union(&rects)
    }
}

/// Solved frames for one viewport.
#[derive(Debug, Clone, PartialEq)]
pub struct Frames {
    pub breakpoint: Breakpoint,
    /// Usable canvas (viewport minus safe areas).
    pub canvas: Rect,
    /// Area navigation: switch rail (desktop) or bottom selector strip.
    pub nav: Rect,
    /// Breadcrumb bar.
    pub top_bar: Rect,
    pub carousel: Option<Carousel>,
    /// Repository rotary module (reference 302×192; scale by `h / 192`).
    /// `None` on the phone, where it opens as a bottom sheet.
    pub rotary: Option<Rect>,
    pub panels: Vec<Panel>,
    /// Number of dashboard pages.
    pub pages: u32,
}

impl Frames {
    /// Top-level regions that must tile the canvas without overlapping
    /// (panels of page 0 only; other pages occupy the same rect).
    #[must_use]
    pub fn regions(&self) -> Vec<Rect> {
        let mut out = vec![self.nav, self.top_bar];
        if let Some(c) = &self.carousel {
            out.push(c.bounds());
        }
        out.extend(self.rotary);
        out.extend(self.panels.iter().filter(|p| p.page == 0).map(|p| p.plate));
        out
    }

    /// Panels on `page`.
    pub fn page(&self, page: u32) -> impl Iterator<Item = &Panel> {
        self.panels.iter().filter(move |p| p.page == page)
    }
}

/// Outer margin around the desk.
pub const MARGIN: f64 = 6.0;
/// Gap between the rail and the content column.
pub const RAIL_GAP: f64 = 8.0;
/// Desktop rail plate width.
pub const RAIL_W: f64 = 80.0;
/// Compact-landscape rail plate width.
pub const RAIL_W_COMPACT: f64 = 72.0;
/// Breadcrumb bar height (desktop / compact).
pub const TOP_BAR_H: f64 = 48.0;
pub const TOP_BAR_H_COMPACT: f64 = 40.0;
pub const TOP_BAR_H_PHONE: f64 = 44.0;
/// Gap below the top bar.
pub const TOP_GAP: f64 = 10.0;
/// Carousel card height: ideal and minimum.
pub const CARD_H: f64 = 192.0;
pub const CARD_H_MIN: f64 = 132.0;
/// Page-dot band below the cards.
pub const DOTS_H: f64 = 26.0;
pub const CARD_GAP: f64 = 12.0;
/// Rotary reference size.
pub const ROTARY_W: f64 = 302.0;
pub const ROTARY_H: f64 = 192.0;
/// CAROUSEL switch plate width (Wide).
pub const CAROUSEL_SWITCH_W: f64 = 92.0;
/// Narrowest useful twin-CRT card.
pub const CARD_MIN_W: f64 = 220.0;
/// Gap between Trainway and Delivery.
pub const PANEL_GAP: f64 = 8.0;
/// Delivery share of the dashboard width (506 of 1404 px at 1512 wide).
pub const DELIVERY_RATIO: f64 = 506.0 / 1404.0;
/// Medium: Delivery collapses to the gate summary.
pub const DELIVERY_RATIO_MEDIUM: f64 = 0.30;
pub const DELIVERY_MIN_W: f64 = 280.0;
/// Minimum dashboard height before cards shrink.
pub const DASH_MIN_WIDE: f64 = 440.0;
pub const DASH_MIN_MEDIUM: f64 = 300.0;
/// Phone: card height, bottom selector strip, gaps.
pub const PHONE_CARD_H: f64 = 168.0;
pub const PHONE_NAV_H: f64 = 60.0;
pub const PHONE_GAP: f64 = 8.0;
/// Phone: hide the card when the dashboard would be shorter than this.
pub const PHONE_PANEL_MIN: f64 = 200.0;

const PANEL_GRID: GridSpec = GridSpec {
    design_px: 16.0,
    floor_px: CRT_FLOOR_DESKTOP_PX,
    pad_x: 10.0,
    pad_y: 8.0,
};
const CARD_GRID: GridSpec = GridSpec {
    design_px: 14.0,
    floor_px: CRT_FLOOR_DESKTOP_PX,
    pad_x: 8.0,
    pad_y: 6.0,
};

fn phone_grid(spec: GridSpec, design_px: f64) -> GridSpec {
    GridSpec {
        design_px,
        floor_px: CRT_FLOOR_PHONE_PX,
        ..spec
    }
}

/// Bounding box of `rects` (empty input gives a zero rect).
#[must_use]
pub fn union(rects: &[Rect]) -> Rect {
    let Some(first) = rects.first() else {
        return Rect::default();
    };
    let (mut x0, mut y0, mut x1, mut y1) = (first.x, first.y, first.right(), first.bottom());
    for r in &rects[1..] {
        x0 = x0.min(r.x);
        y0 = y0.min(r.y);
        x1 = x1.max(r.right());
        y1 = y1.max(r.bottom());
    }
    Rect::new(x0, y0, x1 - x0, y1 - y0)
}

fn card(
    plate: Rect,
    top: f64,
    bottom: f64,
    side: f64,
    gap: f64,
    spec: GridSpec,
    font: &CrtFontMetrics,
) -> Card {
    let crt_w = ((plate.w - 2.0 * side - gap) / 2.0).max(0.0);
    let crt_h = (plate.h - top - bottom).max(0.0);
    let ci_crt = Rect::new(plate.x + side, plate.y + top, crt_w, crt_h);
    let cd_crt = Rect::new(plate.right() - side - crt_w, plate.y + top, crt_w, crt_h);
    Card {
        plate,
        ci_crt,
        cd_crt,
        ci_grid: solve_grid(ci_crt, spec, font),
        cd_grid: solve_grid(cd_crt, spec, font),
    }
}

/// Desktop card: LCD name strip on top, twin CRTs below (reference 324×192
/// gives two 140×132 screens).
fn desktop_card(plate: Rect, font: &CrtFontMetrics) -> Card {
    card(plate, 42.0, 18.0, 16.0, 12.0, CARD_GRID, font)
}

fn panel(
    kind: PanelKind,
    page: u32,
    plate: Rect,
    insets: (f64, f64, f64, f64),
    spec: GridSpec,
    font: &CrtFontMetrics,
) -> Panel {
    let crt = plate.inset(insets.0, insets.1, insets.2, insets.3);
    Panel {
        kind,
        page,
        plate,
        crt,
        grid: solve_grid(crt, spec, font),
    }
}

/// Plate → CRT insets: header strip with title, LCDs and switches.
const TRAINWAY_INSETS: (f64, f64, f64, f64) = (22.0, 64.0, 22.0, 24.0);
const DELIVERY_INSETS: (f64, f64, f64, f64) = (22.0, 74.0, 22.0, 24.0);
const COMPACT_INSETS: (f64, f64, f64, f64) = (14.0, 44.0, 14.0, 14.0);
const PHONE_INSETS: (f64, f64, f64, f64) = (12.0, 40.0, 12.0, 12.0);

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]
fn cards_fitting(width: f64, max: usize) -> usize {
    let n = ((width + CARD_GAP) / (CARD_MIN_W + CARD_GAP)).floor();
    if n.is_finite() && n >= 1.0 {
        (n as usize).min(max)
    } else {
        0
    }
}

#[allow(clippy::cast_precision_loss)]
fn row_of_cards(x: f64, y: f64, width: f64, h: f64, n: usize, font: &CrtFontMetrics) -> Vec<Card> {
    if n == 0 {
        return Vec::new();
    }
    let w = ((width - CARD_GAP * (n - 1) as f64) / n as f64).floor();
    (0..n)
        .map(|i| desktop_card(Rect::new(x + i as f64 * (w + CARD_GAP), y, w, h), font))
        .collect()
}

/// Wide and Medium share the vertical stack; they differ in the carousel
/// row and the dashboard split.
fn solve_desktop(c: Rect, bp: Breakpoint, area: Area, font: &CrtFontMetrics) -> Frames {
    let wide = bp == Breakpoint::Wide;
    let nav = Rect::new(c.x + MARGIN, c.y + MARGIN, RAIL_W, c.h - 2.0 * MARGIN);
    let x0 = nav.right() + RAIL_GAP;
    let x1 = c.right() - MARGIN;
    let cw = x1 - x0;
    let top_bar = Rect::new(x0, c.y + MARGIN, cw, TOP_BAR_H);
    let fixed = MARGIN + TOP_BAR_H + TOP_GAP + DOTS_H + MARGIN;
    let dash_min = if wide { DASH_MIN_WIDE } else { DASH_MIN_MEDIUM };
    let card_h = (c.h - fixed - dash_min).clamp(CARD_H_MIN, CARD_H);
    let row_y = top_bar.bottom() + TOP_GAP;

    // Carousel row, right to left: rotary, (switch), cards.
    let rotary_w = ROTARY_W * card_h / ROTARY_H;
    let mut right = x1;
    let mut rotary = Some(Rect::new(right - rotary_w, row_y, rotary_w, card_h));
    right -= rotary_w + 10.0;
    let mut switch = None;
    if wide {
        switch = Some(Rect::new(
            right - CAROUSEL_SWITCH_W,
            row_y,
            CAROUSEL_SWITCH_W,
            card_h,
        ));
        right -= CAROUSEL_SWITCH_W + CARD_GAP;
    }
    let max_cards = if wide { 3 } else { 2 };
    let mut n = cards_fitting(right - x0, max_cards);
    if n == 0 {
        // Too narrow for a card next to the rotary: the rotary moves to a
        // sheet opened from the top bar.
        rotary = None;
        right = x1;
        n = cards_fitting(right - x0, max_cards).max(1);
    }
    let cards_w = right - x0;
    let cards = row_of_cards(x0, row_y, cards_w, card_h, n, font);
    let dots = Rect::new(x0, row_y + card_h, cards_w, DOTS_H);
    let carousel = Carousel {
        cards,
        switch,
        dots,
    };

    let dash_y = dots.bottom();
    let dash_h = c.bottom() - MARGIN - dash_y;
    let panels = if area == Area::Fleet {
        vec![panel(
            PanelKind::Fleet,
            0,
            Rect::new(x0, dash_y, cw, dash_h),
            TRAINWAY_INSETS,
            PANEL_GRID,
            font,
        )]
    } else {
        let ratio = if wide {
            DELIVERY_RATIO
        } else {
            DELIVERY_RATIO_MEDIUM
        };
        let delivery_w = ((cw - PANEL_GAP) * ratio)
            .round()
            .max(DELIVERY_MIN_W)
            .min((cw - PANEL_GAP) / 2.0);
        let trainway_w = cw - PANEL_GAP - delivery_w;
        vec![
            panel(
                PanelKind::Trainway,
                0,
                Rect::new(x0, dash_y, trainway_w, dash_h),
                TRAINWAY_INSETS,
                PANEL_GRID,
                font,
            ),
            panel(
                PanelKind::Delivery,
                0,
                Rect::new(x0 + trainway_w + PANEL_GAP, dash_y, delivery_w, dash_h),
                DELIVERY_INSETS,
                PANEL_GRID,
                font,
            ),
        ]
    };
    Frames {
        breakpoint: bp,
        canvas: c,
        nav,
        top_bar,
        carousel: Some(carousel),
        rotary,
        panels,
        pages: 1,
    }
}

fn paged_panels(
    plate: Rect,
    area: Area,
    insets: (f64, f64, f64, f64),
    spec: GridSpec,
    font: &CrtFontMetrics,
) -> (Vec<Panel>, u32) {
    if area == Area::Fleet {
        (
            vec![panel(PanelKind::Fleet, 0, plate, insets, spec, font)],
            1,
        )
    } else {
        (
            vec![
                panel(PanelKind::Trainway, 0, plate, insets, spec, font),
                panel(PanelKind::Delivery, 1, plate, insets, spec, font),
            ],
            2,
        )
    }
}

fn solve_compact(c: Rect, area: Area, font: &CrtFontMetrics) -> Frames {
    let nav = Rect::new(
        c.x + MARGIN,
        c.y + MARGIN,
        RAIL_W_COMPACT,
        c.h - 2.0 * MARGIN,
    );
    let x0 = nav.right() + RAIL_GAP;
    let cw = c.right() - MARGIN - x0;
    let top_bar = Rect::new(x0, c.y + MARGIN, cw, TOP_BAR_H_COMPACT);
    let y = top_bar.bottom() + PHONE_GAP;
    let plate = Rect::new(x0, y, cw, c.bottom() - MARGIN - y);
    let (panels, pages) = paged_panels(
        plate,
        area,
        COMPACT_INSETS,
        phone_grid(PANEL_GRID, 15.0),
        font,
    );
    Frames {
        breakpoint: Breakpoint::CompactLandscape,
        canvas: c,
        nav,
        top_bar,
        carousel: None,
        rotary: None,
        panels,
        pages,
    }
}

fn solve_phone(c: Rect, area: Area, font: &CrtFontMetrics) -> Frames {
    let x0 = c.x + MARGIN;
    let cw = c.w - 2.0 * MARGIN;
    let top_bar = Rect::new(x0, c.y + MARGIN, cw, TOP_BAR_H_PHONE);
    let nav = Rect::new(x0, c.bottom() - MARGIN - PHONE_NAV_H, cw, PHONE_NAV_H);
    let mut y = top_bar.bottom() + PHONE_GAP;
    let with_card = nav.y - PHONE_GAP - (y + PHONE_CARD_H + PHONE_GAP) >= PHONE_PANEL_MIN;
    let carousel = with_card.then(|| {
        let plate = Rect::new(x0, y, cw, PHONE_CARD_H);
        y = plate.bottom() + PHONE_GAP;
        let card = card(
            plate,
            34.0,
            22.0,
            12.0,
            10.0,
            phone_grid(CARD_GRID, 15.0),
            font,
        );
        Carousel {
            cards: vec![card],
            switch: None,
            dots: Rect::new(plate.x, plate.bottom() - 20.0, plate.w, 20.0),
        }
    });
    let plate = Rect::new(x0, y, cw, nav.y - PHONE_GAP - y);
    let (panels, pages) = paged_panels(
        plate,
        area,
        PHONE_INSETS,
        phone_grid(PANEL_GRID, 16.0),
        font,
    );
    Frames {
        breakpoint: Breakpoint::Phone,
        canvas: c,
        nav,
        top_bar,
        carousel,
        rotary: None,
        panels,
        pages,
    }
}

/// Solve the frames for a viewport, safe-area insets, area and CRT font.
#[must_use]
pub fn solve(viewport: Viewport, safe: SafeArea, area: Area, font: &CrtFontMetrics) -> Frames {
    let canvas = Rect::new(
        safe.left,
        safe.top,
        (viewport.width - safe.left - safe.right).max(0.0),
        (viewport.height - safe.top - safe.bottom).max(0.0),
    );
    match classify(canvas.w, canvas.h) {
        bp @ (Breakpoint::Wide | Breakpoint::Medium) => solve_desktop(canvas, bp, area, font),
        Breakpoint::CompactLandscape => solve_compact(canvas, area, font),
        Breakpoint::Phone => solve_phone(canvas, area, font),
    }
}

/// Solve for a full route (only the area affects the frames).
#[must_use]
pub fn solve_route(
    viewport: Viewport,
    safe: SafeArea,
    route: &muconsole_protocol::Route,
    font: &CrtFontMetrics,
) -> Frames {
    solve(viewport, safe, route.area, font)
}

#[cfg(test)]
mod tests;
