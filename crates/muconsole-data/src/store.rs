// SPDX-License-Identifier: AGPL-3.0-only
//! Normalized store keyed by repository, job and deployment.
//!
//! Every upstream fact (REST payload, SSE event, scenario step) becomes an
//! [`Input`] and goes through [`Store::apply`]; [`Store::view`] projects the
//! store into the canonical [`ViewModel`] for the current selection. Live
//! data and fixture scenarios share this exact path.

use std::collections::BTreeMap;

use muconsole_protocol::dto::{
    BranchCreatedDto, BranchDeletedDto, DeploymentDto, GateDto, ItProgressDto, JobStageChangedDto,
    LeaseExpiredDto, NodeMembershipChangedDto, PrStateChangedDto, PromotedDto, RepoDto,
    RepoSummaryDto, RolledBackDto, TaintedDto, TopologyDto,
};
use muconsole_protocol::{
    AreaBadges, Branch, CdScore, DataSource, DeploymentGate, DeploymentSummary, FabricNode,
    FadeReason, GateOutcome, GateTest, ParentRef, PrState, RepoId, RepoScore, RepoStatus,
    RepoTopology, RollbackReason, Selection, SidingPhase, StageState, Station, SubmoduleRef, Train,
    ViewModel,
};

/// A merged or deleted siding is removed this long after it starts fading.
pub const FADE_MS: u64 = 2_000;

/// One upstream fact.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, PartialEq)]
pub enum Input {
    Repos(Vec<RepoDto>),
    Summary(RepoSummaryDto),
    Topology(TopologyDto),
    Deployments {
        repo: RepoId,
        list: Vec<DeploymentDto>,
    },
    Gate(GateDto),
    BranchCreated(BranchCreatedDto),
    BranchDeleted(BranchDeletedDto),
    PrStateChanged(PrStateChangedDto),
    JobStageChanged(JobStageChangedDto),
    NodeMembership(NodeMembershipChangedDto),
    ItProgress(ItProgressDto),
    Promoted(PromotedDto),
    RolledBack(RolledBackDto),
    Tainted(TaintedDto),
    LeaseExpired(LeaseExpiredDto),
    Select(Selection),
    /// Offline snapshot restored from CacheStorage (`saved_epoch_ms`, view).
    Restore {
        saved_epoch_ms: u64,
        view: ViewModel,
    },
}

#[derive(Debug, Clone)]
struct Topo {
    t: RepoTopology,
    head_sha: String,
}

/// Normalized worker state.
#[derive(Debug, Default)]
pub struct Store {
    source: DataSource,
    repos: BTreeMap<RepoId, RepoScore>,
    topologies: BTreeMap<RepoId, Topo>,
    gates: BTreeMap<String, DeploymentGate>,
    history: BTreeMap<RepoId, Vec<DeploymentSummary>>,
    latest: BTreeMap<RepoId, String>,
    nodes: BTreeMap<String, FabricNode>,
    selection: Option<Selection>,
    /// Offline snapshot shown until live data arrives.
    offline: Option<(u64, ViewModel)>,
    loaded: bool,
    dirty: bool,
}

fn pct_to_bp(pct: f64) -> u32 {
    if pct.is_nan() {
        return 0;
    }
    // Clamped to 0..=10_000 first, so the cast cannot truncate.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let bp = (pct * 100.0).round().clamp(0.0, 10_000.0) as u32;
    bp
}

fn signed_pct_to_bp(pct: f64) -> i32 {
    if pct.is_nan() {
        return 0;
    }
    #[allow(clippy::cast_possible_truncation)]
    let bp = (pct * 100.0).round().clamp(-1_000_000.0, 1_000_000.0) as i32;
    bp
}

fn station_index(stations: &[Station], id: &str) -> u32 {
    stations
        .iter()
        .position(|s| s.id == id)
        .and_then(|i| u32::try_from(i).ok())
        .unwrap_or(0)
}

fn upsert_sorted<T, K: Ord>(v: &mut Vec<T>, item: T, key: impl Fn(&T) -> K) {
    match v.binary_search_by(|x| key(x).cmp(&key(&item))) {
        Ok(i) => v[i] = item,
        Err(i) => v.insert(i, item),
    }
}

impl Store {
    #[must_use]
    pub fn new(source: DataSource) -> Self {
        Self {
            source,
            dirty: true,
            ..Self::default()
        }
    }

    /// Whether the projected view may have changed since [`Self::take_dirty`].
    #[must_use]
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Read and clear the dirty flag.
    pub fn take_dirty(&mut self) -> bool {
        std::mem::take(&mut self.dirty)
    }

    /// Whether live (or scenario) data has arrived.
    #[must_use]
    pub fn has_live_data(&self) -> bool {
        self.loaded
    }

    #[must_use]
    pub fn selection(&self) -> Option<&Selection> {
        self.selection.as_ref()
    }

    /// Repository whose topology the selection shows (submodule repo when a
    /// submodule is selected).
    #[must_use]
    pub fn topology_repo(&self) -> Option<RepoId> {
        let sel = self.selection.as_ref()?;
        match &sel.submodule {
            None => Some(sel.repo.clone()),
            Some(sub) => self
                .repos
                .get(&sel.repo)?
                .submodules
                .iter()
                .find(|s| s.path == sub.path)
                .map(|s| s.repo.clone()),
        }
    }

    /// Latest deployment id of a repository, if known.
    #[must_use]
    pub fn latest_deployment(&self, repo: &RepoId) -> Option<&str> {
        self.latest.get(repo).map(String::as_str)
    }

    /// Whether the gate of a deployment is loaded.
    #[must_use]
    pub fn has_gate(&self, id: &str) -> bool {
        self.gates.contains_key(id)
    }

    /// Whether a topology is loaded for a repository.
    #[must_use]
    pub fn has_topology(&self, repo: &RepoId) -> bool {
        self.topologies.contains_key(repo)
    }

    /// Ordered repository ids.
    pub fn repo_ids(&self) -> impl Iterator<Item = &RepoId> {
        self.repos.keys()
    }

    /// Change the data source label.
    pub fn set_source(&mut self, source: DataSource) {
        if self.source != source {
            self.source = source;
            self.dirty = true;
        }
    }

    /// Apply one input. Returns whether anything changed.
    pub fn apply(&mut self, input: Input, now_ms: u64) -> bool {
        let changed = match input {
            Input::Repos(list) => self.repos_loaded(list),
            Input::Summary(s) => self.summary(&s),
            Input::Topology(t) => self.topology(t),
            Input::Deployments { repo, list } => self.deployments(&repo, &list),
            Input::Gate(g) => self.gate(g),
            Input::BranchCreated(e) => self.branch_created(&e),
            Input::BranchDeleted(e) => self.branch_deleted(&e, now_ms),
            Input::PrStateChanged(e) => self.pr_state(&e, now_ms),
            Input::JobStageChanged(e) => self.job_stage(&e),
            Input::NodeMembership(e) => self.node_membership(&e),
            Input::ItProgress(e) => self.it_progress(&e),
            Input::Promoted(e) => self.with_gate(&e.deployment_id, |g| {
                g.outcome = GateOutcome::Promoted;
                g.restore_slot = e.slot.other();
            }),
            Input::RolledBack(e) => self.with_gate(&e.deployment_id, |g| {
                g.outcome = GateOutcome::RolledBack;
                g.rollback_reason = Some(e.reason);
                g.restore_slot = e.restore_slot;
            }),
            Input::Tainted(e) => self.with_gate(&e.deployment_id, |g| {
                g.outcome = GateOutcome::Tainted;
                g.tainted = true;
            }),
            Input::LeaseExpired(e) => self.with_gate(&e.deployment_id, |g| {
                if !g.outcome.is_final() {
                    g.outcome = GateOutcome::RolledBack;
                    g.rollback_reason = Some(RollbackReason::LeaseExpired);
                }
            }),
            Input::Select(sel) => {
                let changed = self.selection.as_ref() != Some(&sel);
                self.selection = Some(sel);
                changed
            }
            Input::Restore {
                saved_epoch_ms,
                view,
            } => {
                if self.loaded {
                    false
                } else {
                    if self.selection.is_none() {
                        self.selection.clone_from(&view.selection);
                    }
                    self.offline = Some((saved_epoch_ms, view));
                    true
                }
            }
        };
        self.dirty |= changed;
        changed
    }

    /// Advance time: remove sidings whose fade-out finished.
    pub fn tick(&mut self, now_ms: u64) -> bool {
        let mut changed = false;
        for topo in self.topologies.values_mut() {
            let t = &mut topo.t;
            let expired: Vec<String> = t
                .branches
                .iter()
                .filter(|b| {
                    matches!(b.phase, SidingPhase::Fading { since_epoch_ms, .. }
                        if now_ms >= since_epoch_ms.saturating_add(FADE_MS))
                })
                .map(|b| b.ref_name.clone())
                .collect();
            if expired.is_empty() {
                continue;
            }
            t.branches.retain(|b| !expired.contains(&b.ref_name));
            t.trains.retain(|tr| !expired.contains(&tr.ref_name));
            t.fabric_rev += 1;
            changed = true;
        }
        self.dirty |= changed;
        changed
    }

    fn repos_loaded(&mut self, list: Vec<RepoDto>) -> bool {
        self.loaded = true;
        self.offline = None;
        let mut next = BTreeMap::new();
        for d in list {
            let id = RepoId::new(&d.owner, &d.repo);
            let cd = self.repos.get(&id).and_then(|r| r.cd.clone());
            let score = RepoScore {
                id: id.clone(),
                visibility: d.visibility,
                default_branch: d.default_branch,
                status: d.status,
                score: d.score.min(100),
                coverage_bp: pct_to_bp(d.coverage),
                bench_delta_bp: signed_pct_to_bp(d.bench_delta),
                latest_job_id: d.latest_job_id,
                submodules: d
                    .submodules
                    .into_iter()
                    .map(|s| SubmoduleRef {
                        path: s.path,
                        repo: RepoId::new(&s.owner, &s.repo),
                        pinned_sha: s.pinned_sha,
                    })
                    .collect(),
                parents: d
                    .parents
                    .into_iter()
                    .map(|p| ParentRef {
                        repo: RepoId::new(&p.owner, &p.repo),
                        path: p.path,
                    })
                    .collect(),
                cd,
            };
            next.insert(id, score);
        }
        self.repos = next;
        true
    }

    fn summary(&mut self, s: &RepoSummaryDto) -> bool {
        let Some(r) = self.repos.get_mut(&RepoId::new(&s.owner, &s.repo)) else {
            return false;
        };
        r.status = s.status;
        r.score = s.score.min(100);
        r.coverage_bp = pct_to_bp(s.coverage);
        r.bench_delta_bp = signed_pct_to_bp(s.bench_delta);
        r.latest_job_id = s.latest_job_id.or(r.latest_job_id);
        true
    }

    fn topology(&mut self, d: TopologyDto) -> bool {
        self.loaded = true;
        let repo = RepoId::new(&d.owner, &d.repo);
        let stations: Vec<Station> = d
            .stations
            .into_iter()
            .map(|s| Station {
                id: s.id,
                label: s.label,
            })
            .collect();
        let n = stations.len();
        let mut branches: Vec<Branch> = d
            .branches
            .into_iter()
            .filter(|b| b.ref_name != d.default_branch)
            .map(|b| Branch {
                fork_stage: station_index(&stations, &b.fork_stage),
                ref_name: b.ref_name,
                base_ref: b.base_ref,
                pr_state: b.pr_state,
                last_job: b.last_job,
                phase: SidingPhase::Active,
            })
            .collect();
        branches.sort_by(|a, b| a.ref_name.cmp(&b.ref_name));
        branches.dedup_by(|a, b| a.ref_name == b.ref_name);
        let mut trains: Vec<Train> = d
            .trains
            .into_iter()
            .map(|t| {
                let mut states = t.stage_states;
                states.resize(n, StageState::Pending);
                Train {
                    job_id: t.job_id,
                    stage: station_index(&stations, &t.stage),
                    node_lost: self.node_offline(t.node_id.as_deref()),
                    ref_name: t.ref_name,
                    stage_states: states,
                    node_id: t.node_id,
                }
            })
            .collect();
        trains.sort_by_key(|t| t.job_id);
        trains.dedup_by_key(|t| t.job_id);
        let prev_rev = self.topologies.get(&repo).map_or(0, |t| t.t.fabric_rev);
        let submodules = self
            .repos
            .get(&repo)
            .map(|r| r.submodules.clone())
            .unwrap_or_default();
        let topo = RepoTopology {
            repo: repo.clone(),
            submodule: None,
            default_branch: d.default_branch,
            stations,
            branches,
            trains,
            submodules,
            fabric_rev: d.fabric_rev.max(prev_rev),
            submodule_bump_pending: false,
        };
        self.topologies.insert(
            repo,
            Topo {
                t: topo,
                head_sha: d.head_sha,
            },
        );
        true
    }

    fn node_offline(&self, node: Option<&str>) -> bool {
        node.and_then(|n| self.nodes.get(n))
            .is_some_and(|n| !n.online)
    }

    fn topo_mut(&mut self, owner: &str, repo: &str, server_rev: u64) -> Option<&mut RepoTopology> {
        let topo = &mut self.topologies.get_mut(&RepoId::new(owner, repo))?.t;
        topo.fabric_rev = server_rev.max(topo.fabric_rev + 1);
        Some(topo)
    }

    fn branch_created(&mut self, e: &BranchCreatedDto) -> bool {
        let Some(t) = self
            .topologies
            .get_mut(&RepoId::new(&e.owner, &e.repo))
            .map(|t| &mut t.t)
        else {
            return false;
        };
        if e.ref_name == t.default_branch || t.branches.iter().any(|b| b.ref_name == e.ref_name) {
            return false;
        }
        let fork_stage = station_index(&t.stations, &e.fork_stage);
        t.fabric_rev = e.fabric_rev.max(t.fabric_rev + 1);
        upsert_sorted(
            &mut t.branches,
            Branch {
                ref_name: e.ref_name.clone(),
                base_ref: e.base_ref.clone(),
                fork_stage,
                pr_state: PrState::None,
                last_job: None,
                phase: SidingPhase::UnderConstruction,
            },
            |b| b.ref_name.clone(),
        );
        true
    }

    fn start_fade(t: &mut RepoTopology, r: &str, reason: FadeReason, now_ms: u64) -> bool {
        let Some(b) = t.branches.iter_mut().find(|b| b.ref_name == r) else {
            return false;
        };
        if matches!(b.phase, SidingPhase::Fading { .. }) {
            return false;
        }
        b.phase = SidingPhase::Fading {
            reason,
            since_epoch_ms: now_ms,
        };
        true
    }

    fn branch_deleted(&mut self, e: &BranchDeletedDto, now_ms: u64) -> bool {
        let Some(t) = self
            .topologies
            .get_mut(&RepoId::new(&e.owner, &e.repo))
            .map(|t| &mut t.t)
        else {
            return false;
        };
        let changed = Self::start_fade(t, &e.ref_name, FadeReason::Deleted, now_ms);
        if changed {
            t.fabric_rev = e.fabric_rev.max(t.fabric_rev + 1);
        }
        changed
    }

    fn pr_state(&mut self, e: &PrStateChangedDto, now_ms: u64) -> bool {
        let Some(t) = self.topo_mut(&e.owner, &e.repo, e.fabric_rev) else {
            return false;
        };
        let Some(b) = t.branches.iter_mut().find(|b| b.ref_name == e.ref_name) else {
            return false;
        };
        b.pr_state = e.pr_state;
        if e.pr_state == PrState::Merged {
            Self::start_fade(t, &e.ref_name, FadeReason::Merged, now_ms);
        }
        true
    }

    fn job_stage(&mut self, e: &JobStageChangedDto) -> bool {
        let repo_id = RepoId::new(&e.owner, &e.repo);
        let node_lost = self.node_offline(e.node_id.as_deref());
        let mut changed = false;
        let mut default_train_status = None;
        if let Some(t) = self.topologies.get_mut(&repo_id).map(|t| &mut t.t) {
            // One train per lane: a newer job on the same ref replaces older
            // ones; events for an older job are stale.
            if t.trains
                .iter()
                .any(|tr| tr.ref_name == e.ref_name && tr.job_id > e.job_id)
            {
                return false;
            }
            let n = t.stations.len();
            let stage = station_index(&t.stations, &e.stage);
            let mut states = t
                .trains
                .iter()
                .find(|tr| tr.job_id == e.job_id)
                .map_or_else(
                    || vec![StageState::Pending; n],
                    |tr| tr.stage_states.clone(),
                );
            if e.stage_states.is_empty() {
                if let Some(slot) = usize::try_from(stage).ok().and_then(|i| states.get_mut(i)) {
                    *slot = e.state;
                }
            } else {
                states.clone_from(&e.stage_states);
                states.resize(n, StageState::Pending);
            }
            t.trains
                .retain(|tr| tr.ref_name != e.ref_name || tr.job_id == e.job_id);
            let train = Train {
                job_id: e.job_id,
                ref_name: e.ref_name.clone(),
                stage,
                stage_states: states,
                node_id: e.node_id.clone(),
                node_lost,
            };
            if e.ref_name == t.default_branch {
                default_train_status = Some(train_status(&train));
            } else if let Some(b) = t.branches.iter_mut().find(|b| b.ref_name == e.ref_name) {
                b.last_job = Some(e.job_id);
                if b.phase == SidingPhase::UnderConstruction && e.node_id.is_some() {
                    b.phase = SidingPhase::Active;
                }
            } else {
                upsert_sorted(
                    &mut t.branches,
                    Branch {
                        ref_name: e.ref_name.clone(),
                        base_ref: t.default_branch.clone(),
                        fork_stage: 0,
                        pr_state: PrState::None,
                        last_job: Some(e.job_id),
                        phase: SidingPhase::Active,
                    },
                    |b| b.ref_name.clone(),
                );
            }
            upsert_sorted(&mut t.trains, train, |tr| tr.job_id);
            t.fabric_rev = e.fabric_rev.max(t.fabric_rev + 1);
            changed = true;
        }
        if let Some(r) = self.repos.get_mut(&repo_id) {
            if r.latest_job_id.is_none_or(|j| j <= e.job_id) {
                r.latest_job_id = Some(e.job_id);
            }
            if let Some(s) = default_train_status {
                r.status = s;
            }
            changed = true;
        }
        changed
    }

    fn node_membership(&mut self, e: &NodeMembershipChangedDto) -> bool {
        let node = FabricNode {
            id: e.node_id.clone(),
            online: e.online,
        };
        let changed = self.nodes.get(&e.node_id) != Some(&node);
        self.nodes.insert(e.node_id.clone(), node);
        for topo in self.topologies.values_mut() {
            let mut touched = false;
            for tr in &mut topo.t.trains {
                if tr.node_id.as_deref() == Some(e.node_id.as_str()) && tr.node_lost == e.online {
                    tr.node_lost = !e.online;
                    touched = true;
                }
            }
            if touched {
                topo.t.fabric_rev = e.fabric_rev.max(topo.t.fabric_rev + 1);
            }
        }
        changed
    }

    fn deployments(&mut self, repo: &RepoId, list: &[DeploymentDto]) -> bool {
        self.loaded = true;
        let mut sorted: Vec<&DeploymentDto> = list.iter().collect();
        sorted.sort_by_key(|a| std::cmp::Reverse(a.started_epoch_ms));
        let history: Vec<DeploymentSummary> = sorted
            .iter()
            .map(|d| {
                self.gates.get(&d.deployment_id).map_or_else(
                    || DeploymentSummary {
                        deployment_id: d.deployment_id.clone(),
                        artifact_digest: d.artifact_digest.clone(),
                        it_passed: d.it_passed,
                        it_total: d.it_total,
                        outcome: d.outcome,
                    },
                    DeploymentGate::summary,
                )
            })
            .collect();
        if let Some(first) = sorted.first() {
            self.latest
                .insert(repo.clone(), first.deployment_id.clone());
            if let Some(r) = self.repos.get_mut(repo) {
                r.cd = Some(CdScore {
                    deployment_id: first.deployment_id.clone(),
                    active_slot: first.active_slot,
                    outcome: first.outcome,
                    it_passed: first.it_passed,
                    it_total: first.it_total,
                });
            }
        } else {
            self.latest.remove(repo);
            if let Some(r) = self.repos.get_mut(repo) {
                r.cd = None;
            }
        }
        self.history.insert(repo.clone(), history);
        let latest = self.latest.get(repo).cloned();
        if let Some(id) = latest {
            self.refresh_cd(&id);
        }
        true
    }

    fn gate(&mut self, d: GateDto) -> bool {
        self.loaded = true;
        let id = d.deployment_id.clone();
        let gate = DeploymentGate {
            deployment_id: d.deployment_id,
            repo: RepoId::new(&d.owner, &d.repo),
            service: d.service,
            artifact_digest: d.artifact_digest,
            target_node: d.target_node,
            staged_slot: d.staged_slot,
            restore_slot: d.restore_slot,
            lease_expires_epoch_ms: d.lease_expires_epoch_ms,
            outcome: d.outcome,
            rollback_reason: d.rollback_reason,
            tainted: d.tainted,
            tests: d
                .tests
                .into_iter()
                .map(|t| GateTest {
                    name: t.name,
                    critical: t.critical,
                    state: t.state,
                    duration_ms: t.duration_ms,
                    crate_name: t.crate_name,
                    log_excerpt: t.log_excerpt,
                })
                .collect(),
        };
        self.gates.insert(id.clone(), gate);
        self.refresh_cd(&id);
        true
    }

    fn it_progress(&mut self, e: &ItProgressDto) -> bool {
        self.with_gate(&e.deployment_id, |g| {
            if let Some(t) = g.tests.iter_mut().find(|t| t.name == e.test_name) {
                t.critical = e.critical;
                t.state = e.state;
                t.duration_ms = e.duration_ms;
            } else {
                g.tests.push(GateTest {
                    name: e.test_name.clone(),
                    critical: e.critical,
                    state: e.state,
                    duration_ms: e.duration_ms,
                    crate_name: None,
                    log_excerpt: None,
                });
            }
        })
    }

    fn with_gate(&mut self, id: &str, f: impl FnOnce(&mut DeploymentGate)) -> bool {
        let Some(g) = self.gates.get_mut(id) else {
            return false;
        };
        let before = g.clone();
        f(g);
        let changed = *g != before;
        if changed {
            self.refresh_cd(id);
        }
        changed
    }

    /// Propagate a gate into its history entry and the repo's CD score.
    fn refresh_cd(&mut self, id: &str) {
        let Some(g) = self.gates.get(id) else {
            return;
        };
        let summary = g.summary();
        if let Some(h) = self.history.get_mut(&g.repo) {
            if let Some(e) = h.iter_mut().find(|e| e.deployment_id == id) {
                *e = summary.clone();
            }
        }
        if self.latest.get(&g.repo).map(String::as_str) == Some(id) {
            if let Some(r) = self.repos.get_mut(&g.repo) {
                let active_slot = match g.outcome {
                    GateOutcome::Promoted => g.staged_slot,
                    GateOutcome::RolledBack | GateOutcome::Tainted => g.restore_slot,
                    GateOutcome::Pending => r.cd.as_ref().map_or(g.restore_slot, |c| c.active_slot),
                };
                r.cd = Some(CdScore {
                    deployment_id: id.to_owned(),
                    active_slot,
                    outcome: summary.outcome,
                    it_passed: summary.it_passed,
                    it_total: summary.it_total,
                });
            }
        }
    }

    /// Alert counts per area.
    #[must_use]
    pub fn badges(&self) -> AreaBadges {
        if let Some((_, v)) = self.offline.as_ref() {
            return badges_of_view(v);
        }
        let ci = u32::try_from(
            self.repos
                .values()
                .filter(|r| {
                    r.status == RepoStatus::Failing
                        || self.topologies.get(&r.id).is_some_and(|t| t.t.stop())
                })
                .count(),
        )
        .unwrap_or(u32::MAX);
        let cd = u32::try_from(
            self.repos
                .values()
                .filter_map(|r| r.cd.as_ref())
                .filter(|c| matches!(c.outcome, GateOutcome::RolledBack | GateOutcome::Tainted))
                .count(),
        )
        .unwrap_or(u32::MAX);
        let fleet =
            u32::try_from(self.nodes.values().filter(|n| !n.online).count()).unwrap_or(u32::MAX);
        AreaBadges {
            overview: ci.saturating_add(cd).saturating_add(fleet),
            ci,
            cd,
            fleet,
        }
    }

    /// Project the store into the view model for the current selection.
    #[must_use]
    pub fn view(&self) -> ViewModel {
        if let Some((saved, v)) = self.offline.as_ref() {
            let mut v = v.clone();
            v.source = DataSource::OfflineCache;
            v.stale_since_epoch_ms = Some(*saved);
            v.selection.clone_from(&self.selection);
            return v;
        }
        let selection = self.selection.clone();
        let topology = self.topology_repo().and_then(|repo| {
            let topo = self.topologies.get(&repo)?;
            let mut t = topo.t.clone();
            t.submodules = self
                .repos
                .get(&repo)
                .map(|r| r.submodules.clone())
                .unwrap_or_default();
            if let Some(sel) = selection.as_ref() {
                if let Some(sub) = sel.submodule.as_ref() {
                    t.submodule = Some(sub.clone());
                    let pinned = self.repos.get(&sel.repo).and_then(|p| {
                        p.submodules
                            .iter()
                            .find(|s| s.path == sub.path)
                            .map(|s| s.pinned_sha.clone())
                    });
                    t.submodule_bump_pending = pinned.is_some_and(|p| {
                        !p.is_empty() && !topo.head_sha.is_empty() && p != topo.head_sha
                    });
                }
            }
            Some(t)
        });
        let parent = selection.as_ref().map(|s| &s.repo);
        let gate = parent
            .and_then(|p| self.latest.get(p))
            .and_then(|id| self.gates.get(id))
            .cloned();
        let history = parent
            .and_then(|p| self.history.get(p))
            .cloned()
            .unwrap_or_default();
        ViewModel {
            source: self.source.clone(),
            repos: self.repos.values().cloned().collect(),
            selection,
            topology,
            gate,
            history,
            nodes: self.nodes.values().cloned().collect(),
            stale_since_epoch_ms: None,
        }
    }
}

fn train_status(t: &Train) -> RepoStatus {
    if t.failed() {
        RepoStatus::Failing
    } else if t
        .stage_states
        .iter()
        .any(|s| matches!(s, StageState::Running | StageState::Pending))
    {
        RepoStatus::Running
    } else {
        RepoStatus::Passing
    }
}

fn badges_of_view(v: &ViewModel) -> AreaBadges {
    let ci = v
        .repos
        .iter()
        .filter(|r| r.status == RepoStatus::Failing)
        .count();
    let cd = v
        .repos
        .iter()
        .filter_map(|r| r.cd.as_ref())
        .filter(|c| matches!(c.outcome, GateOutcome::RolledBack | GateOutcome::Tainted))
        .count();
    let fleet = v.nodes.iter().filter(|n| !n.online).count();
    let c = |n: usize| u32::try_from(n).unwrap_or(u32::MAX);
    AreaBadges {
        overview: c(ci + cd + fleet),
        ci: c(ci),
        cd: c(cd),
        fleet: c(fleet),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percent_conversions_clamp() {
        assert_eq!(pct_to_bp(83.456), 8_346);
        assert_eq!(pct_to_bp(-3.0), 0);
        assert_eq!(pct_to_bp(250.0), 10_000);
        assert_eq!(pct_to_bp(f64::NAN), 0);
        assert_eq!(signed_pct_to_bp(-1.25), -125);
        assert_eq!(signed_pct_to_bp(f64::NAN), 0);
    }

    #[test]
    fn events_for_unknown_targets_are_ignored() {
        let mut s = Store::new(DataSource::Live);
        s.take_dirty();
        assert!(!s.apply(
            Input::LeaseExpired(LeaseExpiredDto {
                deployment_id: "nope".into()
            }),
            0
        ));
        assert!(!s.apply(
            Input::BranchDeleted(BranchDeletedDto {
                owner: "o".into(),
                repo: "r".into(),
                ref_name: "feat/x".into(),
                fabric_rev: 0,
            }),
            0
        ));
        assert!(!s.is_dirty());
        assert_eq!(s.view(), ViewModel::default());
    }
}
