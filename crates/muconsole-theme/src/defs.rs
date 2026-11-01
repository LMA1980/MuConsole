// SPDX-License-Identifier: AGPL-3.0-only
//! Shared `<defs>`: filters, gradients, patterns and LCD glyph symbols.
//!
//! Instruments reference these by id (see [`ids`]); write them once per root
//! `<svg>` with [`write_defs`]. Filters for the CRT pipeline are only emitted
//! when the resolved [`Effects`] need them.

use crate::lcd::{self, GLYPHS, GLYPH_COLS, GLYPH_ROWS};
use crate::svg::{num, SvgWriter};
use crate::tokens::{self, Effects, Phosphor, CRT_CD, CRT_CI};

/// Element ids used by the instrument library.
pub mod ids {
    pub const WOODGRAIN: &str = "mu-woodgrain";
    pub const BRUSHED: &str = "mu-brushed";
    pub const SHEEN: &str = "mu-sheen";
    pub const TUNGSTEN: &str = "mu-tungsten";
    pub const STEEL: &str = "mu-steel";
    pub const PARKERIZED: &str = "mu-park";
    pub const RUBBER: &str = "mu-rubber";
    pub const BAKELITE: &str = "mu-bakelite";
    pub const ESCUTCHEON: &str = "mu-esc";
    pub const LENS_RED: &str = "mu-lens-red";
    pub const LENS_GREEN: &str = "mu-lens-green";
    pub const LENS_AMBER: &str = "mu-lens-amber";
    pub const LENS_OFF: &str = "mu-lens-off";
    pub const LCD_BACKLIGHT: &str = "mu-lcdbg";
    pub const CRT_CI_GLASS: &str = "mu-crt-ci";
    pub const CRT_CD_GLASS: &str = "mu-crt-cd";
    pub const VIGNETTE: &str = "mu-vignette";
    pub const GLASS: &str = "mu-glass";
    pub const SCANLINES: &str = "mu-scan";
    pub const PHOSPHOR: &str = "mu-phos";
    pub const LAMP_GLOW: &str = "mu-lamp";
    pub const LCD_GHOST: &str = "mu-lcd-ghost";
}

/// `url(#id)` reference.
#[must_use]
pub fn url(id: &str) -> String {
    format!("url(#{id})")
}

/// Symbol id of an LCD glyph (`mu-lcd-41` for `A`).
#[must_use]
pub fn lcd_symbol_id(c: char) -> String {
    format!("mu-lcd-{:02x}", u32::from(c))
}

/// Glass gradient id for a phosphor.
#[must_use]
pub fn crt_glass_id(p: &Phosphor) -> &'static str {
    if p.name == CRT_CD.name {
        ids::CRT_CD_GLASS
    } else {
        ids::CRT_CI_GLASS
    }
}

type Stop<'a> = (f64, &'a str, Option<f64>);

fn stops(w: &mut SvgWriter, list: &[Stop<'_>]) {
    for &(offset, color, opacity) in list {
        w.tag("stop")
            .attr("offset", offset)
            .attr("stop-color", color)
            .attr_opt("stop-opacity", opacity)
            .end();
    }
}

fn linear(w: &mut SvgWriter, id: &str, dir: (f64, f64, f64, f64), list: &[Stop<'_>]) {
    w.tag("linearGradient")
        .attr("id", id)
        .attr("x1", dir.0)
        .attr("y1", dir.1)
        .attr("x2", dir.2)
        .attr("y2", dir.3)
        .open();
    stops(w, list);
    w.close();
}

fn radial(w: &mut SvgWriter, id: &str, c: (f64, f64, f64), list: &[Stop<'_>]) {
    w.tag("radialGradient")
        .attr("id", id)
        .attr("cx", c.0)
        .attr("cy", c.1)
        .attr("r", c.2)
        .open();
    stops(w, list);
    w.close();
}

fn turbulence(w: &mut SvgWriter, freq: (f64, f64), octaves: u32, seed: u32, result: &str) {
    w.tag("feTurbulence")
        .attr("type", "fractalNoise")
        .attr("baseFrequency", format!("{} {}", num(freq.0), num(freq.1)))
        .attr("numOctaves", octaves)
        .attr("seed", seed)
        .attr("result", result)
        .end();
}

fn color_matrix(w: &mut SvgWriter, input: &str, values: &str, result: &str) {
    w.tag("feColorMatrix")
        .attr("in", input)
        .attr("type", "matrix")
        .attr("values", values)
        .attr("result", result)
        .end();
}

fn merge(w: &mut SvgWriter, inputs: &[&str], result: Option<&str>) {
    w.tag("feMerge").attr_opt("result", result).open();
    for i in inputs {
        w.tag("feMergeNode").attr("in", *i).end();
    }
    w.close();
}

fn full_filter(w: &mut SvgWriter, id: &str) {
    w.tag("filter")
        .attr("id", id)
        .attr("x", "0")
        .attr("y", "0")
        .attr("width", "100%")
        .attr("height", "100%")
        .open();
}

fn glow_filter(w: &mut SvgWriter, id: &str, std_dev: f64, pad_pct: u32) {
    let pad = format!("-{pad_pct}%");
    let size = format!("{}%", 100 + 2 * pad_pct);
    w.tag("filter")
        .attr("id", id)
        .attr("x", pad.as_str())
        .attr("y", pad.as_str())
        .attr("width", size.as_str())
        .attr("height", size.as_str())
        .open();
    w.tag("feGaussianBlur")
        .attr("stdDeviation", std_dev)
        .attr("result", "b")
        .end();
    merge(w, &["b", "SourceGraphic"], None);
    w.close();
}

/// Cabinet-scale wood veneer filter (fine long grain, broad figure, pores).
fn wood_filter(w: &mut SvgWriter) {
    use tokens::wood::{FIGURE_FREQ, GRAIN_FREQ};
    full_filter(w, ids::WOODGRAIN);
    turbulence(w, GRAIN_FREQ, 4, 11, "n");
    color_matrix(
        w,
        "n",
        "0 0 0 0 0.25  0 0 0 0 0.15  0 0 0 0 0.085  2.0 0 0 0 -0.88",
        "grain",
    );
    turbulence(w, FIGURE_FREQ, 2, 4, "f");
    color_matrix(
        w,
        "f",
        "0 0 0 0 0.20  0 0 0 0 0.12  0 0 0 0 0.07  1.2 0 0 0 -0.45",
        "figure",
    );
    turbulence(w, (0.035, 1.1), 1, 9, "p");
    color_matrix(
        w,
        "p",
        "0 0 0 0 0.02  0 0 0 0 0.01  0 0 0 0 0.005  3.0 0 0 0 -1.75",
        "pores",
    );
    merge(w, &["figure", "grain", "pores"], None);
    w.close();
}

/// Fine satin-brushed tungsten filter (hairlines + dark counter-lines).
fn brushed_filter(w: &mut SvgWriter) {
    use tokens::tungsten::{COUNTER_FREQ, HAIR_FREQ};
    full_filter(w, ids::BRUSHED);
    turbulence(w, HAIR_FREQ, 3, 5, "t");
    color_matrix(
        w,
        "t",
        "0 0 0 0 0.78  0 0 0 0 0.80  0 0 0 0 0.83  0.22 0 0 0 -0.085",
        "hair",
    );
    turbulence(w, COUNTER_FREQ, 2, 17, "t2");
    color_matrix(
        w,
        "t2",
        "0 0 0 0 0  0 0 0 0 0  0 0 0 0 0  0.20 0 0 0 -0.08",
        "dark",
    );
    merge(w, &["hair", "dark"], Some("m"));
    w.tag("feComposite")
        .attr("in", "m")
        .attr("in2", "SourceGraphic")
        .attr("operator", "in")
        .attr("result", "sc")
        .end();
    merge(w, &["SourceGraphic", "sc"], None);
    w.close();
}

fn materials(w: &mut SvgWriter) {
    use tokens::hardware as hw;
    let white = "#fff";
    linear(
        w,
        ids::SHEEN,
        (0.0, 0.0, 1.0, 0.0),
        &[
            (0.0, white, Some(0.0)),
            (0.42, white, Some(0.045)),
            (0.5, white, Some(tokens::tungsten::SHEEN_PEAK)),
            (0.58, white, Some(0.045)),
            (1.0, white, Some(0.0)),
        ],
    );
    linear(
        w,
        ids::TUNGSTEN,
        (0.0, 0.0, 0.0, 1.0),
        &[
            (0.0, tokens::tungsten::TOP, None),
            (1.0, tokens::tungsten::BOTTOM, None),
        ],
    );
    linear(
        w,
        ids::STEEL,
        (0.0, 0.0, 1.0, 0.0),
        &[
            (0.0, hw::STEEL_DARK, None),
            (0.4, hw::STEEL_LIGHT, None),
            (0.62, "#8a8e94", None),
            (1.0, "#3a3d42", None),
        ],
    );
    linear(
        w,
        ids::PARKERIZED,
        (0.0, 0.0, 0.0, 1.0),
        &[
            (0.0, hw::PARKERIZED_TOP, None),
            (0.5, "#26282b", None),
            (1.0, hw::PARKERIZED_BOTTOM, None),
        ],
    );
    radial(
        w,
        ids::RUBBER,
        (0.35, 0.3, 0.8),
        &[
            (0.0, "#3a3a3a", None),
            (0.6, hw::RUBBER, None),
            (1.0, "#0a0a0a", None),
        ],
    );
    radial(
        w,
        ids::BAKELITE,
        (0.35, 0.3, 0.85),
        &[
            (0.0, tokens::switch::BAKELITE_HIGHLIGHT, None),
            (0.35, "#1c1c1e", None),
            (1.0, tokens::switch::BAKELITE_SHADOW, None),
        ],
    );
    linear(
        w,
        ids::ESCUTCHEON,
        (0.0, 0.0, 0.0, 1.0),
        &[
            (0.0, tokens::switch::ESCUTCHEON_TOP, None),
            (1.0, tokens::switch::ESCUTCHEON_BOTTOM, None),
        ],
    );
    for (id, hi, mid, lo) in [
        (ids::LENS_RED, "#ffd0c8", tokens::indicator::RED, "#7a0d08"),
        (
            ids::LENS_GREEN,
            "#d8ffe0",
            tokens::indicator::GREEN,
            "#0b5a22",
        ),
        (
            ids::LENS_AMBER,
            "#fff0c8",
            tokens::indicator::AMBER,
            "#7a4a08",
        ),
    ] {
        radial(
            w,
            id,
            (0.4, 0.35, 0.7),
            &[(0.0, hi, None), (0.35, mid, None), (1.0, lo, None)],
        );
    }
    radial(
        w,
        ids::LENS_OFF,
        (0.4, 0.35, 0.7),
        &[
            (0.0, "#5a4a48", None),
            (0.5, "#2a1e1d", None),
            (1.0, "#100b0b", None),
        ],
    );
    linear(
        w,
        ids::LCD_BACKLIGHT,
        (0.0, 0.0, 0.0, 1.0),
        &[
            (0.0, tokens::lcd::BACKLIGHT_TOP, None),
            (1.0, tokens::lcd::BACKLIGHT_BOTTOM, None),
        ],
    );
    for p in [CRT_CI, CRT_CD] {
        radial(
            w,
            crt_glass_id(&p),
            (0.5, 0.5, 0.75),
            &[(0.0, p.glass_center, None), (1.0, p.glass_edge, None)],
        );
    }
    radial(
        w,
        ids::VIGNETTE,
        (0.5, 0.5, 0.72),
        &[(0.6, "#000", Some(0.0)), (1.0, "#000", Some(0.75))],
    );
    linear(
        w,
        ids::GLASS,
        (0.0, 0.0, 1.0, 1.0),
        &[(0.0, white, Some(0.09)), (0.35, white, Some(0.0))],
    );
}

fn scanlines(w: &mut SvgWriter) {
    use tokens::crt::{SCANLINE_HEIGHT, SCANLINE_OPACITY, SCANLINE_PITCH};
    w.tag("pattern")
        .attr("id", ids::SCANLINES)
        .attr("width", SCANLINE_PITCH)
        .attr("height", SCANLINE_PITCH)
        .attr("patternUnits", "userSpaceOnUse")
        .open();
    w.tag("rect")
        .attr("width", SCANLINE_PITCH)
        .attr("height", SCANLINE_HEIGHT)
        .attr("fill", "#000")
        .attr("opacity", SCANLINE_OPACITY)
        .end();
    w.close();
}

fn lcd_symbols(w: &mut SvgWriter) {
    let viewbox = format!("0 0 {GLYPH_COLS} {GLYPH_ROWS}");
    for (c, g) in GLYPHS {
        if *c == ' ' {
            continue;
        }
        w.tag("symbol")
            .attr("id", lcd_symbol_id(*c))
            .attr("viewBox", viewbox.as_str())
            .attr("overflow", "visible")
            .open();
        for (col, row) in lcd::dots(g) {
            dot(w, col, row);
        }
        w.close();
    }
    w.tag("symbol")
        .attr("id", ids::LCD_GHOST)
        .attr("viewBox", viewbox.as_str())
        .open();
    for row in 0..GLYPH_ROWS {
        for col in 0..GLYPH_COLS {
            dot(w, col, row);
        }
    }
    w.close();
}

fn dot(w: &mut SvgWriter, col: usize, row: usize) {
    w.tag("rect")
        .attr("x", col)
        .attr("y", row)
        .attr("width", tokens::lcd::DOT_FILL)
        .attr("height", tokens::lcd::DOT_FILL)
        .end();
}

/// Write the shared `<defs>` block.
pub fn write_defs(w: &mut SvgWriter, effects: Effects) {
    w.tag("defs").open();
    wood_filter(w);
    brushed_filter(w);
    materials(w);
    if effects.scanlines {
        scanlines(w);
    }
    if effects.bloom {
        glow_filter(w, ids::PHOSPHOR, tokens::crt::BLOOM_STD_DEV, 20);
    }
    // The lamp glow is the only filter kept in Lite mode (L1 layer).
    glow_filter(w, ids::LAMP_GLOW, 3.0, 60);
    lcd_symbols(w);
    w.close();
}
