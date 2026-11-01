// SPDX-License-Identifier: AGPL-3.0-only
//! Measured font metrics (design spec section 3.2).
//!
//! The layout solver keeps a fixed column grid per CRT and derives the font
//! size from the cell width: `size = column_px / advance_em`.

/// CSS `font-family` alias the UI binds (via `@font-face`) to the selected
/// CRT font. Instruments never name the concrete family, so the font switch
/// only swaps the `@font-face` source.
pub const CRT_FAMILY_ALIAS: &str = "MuCRT";
/// CSS `font-family` alias bound to HeavyData for etched / embossed text.
pub const ETCH_FAMILY_ALIAS: &str = "MuEtch";

/// Metrics of a monospaced CRT font.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CrtFontMetrics {
    /// CSS `font-family` name of the self-hosted subset.
    pub family: &'static str,
    /// Advance width of every glyph, in em.
    pub advance_em: f64,
    /// WOFF2 file stem in `dist/fonts/` (regular weight).
    pub regular_file: &'static str,
    /// Bold file stem, if the family ships a bold weight.
    pub bold_file: Option<&'static str>,
    /// Lazy-loaded only when selected.
    pub lazy: bool,
}

/// Hurmit Nerd Font Mono (the Hermit family renamed by Nerd Fonts because of
/// the OFL Reserved Font Name). Default CRT font.
pub const HURMIT: CrtFontMetrics = CrtFontMetrics {
    family: "Hurmit Nerd Font Mono",
    advance_em: 0.618,
    regular_file: "HurmitNerdFontMono-Regular",
    bold_file: Some("HurmitNerdFontMono-Bold"),
    lazy: false,
};

/// OpenDyslexicM Nerd Font Mono: accessibility switch, about 18 % wider.
pub const OPEN_DYSLEXIC: CrtFontMetrics = CrtFontMetrics {
    family: "OpenDyslexicM Nerd Font Mono",
    advance_em: 0.732,
    regular_file: "OpenDyslexicMNerdFontMono-Regular",
    bold_file: None,
    lazy: true,
};

/// Every CRT font, default first.
pub const CRT_FONTS: [CrtFontMetrics; 2] = [HURMIT, OPEN_DYSLEXIC];

/// HeavyData Nerd Font (etched / embossed plate labels).
pub mod heavy_data {
    pub const FAMILY: &str = "HeavyData Nerd Font";
    pub const FILE: &str = "HeavyDataNerdFont-Regular";
    /// Cap height in em.
    pub const CAP_HEIGHT_EM: f64 = 0.59;
    /// Minimum rendered size in px (cap height ~6.5 px).
    pub const MIN_SIZE_PX: f64 = 11.0;
}

/// Minimum CRT text size on desktop breakpoints (px).
pub const CRT_FLOOR_DESKTOP_PX: f64 = 12.0;
/// Minimum CRT text size on the phone breakpoint (px).
pub const CRT_FLOOR_PHONE_PX: f64 = 13.0;

/// Ratio applied to a design size when switching from Hurmit to `font` so the
/// column grid stays the same (about 0.844 for OpenDyslexicM).
#[must_use]
pub fn grid_scale(font: &CrtFontMetrics) -> f64 {
    HURMIT.advance_em / font.advance_em
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spec_metrics() {
        assert!((HURMIT.advance_em - 0.618).abs() < 1e-9);
        assert!((OPEN_DYSLEXIC.advance_em - 0.732).abs() < 1e-9);
        assert!((heavy_data::CAP_HEIGHT_EM - 0.59).abs() < 1e-9);
        // OpenDyslexicM is ~18 % wider, so the size shrinks ~16 %.
        let wider = OPEN_DYSLEXIC.advance_em / HURMIT.advance_em - 1.0;
        assert!((0.17..0.19).contains(&wider));
        let shrink = 1.0 - grid_scale(&OPEN_DYSLEXIC);
        assert!((0.15..0.165).contains(&shrink));
        assert!((grid_scale(&HURMIT) - 1.0).abs() < 1e-12);
        const { assert!(heavy_data::MIN_SIZE_PX * heavy_data::CAP_HEIGHT_EM > 6.0) };
    }
}
