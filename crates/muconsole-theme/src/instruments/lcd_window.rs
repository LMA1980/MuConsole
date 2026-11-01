// SPDX-License-Identifier: AGPL-3.0-only
//! Dot-matrix STN LCD window for single-line dynamic labels.

use crate::defs::{ids, lcd_symbol_id, url};
use crate::lcd::{self, CELL_ADVANCE, GLYPH_COLS, GLYPH_ROWS};
use crate::svg::SvgWriter;
use crate::tokens::lcd as tok;

/// LCD window geometry.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LcdSpec {
    /// Number of character cells; `None` uses the text length.
    pub cols: Option<usize>,
    /// Dot pitch in px.
    pub pitch: f64,
    /// Inner padding in px.
    pub pad: f64,
}

impl Default for LcdSpec {
    fn default() -> Self {
        Self {
            cols: None,
            pitch: 2.0,
            pad: 4.0,
        }
    }
}

impl LcdSpec {
    #[must_use]
    pub fn new(pitch: f64, pad: f64) -> Self {
        Self {
            cols: None,
            pitch,
            pad,
        }
    }

    #[must_use]
    pub fn cols(mut self, cols: usize) -> Self {
        self.cols = Some(cols);
        self
    }

    /// Outer size (without the 2 px bezel) for `cols` cells.
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn size_for(&self, cols: usize) -> (f64, f64) {
        let cell = CELL_ADVANCE as f64 * self.pitch;
        let w = cols as f64 * cell - self.pitch + 2.0 * self.pad;
        let h = GLYPH_ROWS as f64 * self.pitch + 2.0 * self.pad;
        (w, h)
    }

    /// Largest number of cells that fit in `width` px.
    #[must_use]
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_precision_loss
    )]
    pub fn cols_fitting(&self, width: f64) -> usize {
        let cell = CELL_ADVANCE as f64 * self.pitch;
        let n = ((width - 2.0 * self.pad + self.pitch) / cell).floor();
        if n.is_finite() && n > 0.0 {
            n as usize
        } else {
            0
        }
    }
}

/// Draw an LCD window at `(x, y)` showing `txt`; returns `(width, height)`.
///
/// Cells beyond the text show only ghost dots; text beyond `cols` is cut off
/// (callers truncate with an ellipsis cell where needed). Characters without
/// a glyph render as ghost-only cells.
#[allow(clippy::cast_precision_loss)]
pub fn lcd(w: &mut SvgWriter, x: f64, y: f64, txt: &str, spec: LcdSpec) -> (f64, f64) {
    let chars: Vec<char> = txt.chars().map(lcd::normalize).collect();
    let cols = spec.cols.unwrap_or(chars.len());
    let (width, height) = spec.size_for(cols);
    let p = spec.pitch;
    w.tag("rect")
        .attr("x", x - 2.0)
        .attr("y", y - 2.0)
        .attr("width", width + 4.0)
        .attr("height", height + 4.0)
        .attr("rx", 2.0)
        .attr("fill", tok::BEZEL)
        .end();
    w.tag("rect")
        .attr("x", x)
        .attr("y", y)
        .attr("width", width)
        .attr("height", height)
        .attr("rx", 1.5)
        .attr("fill", url(ids::LCD_BACKLIGHT))
        .end();
    w.tag("g").attr("fill", tok::PIXEL).open();
    let gw = GLYPH_COLS as f64 * p;
    let gh = GLYPH_ROWS as f64 * p;
    let ghost = format!("#{}", ids::LCD_GHOST);
    for i in 0..cols {
        let gx = x + spec.pad + i as f64 * CELL_ADVANCE as f64 * p;
        let gy = y + spec.pad;
        w.tag("use")
            .attr("href", ghost.as_str())
            .attr("x", gx)
            .attr("y", gy)
            .attr("width", gw)
            .attr("height", gh)
            .attr("opacity", tok::GHOST_OPACITY)
            .end();
        let ch = chars.get(i).copied().unwrap_or(' ');
        if ch != ' ' && lcd::has_glyph(ch) {
            w.tag("use")
                .attr("href", format!("#{}", lcd_symbol_id(ch)))
                .attr("x", gx)
                .attr("y", gy)
                .attr("width", gw)
                .attr("height", gh)
                .attr("opacity", tok::PIXEL_OPACITY)
                .end();
        }
    }
    w.close();
    w.tag("rect")
        .attr("x", x)
        .attr("y", y)
        .attr("width", width)
        .attr("height", height / 2.0)
        .attr("rx", 1.5)
        .attr("fill", "#fff")
        .attr("opacity", 0.07)
        .end();
    (width, height)
}
