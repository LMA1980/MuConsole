// SPDX-License-Identifier: AGPL-3.0-only
//! "Avionics Terminal" theme for MuConsole (dark only).
//!
//! Rule of thumb: **etched means static, LCD means a dynamic label, CRT means
//! a live view, and a lamp or annunciator means state.**
//!
//! - [`tokens`]: material and colour tokens, intensity setting.
//! - [`lcd`]: built-in 5×7 dot-matrix glyph table for STN LCD windows.
//! - [`glyphs`]: Nerd Font icon table; also drives the font subset list.
//! - [`metrics`]: measured font metrics used by the layout solver.
//! - [`svg`]: dependency-free, deterministic SVG writer.
//! - [`defs`]: shared `<defs>` (filters, gradients, patterns, LCD symbols).
//! - [`instruments`]: pure generators for the instrument primitives.
//! - [`interaction`]: switch state machine (maintained, spring-return,
//!   PULL-TO-TURN).
//! - [`css`]: generated `tokens.css`.
#![forbid(unsafe_code)]

pub mod css;
pub mod defs;
pub mod glyphs;
pub mod instruments;
pub mod interaction;
pub mod lcd;
pub mod metrics;
pub mod svg;
pub mod tokens;
