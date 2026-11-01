// SPDX-License-Identifier: AGPL-3.0-only
//! Static-layer surfaces: wood veneer, tungsten plates, punched features.

use super::Rect;
use crate::defs::{ids, url};
use crate::svg::{PathData, SvgWriter};
use crate::tokens::{punch, tungsten, wood as wood_tok};

/// Cabinet-scale wood veneer covering `width`×`height` from the origin.
///
/// Grain density and leaf width are fixed in CSS px, so a smaller viewport
/// shows a smaller window onto the same large cabinet.
pub fn wood(w: &mut SvgWriter, width: f64, height: f64) {
    w.tag("rect")
        .attr("width", width)
        .attr("height", height)
        .attr("fill", wood_tok::BASE)
        .end();
    w.tag("rect")
        .attr("width", width)
        .attr("height", height)
        .attr("filter", url(ids::WOODGRAIN))
        .end();
    // Book-matched leaves: alternate tone plus hairline seams.
    let leaf = wood_tok::LEAF_PX;
    let mut k = 0usize;
    let mut x0 = 0.0;
    while x0 < width {
        if k % 2 == 1 {
            w.tag("rect")
                .attr("x", x0)
                .attr("y", 0.0)
                .attr("width", leaf)
                .attr("height", height)
                .attr("fill", "#000")
                .attr("opacity", 0.10)
                .end();
        }
        if k > 0 {
            w.tag("line")
                .attr("x1", x0)
                .attr("y1", 0.0)
                .attr("x2", x0)
                .attr("y2", height)
                .attr("stroke", wood_tok::SEAM_DARK)
                .attr("stroke-width", 1.0)
                .end();
            w.tag("line")
                .attr("x1", x0 + 1.0)
                .attr("y1", 0.0)
                .attr("x2", x0 + 1.0)
                .attr("y2", height)
                .attr("stroke", wood_tok::SEAM_LIP)
                .attr("stroke-width", 0.5)
                .attr("opacity", 0.6)
                .end();
        }
        k += 1;
        x0 += leaf;
    }
    w.tag("rect")
        .attr("width", width)
        .attr("height", height)
        .attr("fill", url(ids::VIGNETTE))
        .attr("opacity", 0.5)
        .end();
}

/// Punched mounting hole with a lit lower lip.
pub fn hole(w: &mut SvgWriter, cx: f64, cy: f64, r: f64) {
    w.tag("circle")
        .attr("cx", cx)
        .attr("cy", cy)
        .attr("r", r)
        .attr("fill", punch::HOLE)
        .end();
    let d = PathData::new()
        .m(cx - r, cy)
        .a(r, false, false, cx + r, cy)
        .build();
    w.tag("path")
        .attr("d", d)
        .attr("fill", "none")
        .attr("stroke", punch::LOWER_LIP)
        .attr("stroke-width", 0.8)
        .end();
}

/// Default mounting-hole radius.
pub const HOLE_RADIUS: f64 = 3.4;

/// Dark tungsten plate: satin brushing, anisotropic sheen, bevel and
/// (optionally) four corner mounting holes.
pub fn plate(w: &mut SvgWriter, r: Rect, holes: bool) {
    let rx = tungsten::CORNER_RADIUS;
    w.tag("rect")
        .attr("x", r.x)
        .attr("y", r.y)
        .attr("width", r.w)
        .attr("height", r.h)
        .attr("rx", rx)
        .attr("fill", url(ids::TUNGSTEN))
        .attr("filter", url(ids::BRUSHED))
        .end();
    w.tag("rect")
        .attr("x", r.x)
        .attr("y", r.y)
        .attr("width", r.w)
        .attr("height", r.h)
        .attr("rx", rx)
        .attr("fill", url(ids::SHEEN))
        .end();
    let light = PathData::new()
        .m(r.x + 0.5, r.bottom() - rx)
        .v(r.y + rx)
        .q(r.x + 0.5, r.y + 0.5, r.x + rx, r.y + 0.5)
        .h(r.right() - rx)
        .build();
    w.tag("path")
        .attr("d", light)
        .attr("fill", "none")
        .attr("stroke", tungsten::BEVEL_LIGHT)
        .attr("stroke-width", 1.0)
        .end();
    let dark = PathData::new()
        .m(r.right() - 0.5, r.y + rx)
        .v(r.bottom() - rx)
        .q(
            r.right() - 0.5,
            r.bottom() - 0.5,
            r.right() - rx,
            r.bottom() - 0.5,
        )
        .h(r.x + rx)
        .build();
    w.tag("path")
        .attr("d", dark)
        .attr("fill", "none")
        .attr("stroke", tungsten::BEVEL_DARK)
        .attr("stroke-width", 1.0)
        .end();
    if holes {
        for (hx, hy) in [
            (r.x + 9.0, r.y + 9.0),
            (r.right() - 9.0, r.y + 9.0),
            (r.x + 9.0, r.bottom() - 9.0),
            (r.right() - 9.0, r.bottom() - 9.0),
        ] {
            hole(w, hx, hy, HOLE_RADIUS);
        }
    }
}

/// Column of punched vent slots.
pub fn vent_slots(w: &mut SvgWriter, x: f64, y: f64, width: f64, count: usize) {
    for i in 0..count {
        #[allow(clippy::cast_precision_loss)]
        let yy = y + i as f64 * 12.0;
        w.tag("rect")
            .attr("x", x)
            .attr("y", yy)
            .attr("width", width)
            .attr("height", 5.0)
            .attr("rx", 2.5)
            .attr("fill", punch::HOLE)
            .end();
    }
}

/// Punched breadcrumb chevron pointing right, centred vertically on `cy`.
pub fn chevron(w: &mut SvgWriter, x: f64, cy: f64) {
    let d = PathData::new()
        .m(x - 4.0, cy - 6.0)
        .l(x + 4.0, cy)
        .l(x - 4.0, cy + 6.0)
        .z()
        .build();
    w.tag("path").attr("d", d).attr("fill", punch::HOLE).end();
}
