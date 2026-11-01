// SPDX-License-Identifier: AGPL-3.0-only
//! Pure interaction model for the pistol-grip control switches.
//!
//! - **Maintained** selectors move to any position and stay there.
//! - **Spring-return** switches fire when turned off the rest position and
//!   spring back to it on release, so they can never be left in an action
//!   position.
//! - **PULL-TO-TURN** switches need two separate motions: pull (press and
//!   hold for at least [`PULL_HOLD_MS`]) and then rotate, or `Shift`+key on
//!   the keyboard. They spring back to rest on release.

use crate::instruments::switch::SwitchAction;
use crate::tokens::switch::PULL_HOLD_MS;

/// Input event. Timestamps are monotonic milliseconds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwitchEvent {
    /// Pointer down on the grip (starts a pull).
    Press { at_ms: u64 },
    /// Turn to position `to` (drag snapped to a detent, wheel or key).
    /// `shift` is the keyboard equivalent of a completed pull.
    Rotate { to: usize, at_ms: u64, shift: bool },
    /// Pointer up / key up.
    Release { at_ms: u64 },
}

/// Result of an event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwitchOutcome {
    /// Nothing changed.
    Idle,
    /// The grip moved to a position (maintained selection or spring-back).
    Moved(usize),
    /// A momentary action fired at this position.
    Fired(usize),
    /// The rotation was refused (PULL-TO-TURN without a pull, or an
    /// out-of-range position).
    Refused,
}

/// Switch state machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SwitchState {
    action: SwitchAction,
    positions: usize,
    rest: usize,
    current: usize,
    pressed_at: Option<u64>,
}

impl SwitchState {
    /// New switch with `positions` detents resting at `rest`.
    #[must_use]
    pub fn new(action: SwitchAction, positions: usize, rest: usize) -> Self {
        let rest = rest.min(positions.saturating_sub(1));
        Self {
            action,
            positions,
            rest,
            current: rest,
            pressed_at: None,
        }
    }

    /// Current grip position.
    #[must_use]
    pub fn current(&self) -> usize {
        self.current
    }

    /// Whether a pull has been held long enough at `now_ms`.
    #[must_use]
    pub fn is_pulled(&self, now_ms: u64) -> bool {
        self.pressed_at
            .is_some_and(|t| now_ms.saturating_sub(t) >= u64::from(PULL_HOLD_MS))
    }

    /// Apply an event.
    pub fn apply(&mut self, ev: SwitchEvent) -> SwitchOutcome {
        match ev {
            SwitchEvent::Press { at_ms } => {
                self.pressed_at = Some(at_ms);
                SwitchOutcome::Idle
            }
            SwitchEvent::Rotate { to, at_ms, shift } => self.rotate(to, at_ms, shift),
            SwitchEvent::Release { .. } => {
                self.pressed_at = None;
                if self.action.springs_back() && self.current != self.rest {
                    self.current = self.rest;
                    SwitchOutcome::Moved(self.rest)
                } else {
                    SwitchOutcome::Idle
                }
            }
        }
    }

    fn rotate(&mut self, to: usize, at_ms: u64, shift: bool) -> SwitchOutcome {
        if to >= self.positions {
            return SwitchOutcome::Refused;
        }
        if self.action.needs_pull() && !shift && !self.is_pulled(at_ms) {
            return SwitchOutcome::Refused;
        }
        if to == self.current {
            return SwitchOutcome::Idle;
        }
        self.current = to;
        match self.action {
            SwitchAction::Maintained => SwitchOutcome::Moved(to),
            SwitchAction::SpringReturn | SwitchAction::PullToTurn => {
                if to == self.rest {
                    SwitchOutcome::Moved(to)
                } else {
                    SwitchOutcome::Fired(to)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rot(to: usize, at_ms: u64) -> SwitchEvent {
        SwitchEvent::Rotate {
            to,
            at_ms,
            shift: false,
        }
    }

    #[test]
    fn maintained_selector_stays() {
        let mut s = SwitchState::new(SwitchAction::Maintained, 4, 1);
        assert_eq!(s.apply(rot(3, 0)), SwitchOutcome::Moved(3));
        assert_eq!(
            s.apply(SwitchEvent::Release { at_ms: 5 }),
            SwitchOutcome::Idle
        );
        assert_eq!(s.current(), 3);
        assert_eq!(s.apply(rot(3, 6)), SwitchOutcome::Idle);
        assert_eq!(s.apply(rot(4, 7)), SwitchOutcome::Refused);
    }

    #[test]
    fn spring_return_fires_and_springs_back() {
        // PREV · N · NEXT resting at N.
        let mut s = SwitchState::new(SwitchAction::SpringReturn, 3, 1);
        assert_eq!(s.apply(rot(2, 0)), SwitchOutcome::Fired(2));
        assert_eq!(
            s.apply(SwitchEvent::Release { at_ms: 10 }),
            SwitchOutcome::Moved(1)
        );
        assert_eq!(s.current(), 1);
        assert_eq!(s.apply(rot(0, 20)), SwitchOutcome::Fired(0));
    }

    #[test]
    fn pull_to_turn_needs_hold_or_shift() {
        // NORM · RST resting at NORM.
        let mut s = SwitchState::new(SwitchAction::PullToTurn, 2, 0);
        assert_eq!(s.apply(rot(1, 0)), SwitchOutcome::Refused, "no pull");
        s.apply(SwitchEvent::Press { at_ms: 100 });
        assert_eq!(s.apply(rot(1, 250)), SwitchOutcome::Refused, "held 150 ms");
        assert!(!s.is_pulled(399));
        assert_eq!(s.apply(rot(1, 400)), SwitchOutcome::Fired(1), "held 300 ms");
        assert_eq!(
            s.apply(SwitchEvent::Release { at_ms: 500 }),
            SwitchOutcome::Moved(0)
        );
        assert_eq!(
            s.apply(rot(1, 600)),
            SwitchOutcome::Refused,
            "pull resets on release"
        );
        let shift = SwitchEvent::Rotate {
            to: 1,
            at_ms: 700,
            shift: true,
        };
        assert_eq!(s.apply(shift), SwitchOutcome::Fired(1), "Shift+key");
    }

    #[test]
    fn rest_is_clamped() {
        let s = SwitchState::new(SwitchAction::Maintained, 2, 9);
        assert_eq!(s.current(), 1);
        let empty = SwitchState::new(SwitchAction::Maintained, 0, 0);
        assert_eq!(empty.current(), 0);
    }
}
