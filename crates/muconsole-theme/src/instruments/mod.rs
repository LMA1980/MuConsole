// SPDX-License-Identifier: AGPL-3.0-only
//! Pure generators for the instrument primitives.
//!
//! Every function appends SVG to a [`SvgWriter`] and depends only on its
//! arguments, so output is deterministic. Instruments reference the shared
//! definitions written by [`crate::defs::write_defs`].
//!
//! | Primitive | Function |
//! |---|---|
//! | Cabinet-scale wood veneer | [`surface::wood`] |
//! | Tungsten plate (brushed + sheen + bevel + holes) | [`surface::plate`] |
//! | Etched / embossed text | [`text::etched`], [`text::embossed`] |
//! | STN LCD window | [`lcd_window::lcd`] |
//! | Pistol-grip control switch | [`switch::pistol_switch`] |
//! | Indicating lamp | [`switch::indicator_lamp`] |
//! | Annunciator | [`annunciator::annunciator`] |
//! | CRT pipeline | [`crt::crt`] |
//! | Concentric rotary | [`rotary::rotary`] |

pub mod annunciator;
pub mod crt;
pub mod lcd_window;
pub mod rotary;
pub mod surface;
pub mod switch;
pub mod text;

/// Axis-aligned rectangle in CSS px.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

impl Rect {
    #[must_use]
    pub const fn new(x: f64, y: f64, w: f64, h: f64) -> Self {
        Self { x, y, w, h }
    }

    #[must_use]
    pub fn right(&self) -> f64 {
        self.x + self.w
    }

    #[must_use]
    pub fn bottom(&self) -> f64 {
        self.y + self.h
    }

    /// Shrink by the given insets.
    #[must_use]
    pub fn inset(&self, left: f64, top: f64, right: f64, bottom: f64) -> Self {
        Self::new(
            self.x + left,
            self.y + top,
            (self.w - left - right).max(0.0),
            (self.h - top - bottom).max(0.0),
        )
    }
}

/// Point on a circle; `deg` is measured clockwise from +x (SVG convention).
#[must_use]
pub(crate) fn polar(cx: f64, cy: f64, r: f64, deg: f64) -> (f64, f64) {
    let rad = deg.to_radians();
    (cx + r * rad.cos(), cy + r * rad.sin())
}

#[cfg(test)]
mod tests;
