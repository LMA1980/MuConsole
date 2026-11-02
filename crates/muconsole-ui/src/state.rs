// SPDX-License-Identifier: AGPL-3.0-only
//! Interactive console state machine for `C1`–`C6`, keyboard shortcuts,
//! rotary 350 ms settle debounce, `PullToTurn` restart guard, and
//! `Hurmit` ↔ `OpenDyslexicM` reading-font persistence.

use std::collections::BTreeMap;

use muconsole_protocol::{
    Area, CrtFont, PanelId, RepoId, RepoScore, Route, SubmoduleId, UiCommand,
};
use muconsole_theme::instruments::switch::SwitchAction;
use muconsole_theme::interaction::{SwitchEvent, SwitchOutcome, SwitchState};
use muconsole_theme::metrics::{CrtFontMetrics, HURMIT, OPEN_DYSLEXIC};

use crate::client::{ClientUpdate, DataClient};
use crate::router::{area_label, escape_up, parse_hash};

/// `localStorage` key persisting the user's selected CRT reading font.
pub const FONT_STORAGE_KEY: &str = "muconsole.font";
/// Rotary settle window before dispatching [`UiCommand::SelectRepo`].
pub const ROTARY_SETTLE_MS: u64 = 350;
/// Armed confirmation window (ms) for the guarded Trainway `RESTART` control.
pub const RESTART_ARM_WINDOW_MS: u64 = 3_000;

/// Ordered areas on the C1 selector rail.
pub const AREAS: [Area; 4] = [Area::Overview, Area::Ci, Area::Cd, Area::Fleet];

/// Parse `?font=dyslexic|hurmit` or stored `localStorage` value.
#[must_use]
pub fn parse_font_pref(query: &str, stored: Option<&str>) -> CrtFont {
    for pair in query.trim_start_matches('?').split('&') {
        if let Some(("font", val)) = pair.split_once('=') {
            if val.eq_ignore_ascii_case("dyslexic") || val.eq_ignore_ascii_case("opendyslexic") {
                return CrtFont::OpenDyslexic;
            }
            if val.eq_ignore_ascii_case("hurmit") {
                return CrtFont::Hurmit;
            }
        }
    }
    if let Some(s) = stored {
        if s.eq_ignore_ascii_case("dyslexic") || s.eq_ignore_ascii_case("opendyslexic") {
            return CrtFont::OpenDyslexic;
        }
    }
    CrtFont::Hurmit
}

/// Serialize [`CrtFont`] for `localStorage` / query parameters.
#[must_use]
pub const fn font_slug(font: CrtFont) -> &'static str {
    match font {
        CrtFont::Hurmit => "hurmit",
        CrtFont::OpenDyslexic => "dyslexic",
    }
}

/// Complete UI state for the console.
#[derive(Debug)]
pub struct ConsoleState {
    pub client: DataClient,
    pub route: Route,
    pub font: CrtFont,
    pub lazy_dyslexic_loaded: bool,
    pub reduced_motion: bool,
    /// Outer rotary ring index into [`Self::filtered_repos`].
    pub outer_index: usize,
    /// Inner rotary ring index (`0` = `PARENT`, `1..=M` = submodules).
    pub inner_index: usize,
    /// `/` type-ahead filter string.
    pub filter_query: String,
    pub filter_focused: bool,
    /// Pending rotary selection waiting for the 350 ms settle window (`due_ms`).
    pub pending_select_at_ms: Option<u64>,
    /// Phone bottom-sheet rotary visibility.
    pub phone_rotary_open: bool,
    /// C4 Scorecard carousel page index.
    pub carousel_page: u32,
    /// Persistent Trainway siding lane assignments (`ref_name -> lane`).
    pub trainway_lanes: BTreeMap<String, u32>,
    /// C5 Trainway `YARD +N` overflow page index.
    pub yard_page: u32,
    /// C5 Trainway `LIVE` follow toggle.
    pub trainway_live: bool,
    /// C5 guarded `PullToTurn` `RESTART` switch (`NORM` = 0, `RST` = 1).
    pub restart_switch: SwitchState,
    /// Timestamp until which the `RESTART` switch remains armed (`Some(until_ms)`).
    pub restart_armed_until_ms: Option<u64>,
    /// Count of confirmed restarts triggered through the `PullToTurn` guard.
    pub restart_count: u32,
    /// C6 Delivery history strip selection (`0` = live / latest gate).
    pub selected_history_idx: usize,
    /// C6 Delivery selected test row for log excerpt paging.
    pub selected_test_idx: usize,
    /// C6 Delivery log excerpt page cursor.
    pub log_page: u32,
    /// Fleet telemetry page cursor (`0` = node locks, `1` = sandbox telemetry).
    pub fleet_page: u32,
    /// Polite `aria-live` announcement text.
    pub aria_polite: String,
    /// Assertive `aria-live` alert text.
    pub aria_assertive: String,
}

impl Default for ConsoleState {
    fn default() -> Self {
        Self::new()
    }
}

impl ConsoleState {
    #[must_use]
    pub fn new() -> Self {
        Self {
            client: DataClient::new(),
            route: Route {
                area: Area::Overview,
                repo: None,
                submodule: None,
                branch: None,
                job: None,
            },
            font: CrtFont::Hurmit,
            lazy_dyslexic_loaded: false,
            reduced_motion: false,
            outer_index: 0,
            inner_index: 0,
            filter_query: String::new(),
            filter_focused: false,
            pending_select_at_ms: None,
            phone_rotary_open: false,
            carousel_page: 0,
            trainway_lanes: BTreeMap::new(),
            yard_page: 0,
            trainway_live: true,
            restart_switch: SwitchState::new(SwitchAction::PullToTurn, 2, 0),
            restart_armed_until_ms: None,
            restart_count: 0,
            selected_history_idx: 0,
            selected_test_idx: 0,
            log_page: 0,
            fleet_page: 0,
            aria_polite: String::new(),
            aria_assertive: String::new(),
        }
    }

    /// Initialize route and font preference from browser location + storage.
    pub fn init_from_env(&mut self, hash: &str, search: &str, stored_font: Option<&str>) {
        self.route = parse_hash(hash);
        let f = parse_font_pref(search, stored_font);
        self.set_font(f);
    }

    /// Active font metrics (`HURMIT` or `OPEN_DYSLEXIC`).
    #[must_use]
    pub const fn font_metrics(&self) -> &'static CrtFontMetrics {
        match self.font {
            CrtFont::Hurmit => &HURMIT,
            CrtFont::OpenDyslexic => &OPEN_DYSLEXIC,
        }
    }

    /// Set the active reading font, lazy-loading `OpenDyslexicM` on first use.
    pub fn set_font(&mut self, font: CrtFont) -> UiCommand {
        self.font = font;
        if font == CrtFont::OpenDyslexic {
            self.lazy_dyslexic_loaded = true;
        }
        self.aria_polite = format!("Font set to {}", self.font_metrics().family);
        UiCommand::SetCrtFont(font)
    }

    /// Toggle between `Hurmit` and `OpenDyslexicM`.
    pub fn toggle_font(&mut self) -> UiCommand {
        let next = match self.font {
            CrtFont::Hurmit => CrtFont::OpenDyslexic,
            CrtFont::OpenDyslexic => CrtFont::Hurmit,
        };
        self.set_font(next)
    }

    /// Feed a worker frame into [`DataClient`] and synchronize selection/route.
    pub fn on_worker_frame(&mut self, frame: &[u8]) -> ClientUpdate {
        let upd = self.client.on_frame(frame);
        match &upd {
            ClientUpdate::View => {
                if let Some(v) = self.client.view() {
                    if let Some(sel) = v.selection.as_ref() {
                        if self.route.repo.is_none() {
                            self.route.repo = Some(sel.repo.clone());
                            self.route.submodule.clone_from(&sel.submodule);
                        }
                    }
                    if v.topology
                        .as_ref()
                        .is_some_and(muconsole_protocol::RepoTopology::stop)
                    {
                        self.aria_assertive = "STOP: CI stage failure detected".into();
                    }
                }
                self.sync_rotary_from_route();
            }
            ClientUpdate::ReloadRequired { expected, found } => {
                self.aria_assertive = crate::client::reload_banner_text(*expected, *found);
            }
            _ => {}
        }
        upd
    }

    /// Repositories matching the current `/` type-ahead filter.
    #[must_use]
    pub fn filtered_repos(&self) -> Vec<RepoScore> {
        let Some(v) = self.client.view() else {
            return Vec::new();
        };
        let q = self.filter_query.trim().to_ascii_lowercase();
        if q.is_empty() {
            return v.repos.clone();
        }
        v.repos
            .iter()
            .filter(|r| {
                format!("{}/{}", r.id.owner, r.id.repo)
                    .to_ascii_lowercase()
                    .contains(&q)
            })
            .cloned()
            .collect()
    }

    /// Scorecards ordered for C4 Carousel: selected parent repo first, then
    /// its submodules in `.gitmodules` order, then remaining repositories.
    #[must_use]
    pub fn ordered_carousel_repos(&self) -> Vec<RepoScore> {
        let Some(v) = self.client.view() else {
            return Vec::new();
        };
        let mut out = Vec::with_capacity(v.repos.len());
        let parent_id = self
            .route
            .repo
            .as_ref()
            .or_else(|| v.selection.as_ref().map(|s| &s.repo));
        if let Some(pid) = parent_id {
            if let Some(parent) = v.repos.iter().find(|r| &r.id == pid) {
                out.push(parent.clone());
                for sub in &parent.submodules {
                    if let Some(sr) = v.repos.iter().find(|r| r.id == sub.repo) {
                        if !out.iter().any(|x| x.id == sr.id) {
                            out.push(sr.clone());
                        }
                    }
                }
            }
        }
        for r in &v.repos {
            if !out.iter().any(|x| x.id == r.id) {
                out.push(r.clone());
            }
        }
        out
    }

    /// Synchronize `outer_index` and `inner_index` with `self.route`.
    pub fn sync_rotary_from_route(&mut self) {
        let repos = self.filtered_repos();
        if repos.is_empty() {
            self.outer_index = 0;
            self.inner_index = 0;
            return;
        }
        if let Some(target) = self.route.repo.as_ref() {
            if let Some(pos) = repos.iter().position(|r| &r.id == target) {
                self.outer_index = pos;
            }
        }
        self.outer_index = self.outer_index.min(repos.len() - 1);
        let parent = &repos[self.outer_index];
        self.inner_index = match self.route.submodule.as_ref() {
            None => 0,
            Some(sub) => parent
                .submodules
                .iter()
                .position(|s| s.path == sub.path)
                .map_or(0, |i| i + 1),
        };
    }

    /// Rotate the outer (repository) ring by `delta` detents and start the
    /// 350 ms settle timer.
    pub fn rotate_outer(&mut self, delta: isize, now_ms: u64) {
        let repos = self.filtered_repos();
        if repos.is_empty() {
            return;
        }
        let len = isize::try_from(repos.len()).unwrap_or(1);
        let cur = isize::try_from(self.outer_index).unwrap_or(0);
        let next = (cur + delta).rem_euclid(len);
        self.outer_index = usize::try_from(next).unwrap_or(0);
        self.inner_index = 0;
        let chosen = &repos[self.outer_index];
        self.route.repo = Some(chosen.id.clone());
        self.route.submodule = None;
        self.pending_select_at_ms = Some(now_ms.saturating_add(ROTARY_SETTLE_MS));
        self.aria_polite = format!("Selected repository {}/{}", chosen.id.owner, chosen.id.repo);
    }

    /// Rotate the inner (`PARENT` + submodules) ring by `delta` detents and
    /// start the 350 ms settle timer.
    pub fn rotate_inner(&mut self, delta: isize, now_ms: u64) {
        let repos = self.filtered_repos();
        let Some(parent) = repos.get(self.outer_index) else {
            return;
        };
        let total = parent.submodules.len() + 1;
        let len = isize::try_from(total).unwrap_or(1);
        let cur = isize::try_from(self.inner_index).unwrap_or(0);
        let next = (cur + delta).rem_euclid(len);
        self.inner_index = usize::try_from(next).unwrap_or(0);
        self.route.repo = Some(parent.id.clone());
        self.route.submodule = if self.inner_index == 0 {
            None
        } else {
            parent
                .submodules
                .get(self.inner_index - 1)
                .map(|s| SubmoduleId {
                    path: s.path.clone(),
                })
        };
        self.pending_select_at_ms = Some(now_ms.saturating_add(ROTARY_SETTLE_MS));
        let label = self
            .route
            .submodule
            .as_ref()
            .map_or("PARENT", |s| s.path.as_str());
        self.aria_polite = format!("Selected submodule {label}");
    }

    /// Update the `/` type-ahead filter and snap `outer_index` to the first
    /// matching repository.
    pub fn set_filter_query(&mut self, query: &str, now_ms: u64) {
        self.filter_query = query.to_owned();
        self.outer_index = 0;
        self.inner_index = 0;
        let repos = self.filtered_repos();
        if let Some(first) = repos.first() {
            self.route.repo = Some(first.id.clone());
            self.route.submodule = None;
            self.pending_select_at_ms = Some(now_ms.saturating_add(ROTARY_SETTLE_MS));
        }
    }

    /// Select an area on C1 (`OVR`, `CI`, `CD`, `FLT`).
    pub fn select_area(&mut self, area: Area) -> UiCommand {
        self.route.area = area;
        self.aria_polite = format!("Area {}", area_label(area));
        UiCommand::Subscribe(self.route.clone())
    }

    /// Step the C4 Scorecard carousel by `delta` pages.
    pub fn step_carousel(&mut self, delta: i32, cards_per_page: usize) -> UiCommand {
        let total_repos = self.ordered_carousel_repos().len();
        let per = cards_per_page.max(1);
        let pages = i32::try_from(total_repos.div_ceil(per).max(1)).unwrap_or(1);
        let cur = i32::try_from(self.carousel_page).unwrap_or(0);
        let next = (cur + delta).rem_euclid(pages);
        self.carousel_page = u32::try_from(next).unwrap_or(0);
        UiCommand::Page {
            panel: PanelId::Carousel,
            cursor: self.carousel_page,
        }
    }

    /// Feed an event into the guarded `PullToTurn` Trainway `RESTART` switch.
    ///
    /// Rotating to position `1` (`RST`) requires either a >= 300 ms pull hold
    /// or `shift: true`. The first valid rotation arms the control for
    /// [`RESTART_ARM_WINDOW_MS`]; confirming within that window dispatches
    /// [`UiCommand::Retry`].
    pub fn on_restart_switch(&mut self, ev: SwitchEvent, now_ms: u64) -> Option<UiCommand> {
        if let Some(until) = self.restart_armed_until_ms {
            if now_ms > until {
                self.restart_armed_until_ms = None;
            }
        }
        match self.restart_switch.apply(ev) {
            SwitchOutcome::Fired(1) => {
                if self
                    .restart_armed_until_ms
                    .is_some_and(|until| now_ms <= until)
                {
                    self.restart_armed_until_ms = None;
                    self.restart_count = self.restart_count.saturating_add(1);
                    self.aria_assertive = "Trainway restart confirmed".into();
                    Some(UiCommand::Retry)
                } else {
                    self.restart_armed_until_ms =
                        Some(now_ms.saturating_add(RESTART_ARM_WINDOW_MS));
                    self.aria_polite = "Restart armed: confirm within 3 seconds to execute".into();
                    None
                }
            }
            SwitchOutcome::Refused => {
                self.aria_polite = "Pull and hold 300ms (or Shift) before turning RESTART".into();
                None
            }
            _ => None,
        }
    }

    /// Handle a keyboard shortcut and return any resulting [`UiCommand`].
    pub fn on_key(&mut self, key: &str, shift: bool, now_ms: u64) -> Option<UiCommand> {
        match key {
            "[" => {
                let idx = AREAS
                    .iter()
                    .position(|a| *a == self.route.area)
                    .unwrap_or(0);
                let prev = AREAS[(idx + AREAS.len() - 1) % AREAS.len()];
                Some(self.select_area(prev))
            }
            "]" => {
                let idx = AREAS
                    .iter()
                    .position(|a| *a == self.route.area)
                    .unwrap_or(0);
                let next = AREAS[(idx + 1) % AREAS.len()];
                Some(self.select_area(next))
            }
            "Escape" => {
                if self.phone_rotary_open {
                    self.phone_rotary_open = false;
                    self.aria_polite = "Closed repository selector sheet".into();
                    None
                } else if !self.filter_query.is_empty() || self.filter_focused {
                    self.filter_query.clear();
                    self.filter_focused = false;
                    self.aria_polite = "Cleared repository filter".into();
                    None
                } else {
                    self.route = escape_up(&self.route);
                    self.aria_polite = "Navigated up one level".into();
                    Some(UiCommand::Subscribe(self.route.clone()))
                }
            }
            "/" => {
                self.filter_focused = true;
                self.aria_polite = "Type-ahead repository filter focused".into();
                None
            }
            "ArrowUp" => {
                if shift {
                    self.rotate_inner(-1, now_ms);
                } else {
                    self.rotate_outer(-1, now_ms);
                }
                None
            }
            "ArrowDown" => {
                if shift {
                    self.rotate_inner(1, now_ms);
                } else {
                    self.rotate_outer(1, now_ms);
                }
                None
            }
            "ArrowLeft" => Some(self.step_carousel(-1, 3)),
            "ArrowRight" => Some(self.step_carousel(1, 3)),
            "f" | "F" if !self.filter_focused => Some(self.toggle_font()),
            _ => None,
        }
    }

    /// Advance timers: emit [`UiCommand::SelectRepo`] once the 350 ms rotary
    /// settle window elapses, and expire the 3 s restart arm window.
    pub fn tick(&mut self, now_ms: u64) -> Vec<UiCommand> {
        let mut cmds = Vec::new();
        if let Some(until) = self.restart_armed_until_ms {
            if now_ms > until {
                self.restart_armed_until_ms = None;
            }
        }
        if let Some(due) = self.pending_select_at_ms {
            if now_ms >= due {
                self.pending_select_at_ms = None;
                let repo = self
                    .route
                    .repo
                    .clone()
                    .unwrap_or_else(|| RepoId::new("fixture", "superproject"));
                cmds.push(UiCommand::SelectRepo {
                    repo,
                    submodule: self.route.submodule.clone(),
                });
            }
        }
        cmds
    }
}
