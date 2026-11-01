// SPDX-License-Identifier: AGPL-3.0-only
//! Annunciator tiles: etched legend on the lens, dark legend when lit.

use super::Rect;
use crate::defs::{ids, url};
use crate::metrics::ETCH_FAMILY_ALIAS;
use crate::svg::SvgWriter;
use crate::tokens::{lamp, Lamp};

/// Draw an annunciator tile.
pub fn annunciator(w: &mut SvgWriter, r: Rect, legend: &str, state: Lamp) {
    w.tag("rect")
        .attr("x", r.x - 2.0)
        .attr("y", r.y - 2.0)
        .attr("width", r.w + 4.0)
        .attr("height", r.h + 4.0)
        .attr("rx", 3.0)
        .attr("fill", "#0b0c0e")
        .end();
    let (fill, filter, ink) = match state.color() {
        Some(c) => (c, Some(url(ids::LAMP_GLOW)), lamp::LIT_LEGEND),
        None => (lamp::OFF, None, lamp::OFF_LEGEND),
    };
    w.tag("rect")
        .attr("x", r.x)
        .attr("y", r.y)
        .attr("width", r.w)
        .attr("height", r.h)
        .attr("rx", 2.0)
        .attr("fill", fill)
        .attr_opt("filter", filter)
        .end();
    w.tag("text")
        .attr("x", r.x + r.w / 2.0)
        .attr("y", r.y + r.h / 2.0 + 4.0)
        .attr("text-anchor", "middle")
        .attr("font-family", ETCH_FAMILY_ALIAS)
        .attr("font-size", 11.0)
        .attr("fill", ink)
        .text(legend);
    w.tag("rect")
        .attr("x", r.x)
        .attr("y", r.y)
        .attr("width", r.w)
        .attr("height", r.h / 2.0)
        .attr("rx", 2.0)
        .attr("fill", "#fff")
        .attr("opacity", 0.08)
        .end();
}

/// Small page-dot lamp (carousel position).
pub fn page_dot(w: &mut SvgWriter, cx: f64, cy: f64, lit: bool) {
    w.tag("circle")
        .attr("cx", cx)
        .attr("cy", cy)
        .attr("r", 3.5)
        .attr("fill", if lit { lamp::OK } else { "#222" })
        .attr_opt("filter", lit.then(|| url(ids::LAMP_GLOW)))
        .attr("stroke", "#0a0a0a")
        .end();
}
