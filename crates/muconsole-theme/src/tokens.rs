// SPDX-License-Identifier: AGPL-3.0-only
//! Material and colour tokens (design spec section 3.1).
//!
//! Colours are `#rrggbb` strings so they can be written straight into SVG
//! attributes and CSS custom properties.

/// Hardwood veneer (wenge), cabinet scale.
pub mod wood {
    pub const BASE: &str = "#120b07";
    pub const GRAIN: &str = "#402616";
    pub const FIGURE: &str = "#331f12";
    pub const PORES: &str = "#050302";
    /// Seam shadow and lip between book-matched leaves.
    pub const SEAM_DARK: &str = "#070403";
    pub const SEAM_LIP: &str = "#3a2618";
    /// Book-matched veneer leaf width in CSS px (about 230 mm at cabinet scale).
    /// Fixed in CSS px: grain density never scales with the viewport.
    pub const LEAF_PX: f64 = 384.0;
    /// Physical scale: millimetres per CSS px.
    pub const MM_PER_PX: f64 = 0.6;
    /// Fine long grain `feTurbulence` base frequency (x, y).
    pub const GRAIN_FREQ: (f64, f64) = (0.0011, 0.46);
    /// Broad figure layer base frequency (x, y).
    pub const FIGURE_FREQ: (f64, f64) = (0.0007, 0.018);
}

/// Dark tungsten plate with fine satin brushing.
pub mod tungsten {
    pub const TOP: &str = "#2b2d31";
    pub const BOTTOM: &str = "#17181b";
    pub const BEVEL_LIGHT: &str = "#5a5e65";
    pub const BEVEL_DARK: &str = "#0c0d0f";
    /// Long hairlines `feTurbulence` base frequency (x, y), 3 octaves.
    pub const HAIR_FREQ: (f64, f64) = (0.004, 2.4);
    /// Hairline alpha amplitude.
    pub const HAIR_AMPLITUDE: f64 = 0.22;
    /// Dark counter-lines base frequency (x, y).
    pub const COUNTER_FREQ: (f64, f64) = (0.006, 1.6);
    /// Peak white opacity of the anisotropic vertical-band sheen (<= 7 %).
    pub const SHEEN_PEAK: f64 = 0.07;
    pub const CORNER_RADIUS: f64 = 4.0;
}

/// Acid-etched (recessed) text, HeavyData.
pub mod etch {
    pub const FACE: &str = "#9499a1";
    pub const RECESS: &str = "#0b0c0e";
    pub const LIP: &str = "#545961";
}

/// Embossed (raised) text, HeavyData.
pub mod emboss {
    pub const FACE: &str = "#6a6f77";
    pub const LIP: &str = "#b4b9c1";
    pub const SHADOW: &str = "#050506";
}

/// Punched-through holes, vent slots and chevrons.
pub mod punch {
    pub const HOLE: &str = "#050505";
    pub const LOWER_LIP: &str = "#5c6067";
}

/// CRT phosphor palette.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Phosphor {
    pub name: &'static str,
    pub lit: &'static str,
    pub dim: &'static str,
    /// Glass gradient: centre, edge.
    pub glass_center: &'static str,
    pub glass_edge: &'static str,
    /// Text colour on a lit (reverse-video) bar.
    pub reverse_ink: &'static str,
}

/// CI screens: green P1 phosphor.
pub const CRT_CI: Phosphor = Phosphor {
    name: "ci",
    lit: "#4dff7a",
    dim: "#1f6b34",
    glass_center: "#0b1a0f",
    glass_edge: "#020604",
    reverse_ink: "#04140a",
};

/// CD screens: amber P3 phosphor.
pub const CRT_CD: Phosphor = Phosphor {
    name: "cd",
    lit: "#ffb43a",
    dim: "#6b4a12",
    glass_center: "#1c1306",
    glass_edge: "#060402",
    reverse_ink: "#1a1004",
};

/// CRT pipeline constants.
pub mod crt {
    /// Phosphor bloom `feGaussianBlur` standard deviation.
    pub const BLOOM_STD_DEV: f64 = 1.6;
    /// Scanline pitch in px.
    pub const SCANLINE_PITCH: f64 = 3.0;
    pub const SCANLINE_HEIGHT: f64 = 1.2;
    pub const SCANLINE_OPACITY: f64 = 0.45;
    pub const BEZEL: &str = "#0a0b0c";
    pub const BEZEL_EDGE: &str = "#3b3e43";
    pub const CORNER_RADIUS: f64 = 16.0;
}

/// Yellow-green STN LCD.
pub mod lcd {
    pub const BACKLIGHT_TOP: &str = "#b6cc4a";
    pub const BACKLIGHT_BOTTOM: &str = "#8ea42a";
    pub const PIXEL: &str = "#1c2608";
    pub const PIXEL_OPACITY: f64 = 0.88;
    pub const GHOST_OPACITY: f64 = 0.07;
    pub const BEZEL: &str = "#08090a";
    /// Dot size as a fraction of the dot pitch.
    pub const DOT_FILL: f64 = 0.82;
}

/// Fasteners, steel and parkerized hardware shared by instruments.
pub mod hardware {
    pub const STEEL_LIGHT: &str = "#c3c7cc";
    pub const STEEL_DARK: &str = "#4b4f55";
    pub const PARKERIZED_TOP: &str = "#3a3c40";
    pub const PARKERIZED_BOTTOM: &str = "#161719";
    pub const RUBBER: &str = "#1a1a1a";
    pub const ENGRAVED_INDEX: &str = "#e8e6dc";
}

/// Nuclear-control-room pistol-grip rotary control switch (GE SBM /
/// Westinghouse W-2 family).
pub mod switch {
    pub const BAKELITE_HIGHLIGHT: &str = "#4a4a4d";
    pub const BAKELITE_SHADOW: &str = "#050506";
    pub const ESCUTCHEON_TOP: &str = "#1d1e20";
    pub const ESCUTCHEON_BOTTOM: &str = "#0c0d0e";
    pub const ESCUTCHEON_EDGE: &str = "#45484d";
    /// Engraved legend / tick white.
    pub const LEGEND: &str = "#e6e3d8";
    /// Pointer inlay on the grip.
    pub const POINTER: &str = "#f2efe6";
    /// Red PULL-TO-TURN collar.
    pub const PULL_COLLAR: &str = "#b3121c";
    /// Press-and-hold time before a PULL-TO-TURN switch may rotate (ms).
    pub const PULL_HOLD_MS: u32 = 300;
    /// Reference escutcheon size in px (all geometry scales from this).
    pub const REFERENCE_SIZE: f64 = 64.0;
}

/// Indicating lamp lenses (control-board convention: green = ready /
/// normal / selected, red = running / energized, amber = caution).
pub mod indicator {
    pub const RED: &str = "#ff3b30";
    pub const GREEN: &str = "#2fe060";
    pub const AMBER: &str = "#ffb43a";
    /// Dark tint over an unlit lens.
    pub const RED_UNLIT: &str = "#5a1410";
    pub const GREEN_UNLIT: &str = "#123d1d";
    pub const AMBER_UNLIT: &str = "#4a3008";
}

/// Annunciator / indicator lamp colours.
pub mod lamp {
    pub const OK: &str = "#3ddc6a";
    pub const CAUTION: &str = "#ffb43a";
    pub const WARNING: &str = "#ff3b3b";
    pub const OFF: &str = "#1d1f22";
    pub const OFF_LEGEND: &str = "#4d5157";
    pub const LIT_LEGEND: &str = "#101010";
}

/// Chrome and shell layout colours (header, status indicators, border accents).
pub mod chrome {
    pub const DARK_SLATE: &str = "#0e1116";
    pub const NEON_CYAN: &str = "#00f0ff";
    pub const GREEN_GLOW: &str = "#4dff7a";
}

/// Annunciator state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Lamp {
    Off,
    Ok,
    Caution,
    Warning,
}

impl Lamp {
    /// Lens colour, or `None` when unlit.
    #[must_use]
    pub const fn color(self) -> Option<&'static str> {
        match self {
            Self::Off => None,
            Self::Ok => Some(lamp::OK),
            Self::Caution => Some(lamp::CAUTION),
            Self::Warning => Some(lamp::WARNING),
        }
    }
}

/// Theme effects intensity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Intensity {
    /// Bloom, scanlines, blink, faint flicker, track-building animation.
    Full,
    /// Static rendering: no bloom, no scanlines.
    Lite,
    /// Lite under reduced motion, low power or slow frames; Full otherwise.
    #[default]
    Auto,
}

/// Frame time above which `Auto` falls back to `Lite` (ms).
pub const AUTO_LITE_FRAME_MS: f64 = 12.0;

/// Effects actually rendered after resolving [`Intensity`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Effects {
    pub bloom: bool,
    pub scanlines: bool,
    pub animation: bool,
}

impl Effects {
    pub const FULL: Self = Self {
        bloom: true,
        scanlines: true,
        animation: true,
    };
    pub const LITE: Self = Self {
        bloom: false,
        scanlines: false,
        animation: false,
    };
}

/// Runtime signals considered by [`Intensity::Auto`].
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct EnvSignals {
    pub prefers_reduced_motion: bool,
    pub low_power: bool,
    /// Recent frame time in ms (rolling average).
    pub frame_ms: f64,
}

impl Intensity {
    /// Resolve the setting against the environment.
    /// `prefers-reduced-motion` always forces Lite.
    #[must_use]
    pub fn resolve(self, env: EnvSignals) -> Effects {
        if env.prefers_reduced_motion {
            return Effects::LITE;
        }
        match self {
            Self::Full => Effects::FULL,
            Self::Lite => Effects::LITE,
            Self::Auto => {
                if env.low_power || env.frame_ms > AUTO_LITE_FRAME_MS {
                    Effects::LITE
                } else {
                    Effects::FULL
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn is_hex(c: &str) -> bool {
        c.len() == 7 && c.starts_with('#') && c[1..].chars().all(|ch| ch.is_ascii_hexdigit())
    }

    #[test]
    fn colours_are_hex() {
        for c in [
            wood::BASE,
            wood::GRAIN,
            wood::FIGURE,
            wood::PORES,
            tungsten::TOP,
            tungsten::BOTTOM,
            tungsten::BEVEL_LIGHT,
            tungsten::BEVEL_DARK,
            etch::FACE,
            etch::RECESS,
            etch::LIP,
            emboss::FACE,
            emboss::LIP,
            emboss::SHADOW,
            punch::HOLE,
            punch::LOWER_LIP,
            CRT_CI.lit,
            CRT_CI.dim,
            CRT_CD.lit,
            CRT_CD.dim,
            lcd::BACKLIGHT_TOP,
            lcd::BACKLIGHT_BOTTOM,
            lcd::PIXEL,
            lamp::OK,
            lamp::CAUTION,
            lamp::WARNING,
            lamp::OFF,
            chrome::DARK_SLATE,
            chrome::NEON_CYAN,
            chrome::GREEN_GLOW,
        ] {
            assert!(is_hex(c), "{c}");
        }
    }

    #[test]
    fn spec_values() {
        assert_eq!(CRT_CI.lit, "#4dff7a");
        assert_eq!(CRT_CD.lit, "#ffb43a");
        const { assert!(tungsten::SHEEN_PEAK <= 0.07) };
        // ~0.9 m wide desk at 1512 px.
        let desk_m = 1512.0 * wood::MM_PER_PX / 1000.0;
        assert!((0.85..0.95).contains(&desk_m));
        assert_eq!(Lamp::Off.color(), None);
        assert_eq!(Lamp::Warning.color(), Some("#ff3b3b"));
    }

    #[test]
    fn intensity_resolution() {
        let calm = EnvSignals {
            frame_ms: 6.0,
            ..EnvSignals::default()
        };
        assert_eq!(Intensity::Full.resolve(calm), Effects::FULL);
        assert_eq!(Intensity::Lite.resolve(calm), Effects::LITE);
        assert_eq!(Intensity::Auto.resolve(calm), Effects::FULL);
        let slow = EnvSignals {
            frame_ms: 14.0,
            ..calm
        };
        assert_eq!(Intensity::Auto.resolve(slow), Effects::LITE);
        assert_eq!(Intensity::Full.resolve(slow), Effects::FULL);
        let low = EnvSignals {
            low_power: true,
            ..calm
        };
        assert_eq!(Intensity::Auto.resolve(low), Effects::LITE);
        let rm = EnvSignals {
            prefers_reduced_motion: true,
            ..calm
        };
        assert_eq!(Intensity::Full.resolve(rm), Effects::LITE);
    }
}
