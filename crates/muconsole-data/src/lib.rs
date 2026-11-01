// SPDX-License-Identifier: AGPL-3.0-only
//! MuConsole data worker (Phase 1 stub).
//!
//! Phase 2 adds the dedicated Web Worker bootstrap, the `fetch` +
//! `ReadableStream` SSE parser, the normalized store, diffing and ≤ 10 Hz
//! coalescing. For now the crate only owns the command handling contract so
//! the protocol is exercised from both sides and the crate builds for
//! `wasm32-unknown-unknown`.
#![forbid(unsafe_code)]

use muconsole_protocol::{decode, encode, CodecError, ConnState, DataEvent, UiCommand, UiError};

/// Upper bound on UI patch rate (Hz) for the coalescer.
pub const MAX_PATCH_HZ: u32 = 10;

/// Minimal worker state.
#[derive(Debug, Default)]
pub struct Worker {
    hidden: bool,
}

impl Worker {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Whether the tab is hidden (heartbeat-only mode).
    #[must_use]
    pub fn is_hidden(&self) -> bool {
        self.hidden
    }

    /// Handle one command and return the events to post back.
    pub fn handle(&mut self, cmd: &UiCommand) -> Vec<DataEvent> {
        match cmd {
            UiCommand::SetVisibility { hidden } => {
                self.hidden = *hidden;
                Vec::new()
            }
            UiCommand::Retry | UiCommand::Subscribe(_) | UiCommand::SelectRepo { .. } => {
                vec![DataEvent::Connection(ConnState::Connecting)]
            }
            UiCommand::Page { .. } | UiCommand::SetCrtFont(_) => Vec::new(),
        }
    }

    /// Decode a frame, handle it and encode the replies. Version mismatches
    /// are reported to the UI as [`UiError::VersionMismatch`].
    ///
    /// # Errors
    /// Returns a [`CodecError`] only if a reply cannot be encoded.
    pub fn handle_frame(&mut self, frame: &[u8]) -> Result<Vec<Vec<u8>>, CodecError> {
        let events = match decode::<UiCommand>(frame) {
            Ok(cmd) => self.handle(&cmd),
            Err(CodecError::VersionMismatch { expected, found }) => {
                vec![DataEvent::Error(UiError::VersionMismatch {
                    expected,
                    found,
                })]
            }
            Err(e) => vec![DataEvent::Error(UiError::Decode(e.to_string()))],
        };
        events.iter().map(encode).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use muconsole_protocol::PROTOCOL_VERSION;

    #[test]
    fn visibility_and_retry() {
        let mut w = Worker::new();
        assert!(w
            .handle(&UiCommand::SetVisibility { hidden: true })
            .is_empty());
        assert!(w.is_hidden());
        assert_eq!(
            w.handle(&UiCommand::Retry),
            vec![DataEvent::Connection(ConnState::Connecting)]
        );
    }

    #[test]
    fn frames_round_trip_and_mismatch_is_reported() {
        let mut w = Worker::new();
        let replies = w
            .handle_frame(&encode(&UiCommand::Retry).expect("encode"))
            .expect("ok");
        assert_eq!(replies.len(), 1);
        let ev: DataEvent = decode(&replies[0]).expect("decode");
        assert_eq!(ev, DataEvent::Connection(ConnState::Connecting));

        let bad = [u8::try_from(PROTOCOL_VERSION + 1).expect("small"), 0];
        let replies = w.handle_frame(&bad).expect("ok");
        let ev: DataEvent = decode(&replies[0]).expect("decode");
        assert!(matches!(
            ev,
            DataEvent::Error(UiError::VersionMismatch { found, .. }) if found == PROTOCOL_VERSION + 1
        ));
        let replies = w.handle_frame(&[]).expect("ok");
        let ev: DataEvent = decode(&replies[0]).expect("decode");
        assert!(matches!(ev, DataEvent::Error(UiError::Decode(_))));
    }
}
