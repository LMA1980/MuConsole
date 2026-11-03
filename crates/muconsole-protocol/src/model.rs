// SPDX-License-Identifier: LGPL-3.0-only WITH LGPL-3.0-linking-exception
//! Messages exchanged by the UI module and the data worker.
//!
//! Identifiers and routes live here; the skin-agnostic view models carried
//! by [`DataEvent::Snapshot`] / [`DataEvent::Patch`] live in [`crate::view`].

use alloc::string::String;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::config::WorkerConfig;
use crate::dto::AuthSessionDto;
use crate::view::{AreaBadges, DataSource, Delta, ViewModel};

/// Top-level console area (one rail position each).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Area {
    Overview,
    Ci,
    Cd,
    Fleet,
}

/// `owner/repo` pair identifying a repository.
///
/// Ordering is `owner`, then `repo`; view-model repository lists are kept in
/// this order so diffs and patches are deterministic.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct RepoId {
    pub owner: String,
    pub repo: String,
}

impl RepoId {
    /// Build a repository id from its two halves.
    #[must_use]
    pub fn new(owner: &str, repo: &str) -> Self {
        Self {
            owner: owner.into(),
            repo: repo.into(),
        }
    }
}

/// Submodule of a superproject, identified by its path in `.gitmodules`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SubmoduleId {
    pub path: String,
}

/// Hash route: `#/{area}/{owner}/{repo}/{branch}/{job}`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Route {
    pub area: Area,
    pub repo: Option<RepoId>,
    pub submodule: Option<SubmoduleId>,
    pub branch: Option<String>,
    pub job: Option<u64>,
}

/// CRT terminal font family. The layout solver re-measures on change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CrtFont {
    /// Hurmit Nerd Font Mono (default).
    Hurmit,
    /// OpenDyslexicM Nerd Font Mono (accessibility switch).
    OpenDyslexic,
}

/// Panels that support panel-local paging.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PanelId {
    Carousel,
    Trainway,
    Delivery,
    Fleet,
    RepoPicker,
    Log,
}

/// UI -> data worker.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiCommand {
    /// First command after spawning the worker: API base URL and optional
    /// fixture scenario, both parsed from the page query string.
    Configure(WorkerConfig),
    Subscribe(Route),
    /// Applied after the rotary settles (350 ms after the last detent).
    SelectRepo {
        repo: RepoId,
        submodule: Option<SubmoduleId>,
    },
    Page {
        panel: PanelId,
        cursor: u32,
    },
    /// Reconnect immediately (live) or replay from the start (scenario).
    Retry,
    /// Tab hidden: the worker throttles to heartbeat-only.
    SetVisibility {
        hidden: bool,
    },
    SetCrtFont(CrtFont),
    /// Redeem a one-time operator invite token (`POST /api/auth/redeem`).
    RedeemInvite {
        token: String,
    },
    /// Directly set the operator authentication session state.
    SetAuthSession(AuthSessionDto),
    /// Restart / retry a specific CI job (`POST /jobs/{id}/restart`).
    RestartJob {
        job_id: u64,
    },
}

/// Connection state of the worker's upstream streams.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConnState {
    Connecting,
    Live,
    /// A stream dropped; the next attempt starts in `retry_in_ms`.
    Reconnecting {
        attempt: u32,
        retry_in_ms: u32,
    },
    /// No upstream; the view comes from the offline snapshot (if any).
    Offline,
    /// Tab hidden: streams stay open but the UI only gets heartbeats.
    Paused,
    /// A fixture scenario is replaying instead of live data.
    Scenario,
}

/// Errors surfaced to the UI.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiError {
    /// Frame carried a different protocol version: a reload is required.
    VersionMismatch {
        expected: u32,
        found: u32,
    },
    Http {
        status: u32,
        message: String,
    },
    Decode(String),
    /// Upstream SSE stream failure (network error, bad status).
    Stream(String),
    /// `?scenario=` named an unknown fixture; the worker fell back to live.
    UnknownScenario(String),
}

/// Data worker -> UI.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataEvent {
    /// Sent once after [`UiCommand::Configure`].
    Ready {
        protocol: u32,
        source: DataSource,
    },
    Snapshot(ViewModel),
    Patch(Vec<Delta>),
    Connection(ConnState),
    Error(UiError),
    /// Alert counts for the area selector lamps (sent when they change).
    AreaBadges(AreaBadges),
    /// Hidden-tab keep-alive (replaces patches while hidden).
    Heartbeat {
        seq: u32,
    },
}
