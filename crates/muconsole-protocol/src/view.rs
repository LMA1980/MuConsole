// SPDX-License-Identifier: LGPL-3.0-only WITH LGPL-3.0-linking-exception
//! Skin-agnostic view models, incremental [`Delta`]s, [`apply`] and [`diff`].
//!
//! The data worker owns the normalized store and projects it into a
//! [`ViewModel`] for the current selection. It sends one
//! [`crate::DataEvent::Snapshot`] and then [`crate::DataEvent::Patch`]es
//! produced by [`diff`]; the UI keeps a mirror up to date with [`apply`].
//!
//! **Canonical order.** Lists keyed by identity are kept sorted so
//! `apply(old, diff(old, new)) == new` holds for every pair of canonical
//! view models:
//!
//! - [`ViewModel::repos`] by [`RepoId`];
//! - [`RepoTopology::branches`] by `ref_name`;
//! - [`RepoTopology::trains`] by `job_id`;
//! - [`ViewModel::nodes`] by `id`.
//!
//! [`RepoTopology::stations`] and [`DeploymentGate::tests`] keep their
//! contract order (workflow definition / integration-test list).
//!
//! The `snake_case` serde names double as the JSON spelling used by the API
//! DTOs in [`crate::dto`]; postcard ignores names, so they cost nothing on
//! the worker wire.

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::dto::AuthSessionDto;
use crate::model::{RepoId, SubmoduleId};

/// Overall CI state of a repository.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RepoStatus {
    #[default]
    Unknown,
    Passing,
    Running,
    Failing,
}

/// Repository visibility. Private repos only reach authenticated sessions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Visibility {
    #[default]
    Public,
    #[serde(alias = "internal")]
    Private,
}

/// One submodule of a superproject, as pinned on its default branch.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubmoduleRef {
    pub path: String,
    pub repo: RepoId,
    pub pinned_sha: String,
}

/// A superproject that includes this repository as a submodule.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParentRef {
    pub repo: RepoId,
    pub path: String,
}

/// A/B deployment slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Slot {
    #[default]
    A,
    B,
}

impl Slot {
    /// The other slot.
    #[must_use]
    pub fn other(self) -> Self {
        match self {
            Self::A => Self::B,
            Self::B => Self::A,
        }
    }
}

/// Outcome of a deployment's integration gate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GateOutcome {
    /// Tests still running (or not started).
    #[default]
    Pending,
    Promoted,
    RolledBack,
    Tainted,
}

impl GateOutcome {
    /// Whether the gate reached a final verdict.
    #[must_use]
    pub fn is_final(self) -> bool {
        !matches!(self, Self::Pending)
    }
}

/// Why a deployment was rolled back.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RollbackReason {
    CriticalTestFailed,
    LeaseExpired,
    Manual,
}

/// CD half of a scorecard.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CdScore {
    pub deployment_id: String,
    pub active_slot: Slot,
    pub outcome: GateOutcome,
    pub it_passed: u32,
    pub it_total: u32,
}

/// Scorecard for one repository (CI + CD joined in the worker).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepoScore {
    pub id: RepoId,
    pub visibility: Visibility,
    pub default_branch: String,
    pub status: RepoStatus,
    /// Quality score in percent (0..=100).
    pub score: u32,
    /// Line coverage in basis points (0..=10_000).
    pub coverage_bp: u32,
    /// Benchmark delta vs. the default branch, basis points (signed).
    pub bench_delta_bp: i32,
    pub latest_job_id: Option<u64>,
    pub submodules: Vec<SubmoduleRef>,
    pub parents: Vec<ParentRef>,
    pub cd: Option<CdScore>,
}

impl RepoScore {
    /// Scorecard with only an id (everything else unknown / empty).
    #[must_use]
    pub fn new(id: RepoId) -> Self {
        Self {
            id,
            visibility: Visibility::Public,
            default_branch: "main".into(),
            status: RepoStatus::Unknown,
            score: 0,
            coverage_bp: 0,
            bench_delta_bp: 0,
            latest_job_id: None,
            submodules: Vec::new(),
            parents: Vec::new(),
            cd: None,
        }
    }
}

/// A pipeline stage, drawn as a station column on the Trainway.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Station {
    /// Stage id from the workflow definition (`clippy-check`).
    pub id: String,
    /// Short label (`CLIPPY`).
    pub label: String,
}

/// Block signal state of one stage for one train.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StageState {
    #[default]
    #[serde(alias = "queued")]
    Pending,
    Running,
    Passed,
    Failed,
    Skipped,
}

/// Pull-request state of a branch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrState {
    #[default]
    None,
    #[serde(alias = "draft")]
    Open,
    Merged,
    Closed,
}

/// Why a siding is fading out.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FadeReason {
    Merged,
    Deleted,
}

/// Life cycle of a siding on the Trainway.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SidingPhase {
    /// Branch just created: dashed track, `NEW SIDING UNDER CONSTRUCTION`.
    /// Ends when the first job on the branch is dequeued on a node.
    UnderConstruction,
    Active,
    /// Merged or deleted; the worker removes the siding 2 s after `since`.
    Fading {
        reason: FadeReason,
        since_epoch_ms: u64,
    },
}

/// A non-default branch drawn as a siding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Branch {
    pub ref_name: String,
    pub base_ref: String,
    /// Station index where the siding leaves the main line.
    pub fork_stage: u32,
    pub pr_state: PrState,
    pub last_job: Option<u64>,
    pub phase: SidingPhase,
}

/// A CI job drawn as a train.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Train {
    pub job_id: u64,
    pub ref_name: String,
    /// Station index of the current stage.
    pub stage: u32,
    /// One signal per station (same order as [`RepoTopology::stations`]).
    pub stage_states: Vec<StageState>,
    /// Fabric node carrying the job; `None` while queued.
    pub node_id: Option<String>,
    /// The carrying node left the fabric (`NODE LOST`).
    pub node_lost: bool,
}

impl Train {
    /// A failed stage stops the train (reverse video + `STOP`).
    #[must_use]
    pub fn failed(&self) -> bool {
        self.stage_states.contains(&StageState::Failed)
    }

    /// Queued: no node yet.
    #[must_use]
    pub fn queued(&self) -> bool {
        self.node_id.is_none()
    }
}

/// Railway model of one repository (or the selected submodule's repo).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepoTopology {
    pub repo: RepoId,
    /// Set when the topology is shown for a submodule of the selection.
    pub submodule: Option<SubmoduleId>,
    pub default_branch: String,
    pub stations: Vec<Station>,
    pub branches: Vec<Branch>,
    pub trains: Vec<Train>,
    /// This repository's own submodules (interchange markers).
    pub submodules: Vec<SubmoduleRef>,
    /// Topology change counter shown as `FABRIC REV n`.
    pub fabric_rev: u64,
    /// The parent's gitlink points at an older commit than this repo's
    /// default branch head ("submodule bump pending" interchange).
    pub submodule_bump_pending: bool,
}

impl RepoTopology {
    /// `STOP` annunciator: any train has a failed stage.
    #[must_use]
    pub fn stop(&self) -> bool {
        self.trains.iter().any(Train::failed)
    }
}

/// Integration-test state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TestState {
    #[default]
    Pending,
    Running,
    Passed,
    Failed,
}

/// One checkpoint on the integration gate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GateTest {
    pub name: String,
    /// `[C]` flag: a failure rolls the deployment back.
    pub critical: bool,
    pub state: TestState,
    pub duration_ms: u64,
    /// Crate the test belongs to (highlighted for the selected submodule).
    pub crate_name: Option<String>,
    pub log_excerpt: Option<String>,
}

/// Live integration gate of a deployment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeploymentGate {
    pub deployment_id: String,
    pub repo: RepoId,
    pub service: String,
    pub artifact_digest: String,
    pub target_node: String,
    pub staged_slot: Slot,
    pub restore_slot: Slot,
    pub lease_expires_epoch_ms: u64,
    pub outcome: GateOutcome,
    pub rollback_reason: Option<RollbackReason>,
    pub tainted: bool,
    pub tests: Vec<GateTest>,
}

impl DeploymentGate {
    /// Passed test count.
    #[must_use]
    pub fn passed(&self) -> u32 {
        count_u32(self.tests.iter().filter(|t| t.state == TestState::Passed))
    }

    /// Total test count.
    #[must_use]
    pub fn total(&self) -> u32 {
        count_u32(self.tests.iter())
    }

    /// Every critical test passed.
    #[must_use]
    pub fn all_critical_passed(&self) -> bool {
        self.tests
            .iter()
            .filter(|t| t.critical)
            .all(|t| t.state == TestState::Passed)
    }

    /// Any critical test failed.
    #[must_use]
    pub fn any_critical_failed(&self) -> bool {
        self.tests
            .iter()
            .any(|t| t.critical && t.state == TestState::Failed)
    }

    /// `PROMOTE` junction lamp.
    #[must_use]
    pub fn promote_lit(&self) -> bool {
        self.outcome == GateOutcome::Promoted
            || (self.outcome == GateOutcome::Pending && self.all_critical_passed())
    }

    /// `ROLLBACK` junction lamp.
    #[must_use]
    pub fn rollback_lit(&self) -> bool {
        self.outcome == GateOutcome::RolledBack || self.any_critical_failed()
    }

    /// History-strip entry for this gate.
    #[must_use]
    pub fn summary(&self) -> DeploymentSummary {
        DeploymentSummary {
            deployment_id: self.deployment_id.clone(),
            artifact_digest: self.artifact_digest.clone(),
            it_passed: self.passed(),
            it_total: self.total(),
            outcome: self.outcome,
        }
    }
}

fn count_u32<I: Iterator>(it: I) -> u32 {
    u32::try_from(it.count()).unwrap_or(u32::MAX)
}

/// History-strip entry: `digest IT n/m`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeploymentSummary {
    pub deployment_id: String,
    pub artifact_digest: String,
    pub it_passed: u32,
    pub it_total: u32,
    pub outcome: GateOutcome,
}

/// Fabric worker node.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FabricNode {
    pub id: String,
    pub online: bool,
}

/// Current repository / submodule selection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Selection {
    pub repo: RepoId,
    pub submodule: Option<SubmoduleId>,
}

/// Where the view model comes from.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum DataSource {
    #[default]
    Live,
    /// Fixture scenario by name (`?scenario=`).
    Scenario(String),
    /// Restored from the CacheStorage offline snapshot.
    OfflineCache,
}

/// Alert counts per area (LCD badges next to the area lamps).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct AreaBadges {
    pub overview: u32,
    pub ci: u32,
    pub cd: u32,
    pub fleet: u32,
}

/// Operator authentication state surfaced in the top bar and used to gate
/// raw execution logs and job restart controls.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthState {
    #[default]
    Anonymous,
    InviteRequired {
        github_login: String,
    },
    Operator {
        github_login: String,
        role: Option<String>,
    },
}

impl AuthState {
    /// Derive [`AuthState`] from an [`AuthSessionDto`] response.
    #[must_use]
    pub fn from_session(dto: &AuthSessionDto) -> Self {
        if dto.authenticated || dto.invite_bound {
            Self::Operator {
                github_login: dto
                    .github_login
                    .clone()
                    .filter(|s| !s.is_empty())
                    .unwrap_or_else(|| "operator".into()),
                role: dto.role.clone(),
            }
        } else if let Some(login) = dto.github_login.as_ref().filter(|s| !s.is_empty()) {
            Self::InviteRequired {
                github_login: login.clone(),
            }
        } else {
            Self::Anonymous
        }
    }

    /// Whether the session is an authenticated, invite-bound operator.
    #[must_use]
    pub fn is_operator(&self) -> bool {
        matches!(self, Self::Operator { .. })
    }

    /// Short top-bar badge label (`READ-ONLY`, `INVITE REQ (@login)`, `@login`).
    #[must_use]
    pub fn badge_label(&self) -> String {
        match self {
            Self::Anonymous => "READ-ONLY".into(),
            Self::InviteRequired { github_login } => format!("INVITE REQ (@{github_login})"),
            Self::Operator { github_login, .. } => format!("@{github_login}"),
        }
    }
}

/// Full view model snapshot.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ViewModel {
    pub source: DataSource,
    /// Scorecards, sorted by [`RepoId`].
    pub repos: Vec<RepoScore>,
    pub selection: Option<Selection>,
    /// Trainway of the selection (submodule repo when one is selected).
    pub topology: Option<RepoTopology>,
    /// Active (or most recent) deployment gate of the selected parent repo.
    pub gate: Option<DeploymentGate>,
    /// History strip, newest first.
    pub history: Vec<DeploymentSummary>,
    /// Fabric nodes, sorted by id.
    pub nodes: Vec<FabricNode>,
    /// Set when the data is stale (offline snapshot): when it was saved.
    pub stale_since_epoch_ms: Option<u64>,
    /// Operator authentication state.
    #[serde(default)]
    pub auth: AuthState,
}

/// Incremental change to the view model.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Delta {
    Source(DataSource),
    RepoUpsert(RepoScore),
    RepoRemoved(RepoId),
    Selection(Option<Selection>),
    /// Replace the whole topology (selection or station set changed).
    Topology(Option<RepoTopology>),
    FabricRev(u64),
    SubmoduleBumpPending(bool),
    BranchUpsert(Branch),
    BranchRemoved(String),
    TrainUpsert(Train),
    TrainRemoved(u64),
    /// Replace the whole gate (deployment or test list changed).
    Gate(Option<DeploymentGate>),
    GateTestUpsert(GateTest),
    GateStatus {
        outcome: GateOutcome,
        rollback_reason: Option<RollbackReason>,
        tainted: bool,
        lease_expires_epoch_ms: u64,
    },
    History(Vec<DeploymentSummary>),
    NodeUpsert(FabricNode),
    NodeRemoved(String),
    Stale(Option<u64>),
    Auth(AuthState),
}

fn upsert_sorted<T, K: Ord + ?Sized>(v: &mut Vec<T>, item: T, key: impl Fn(&T) -> &K) {
    match v.binary_search_by(|x| key(x).cmp(key(&item))) {
        Ok(i) => v[i] = item,
        Err(i) => v.insert(i, item),
    }
}

fn remove_sorted<T, K: Ord + ?Sized>(v: &mut Vec<T>, k: &K, key: impl Fn(&T) -> &K) {
    if let Ok(i) = v.binary_search_by(|x| key(x).cmp(k)) {
        v.remove(i);
    }
}

/// Apply one delta. Deltas that target an absent topology / gate are
/// ignored (the next snapshot or replace delta carries them).
pub fn apply(vm: &mut ViewModel, delta: &Delta) {
    match delta {
        Delta::Source(s) => vm.source = s.clone(),
        Delta::RepoUpsert(r) => upsert_sorted(&mut vm.repos, r.clone(), |x| &x.id),
        Delta::RepoRemoved(id) => remove_sorted(&mut vm.repos, id, |x| &x.id),
        Delta::Selection(s) => vm.selection = s.clone(),
        Delta::Topology(t) => vm.topology = t.clone(),
        Delta::Gate(g) => vm.gate = g.clone(),
        Delta::History(h) => vm.history = h.clone(),
        Delta::NodeUpsert(n) => upsert_sorted(&mut vm.nodes, n.clone(), |x| x.id.as_str()),
        Delta::NodeRemoved(id) => remove_sorted(&mut vm.nodes, id.as_str(), |x| x.id.as_str()),
        Delta::Stale(s) => vm.stale_since_epoch_ms = *s,
        Delta::Auth(a) => vm.auth = a.clone(),
        Delta::FabricRev(_)
        | Delta::SubmoduleBumpPending(_)
        | Delta::BranchUpsert(_)
        | Delta::BranchRemoved(_)
        | Delta::TrainUpsert(_)
        | Delta::TrainRemoved(_) => {
            if let Some(t) = vm.topology.as_mut() {
                apply_topology(t, delta);
            }
        }
        Delta::GateTestUpsert(_) | Delta::GateStatus { .. } => {
            if let Some(g) = vm.gate.as_mut() {
                apply_gate(g, delta);
            }
        }
    }
}

fn apply_topology(t: &mut RepoTopology, delta: &Delta) {
    match delta {
        Delta::FabricRev(r) => t.fabric_rev = *r,
        Delta::SubmoduleBumpPending(b) => t.submodule_bump_pending = *b,
        Delta::BranchUpsert(b) => {
            upsert_sorted(&mut t.branches, b.clone(), |x| x.ref_name.as_str());
        }
        Delta::BranchRemoved(r) => {
            remove_sorted(&mut t.branches, r.as_str(), |x| x.ref_name.as_str());
        }
        Delta::TrainUpsert(tr) => upsert_sorted(&mut t.trains, tr.clone(), |x| &x.job_id),
        Delta::TrainRemoved(id) => remove_sorted(&mut t.trains, id, |x| &x.job_id),
        _ => {}
    }
}

fn apply_gate(g: &mut DeploymentGate, delta: &Delta) {
    match delta {
        Delta::GateTestUpsert(t) => {
            if let Some(slot) = g.tests.iter_mut().find(|x| x.name == t.name) {
                *slot = t.clone();
            } else {
                g.tests.push(t.clone());
            }
        }
        Delta::GateStatus {
            outcome,
            rollback_reason,
            tainted,
            lease_expires_epoch_ms,
        } => {
            g.outcome = *outcome;
            g.rollback_reason = *rollback_reason;
            g.tainted = *tainted;
            g.lease_expires_epoch_ms = *lease_expires_epoch_ms;
        }
        _ => {}
    }
}

/// Apply a patch in order.
pub fn apply_all(vm: &mut ViewModel, deltas: &[Delta]) {
    for d in deltas {
        apply(vm, d);
    }
}

/// Merge-walk two sorted lists, emitting upserts and removals.
fn diff_sorted<T: PartialEq + Clone, K: Ord + ?Sized>(
    old: &[T],
    new: &[T],
    key: impl Fn(&T) -> &K,
    upsert: impl Fn(T) -> Delta,
    remove: impl Fn(&T) -> Delta,
    out: &mut Vec<Delta>,
) {
    let (mut i, mut j): (usize, usize) = (0, 0);
    while i < old.len() || j < new.len() {
        match (old.get(i), new.get(j)) {
            (Some(a), Some(b)) => match key(a).cmp(key(b)) {
                core::cmp::Ordering::Equal => {
                    if a != b {
                        out.push(upsert(b.clone()));
                    }
                    i += 1;
                    j += 1;
                }
                core::cmp::Ordering::Less => {
                    out.push(remove(a));
                    i += 1;
                }
                core::cmp::Ordering::Greater => {
                    out.push(upsert(b.clone()));
                    j += 1;
                }
            },
            (Some(a), None) => {
                out.push(remove(a));
                i += 1;
            }
            (None, Some(b)) => {
                out.push(upsert(b.clone()));
                j += 1;
            }
            (None, None) => break,
        }
    }
}

/// Smallest practical delta list turning `old` into `new` (both canonical).
#[must_use]
pub fn diff(old: &ViewModel, new: &ViewModel) -> Vec<Delta> {
    let mut out = Vec::new();
    if old.source != new.source {
        out.push(Delta::Source(new.source.clone()));
    }
    diff_sorted(
        &old.repos,
        &new.repos,
        |x| &x.id,
        Delta::RepoUpsert,
        |x| Delta::RepoRemoved(x.id.clone()),
        &mut out,
    );
    if old.selection != new.selection {
        out.push(Delta::Selection(new.selection.clone()));
    }
    diff_topology(old.topology.as_ref(), new.topology.as_ref(), &mut out);
    diff_gate(old.gate.as_ref(), new.gate.as_ref(), &mut out);
    if old.history != new.history {
        out.push(Delta::History(new.history.clone()));
    }
    diff_sorted(
        &old.nodes,
        &new.nodes,
        |x| x.id.as_str(),
        Delta::NodeUpsert,
        |x| Delta::NodeRemoved(x.id.clone()),
        &mut out,
    );
    if old.stale_since_epoch_ms != new.stale_since_epoch_ms {
        out.push(Delta::Stale(new.stale_since_epoch_ms));
    }
    if old.auth != new.auth {
        out.push(Delta::Auth(new.auth.clone()));
    }
    out
}

fn diff_topology(old: Option<&RepoTopology>, new: Option<&RepoTopology>, out: &mut Vec<Delta>) {
    match (old, new) {
        (None, None) => {}
        (Some(a), Some(b))
            if a.repo == b.repo
                && a.submodule == b.submodule
                && a.default_branch == b.default_branch
                && a.stations == b.stations
                && a.submodules == b.submodules =>
        {
            if a.fabric_rev != b.fabric_rev {
                out.push(Delta::FabricRev(b.fabric_rev));
            }
            if a.submodule_bump_pending != b.submodule_bump_pending {
                out.push(Delta::SubmoduleBumpPending(b.submodule_bump_pending));
            }
            diff_sorted(
                &a.branches,
                &b.branches,
                |x| x.ref_name.as_str(),
                Delta::BranchUpsert,
                |x| Delta::BranchRemoved(x.ref_name.clone()),
                out,
            );
            diff_sorted(
                &a.trains,
                &b.trains,
                |x| &x.job_id,
                Delta::TrainUpsert,
                |x| Delta::TrainRemoved(x.job_id),
                out,
            );
        }
        (_, b) => out.push(Delta::Topology(b.cloned())),
    }
}

fn diff_gate(old: Option<&DeploymentGate>, new: Option<&DeploymentGate>, out: &mut Vec<Delta>) {
    match (old, new) {
        (None, None) => {}
        (Some(a), Some(b))
            if a.deployment_id == b.deployment_id
                && a.repo == b.repo
                && a.service == b.service
                && a.artifact_digest == b.artifact_digest
                && a.target_node == b.target_node
                && a.staged_slot == b.staged_slot
                && a.restore_slot == b.restore_slot
                && b.tests.len() >= a.tests.len()
                && a.tests.iter().zip(&b.tests).all(|(x, y)| x.name == y.name) =>
        {
            for (i, t) in b.tests.iter().enumerate() {
                if a.tests.get(i) != Some(t) {
                    out.push(Delta::GateTestUpsert(t.clone()));
                }
            }
            if a.outcome != b.outcome
                || a.rollback_reason != b.rollback_reason
                || a.tainted != b.tainted
                || a.lease_expires_epoch_ms != b.lease_expires_epoch_ms
            {
                out.push(Delta::GateStatus {
                    outcome: b.outcome,
                    rollback_reason: b.rollback_reason,
                    tainted: b.tainted,
                    lease_expires_epoch_ms: b.lease_expires_epoch_ms,
                });
            }
        }
        (_, b) => out.push(Delta::Gate(b.cloned())),
    }
}
