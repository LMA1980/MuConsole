// SPDX-License-Identifier: LGPL-3.0-only WITH LGPL-3.0-linking-exception
//! JSON DTOs of the read-only REST + SSE API the data worker consumes.
//!
//! This module is the client side of the contract documented in
//! `docs/api-contract.md`. ActionRunner-controller serves `/api/repos*` and
//! the new CI event kinds on `/api/events`; MuConDe-controller serves
//! `/api/cd/*`. All field names are `snake_case`, enums are `snake_case`
//! strings, timestamps are Unix epoch milliseconds and unknown fields are
//! ignored, so servers may add fields without breaking the console.
//!
//! Optional fields default when absent or `null` (`#[serde(default,
//! deserialize_with = "null_default")]`), and string job identifiers from
//! `ActionRunner-controller` (`"job-1793563200-44444444"`) map
//! deterministically to monotonic `u64` ids alongside numeric fixture ids.

use alloc::string::String;
use alloc::vec::Vec;
use serde::{Deserialize, Deserializer, Serialize};

pub use crate::view::{
    GateOutcome, PrState, RepoStatus, RollbackReason, Slot, StageState, TestState, Visibility,
};

fn null_default<'de, D, T>(d: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de> + Default,
{
    Ok(Option::<T>::deserialize(d)?.unwrap_or_default())
}

/// Convert an `ActionRunner-controller` job identifier (e.g.
/// `"job-1793563200-44444444"` or `"1234"`) into a deterministic `u64` that
/// preserves chronological ordering across jobs on the same branch.
#[must_use]
pub fn parse_job_id_str(s: &str) -> u64 {
    if let Ok(n) = s.parse::<u64>() {
        return n;
    }
    if let Some(rest) = s.strip_prefix("job-") {
        let mut parts = rest.split('-');
        let ts = parts
            .next()
            .and_then(|p| p.parse::<u64>().ok())
            .unwrap_or(0);
        let suffix = parts.next().unwrap_or("");
        let hex_prefix: String = suffix
            .chars()
            .filter(char::is_ascii_hexdigit)
            .take(6)
            .collect();
        let low = u64::from_str_radix(&hex_prefix, 16).unwrap_or(0);
        if ts > 0 || low > 0 {
            return (ts << 24) | (low & 0x00FF_FFFF);
        }
    }
    // Fallback deterministic FNV-1a 64-bit hash.
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for b in s.as_bytes() {
        hash ^= u64::from(*b);
        hash = hash.wrapping_mul(0x0100_0000_01b3);
    }
    hash
}

#[derive(Deserialize)]
#[serde(untagged)]
enum RawJobId {
    Num(u64),
    Str(String),
    Brief {
        #[serde(deserialize_with = "de_job_id")]
        job_id: u64,
    },
}

impl RawJobId {
    fn into_u64(self) -> u64 {
        match self {
            Self::Num(n) => n,
            Self::Str(s) => parse_job_id_str(&s),
            Self::Brief { job_id } => job_id,
        }
    }
}

fn de_job_id<'de, D>(d: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(RawJobId::deserialize(d)?.into_u64())
}

fn de_opt_job_id<'de, D>(d: D) -> Result<Option<u64>, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(Option::<RawJobId>::deserialize(d)?.map(RawJobId::into_u64))
}

#[derive(Deserialize)]
#[serde(untagged)]
enum RawStageSignal {
    State(StageState),
    Signal { state: StageState },
}

fn de_stage_states<'de, D>(d: D) -> Result<Vec<StageState>, D::Error>
where
    D: Deserializer<'de>,
{
    let raw = Option::<Vec<RawStageSignal>>::deserialize(d)?.unwrap_or_default();
    Ok(raw
        .into_iter()
        .map(|s| match s {
            RawStageSignal::State(st) | RawStageSignal::Signal { state: st } => st,
        })
        .collect())
}

/// `submodules[]` entry of [`RepoDto`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubmoduleDto {
    pub path: String,
    pub owner: String,
    pub repo: String,
    #[serde(default, deserialize_with = "null_default")]
    pub pinned_sha: String,
}

/// `parents[]` entry of [`RepoDto`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParentDto {
    pub owner: String,
    pub repo: String,
    pub path: String,
}

/// `GET /api/repos` -> `[RepoDto]` or `{ "fabric_rev", "authenticated", "repos": [RepoDto] }`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RepoDto {
    pub owner: String,
    pub repo: String,
    #[serde(default, deserialize_with = "null_default")]
    pub visibility: Visibility,
    #[serde(default, deserialize_with = "null_default")]
    pub default_branch: String,
    #[serde(default, deserialize_with = "de_opt_job_id")]
    pub latest_job_id: Option<u64>,
    /// Quality score, percent 0..=100.
    #[serde(default, deserialize_with = "null_default")]
    pub score: u32,
    /// Line coverage, percent 0.0..=100.0.
    #[serde(default, deserialize_with = "null_default")]
    pub coverage: f64,
    /// Benchmark delta vs. default branch, signed percent.
    #[serde(default, deserialize_with = "null_default")]
    pub bench_delta: f64,
    #[serde(default, deserialize_with = "null_default")]
    pub status: RepoStatus,
    #[serde(default, deserialize_with = "null_default")]
    pub submodules: Vec<SubmoduleDto>,
    #[serde(default, deserialize_with = "null_default")]
    pub parents: Vec<ParentDto>,
}

/// Envelope returned by `ActionRunner-controller` on `GET /api/repos`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RepoListResponseDto {
    #[serde(default)]
    pub fabric_rev: u64,
    #[serde(default)]
    pub authenticated: bool,
    #[serde(default, deserialize_with = "null_default")]
    pub repos: Vec<RepoDto>,
}

/// `history[]` entry of [`RepoSummaryDto`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SummaryHistoryDto {
    #[serde(default, deserialize_with = "de_job_id")]
    pub job_id: u64,
    #[serde(default, deserialize_with = "null_default")]
    pub score: u32,
    #[serde(default, deserialize_with = "null_default")]
    pub coverage: f64,
}

/// `GET /api/repos/{owner}/{repo}/summary`: the CI half of the scorecard.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RepoSummaryDto {
    pub owner: String,
    pub repo: String,
    #[serde(default, deserialize_with = "null_default")]
    pub status: RepoStatus,
    #[serde(default, deserialize_with = "null_default")]
    pub score: u32,
    #[serde(default, deserialize_with = "null_default")]
    pub coverage: f64,
    #[serde(default, deserialize_with = "null_default")]
    pub bench_delta: f64,
    #[serde(default, deserialize_with = "de_opt_job_id")]
    pub latest_job_id: Option<u64>,
    /// Recent quality scores, oldest first (sparkline).
    #[serde(default, deserialize_with = "null_default")]
    pub recent_scores: Vec<u32>,
    /// Structured sparkline history returned by `ActionRunner-controller`.
    #[serde(default, deserialize_with = "null_default")]
    pub history: Vec<SummaryHistoryDto>,
}

/// `stations[]` entry of [`TopologyDto`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StationDto {
    pub id: String,
    #[serde(alias = "name")]
    pub label: String,
}

/// `branches[]` entry of [`TopologyDto`] (non-default refs only).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BranchDto {
    #[serde(rename = "ref")]
    pub ref_name: String,
    pub base_ref: String,
    /// Station id where the branch was cut.
    #[serde(default, deserialize_with = "null_default")]
    pub fork_stage: String,
    #[serde(default, deserialize_with = "null_default")]
    pub pr_state: PrState,
    #[serde(default, deserialize_with = "de_opt_job_id")]
    pub last_job: Option<u64>,
}

/// `trains[]` entry of [`TopologyDto`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrainDto {
    #[serde(deserialize_with = "de_job_id")]
    pub job_id: u64,
    #[serde(rename = "ref")]
    pub ref_name: String,
    /// Station id of the current stage.
    #[serde(default, deserialize_with = "null_default")]
    pub stage: String,
    /// One state per station, in `stations[]` order.
    #[serde(default, deserialize_with = "de_stage_states")]
    pub stage_states: Vec<StageState>,
    #[serde(default)]
    pub node_id: Option<String>,
}

/// `GET /api/repos/{owner}/{repo}/topology`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TopologyDto {
    pub owner: String,
    pub repo: String,
    pub default_branch: String,
    /// Head commit of the default branch (submodule-bump detection).
    #[serde(default, deserialize_with = "null_default")]
    pub head_sha: String,
    pub stations: Vec<StationDto>,
    #[serde(default, deserialize_with = "null_default")]
    pub branches: Vec<BranchDto>,
    #[serde(default, deserialize_with = "null_default")]
    pub trains: Vec<TrainDto>,
    #[serde(default)]
    pub fabric_rev: u64,
}

/// SSE `event: branch_created` on `/api/events`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BranchCreatedDto {
    pub owner: String,
    pub repo: String,
    #[serde(rename = "ref")]
    pub ref_name: String,
    pub base_ref: String,
    #[serde(default, deserialize_with = "null_default")]
    pub fork_stage: String,
    #[serde(default)]
    pub fabric_rev: u64,
}

/// SSE `event: branch_deleted` on `/api/events`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BranchDeletedDto {
    pub owner: String,
    pub repo: String,
    #[serde(rename = "ref")]
    pub ref_name: String,
    #[serde(default)]
    pub fabric_rev: u64,
}

/// SSE `event: pr_state_changed` on `/api/events`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrStateChangedDto {
    pub owner: String,
    pub repo: String,
    #[serde(rename = "ref")]
    pub ref_name: String,
    pub pr_state: PrState,
    #[serde(default)]
    pub pr_number: Option<u64>,
    #[serde(default)]
    pub fabric_rev: u64,
}

/// SSE `event: job_stage_changed` on `/api/events`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JobStageChangedDto {
    pub owner: String,
    pub repo: String,
    #[serde(deserialize_with = "de_job_id")]
    pub job_id: u64,
    #[serde(rename = "ref")]
    pub ref_name: String,
    /// Station id of the stage that changed.
    #[serde(default, deserialize_with = "null_default")]
    pub stage: String,
    pub state: StageState,
    /// Optional full signal row (one per station); wins over `state`.
    #[serde(default, deserialize_with = "de_stage_states")]
    pub stage_states: Vec<StageState>,
    #[serde(default)]
    pub node_id: Option<String>,
    #[serde(default)]
    pub fabric_rev: u64,
}

#[derive(Deserialize)]
struct RawNodeMembership {
    node_id: String,
    #[serde(default)]
    online: Option<bool>,
    #[serde(default)]
    change: Option<String>,
    #[serde(default)]
    fabric_rev: u64,
}

/// SSE `event: node_membership_changed` on `/api/events`.
///
/// Accepts both `{"node_id", "online": bool, "fabric_rev"}` and
/// `ActionRunner-controller`'s `{"node_id", "change": "joined"|"left"|"online"|"offline",
/// "online_nodes": [...], "fabric_rev"}`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "RawNodeMembership")]
pub struct NodeMembershipChangedDto {
    pub node_id: String,
    pub online: bool,
    #[serde(default)]
    pub fabric_rev: u64,
}

impl From<RawNodeMembership> for NodeMembershipChangedDto {
    fn from(raw: RawNodeMembership) -> Self {
        let online = raw
            .online
            .unwrap_or(matches!(raw.change.as_deref(), Some("joined" | "online")));
        Self {
            node_id: raw.node_id,
            online,
            fabric_rev: raw.fabric_rev,
        }
    }
}

/// `GET /api/cd/deployments?repo={owner}/{repo}[&limit=N]` -> `[DeploymentDto]`,
/// newest first.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeploymentDto {
    pub deployment_id: String,
    pub owner: String,
    pub repo: String,
    pub service: String,
    pub artifact_digest: String,
    pub target_node: String,
    /// Slot currently serving traffic on the target.
    pub active_slot: Slot,
    /// Slot the candidate was staged into.
    pub staged_slot: Slot,
    pub outcome: GateOutcome,
    #[serde(default)]
    pub it_passed: u32,
    #[serde(default)]
    pub it_total: u32,
    #[serde(default)]
    pub started_epoch_ms: u64,
}

/// `tests[]` entry of [`GateDto`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GateTestDto {
    pub name: String,
    pub critical: bool,
    pub state: TestState,
    #[serde(default)]
    pub duration_ms: u64,
    #[serde(default)]
    pub crate_name: Option<String>,
    #[serde(default)]
    pub log_excerpt: Option<String>,
}

/// `GET /api/cd/deployments/{id}/gate`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GateDto {
    pub deployment_id: String,
    pub owner: String,
    pub repo: String,
    pub service: String,
    pub artifact_digest: String,
    pub target_node: String,
    pub staged_slot: Slot,
    pub restore_slot: Slot,
    #[serde(default)]
    pub lease_expires_epoch_ms: u64,
    pub outcome: GateOutcome,
    #[serde(default)]
    pub rollback_reason: Option<RollbackReason>,
    #[serde(default)]
    pub tainted: bool,
    /// Contract order (`.muconde/MuConDe.yml` -> `integration_tests`).
    pub tests: Vec<GateTestDto>,
}

/// `GET /api/cd/leases` -> `[LeaseDto]`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LeaseDto {
    pub lease_id: String,
    pub deployment_id: String,
    pub target_node: String,
    pub service: String,
    pub artifact_digest: String,
    pub active_slot: Slot,
    pub candidate_slot: Slot,
    #[serde(default)]
    pub expires_epoch_ms: u64,
    #[serde(default)]
    pub ttl_remaining_ms: u64,
    #[serde(default)]
    pub promoted: bool,
}

/// SSE `event: it_progress` on `/api/cd/events` (mirrors the
/// `IntegrationTestProgress` IPC message).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItProgressDto {
    pub deployment_id: String,
    pub test_name: String,
    pub critical: bool,
    pub state: TestState,
    #[serde(default)]
    pub duration_ms: u64,
    #[serde(default)]
    pub timestamp_epoch_ms: u64,
}

/// SSE `event: promoted` on `/api/cd/events`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromotedDto {
    pub deployment_id: String,
    /// Slot now serving traffic.
    pub slot: Slot,
}

/// SSE `event: rolled_back` on `/api/cd/events`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RolledBackDto {
    pub deployment_id: String,
    pub restore_slot: Slot,
    pub reason: RollbackReason,
}

/// SSE `event: tainted` on `/api/cd/events`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaintedDto {
    pub deployment_id: String,
    pub artifact_digest: String,
    #[serde(default)]
    pub reason: String,
}

/// SSE `event: lease_expired` on `/api/cd/events`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LeaseExpiredDto {
    pub deployment_id: String,
}

/// `GET /api/auth/session` and `POST /api/auth/redeem` response (`ActionRunner-controller`).
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct AuthSessionDto {
    #[serde(default)]
    pub authenticated: bool,
    #[serde(default)]
    pub github_login: Option<String>,
    #[serde(default)]
    pub invite_bound: bool,
    #[serde(default)]
    pub role: Option<String>,
}

/// REST paths (relative to the configured API base).
pub mod paths {
    use alloc::format;
    use alloc::string::String;

    /// CI event stream (existing endpoint, new event kinds).
    pub const CI_EVENTS: &str = "/api/events";
    /// CD event stream.
    pub const CD_EVENTS: &str = "/api/cd/events";
    /// Repository list.
    pub const REPOS: &str = "/api/repos";
    /// Active CD deployment leases.
    pub const LEASES: &str = "/api/cd/leases";
    /// Operator authentication session probe.
    pub const AUTH_SESSION: &str = "/api/auth/session";
    /// One-time invite token redemption endpoint.
    pub const AUTH_REDEEM: &str = "/api/auth/redeem";

    /// `/api/repos/{owner}/{repo}/summary`.
    #[must_use]
    pub fn summary(owner: &str, repo: &str) -> String {
        format!("/api/repos/{owner}/{repo}/summary")
    }

    /// `/api/repos/{owner}/{repo}/topology`.
    #[must_use]
    pub fn topology(owner: &str, repo: &str) -> String {
        format!("/api/repos/{owner}/{repo}/topology")
    }

    /// `/api/cd/deployments?repo={owner}/{repo}&limit={limit}`.
    #[must_use]
    pub fn deployments(owner: &str, repo: &str, limit: u32) -> String {
        format!("/api/cd/deployments?repo={owner}/{repo}&limit={limit}")
    }

    /// `/api/cd/deployments/{id}/gate`.
    #[must_use]
    pub fn gate(deployment_id: &str) -> String {
        format!("/api/cd/deployments/{deployment_id}/gate")
    }
}
