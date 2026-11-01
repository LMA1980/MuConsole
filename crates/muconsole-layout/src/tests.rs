// SPDX-License-Identifier: AGPL-3.0-only
//! Zero-overflow / no-scroll tests at every target viewport, for both fonts.

use super::*;
use muconsole_theme::metrics::{CRT_FONTS, HURMIT, OPEN_DYSLEXIC};

const EPS: f64 = 0.01;
const AREAS: [Area; 4] = [Area::Overview, Area::Ci, Area::Cd, Area::Fleet];

/// Design targets: (width, height, expected breakpoint).
const TARGETS: [(f64, f64, Breakpoint); 6] = [
    (1512.0, 860.0, Breakpoint::Wide),
    (1512.0, 982.0, Breakpoint::Wide),
    (1800.0, 1040.0, Breakpoint::Wide),
    (402.0, 720.0, Breakpoint::Phone),
    (402.0, 778.0, Breakpoint::Phone),
    (874.0, 340.0, Breakpoint::CompactLandscape),
];

fn inside(inner: &Rect, outer: &Rect) -> bool {
    inner.x >= outer.x - EPS
        && inner.y >= outer.y - EPS
        && inner.right() <= outer.right() + EPS
        && inner.bottom() <= outer.bottom() + EPS
}

fn overlap(a: &Rect, b: &Rect) -> bool {
    a.x < b.right() - EPS
        && b.x < a.right() - EPS
        && a.y < b.bottom() - EPS
        && b.y < a.bottom() - EPS
}

fn assert_grid(g: &CrtGrid, screen: &Rect, floor: f64, what: &str) {
    assert!(g.cols >= 1, "{what}: no columns");
    assert!(g.rows >= 1, "{what}: no rows");
    assert!(
        g.font_px >= floor - EPS,
        "{what}: {} px below floor {floor}",
        g.font_px
    );
    assert!(
        g.text_width() <= g.text_area.w + EPS,
        "{what}: text overflows width"
    );
    assert!(
        g.text_height() <= g.text_area.h + EPS,
        "{what}: text overflows height"
    );
    assert!(
        inside(&g.text_area, screen),
        "{what}: text area outside screen"
    );
}

/// The invariants behind "nothing overflows and nothing scrolls".
fn assert_no_overflow(f: &Frames, label: &str) {
    let c = f.canvas;
    let regions = f.regions();
    for r in &regions {
        assert!(r.w > 0.0 && r.h > 0.0, "{label}: empty region {r:?}");
        assert!(inside(r, &c), "{label}: {r:?} outside canvas {c:?}");
    }
    for (i, a) in regions.iter().enumerate() {
        for b in &regions[i + 1..] {
            assert!(!overlap(a, b), "{label}: {a:?} overlaps {b:?}");
        }
    }
    // Frames span the full canvas height (minus margins): nothing to scroll,
    // nothing left empty.
    let top = regions.iter().map(|r| r.y).fold(f64::INFINITY, f64::min);
    let bottom = regions.iter().map(Rect::bottom).fold(0.0, f64::max);
    assert!((top - (c.y + MARGIN)).abs() < 0.5, "{label}: top {top}");
    assert!(
        (bottom - (c.bottom() - MARGIN)).abs() < 0.5,
        "{label}: bottom {bottom}"
    );
    let right = regions.iter().map(Rect::right).fold(0.0, f64::max);
    assert!(
        (right - (c.right() - MARGIN)).abs() < 0.5,
        "{label}: right {right}"
    );

    let floor = match f.breakpoint {
        Breakpoint::Wide | Breakpoint::Medium => CRT_FLOOR_DESKTOP_PX,
        Breakpoint::CompactLandscape | Breakpoint::Phone => CRT_FLOOR_PHONE_PX,
    };
    for p in &f.panels {
        assert!(p.page < f.pages, "{label}: page out of range");
        assert!(
            inside(&p.crt, &p.plate),
            "{label}: {:?} CRT outside plate",
            p.kind
        );
        assert_grid(&p.grid, &p.crt, floor, &format!("{label} {:?}", p.kind));
    }
    for page in 0..f.pages {
        assert!(f.page(page).count() >= 1, "{label}: empty page {page}");
    }
    if let Some(car) = &f.carousel {
        assert!(!car.cards.is_empty(), "{label}: carousel without cards");
        for (i, card) in car.cards.iter().enumerate() {
            assert!(
                inside(&card.ci_crt, &card.plate),
                "{label}: card {i} CI CRT"
            );
            assert!(
                inside(&card.cd_crt, &card.plate),
                "{label}: card {i} CD CRT"
            );
            assert!(
                !overlap(&card.ci_crt, &card.cd_crt),
                "{label}: card {i} CRTs overlap"
            );
            assert_grid(
                &card.ci_grid,
                &card.ci_crt,
                floor,
                &format!("{label} card {i} CI"),
            );
            assert_grid(
                &card.cd_grid,
                &card.cd_crt,
                floor,
                &format!("{label} card {i} CD"),
            );
            for other in &car.cards[i + 1..] {
                assert!(
                    !overlap(&card.plate, &other.plate),
                    "{label}: cards overlap"
                );
            }
            if let Some(sw) = car.switch {
                assert!(!overlap(&card.plate, &sw), "{label}: card overlaps switch");
            }
        }
    }
}

#[test]
fn target_viewports_never_overflow() {
    for (w, h, bp) in TARGETS {
        for font in &CRT_FONTS {
            for area in AREAS {
                let f = solve(Viewport::new(w, h), SafeArea::ZERO, area, font);
                let label = format!("{w}x{h} {} {area:?}", font.family);
                assert_eq!(f.breakpoint, bp, "{label}");
                assert_no_overflow(&f, &label);
            }
        }
    }
}

#[test]
fn wide_matches_concept_geometry() {
    let f = solve(
        Viewport::new(1512.0, 860.0),
        SafeArea::ZERO,
        Area::Ci,
        &HURMIT,
    );
    assert_eq!(f.nav, Rect::new(6.0, 6.0, 80.0, 848.0));
    assert_eq!(f.top_bar, Rect::new(94.0, 6.0, 1412.0, 48.0));
    let car = f.carousel.as_ref().expect("carousel");
    assert_eq!(car.cards.len(), 3);
    assert_eq!(car.cards[0].plate, Rect::new(94.0, 64.0, 324.0, 192.0));
    assert_eq!(car.cards[2].plate.x, 766.0);
    assert_eq!(car.cards[0].ci_crt, Rect::new(110.0, 106.0, 140.0, 132.0));
    assert_eq!(car.switch, Some(Rect::new(1102.0, 64.0, 92.0, 192.0)));
    assert_eq!(f.rotary, Some(Rect::new(1204.0, 64.0, 302.0, 192.0)));
    assert_eq!(f.panels[0].kind, PanelKind::Trainway);
    assert_eq!(f.panels[0].plate, Rect::new(94.0, 282.0, 898.0, 572.0));
    assert_eq!(f.panels[1].plate, Rect::new(1000.0, 282.0, 506.0, 572.0));
    assert_eq!(f.panels[0].crt, Rect::new(116.0, 346.0, 854.0, 484.0));
}

#[test]
fn font_switch_keeps_the_column_grid() {
    for (w, h, _) in TARGETS {
        let a = solve(Viewport::new(w, h), SafeArea::ZERO, Area::Ci, &HURMIT);
        let b = solve(
            Viewport::new(w, h),
            SafeArea::ZERO,
            Area::Ci,
            &OPEN_DYSLEXIC,
        );
        assert_eq!(
            a.regions(),
            b.regions(),
            "{w}x{h}: frames move with the font"
        );
        for (pa, pb) in a.panels.iter().zip(&b.panels) {
            assert_eq!(pa.grid.rows, pb.grid.rows, "{w}x{h} {:?} rows", pa.kind);
            if !pb.grid.clamped {
                assert_eq!(pa.grid.cols, pb.grid.cols, "{w}x{h} {:?} cols", pa.kind);
                let shrink = pb.grid.font_px / pa.grid.font_px;
                assert!((shrink - 0.618 / 0.732).abs() < 1e-9);
            }
        }
    }
}

#[test]
fn desktop_panels_keep_design_size() {
    let f = solve(
        Viewport::new(1512.0, 860.0),
        SafeArea::ZERO,
        Area::Ci,
        &OPEN_DYSLEXIC,
    );
    for p in &f.panels {
        assert!(!p.grid.clamped, "{:?} clamped at desktop size", p.kind);
        assert!(p.grid.cols >= 40, "{:?} only {} cols", p.kind, p.grid.cols);
    }
    let trainway = &f.panels[0].grid;
    // 8 stations at ≥ 7 columns each fit on one row.
    assert!(trainway.cols >= 8 * 7);
    assert!(trainway.rows >= 18);
}

#[test]
fn shorter_canvas_shrinks_cards_first() {
    let tall = solve(
        Viewport::new(1512.0, 982.0),
        SafeArea::ZERO,
        Area::Ci,
        &HURMIT,
    );
    let car = tall.carousel.as_ref().expect("carousel");
    assert!(
        (car.cards[0].plate.h - CARD_H).abs() < EPS,
        "cards stay at ideal height"
    );
    assert!(tall.panels[0].plate.h > 572.0, "dashboard grows");

    let medium = solve(
        Viewport::new(1280.0, 560.0),
        SafeArea::ZERO,
        Area::Ci,
        &HURMIT,
    );
    assert_eq!(medium.breakpoint, Breakpoint::Medium);
    let card_h = medium.carousel.as_ref().expect("carousel").cards[0].plate.h;
    assert!(
        (CARD_H_MIN - EPS..CARD_H).contains(&card_h),
        "card {card_h}"
    );
    assert_no_overflow(&medium, "1280x560");
}

#[test]
fn phone_layout_and_safe_areas() {
    let f = solve(
        Viewport::new(402.0, 720.0),
        SafeArea::ZERO,
        Area::Ci,
        &HURMIT,
    );
    assert_eq!(f.pages, 2);
    assert!(f.rotary.is_none());
    assert_eq!(f.carousel.as_ref().map(|c| c.cards.len()), Some(1));
    assert!((f.nav.h - PHONE_NAV_H).abs() < EPS);
    assert!(f.page(0).all(|p| p.kind == PanelKind::Trainway));
    assert!(f.page(1).all(|p| p.kind == PanelKind::Delivery));

    // Full 402×874 with Dynamic Island / home indicator insets ≈ the PWA canvas.
    let safe = SafeArea {
        top: 62.0,
        bottom: 34.0,
        ..SafeArea::ZERO
    };
    let pwa = solve(Viewport::new(402.0, 874.0), safe, Area::Ci, &OPEN_DYSLEXIC);
    assert_eq!(pwa.canvas, Rect::new(0.0, 62.0, 402.0, 778.0));
    assert_no_overflow(&pwa, "402x874 safe");

    // Landscape with side insets.
    let side = SafeArea {
        left: 62.0,
        right: 62.0,
        bottom: 21.0,
        ..SafeArea::ZERO
    };
    let land = solve(Viewport::new(874.0, 402.0), side, Area::Cd, &OPEN_DYSLEXIC);
    assert_eq!(land.breakpoint, Breakpoint::CompactLandscape);
    assert_no_overflow(&land, "874x402 safe");
}

#[test]
fn fleet_uses_a_single_panel() {
    for (w, h, _) in TARGETS {
        let f = solve(Viewport::new(w, h), SafeArea::ZERO, Area::Fleet, &HURMIT);
        assert_eq!(f.pages, 1);
        assert_eq!(f.panels.len(), 1);
        assert_eq!(f.panels[0].kind, PanelKind::Fleet);
    }
}

#[test]
fn classification_boundaries() {
    assert_eq!(classify(1280.0, 760.0), Breakpoint::Wide);
    assert_eq!(classify(1279.0, 760.0), Breakpoint::Medium);
    assert_eq!(classify(1280.0, 759.0), Breakpoint::Medium);
    assert_eq!(classify(900.0, 700.0), Breakpoint::Medium);
    assert_eq!(classify(874.0, 479.0), Breakpoint::CompactLandscape);
    assert_eq!(classify(519.0, 2000.0), Breakpoint::Phone);
    assert_eq!(classify(520.0, 480.0), Breakpoint::Medium);
}

#[test]
fn sweep_never_overflows() {
    // Every size between a small phone and a large desktop, both fonts.
    let mut w = 320.0;
    while w <= 2560.0 {
        let mut h = 300.0;
        while h <= 1600.0 {
            for font in &CRT_FONTS {
                let f = solve(Viewport::new(w, h), SafeArea::ZERO, Area::Ci, font);
                assert_no_overflow(&f, &format!("{w}x{h} {}", font.family));
            }
            h += 23.0;
        }
        w += 37.0;
    }
}

#[test]
fn solver_is_deterministic() {
    for (w, h, _) in TARGETS {
        let a = solve(Viewport::new(w, h), SafeArea::ZERO, Area::Overview, &HURMIT);
        let b = solve(Viewport::new(w, h), SafeArea::ZERO, Area::Overview, &HURMIT);
        assert_eq!(a, b);
    }
}

#[test]
fn union_of_rects() {
    assert_eq!(union(&[]), Rect::default());
    let u = union(&[
        Rect::new(0.0, 0.0, 10.0, 10.0),
        Rect::new(5.0, -5.0, 10.0, 10.0),
    ]);
    assert_eq!(u, Rect::new(0.0, -5.0, 15.0, 15.0));
}

#[test]
fn visual_viewport_mapping_and_routes() {
    let v = Viewport::from_visual_viewport(201.0, 360.0, 2.0);
    assert_eq!(v, Viewport::new(402.0, 720.0));
    assert_eq!(
        Viewport::from_visual_viewport(402.0, 720.0, f64::NAN),
        Viewport::new(402.0, 720.0)
    );
    assert_eq!(
        Viewport::from_visual_viewport(402.0, 720.0, 0.0),
        Viewport::new(402.0, 720.0)
    );
    let route = muconsole_protocol::Route {
        area: Area::Fleet,
        repo: None,
        submodule: None,
        branch: None,
        job: None,
    };
    let f = solve_route(
        Viewport::new(1512.0, 860.0),
        SafeArea::ZERO,
        &route,
        &HURMIT,
    );
    assert_eq!(f.panels[0].kind, PanelKind::Fleet);
}
