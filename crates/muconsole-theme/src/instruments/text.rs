// SPDX-License-Identifier: AGPL-3.0-only
//! Etched (recessed) and embossed (raised) HeavyData plate text.

use crate::metrics::{heavy_data, ETCH_FAMILY_ALIAS};
use crate::svg::SvgWriter;
use crate::tokens::{emboss, etch};

/// `text-anchor` value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Anchor {
    #[default]
    Start,
    Middle,
    End,
}

impl Anchor {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::Middle => "middle",
            Self::End => "end",
        }
    }
}

/// Plate text style.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextStyle {
    /// Nominal size in px (rendered at `max(11, size * 1.05)`).
    pub size: f64,
    pub anchor: Anchor,
    pub letter_spacing: f64,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            size: 11.0,
            anchor: Anchor::Start,
            letter_spacing: 1.0,
        }
    }
}

impl TextStyle {
    #[must_use]
    pub fn sized(size: f64) -> Self {
        Self {
            size,
            ..Self::default()
        }
    }

    #[must_use]
    pub fn anchor(mut self, anchor: Anchor) -> Self {
        self.anchor = anchor;
        self
    }

    #[must_use]
    pub fn spacing(mut self, letter_spacing: f64) -> Self {
        self.letter_spacing = letter_spacing;
        self
    }

    /// Rendered size, never below the HeavyData legibility floor.
    #[must_use]
    pub fn rendered_size(&self) -> f64 {
        heavy_data::MIN_SIZE_PX.max(self.size * 1.05)
    }
}

fn pass(w: &mut SvgWriter, x: f64, y: f64, txt: &str, style: &TextStyle, fill: &str) {
    w.tag("text")
        .attr("x", x)
        .attr("y", y)
        .attr("font-family", ETCH_FAMILY_ALIAS)
        .attr("font-size", style.rendered_size())
        .attr("letter-spacing", style.letter_spacing)
        .attr("text-anchor", style.anchor.as_str())
        .attr("fill", fill)
        .text(txt);
}

/// Acid-etched recess: dark shadow top-left, light lip bottom-right,
/// frosted face. Used for small static captions.
pub fn etched(w: &mut SvgWriter, x: f64, y: f64, txt: &str, style: TextStyle) {
    pass(w, x - 0.7, y - 0.7, txt, &style, etch::RECESS);
    pass(w, x + 0.6, y + 0.6, txt, &style, etch::LIP);
    pass(w, x, y, txt, &style, etch::FACE);
}

/// Embossed (raised): light lip top-left, deep shadow bottom-right.
/// Used for panel titles and module names.
pub fn embossed(w: &mut SvgWriter, x: f64, y: f64, txt: &str, style: TextStyle) {
    pass(w, x + 1.3, y + 1.5, txt, &style, emboss::SHADOW);
    pass(w, x - 0.7, y - 0.7, txt, &style, emboss::LIP);
    pass(w, x, y, txt, &style, emboss::FACE);
}
