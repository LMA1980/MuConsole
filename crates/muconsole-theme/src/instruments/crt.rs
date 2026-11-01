// SPDX-License-Identifier: AGPL-3.0-only
//! CRT pipeline: bezel, phosphor glass, bloom (per screen group),
//! scanlines, vignette and glass sheen.

use super::text::Anchor;
use super::Rect;
use crate::defs::{crt_glass_id, ids, url};
use crate::svg::SvgWriter;
use crate::tokens::{crt as tok, Effects, Phosphor};

fn screen_rect(w: &mut SvgWriter, r: Rect, fill: String) {
    w.tag("rect")
        .attr("x", r.x)
        .attr("y", r.y)
        .attr("width", r.w)
        .attr("height", r.h)
        .attr("rx", tok::CORNER_RADIUS)
        .attr("fill", fill)
        .end();
}

/// Draw a CRT screen at `r` and render `content` inside the phosphor group.
///
/// Bloom is one filter on the whole screen group (never per element), and
/// the overlay layer (scanlines) is skipped when the effects are Lite.
pub fn crt(
    w: &mut SvgWriter,
    r: Rect,
    phosphor: &Phosphor,
    effects: Effects,
    content: impl FnOnce(&mut SvgWriter),
) {
    w.tag("rect")
        .attr("x", r.x - 6.0)
        .attr("y", r.y - 6.0)
        .attr("width", r.w + 12.0)
        .attr("height", r.h + 12.0)
        .attr("rx", tok::CORNER_RADIUS + 4.0)
        .attr("fill", tok::BEZEL)
        .attr("stroke", tok::BEZEL_EDGE)
        .attr("stroke-width", 1.0)
        .end();
    screen_rect(w, r, url(crt_glass_id(phosphor)));
    w.tag("g")
        .attr("class", format!("mu-crt mu-crt-{}", phosphor.name))
        .attr_opt("filter", effects.bloom.then(|| url(ids::PHOSPHOR)))
        .open();
    content(w);
    w.close();
    if effects.scanlines {
        screen_rect(w, r, url(ids::SCANLINES));
    }
    screen_rect(w, r, url(ids::VIGNETTE));
    screen_rect(w, r, url(ids::GLASS));
}

/// Phosphor text line at an explicit pixel size (from the layout grid).
pub fn crt_text(
    w: &mut SvgWriter,
    x: f64,
    y: f64,
    txt: &str,
    color: &str,
    font_px: f64,
    anchor: Anchor,
) {
    w.tag("text")
        .attr("x", x)
        .attr("y", y)
        .attr("fill", color)
        .attr("font-size", font_px)
        .attr("text-anchor", anchor.as_str())
        .text(txt);
}

/// Reverse-video bar (failed train, rollback, taint) behind dark text.
pub fn reverse_video(w: &mut SvgWriter, r: Rect, phosphor: &Phosphor) {
    w.tag("rect")
        .attr("x", r.x)
        .attr("y", r.y)
        .attr("width", r.w)
        .attr("height", r.h)
        .attr("fill", phosphor.lit)
        .end();
}
