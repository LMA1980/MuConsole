// SPDX-License-Identifier: LGPL-3.0-only WITH LGPL-3.0-linking-exception
//! Message and view-model skeleton shared by the UI and the data worker.
//!
//! Phase 1 only fixes the shape of the envelope and the commands the layout
//! and theme already depend on (`SelectRepo`, `SetCrtFont`). View models grow
//! in Phase 2 together with the data worker.

use alloc::string::String;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

/// Top-level console area (one rail position each).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Area {
    Overview,
    Ci,
    Cd,
    Fleet,
}

/// `owner/repo` pair identifying a repository.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RepoId {
    pub owner: String,
    pub repo: String,
}

/// Submodule of a superproject, identified by its path in `.gitmodules`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
    Retry,
    /// Tab hidden: the worker throttles to heartbeat-only.
    SetVisibility {
        hidden: bool,
    },
    SetCrtFont(CrtFont),
}

/// Overall CI / CD state of a repository.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RepoStatus {
    Unknown,
    Passing,
    Running,
    Failing,
}

/// Scorecard summary (CI half; CD fields join in Phase 2).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepoScore {
    pub id: RepoId,
    pub status: RepoStatus,
    /// Quality score in percent (0..=100).
    pub score: u32,
    /// Line coverage in basis points (0..=10_000).
    pub coverage_bp: u32,
}

/// Full view model snapshot.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ViewModel {
    /// Topology change counter shown as `FABRIC REV n`.
    pub fabric_rev: u64,
    pub repos: Vec<RepoScore>,
}

/// Incremental change to the view model.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Delta {
    FabricRev(u64),
    RepoUpsert(RepoScore),
    RepoRemoved(RepoId),
}

/// Connection state of the worker's upstream streams.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConnState {
    Connecting,
    Live,
    Reconnecting { attempt: u32 },
    Offline,
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
}

/// Data worker -> UI.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataEvent {
    Snapshot(ViewModel),
    Patch(Vec<Delta>),
    Connection(ConnState),
    Error(UiError),
}
