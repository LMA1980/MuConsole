// SPDX-License-Identifier: AGPL-3.0-only
//! Nuclear-control-room pistol-grip rotary control switch and indicating lamp.
//!
//! The switch follows the GE SBM / Westinghouse W-2 family: a square black
//! engraved escutcheon with two screws, position ticks and white legends, and
//! a black bakelite pistol grip with a white pointer inlay and a steel hub
//! screw. Position angles are in degrees with **0 = up, clockwise positive**.
//!
//! There are no guards or covers: protection comes from the switch type
//! (maintained selectors need a deliberate rotation, spring-return switches
//! cannot be left in an action position, PULL-TO-TURN needs two motions).

use super::polar;
use crate::defs::{ids, url};
use crate::metrics::ETCH_FAMILY_ALIAS;
use crate::svg::{num, PathData, SvgWriter};
use crate::tokens::{indicator, switch as tok};

/// Phillips-head screw.
pub fn screw(w: &mut SvgWriter, x: f64, y: f64, r: f64, rot_deg: f64) {
    w.tag("circle")
        .attr("cx", x)
        .attr("cy", y)
        .attr("r", r + 0.6)
        .attr("fill", "#0b0b0c")
        .end();
    w.tag("circle")
        .attr("cx", x)
        .attr("cy", y)
        .attr("r", r)
        .attr("fill", url(ids::STEEL))
        .attr("stroke", "#111")
        .attr("stroke-width", 0.5)
        .end();
    w.tag("g")
        .attr(
            "transform",
            format!("rotate({} {} {})", num(rot_deg), num(x), num(y)),
        )
        .attr("stroke", "#1a1b1d")
        .attr("stroke-width", 1.0)
        .attr("stroke-linecap", "round")
        .open();
    let a = r * 0.65;
    w.tag("line")
        .attr("x1", x - a)
        .attr("y1", y)
        .attr("x2", x + a)
        .attr("y2", y)
        .end();
    w.tag("line")
        .attr("x1", x)
        .attr("y1", y - a)
        .attr("x2", x)
        .attr("y2", y + a)
        .end();
    w.close();
}

/// Indicating lamp lens colour.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LensColor {
    /// Running / energized.
    Red,
    /// Ready / normal / selected.
    Green,
    /// Caution.
    Amber,
}

impl LensColor {
    const fn lit_id(self) -> &'static str {
        match self {
            Self::Red => ids::LENS_RED,
            Self::Green => ids::LENS_GREEN,
            Self::Amber => ids::LENS_AMBER,
        }
    }

    const fn unlit_tint(self) -> &'static str {
        match self {
            Self::Red => indicator::RED_UNLIT,
            Self::Green => indicator::GREEN_UNLIT,
            Self::Amber => indicator::AMBER_UNLIT,
        }
    }
}

/// Control-room indicating lamp: knurled steel bezel and domed lens.
pub fn indicator_lamp(w: &mut SvgWriter, x: f64, y: f64, color: LensColor, lit: bool, r: f64) {
    w.tag("circle")
        .attr("cx", x + 1.2)
        .attr("cy", y + 1.6)
        .attr("r", r + 2.6)
        .attr("fill", "#000")
        .attr("opacity", 0.5)
        .end();
    w.tag("circle")
        .attr("cx", x)
        .attr("cy", y)
        .attr("r", r + 2.6)
        .attr("fill", url(ids::STEEL))
        .attr("stroke", "#111")
        .attr("stroke-width", 0.6)
        .end();
    w.tag("g")
        .attr("stroke", "#2a2c30")
        .attr("stroke-width", 0.6)
        .open();
    for k in 0..24u32 {
        let deg = f64::from(k * 15);
        let (x1, y1) = polar(x, y, r + 2.5, deg);
        let (x2, y2) = polar(x, y, r + 1.2, deg);
        w.tag("line")
            .attr("x1", x1)
            .attr("y1", y1)
            .attr("x2", x2)
            .attr("y2", y2)
            .end();
    }
    w.close();
    let lens = if lit { color.lit_id() } else { ids::LENS_OFF };
    w.tag("circle")
        .attr("cx", x)
        .attr("cy", y)
        .attr("r", r)
        .attr("fill", url(lens))
        .attr_opt("filter", lit.then(|| url(ids::LAMP_GLOW)))
        .end();
    if !lit {
        w.tag("circle")
            .attr("cx", x)
            .attr("cy", y)
            .attr("r", r)
            .attr("fill", color.unlit_tint())
            .attr("opacity", 0.55)
            .end();
    }
    w.tag("ellipse")
        .attr("cx", x - r * 0.3)
        .attr("cy", y - r * 0.38)
        .attr("rx", r * 0.42)
        .attr("ry", r * 0.22)
        .attr("fill", "#fff")
        .attr("opacity", if lit { 0.55 } else { 0.18 })
        .end();
}

/// One labelled switch position.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SwitchPosition<'a> {
    /// Degrees, 0 = up, clockwise positive.
    pub angle_deg: f64,
    pub legend: &'a str,
}

impl<'a> SwitchPosition<'a> {
    #[must_use]
    pub const fn new(angle_deg: f64, legend: &'a str) -> Self {
        Self { angle_deg, legend }
    }
}

/// Switch action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SwitchAction {
    /// Maintained N-position selector (area, CRT font, live feed).
    #[default]
    Maintained,
    /// Spring-return-to-centre; engraved return arrows.
    SpringReturn,
    /// PULL-TO-TURN with a red collar, spring-return (state-changing actions).
    PullToTurn,
}

impl SwitchAction {
    #[must_use]
    pub const fn springs_back(self) -> bool {
        matches!(self, Self::SpringReturn | Self::PullToTurn)
    }

    #[must_use]
    pub const fn needs_pull(self) -> bool {
        matches!(self, Self::PullToTurn)
    }
}

/// Pistol-grip switch description.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PistolSwitch<'a> {
    pub cx: f64,
    pub cy: f64,
    pub positions: &'a [SwitchPosition<'a>],
    /// Index into `positions` (clamped).
    pub current: usize,
    /// Escutcheon edge length in px (reference 64).
    pub size: f64,
    pub action: SwitchAction,
    /// Engraved nameplate at the bottom of the escutcheon.
    pub nameplate: Option<&'a str>,
}

impl<'a> PistolSwitch<'a> {
    #[must_use]
    pub const fn new(
        cx: f64,
        cy: f64,
        positions: &'a [SwitchPosition<'a>],
        current: usize,
    ) -> Self {
        Self {
            cx,
            cy,
            positions,
            current,
            size: tok::REFERENCE_SIZE,
            action: SwitchAction::Maintained,
            nameplate: None,
        }
    }

    #[must_use]
    pub const fn size(mut self, size: f64) -> Self {
        self.size = size;
        self
    }

    #[must_use]
    pub const fn action(mut self, action: SwitchAction) -> Self {
        self.action = action;
        self
    }

    #[must_use]
    pub const fn nameplate(mut self, name: &'a str) -> Self {
        self.nameplate = Some(name);
        self
    }

    /// Grip angle (0 = up) for the current position; 0 when there are none.
    #[must_use]
    pub fn grip_angle(&self) -> f64 {
        if self.positions.is_empty() {
            return 0.0;
        }
        let i = self.current.min(self.positions.len() - 1);
        self.positions[i].angle_deg
    }

    /// Index of the position nearest to `angle_deg` (pointer drag snapping).
    #[must_use]
    pub fn nearest_position(&self, angle_deg: f64) -> Option<usize> {
        self.positions
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| {
                let da = angle_distance(a.angle_deg, angle_deg);
                let db = angle_distance(b.angle_deg, angle_deg);
                da.total_cmp(&db)
            })
            .map(|(i, _)| i)
    }
}

/// Smallest absolute difference between two angles in degrees.
#[must_use]
pub fn angle_distance(a: f64, b: f64) -> f64 {
    let d = (a - b).rem_euclid(360.0);
    d.min(360.0 - d)
}

/// Grip outline in the local frame (pointer up), scaled by `k`.
fn grip_path(k: f64) -> String {
    PathData::new()
        .m(-4.6 * k, -2.0 * k)
        .l(-2.3 * k, -18.5 * k)
        .q(0.0, -22.5 * k, 2.3 * k, -18.5 * k)
        .l(4.6 * k, -2.0 * k)
        .c(7.4 * k, 4.0 * k, 8.0 * k, 10.0 * k, 6.8 * k, 13.5 * k)
        .c(5.0 * k, 18.5 * k, -5.0 * k, 18.5 * k, -6.8 * k, 13.5 * k)
        .c(-8.0 * k, 10.0 * k, -7.4 * k, 4.0 * k, -4.6 * k, -2.0 * k)
        .z()
        .build()
}

fn legend_text(w: &mut SvgWriter, x: f64, y: f64, size: f64, txt: &str, opacity: Option<f64>) {
    w.tag("text")
        .attr("x", x)
        .attr("y", y)
        .attr("font-family", ETCH_FAMILY_ALIAS)
        .attr("fill", tok::LEGEND)
        .attr("text-anchor", "middle")
        .attr("font-size", size)
        .attr_opt("opacity", opacity)
        .text(txt);
}

fn escutcheon(w: &mut SvgWriter, cx: f64, cy: f64, size: f64, k: f64) {
    let h = size / 2.0;
    w.tag("rect")
        .attr("x", cx - h + 2.0)
        .attr("y", cy - h + 3.0)
        .attr("width", size)
        .attr("height", size)
        .attr("rx", 4.0 * k)
        .attr("fill", "#000")
        .attr("opacity", 0.5)
        .end();
    w.tag("rect")
        .attr("x", cx - h)
        .attr("y", cy - h)
        .attr("width", size)
        .attr("height", size)
        .attr("rx", 4.0 * k)
        .attr("fill", url(ids::ESCUTCHEON))
        .attr("stroke", tok::ESCUTCHEON_EDGE)
        .attr("stroke-width", 0.8)
        .end();
    let d = PathData::new()
        .m(cx - h + 1.0, cy + h - 4.0 * k)
        .v(cy - h + 4.0 * k)
        .q(cx - h + 1.0, cy - h + 1.0, cx - h + 4.0 * k, cy - h + 1.0)
        .h(cx + h - 4.0 * k)
        .build();
    w.tag("path")
        .attr("d", d)
        .attr("fill", "none")
        .attr("stroke", "#5c6066")
        .attr("stroke-width", 0.6)
        .attr("opacity", 0.7)
        .end();
    screw(w, cx - h + 6.0 * k, cy - h + 6.0 * k, 2.0 * k, 20.0);
    screw(w, cx + h - 6.0 * k, cy + h - 6.0 * k, 2.0 * k, -30.0);
}

fn return_arrows(w: &mut SvgWriter, cx: f64, cy: f64, k: f64) {
    for sgn in [-1.0f64, 1.0] {
        let rr = 15.0 * k;
        let (x0, y0) = polar(cx, cy, rr, sgn * 40.0 - 90.0);
        let (x1, y1) = polar(cx, cy, rr, sgn * 12.0 - 90.0);
        let d = PathData::new()
            .m(x0, y0)
            .a(rr, false, sgn < 0.0, x1, y1)
            .build();
        w.tag("path")
            .attr("d", d)
            .attr("fill", "none")
            .attr("stroke", tok::LEGEND)
            .attr("stroke-width", 0.9 * k)
            .attr("opacity", 0.7)
            .end();
        w.tag("circle")
            .attr("cx", x1)
            .attr("cy", y1)
            .attr("r", 1.2 * k)
            .attr("fill", tok::LEGEND)
            .attr("opacity", 0.7)
            .end();
    }
}

fn grip(w: &mut SvgWriter, cx: f64, cy: f64, k: f64, angle: f64) {
    let d = grip_path(k);
    w.tag("g")
        .attr(
            "transform",
            format!(
                "translate({} {}) rotate({})",
                num(cx + 2.8 * k),
                num(cy + 3.8 * k),
                num(angle)
            ),
        )
        .open();
    w.tag("path")
        .attr("d", d.as_str())
        .attr("fill", "#000")
        .attr("opacity", 0.55)
        .end();
    w.close();
    w.tag("g")
        .attr(
            "transform",
            format!("translate({} {}) rotate({})", num(cx), num(cy), num(angle)),
        )
        .open();
    w.tag("path")
        .attr("d", d.as_str())
        .attr("fill", url(ids::BAKELITE))
        .attr("stroke", "#000")
        .attr("stroke-width", 0.8)
        .end();
    let edge = PathData::new()
        .m(-3.6 * k, -4.0 * k)
        .l(-1.9 * k, -17.0 * k)
        .build();
    w.tag("path")
        .attr("d", edge)
        .attr("stroke", "#fff")
        .attr("stroke-width", k)
        .attr("opacity", 0.22)
        .attr("stroke-linecap", "round")
        .end();
    let butt = PathData::new()
        .m(-5.6 * k, 6.0 * k)
        .c(-6.2 * k, 10.0 * k, -5.0 * k, 14.0 * k, -3.0 * k, 15.5 * k)
        .build();
    w.tag("path")
        .attr("d", butt)
        .attr("fill", "none")
        .attr("stroke", "#fff")
        .attr("stroke-width", 0.9 * k)
        .attr("opacity", 0.18)
        .end();
    w.tag("line")
        .attr("x1", 0.0)
        .attr("y1", -6.0 * k)
        .attr("x2", 0.0)
        .attr("y2", -18.5 * k)
        .attr("stroke", tok::POINTER)
        .attr("stroke-width", 1.6 * k)
        .attr("stroke-linecap", "round")
        .end();
    w.tag("circle")
        .attr("cx", 0.0)
        .attr("cy", 0.0)
        .attr("r", 3.4 * k)
        .attr("fill", url(ids::STEEL))
        .attr("stroke", "#111")
        .attr("stroke-width", 0.5)
        .end();
    w.tag("line")
        .attr("x1", -2.0 * k)
        .attr("y1", 0.0)
        .attr("x2", 2.0 * k)
        .attr("y2", 0.0)
        .attr("stroke", "#222")
        .attr("stroke-width", 0.8)
        .end();
    w.close();
}

/// Draw a pistol-grip rotary control switch.
pub fn pistol_switch(w: &mut SvgWriter, s: &PistolSwitch<'_>) {
    let k = s.size / tok::REFERENCE_SIZE;
    let h = s.size / 2.0;
    let (cx, cy) = (s.cx, s.cy);
    escutcheon(w, cx, cy, s.size, k);
    for p in s.positions {
        let deg = p.angle_deg - 90.0;
        let (x1, y1) = polar(cx, cy, 22.5 * k, deg);
        let (x2, y2) = polar(cx, cy, 25.5 * k, deg);
        w.tag("line")
            .attr("x1", x1)
            .attr("y1", y1)
            .attr("x2", x2)
            .attr("y2", y2)
            .attr("stroke", tok::LEGEND)
            .attr("stroke-width", 1.4 * k)
            .end();
        let (lx, ly) = polar(cx, cy, 29.5 * k, deg);
        legend_text(w, lx, ly + 2.6 * k, 7.2 * k, p.legend, None);
    }
    if s.action.springs_back() {
        return_arrows(w, cx, cy, k);
    }
    if let Some(name) = s.nameplate {
        legend_text(w, cx, cy + h - 5.0 * k, 6.4 * k, name, Some(0.9));
    }
    if s.action.needs_pull() {
        w.tag("circle")
            .attr("cx", cx)
            .attr("cy", cy)
            .attr("r", 11.5 * k)
            .attr("fill", tok::PULL_COLLAR)
            .attr("stroke", "#3a0407")
            .attr("stroke-width", 0.8)
            .end();
    }
    if !s.positions.is_empty() {
        grip(w, cx, cy, k, s.grip_angle());
    }
}
