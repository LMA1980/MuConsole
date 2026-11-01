// SPDX-License-Identifier: AGPL-3.0-only
//! Rugged concentric rotary: outer knob = repository, inner knob = submodule.

use super::lcd_window::{lcd, LcdSpec};
use super::surface::plate;
use super::switch::screw;
use super::text::{embossed, etched, TextStyle};
use super::{polar, Rect};
use crate::defs::{ids, url};
use crate::svg::SvgWriter;
use crate::tokens::{hardware, lamp};

/// Rotary state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RotarySpec<'a> {
    pub repos: &'a [&'a str],
    /// Selected repository index (wraps).
    pub selected: usize,
    pub submodules: &'a [&'a str],
    /// `None` = PARENT (no submodule selected).
    pub selected_sub: Option<usize>,
}

/// LCD drum width in character cells.
pub const DRUM_COLS: usize = 14;

/// Position-lamp angle (SVG degrees) of repository slot `k` of `n` on the
/// 270° arc starting at 135°.
#[must_use]
#[allow(clippy::cast_precision_loss)]
pub fn lamp_angle(k: usize, n: usize) -> f64 {
    let span = n.saturating_sub(1).max(1) as f64;
    135.0 + 270.0 * k as f64 / span
}

/// Inner-knob index angle (SVG degrees): PARENT points up, then one slot per
/// submodule clockwise.
#[must_use]
#[allow(clippy::cast_precision_loss)]
pub fn inner_angle(selected_sub: Option<usize>, n_subs: usize) -> f64 {
    let slot = selected_sub.map_or(0, |i| i + 1);
    -90.0 + 360.0 * slot as f64 / (n_subs + 1) as f64
}

/// The three drum lines: previous, `>`selected, next.
#[must_use]
pub fn drum_lines(repos: &[&str], selected: usize) -> [String; 3] {
    let n = repos.len();
    if n == 0 {
        return [String::new(), ">".to_owned(), String::new()];
    }
    let i = selected % n;
    let prev = (i + n - 1) % n;
    let next = (i + 1) % n;
    [
        format!(" {}", repos[prev]),
        format!(">{}", repos[i]),
        format!(" {}", repos[next]),
    ]
}

fn circle(w: &mut SvgWriter, cx: f64, cy: f64, r: f64) -> crate::svg::Tag<'_> {
    w.tag("circle").attr("cx", cx).attr("cy", cy).attr("r", r)
}

fn knob(w: &mut SvgWriter, cx: f64, cy: f64, spec: &RotarySpec<'_>) {
    // Parkerized flange with four screws.
    circle(w, cx + 2.0, cy + 3.0, 60.0)
        .attr("fill", "#000")
        .attr("opacity", 0.5)
        .end();
    circle(w, cx, cy, 60.0)
        .attr("fill", url(ids::PARKERIZED))
        .attr("stroke", "#55585e")
        .attr("stroke-width", 0.6)
        .end();
    for k in 0..4u32 {
        let (sx, sy) = polar(cx, cy, 54.0, 45.0 + 90.0 * f64::from(k));
        screw(w, sx, sy, 2.2, 15.0 * f64::from(k));
    }
    // Position lamp arc: one lamp per repository, lit = selected.
    let n = spec.repos.len();
    let sel = if n == 0 { 0 } else { spec.selected % n };
    for k in 0..n {
        let (lx, ly) = polar(cx, cy, 52.0, lamp_angle(k, n));
        let on = k == sel;
        circle(w, lx, ly, 2.4)
            .attr("fill", if on { lamp::OK } else { "#1b1d1f" })
            .attr("stroke", "#000")
            .attr("stroke-width", 0.5)
            .attr_opt("filter", on.then(|| url(ids::LAMP_GLOW)))
            .end();
    }
    // Outer knob: skirted, fluted rubber grip.
    circle(w, cx + 3.0, cy + 4.0, 46.0)
        .attr("fill", "#000")
        .attr("opacity", 0.55)
        .end();
    circle(w, cx, cy, 46.0)
        .attr("fill", url(ids::RUBBER))
        .attr("stroke", "#000")
        .attr("stroke-width", 1.0)
        .end();
    for k in 0..30u32 {
        let (fx, fy) = polar(cx, cy, 45.0, f64::from(k * 12));
        circle(w, fx, fy, 3.4).attr("fill", "#070707").end();
    }
    circle(w, cx, cy, 36.0)
        .attr("fill", url(ids::PARKERIZED))
        .attr("stroke", "#0a0a0a")
        .attr("stroke-width", 1.0)
        .end();
    for r in [33.0, 29.0, 25.0] {
        circle(w, cx, cy, r)
            .attr("fill", "none")
            .attr("stroke", "#fff")
            .attr("stroke-opacity", 0.04)
            .end();
    }
    // Engraved index line on the skirt.
    if n > 0 {
        let pa = lamp_angle(sel, n);
        let (x1, y1) = polar(cx, cy, 24.0, pa);
        let (x2, y2) = polar(cx, cy, 47.0, pa);
        w.tag("line")
            .attr("x1", x1)
            .attr("y1", y1)
            .attr("x2", x2)
            .attr("y2", y2)
            .attr("stroke", hardware::ENGRAVED_INDEX)
            .attr("stroke-width", 3.2)
            .attr("stroke-linecap", "round")
            .end();
    }
    // Inner knob: knurled steel; no index line without submodules.
    circle(w, cx + 2.0, cy + 3.0, 17.0)
        .attr("fill", "#000")
        .attr("opacity", 0.5)
        .end();
    circle(w, cx, cy, 17.0)
        .attr("fill", url(ids::STEEL))
        .attr("stroke", "#111")
        .attr("stroke-width", 0.8)
        .end();
    w.tag("g")
        .attr("stroke", "#2a2c30")
        .attr("stroke-width", 0.9)
        .open();
    for k in 0..36u32 {
        let deg = f64::from(k * 10);
        let (x1, y1) = polar(cx, cy, 16.5, deg);
        let (x2, y2) = polar(cx, cy, 13.0, deg);
        w.tag("line")
            .attr("x1", x1)
            .attr("y1", y1)
            .attr("x2", x2)
            .attr("y2", y2)
            .end();
    }
    w.close();
    circle(w, cx, cy, 11.0)
        .attr("fill", "#2c2e32")
        .attr("stroke", "#111")
        .attr("stroke-width", 0.6)
        .end();
    if !spec.submodules.is_empty() {
        let sa = inner_angle(spec.selected_sub, spec.submodules.len());
        let (x2, y2) = polar(cx, cy, 10.0, sa);
        w.tag("line")
            .attr("x1", cx)
            .attr("y1", cy)
            .attr("x2", x2)
            .attr("y2", y2)
            .attr("stroke", lamp::WARNING)
            .attr("stroke-width", 2.4)
            .attr("stroke-linecap", "round")
            .end();
    }
}

/// Draw the rotary module in plate `r` (reference size 302×192).
pub fn rotary(w: &mut SvgWriter, r: Rect, spec: &RotarySpec<'_>) {
    plate(w, r, true);
    embossed(
        w,
        r.x + 20.0,
        r.y + 26.0,
        "REPOSITORY",
        TextStyle::sized(12.0),
    );
    knob(w, r.x + 84.0, r.y + 106.0, spec);
    let lx = r.x + 168.0;
    let drum = LcdSpec::new(1.3, 3.0).cols(DRUM_COLS);
    etched(
        w,
        r.x + 20.0,
        r.bottom() - 6.0,
        "OUTER REPO  INNER SUB",
        TextStyle::sized(8.0).spacing(0.5),
    );
    etched(w, lx, r.y + 26.0, "REPO", TextStyle::sized(9.0));
    for (line, dy) in drum_lines(spec.repos, spec.selected)
        .iter()
        .zip([34.0, 56.0, 78.0])
    {
        lcd(w, lx, r.y + dy, line, drum);
    }
    let n_subs = spec.submodules.len();
    let sel_sub = spec.selected_sub.filter(|i| *i < n_subs);
    let caption = match (n_subs, sel_sub) {
        (0, _) => "SUBMODULE".to_owned(),
        (n, Some(i)) => format!("SUBMODULE {} OF {n}", i + 1),
        (n, None) => format!("PARENT  {n} SUBMODULES"),
    };
    etched(w, lx, r.y + 112.0, &caption, TextStyle::sized(9.0));
    let readout = match sel_sub {
        _ if n_subs == 0 => " -- NONE --".to_owned(),
        Some(i) => format!(">{}", spec.submodules[i]),
        None => ">PARENT".to_owned(),
    };
    lcd(w, lx, r.y + 118.0, &readout, drum);
    for k in 0..n_subs {
        let on = sel_sub == Some(k);
        #[allow(clippy::cast_precision_loss)]
        let x = lx + 3.0 + k as f64 * 16.0;
        w.tag("rect")
            .attr("x", x)
            .attr("y", r.y + 146.0)
            .attr("width", 11.0)
            .attr("height", 6.0)
            .attr("rx", 1.0)
            .attr("fill", if on { lamp::CAUTION } else { "#1b1d1f" })
            .attr("stroke", "#000")
            .attr("stroke-width", 0.5)
            .attr_opt("filter", on.then(|| url(ids::LAMP_GLOW)))
            .end();
    }
}
