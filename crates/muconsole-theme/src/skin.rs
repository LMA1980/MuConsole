// SPDX-License-Identifier: AGPL-3.0-only
//! Semantic instrument models and the [`Skin`] rendering trait.
//!
//! Every Phase 3 UI component (`C1`–`C6`) emits semantic models (`Panel`,
//! `Selector`, `Rotary`, `Lamp`, `Readout`, `Screen`) through [`Skin`] so
//! that UX layout, routing, keyboard/pointer interaction, font switching and
//! fixture scenarios are verified independently of surface ornamentation.

use crate::instruments::switch::SwitchAction;
use crate::instruments::Rect;
use crate::svg::SvgWriter;

/// Semantic status tone shared across skins.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SemanticTone {
    #[default]
    Neutral,
    Pass,
    Warn,
    Fail,
    Info,
}

/// Geometric block / checkpoint signal shape (never colour-alone).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SignalGlyph {
    /// `■` CI stage passed.
    BlockPassed,
    /// `▢` CI stage running.
    BlockRunning,
    /// `✕` CI stage or CD test failed.
    BlockFailed,
    /// `▫` CI stage pending / skipped.
    BlockPending,
    /// `●` CD integration test passed.
    TestPassed,
    /// `◯` CD integration test running.
    TestRunning,
    /// `○` CD integration test pending.
    TestPending,
}

impl SignalGlyph {
    /// Unicode geometric symbol used in text/wireframe readouts.
    #[must_use]
    pub const fn symbol(self) -> &'static str {
        match self {
            Self::BlockPassed => "■",
            Self::BlockRunning => "▢",
            Self::BlockFailed => "✕",
            Self::BlockPending => "▫",
            Self::TestPassed => "●",
            Self::TestRunning => "◯",
            Self::TestPending => "○",
        }
    }
}

/// Structural panel container.
#[derive(Debug, Clone, Copy)]
pub struct Panel<'a> {
    pub id: &'a str,
    pub title: &'a str,
    pub rect: Rect,
    pub role: Option<&'a str>,
    pub aria_label: Option<&'a str>,
}

/// One position of a [`Selector`].
#[derive(Debug, Clone, Copy)]
pub struct SelectorOption<'a> {
    pub id: &'a str,
    pub label: &'a str,
    pub badge: Option<u32>,
    pub tone: SemanticTone,
}

/// Multi-position selector (`Maintained`, `SpringReturn`, or `PullToTurn`).
#[derive(Debug, Clone, Copy)]
pub struct Selector<'a> {
    pub control_id: &'a str,
    pub label: &'a str,
    pub rect: Rect,
    pub action: SwitchAction,
    pub options: &'a [SelectorOption<'a>],
    pub selected: usize,
    pub pulled: bool,
    pub horizontal: bool,
}

/// Dual-ring repository + submodule selector.
#[derive(Debug, Clone, Copy)]
pub struct Rotary<'a> {
    pub control_id: &'a str,
    pub rect: Rect,
    pub outer_label: &'a str,
    pub outer_index: usize,
    pub outer_total: usize,
    pub inner_label: &'a str,
    pub inner_index: usize,
    pub inner_total: usize,
    pub filter_query: &'a str,
    pub settling: bool,
}

/// Status lamp / annunciator pill.
#[derive(Debug, Clone, Copy)]
pub struct Lamp<'a> {
    pub control_id: &'a str,
    pub label: &'a str,
    pub rect: Rect,
    pub lit: bool,
    pub tone: SemanticTone,
}

/// Dynamic label / value readout window.
#[derive(Debug, Clone, Copy)]
pub struct Readout<'a> {
    pub control_id: &'a str,
    pub label: &'a str,
    pub value: &'a str,
    pub rect: Rect,
    pub tone: SemanticTone,
    pub interactive: bool,
}

/// One text/checkpoint row inside a [`Screen`].
#[derive(Debug, Clone)]
pub struct ScreenRow<'a> {
    pub text: &'a str,
    pub tone: SemanticTone,
    pub signal: Option<SignalGlyph>,
    pub critical: bool,
    pub control_id: Option<&'a str>,
    pub selected: bool,
}

/// One railway track (main line or siding) inside a [`Screen`].
#[derive(Debug, Clone)]
pub struct ScreenTrack<'a> {
    pub ref_name: &'a str,
    pub lane: u32,
    pub fork_stage: u32,
    pub dashed: bool,
    pub fading: bool,
    pub signals: &'a [(u32, SignalGlyph, SemanticTone)],
    pub node_tag: Option<&'a str>,
    pub node_lost: bool,
}

/// Monospaced screen region (scorecard half-screen, Trainway, Delivery gate, or Fleet).
#[derive(Debug, Clone)]
pub struct Screen<'a> {
    pub control_id: &'a str,
    pub title: &'a str,
    pub rect: Rect,
    pub font_family: &'a str,
    pub font_px: f64,
    pub cols: usize,
    pub rows: usize,
    pub lines: &'a [ScreenRow<'a>],
    pub stations: &'a [&'a str],
    pub tracks: &'a [ScreenTrack<'a>],
    pub page: u32,
    pub total_pages: u32,
}

/// Pluggable visual skin for the console.
pub trait Skin {
    /// Identifier of this skin (`"wireframe"` or `"avionics"`).
    fn name(&self) -> &'static str;

    /// Initialize canvas background and any shared definitions.
    fn begin_frame(&self, w: &mut SvgWriter, width: f64, height: f64);

    /// Render a structural panel container.
    fn panel(&self, w: &mut SvgWriter, p: &Panel<'_>);

    /// Render a multi-position selector (`Maintained`, `SpringReturn`, `PullToTurn`).
    fn selector(&self, w: &mut SvgWriter, s: &Selector<'_>);

    /// Render the two-level repository / submodule rotary selector.
    fn rotary(&self, w: &mut SvgWriter, r: &Rotary<'_>);

    /// Render a status lamp or annunciator pill.
    fn lamp(&self, w: &mut SvgWriter, l: &Lamp<'_>);

    /// Render a dynamic readout window.
    fn readout(&self, w: &mut SvgWriter, r: &Readout<'_>);

    /// Render a monospaced screen with optional railway tracks and pager.
    fn screen(&self, w: &mut SvgWriter, s: &Screen<'_>);
}
