// SPDX-License-Identifier: AGPL-3.0-only
//! Semantic SVG component renderer (`C1`–`C6`, phone bottom-sheet rotary,
//! Fleet telemetry, and HTML accessibility shell) driven through [`Skin`].

use muconsole_layout::trainway::allocate_lanes;
use muconsole_layout::{
    solve, Breakpoint, Card, Carousel, Frames, Panel as LayoutPanel, PanelKind, SafeArea, Viewport,
};
use muconsole_protocol::{
    Area, ConnState, CrtFont, GateOutcome, GateTest, PrState, RepoScore, RepoStatus, RepoTopology,
    SidingPhase, StageState, TestState,
};
use muconsole_theme::instruments::switch::SwitchAction;
use muconsole_theme::instruments::Rect;
use muconsole_theme::skin::{
    Lamp, Panel, Readout, Rotary, Screen, ScreenRow, ScreenTrack, Selector, SelectorOption,
    SemanticTone, SignalGlyph, Skin,
};
use muconsole_theme::svg::SvgWriter;

use crate::router::{area_label, breadcrumb_segments};
use crate::state::{font_slug, ConsoleState};

const fn stage_glyph_and_tone(state: StageState) -> (SignalGlyph, SemanticTone) {
    match state {
        StageState::Passed => (SignalGlyph::BlockPassed, SemanticTone::Pass),
        StageState::Running => (SignalGlyph::BlockRunning, SemanticTone::Warn),
        StageState::Failed => (SignalGlyph::BlockFailed, SemanticTone::Fail),
        StageState::Pending | StageState::Skipped => {
            (SignalGlyph::BlockPending, SemanticTone::Neutral)
        }
    }
}

const fn test_glyph_and_tone(state: TestState) -> (SignalGlyph, SemanticTone) {
    match state {
        TestState::Passed => (SignalGlyph::TestPassed, SemanticTone::Pass),
        TestState::Running => (SignalGlyph::TestRunning, SemanticTone::Warn),
        TestState::Failed => (SignalGlyph::BlockFailed, SemanticTone::Fail),
        TestState::Pending => (SignalGlyph::TestPending, SemanticTone::Neutral),
    }
}

/// Clamp `rect` strictly inside `bounds`.
fn clamp_rect(rect: Rect, bounds: Rect) -> Rect {
    let x = rect.x.clamp(bounds.x, bounds.right());
    let y = rect.y.clamp(bounds.y, bounds.bottom());
    let w = rect.w.min((bounds.right() - x).max(0.0)).max(0.0);
    let h = rect.h.min((bounds.bottom() - y).max(0.0)).max(0.0);
    Rect::new(x, y, w, h)
}

/// Render the full MuConsole SVG document for `viewport`, `safe`, `state`, and `skin`.
#[must_use]
pub fn render_console(
    skin: &dyn Skin,
    state: &ConsoleState,
    viewport: Viewport,
    safe: SafeArea,
) -> String {
    let font = state.font_metrics();
    let frames = solve(viewport, safe, state.route.area, font);
    let mut w = SvgWriter::document(viewport.width, viewport.height);
    skin.begin_frame(&mut w, viewport.width, viewport.height);

    render_c1_nav(skin, &mut w, state, &frames);
    render_c2_top_bar(skin, &mut w, state, &frames);

    if let Some(carousel) = frames.carousel.as_ref() {
        render_c4_carousel(skin, &mut w, state, &frames, carousel);
    }

    if let Some(rotary_rect) = frames.rotary {
        render_c3_rotary(skin, &mut w, state, &frames, rotary_rect);
    }

    render_area_panels(skin, &mut w, state, &frames);

    if frames.breakpoint == Breakpoint::Phone && state.phone_rotary_open {
        render_c3_phone_sheet(skin, &mut w, state, &frames);
    }

    w.finish_closing()
}

/// Render `C1` Area Navigation selector (`OVR`, `CI`, `CD`, `FLT`).
fn render_c1_nav(skin: &dyn Skin, w: &mut SvgWriter, state: &ConsoleState, frames: &Frames) {
    let r = clamp_rect(frames.nav, frames.canvas);
    if r.w <= 0.0 || r.h <= 0.0 {
        return;
    }
    skin.panel(
        w,
        &Panel {
            id: "c1-nav-panel",
            title: if frames.breakpoint == Breakpoint::Phone {
                ""
            } else {
                "NAV"
            },
            rect: r,
            role: Some("navigation"),
            aria_label: Some("Area Navigation"),
        },
    );

    let badges = state.client.badges();
    let opts = [
        SelectorOption {
            id: "ovr",
            label: "OVERVIEW",
            badge: (badges.overview > 0).then_some(badges.overview),
            tone: SemanticTone::Info,
        },
        SelectorOption {
            id: "ci",
            label: "CI",
            badge: (badges.ci > 0).then_some(badges.ci),
            tone: if badges.ci > 0 {
                SemanticTone::Fail
            } else {
                SemanticTone::Pass
            },
        },
        SelectorOption {
            id: "cd",
            label: "CD",
            badge: (badges.cd > 0).then_some(badges.cd),
            tone: if badges.cd > 0 {
                SemanticTone::Warn
            } else {
                SemanticTone::Pass
            },
        },
        SelectorOption {
            id: "flt",
            label: "FLEET",
            badge: (badges.fleet > 0).then_some(badges.fleet),
            tone: if badges.fleet > 0 {
                SemanticTone::Fail
            } else {
                SemanticTone::Info
            },
        },
    ];

    let selected = match state.route.area {
        Area::Overview => 0,
        Area::Ci => 1,
        Area::Cd => 2,
        Area::Fleet => 3,
    };

    let sel_rect = if frames.breakpoint == Breakpoint::Phone {
        clamp_rect(r.inset(2.0, 4.0, 2.0, 4.0), r)
    } else {
        clamp_rect(
            Rect::new(
                r.x + 4.0,
                r.y + 22.0,
                (r.w - 8.0).max(16.0),
                (r.h - 28.0).clamp(44.0, 220.0),
            ),
            r,
        )
    };

    skin.selector(
        w,
        &Selector {
            control_id: "c1-area-selector",
            label: "Area Navigation Selector",
            rect: sel_rect,
            action: SwitchAction::Maintained,
            options: &opts,
            selected,
            pulled: false,
            horizontal: frames.breakpoint == Breakpoint::Phone,
        },
    );
}

/// Render `C2` Top Bar: Breadcrumb segments, `FONT` switch, and status annunciators.
#[allow(clippy::cast_precision_loss)]
fn render_c2_top_bar(skin: &dyn Skin, w: &mut SvgWriter, state: &ConsoleState, frames: &Frames) {
    let r = clamp_rect(frames.top_bar, frames.canvas);
    if r.w <= 0.0 || r.h <= 0.0 {
        return;
    }
    skin.panel(
        w,
        &Panel {
            id: "c2-top-bar",
            title: "",
            rect: r,
            role: Some("banner"),
            aria_label: Some("Breadcrumb and Status Bar"),
        },
    );

    let inner = clamp_rect(r.inset(4.0, 6.0, 4.0, 6.0), r);
    let right_reserve = if frames.breakpoint == Breakpoint::Phone {
        218.0_f64.min(inner.w * 0.58)
    } else {
        436.0_f64.min(inner.w * 0.52)
    };

    // Breadcrumb segments on the left.
    let crumbs = breadcrumb_segments(&state.route, frames.breakpoint);
    let crumb_area_w = (inner.w - right_reserve - 6.0).max(40.0);
    let n_crumbs = crumbs.len().max(1);
    let cw = (crumb_area_w / n_crumbs as f64).max(24.0);
    for (i, c) in crumbs.iter().enumerate() {
        let cr = clamp_rect(
            Rect::new(
                inner.x + i as f64 * cw,
                inner.y,
                (cw - 4.0).max(16.0),
                inner.h,
            ),
            inner,
        );
        let (lbl, val) = if frames.breakpoint == Breakpoint::Phone {
            match c.id {
                "c2-area" => ("", c.value.as_str()),
                "c2-repo" => (
                    "",
                    c.value
                        .rsplit_once('/')
                        .map_or(c.value.as_str(), |(_, repo_name)| repo_name),
                ),
                _ => (c.label, c.value.as_str()),
            }
        } else {
            (c.label, c.value.as_str())
        };
        skin.readout(
            w,
            &Readout {
                control_id: c.id,
                label: lbl,
                value: val,
                rect: cr,
                tone: SemanticTone::Info,
                interactive: c.id != "c2-ellipsis",
            },
        );
    }

    let rx = (inner.right() - right_reserve).max(inner.x);
    if frames.breakpoint == Breakpoint::Phone {
        let trigger_w = 70.0_f64.min(right_reserve * 0.34).max(48.0);
        let trigger_rect = clamp_rect(
            Rect::new(
                rx,
                r.y,
                trigger_w,
                r.h.max(44.0).min(frames.canvas.bottom() - r.y),
            ),
            frames.canvas,
        );
        skin.readout(
            w,
            &Readout {
                control_id: "c3-phone-trigger",
                label: "",
                value: "▾ REPO",
                rect: trigger_rect,
                tone: SemanticTone::Warn,
                interactive: true,
            },
        );
        let font_rect = clamp_rect(
            Rect::new(
                (trigger_rect.right() + 4.0).min(r.right() - 48.0),
                r.y,
                (r.right() - trigger_rect.right() - 4.0).max(48.0),
                r.h.max(44.0).min(frames.canvas.bottom() - r.y),
            ),
            frames.canvas,
        );
        render_font_switch(skin, w, state, font_rect);
    } else {
        let view = state.client.view();
        let stale = view.is_some_and(|v| v.stale_since_epoch_ms.is_some());
        let offline = matches!(state.client.conn(), Some(ConnState::Offline));
        let stop = view
            .and_then(|v| v.topology.as_ref())
            .is_some_and(RepoTopology::stop);
        let reload = state.client.reload_required().is_some();

        let font_w = 212.0_f64.min(right_reserve * 0.50).max(80.0);
        let lamp_w = ((right_reserve - font_w - 16.0) / 3.0).clamp(48.0, 72.0);
        let lamps = [
            (
                "lamp-stale",
                if reload { "RELOAD" } else { "STALE" },
                stale || reload,
                SemanticTone::Warn,
            ),
            ("lamp-offline", "OFFLINE", offline, SemanticTone::Fail),
            ("lamp-stop", "STOP", stop, SemanticTone::Fail),
        ];
        for (idx, (id, lbl, lit, tone)) in lamps.into_iter().enumerate() {
            let lr = clamp_rect(
                Rect::new(rx + idx as f64 * (lamp_w + 4.0), inner.y, lamp_w, inner.h),
                inner,
            );
            skin.lamp(
                w,
                &Lamp {
                    control_id: id,
                    label: lbl,
                    rect: lr,
                    lit,
                    tone,
                },
            );
        }

        let font_x = (inner.right() - font_w).max(rx);
        let font_rect = clamp_rect(
            Rect::new(font_x, inner.y, (inner.right() - font_x).max(48.0), inner.h),
            inner,
        );
        render_font_switch(skin, w, state, font_rect);
    }
}

fn render_font_switch(skin: &dyn Skin, w: &mut SvgWriter, state: &ConsoleState, rect: Rect) {
    let opts = [
        SelectorOption {
            id: "hurmit",
            label: "Standard font",
            badge: None,
            tone: SemanticTone::Info,
        },
        SelectorOption {
            id: "dyslexic",
            label: "Dyslexia-friendly font",
            badge: None,
            tone: SemanticTone::Pass,
        },
    ];
    let selected = match state.font {
        CrtFont::Hurmit => 0,
        CrtFont::OpenDyslexic => 1,
    };
    skin.selector(
        w,
        &Selector {
            control_id: "c2-font-switch",
            label: "Use dyslexia-friendly font",
            rect,
            action: SwitchAction::Maintained,
            options: &opts,
            selected,
            pulled: false,
            horizontal: true,
        },
    );
}

/// Render `C3` Dual-Concentric Repository & Submodule Rotary Selector.
fn render_c3_rotary(
    skin: &dyn Skin,
    w: &mut SvgWriter,
    state: &ConsoleState,
    frames: &Frames,
    rotary_rect: Rect,
) {
    let r = clamp_rect(rotary_rect, frames.canvas);
    let repos = state.filtered_repos();
    let outer_total = repos.len();
    let current_repo = repos.get(state.outer_index);
    let outer_label = current_repo.map_or("—", |x| x.id.repo.as_str());
    let inner_total = current_repo.map_or(1, |x| x.submodules.len() + 1);
    let inner_label = if state.inner_index == 0 {
        "PARENT"
    } else {
        current_repo
            .and_then(|x| x.submodules.get(state.inner_index - 1))
            .map_or("PARENT", |s| s.path.as_str())
    };

    skin.rotary(
        w,
        &Rotary {
            control_id: "c3-rotary",
            rect: r,
            outer_label,
            outer_index: state.outer_index,
            outer_total,
            inner_label,
            inner_index: state.inner_index,
            inner_total,
            filter_query: &state.filter_query,
            settling: state.pending_select_at_ms.is_some(),
        },
    );
}

/// Render the phone bottom-sheet rotary modal with `>= 44×44 pt` touch targets.
fn render_c3_phone_sheet(
    skin: &dyn Skin,
    w: &mut SvgWriter,
    state: &ConsoleState,
    frames: &Frames,
) {
    let sheet_h = (frames.canvas.h * 0.52).clamp(220.0, frames.canvas.h);
    let sheet = clamp_rect(
        Rect::new(
            frames.canvas.x,
            (frames.canvas.bottom() - sheet_h).max(frames.canvas.y),
            frames.canvas.w,
            sheet_h,
        ),
        frames.canvas,
    );
    skin.panel(
        w,
        &Panel {
            id: "c3-phone-sheet",
            title: "REPOSITORY & SUBMODULE SELECTOR",
            rect: sheet,
            role: Some("dialog"),
            aria_label: Some("Repository and Submodule Selector Sheet"),
        },
    );

    let rotary_rect = clamp_rect(
        Rect::new(
            sheet.x + 8.0,
            sheet.y + 24.0,
            (sheet.w - 16.0).max(44.0),
            (sheet.h - 86.0).max(80.0),
        ),
        sheet,
    );
    let repos = state.filtered_repos();
    let current_repo = repos.get(state.outer_index);
    let outer_label = current_repo.map_or("—", |x| x.id.repo.as_str());
    let inner_total = current_repo.map_or(1, |x| x.submodules.len() + 1);
    let inner_label = if state.inner_index == 0 {
        "PARENT"
    } else {
        current_repo
            .and_then(|x| x.submodules.get(state.inner_index - 1))
            .map_or("PARENT", |s| s.path.as_str())
    };
    skin.rotary(
        w,
        &Rotary {
            control_id: "c3-rotary",
            rect: rotary_rect,
            outer_label,
            outer_index: state.outer_index,
            outer_total: repos.len(),
            inner_label,
            inner_index: state.inner_index,
            inner_total,
            filter_query: &state.filter_query,
            settling: state.pending_select_at_ms.is_some(),
        },
    );

    let btn_y = (sheet.bottom() - 52.0).max(sheet.y + 4.0);
    let btn_h = 46.0_f64.min((sheet.bottom() - btn_y).max(44.0));
    let btn_w = ((sheet.w - 24.0) / 3.0).max(44.0);
    let buttons = [
        ("c3-phone-outer-next", "REPO +"),
        ("c3-phone-inner-next", "SUB +"),
        ("c3-phone-close", "CLOSE"),
    ];
    for (i, (id, lbl)) in buttons.into_iter().enumerate() {
        #[allow(clippy::cast_precision_loss)]
        let bx = sheet.x + 6.0 + i as f64 * (btn_w + 4.0);
        let br = clamp_rect(Rect::new(bx, btn_y, btn_w, btn_h), sheet);
        skin.readout(
            w,
            &Readout {
                control_id: id,
                label: "",
                value: lbl,
                rect: br,
                tone: SemanticTone::Info,
                interactive: true,
            },
        );
    }
}

/// Render `C4` Twin-Screen Repository Scorecard Carousel using the solved [`Carousel`] rects.
fn render_c4_carousel(
    skin: &dyn Skin,
    w: &mut SvgWriter,
    state: &ConsoleState,
    frames: &Frames,
    carousel: &Carousel,
) {
    let repos = state.ordered_carousel_repos();
    let cards_per_page = carousel.cards.len().max(1);
    let total_pages = repos.len().div_ceil(cards_per_page).max(1);
    let page = (state.carousel_page as usize).min(total_pages - 1);
    let start = page * cards_per_page;
    let slice = repos
        .get(start..(start + cards_per_page).min(repos.len()))
        .unwrap_or(&[]);

    if let Some(sw_rect) = carousel.switch {
        let step_rect = clamp_rect(sw_rect, frames.canvas);
        let step_opts = [
            SelectorOption {
                id: "prev",
                label: "◀ PREV",
                badge: None,
                tone: SemanticTone::Info,
            },
            SelectorOption {
                id: "next",
                label: "NEXT ▶",
                badge: None,
                tone: SemanticTone::Info,
            },
        ];
        skin.selector(
            w,
            &Selector {
                control_id: "c4-step-switch",
                label: "Carousel Step Switch",
                rect: step_rect,
                action: SwitchAction::SpringReturn,
                options: &step_opts,
                selected: 1,
                pulled: false,
                horizontal: false,
            },
        );
    }

    let dots_rect = clamp_rect(carousel.dots, frames.canvas);
    let mut dot_glyphs = String::new();
    for p in 0..total_pages {
        if p > 0 {
            dot_glyphs.push(' ');
        }
        dot_glyphs.push(if p == page { '●' } else { '○' });
    }
    let dots_val = format!(
        "{}/{total_pages}  {dot_glyphs}  (Page {} of {total_pages})",
        page + 1,
        page + 1
    );
    skin.readout(
        w,
        &Readout {
            control_id: "c4-page-dots",
            label: "PG",
            value: &dots_val,
            rect: dots_rect,
            tone: SemanticTone::Info,
            interactive: false,
        },
    );

    let font = state.font_metrics();
    for (idx, card_slot) in carousel.cards.iter().enumerate() {
        let plate_rect = clamp_rect(card_slot.plate, frames.canvas);
        let repo = slice.get(idx);
        let title = repo.map_or_else(
            || "EMPTY SLOT".to_owned(),
            |r| {
                let sub = if r.submodules.is_empty() {
                    ""
                } else {
                    " · SUBMODULES"
                };
                format!("{}/{}{sub}", r.id.owner, r.id.repo)
            },
        );
        skin.panel(
            w,
            &Panel {
                id: "c4-card-plate",
                title: &title,
                rect: plate_rect,
                role: Some("region"),
                aria_label: Some(&title),
            },
        );
        if let Some(r) = repo {
            render_twin_scorecard(skin, w, r, idx, card_slot, frames.canvas, font.family);
        }
    }
}

#[allow(clippy::too_many_lines)]
fn render_twin_scorecard(
    skin: &dyn Skin,
    w: &mut SvgWriter,
    repo: &RepoScore,
    idx: usize,
    card: &Card,
    canvas: Rect,
    font_family: &str,
) {
    let left_rect = clamp_rect(card.ci_crt, canvas);
    let right_rect = clamp_rect(card.cd_crt, canvas);

    let (left_tone, st_tag) = match repo.status {
        RepoStatus::Passing => (SemanticTone::Pass, "PASS"),
        RepoStatus::Running => (SemanticTone::Warn, "RUN "),
        RepoStatus::Failing => (SemanticTone::Fail, "FAIL"),
        RepoStatus::Unknown => (SemanticTone::Neutral, "UNK "),
    };
    let grade = match repo.score {
        90..=100 => 'A',
        80..=89 => 'B',
        70..=79 => 'C',
        1..=69 => 'D',
        _ => '-',
    };
    let l0 = "CI · QUALITY".to_owned();
    let l1 = format!("{st_tag} {:>2}/100 {grade}", repo.score.min(99));
    let l2 = format!(
        "COV {:>2}% {:+.1}%",
        (repo.coverage_bp / 100).min(99),
        f64::from(repo.bench_delta_bp) / 100.0
    );
    let l3 = format!(
        "SUB {} #{}",
        repo.submodules.len(),
        repo.latest_job_id.unwrap_or(0)
    );
    let l4 = match repo.status {
        RepoStatus::Passing => "TRND: ▅▆▆▇▇██".to_owned(),
        RepoStatus::Running => "TRND: ▄▅▆▆▇▆▊".to_owned(),
        RepoStatus::Failing => "TRND: ▇▆▅▄▃▂▁".to_owned(),
        RepoStatus::Unknown => "TRND: ───────".to_owned(),
    };
    let left_lines = [
        ScreenRow {
            text: &l0,
            tone: SemanticTone::Info,
            signal: None,
            critical: false,
            control_id: None,
            selected: false,
        },
        ScreenRow {
            text: &l1,
            tone: left_tone,
            signal: None,
            critical: false,
            control_id: None,
            selected: false,
        },
        ScreenRow {
            text: &l2,
            tone: left_tone,
            signal: None,
            critical: false,
            control_id: None,
            selected: false,
        },
        ScreenRow {
            text: &l3,
            tone: SemanticTone::Neutral,
            signal: None,
            critical: false,
            control_id: None,
            selected: false,
        },
        ScreenRow {
            text: &l4,
            tone: left_tone,
            signal: None,
            critical: false,
            control_id: None,
            selected: false,
        },
    ];
    let left_id = format!("c4-card-{idx}-left");
    skin.screen(
        w,
        &Screen {
            control_id: &left_id,
            title: &l0,
            rect: left_rect,
            font_family,
            font_px: card.ci_grid.font_px,
            cols: card.ci_grid.cols,
            rows: card.ci_grid.rows,
            lines: &left_lines,
            stations: &[],
            tracks: &[],
            page: 0,
            total_pages: 1,
        },
    );

    let (r0, r1, r2, r3, r4, cd_tone) = match repo.cd.as_ref() {
        Some(cd) => {
            let (tone, gate_tag, crit_tag, ttl_tag) = match cd.outcome {
                GateOutcome::Promoted => (SemanticTone::Pass, "PROMOTED", "OK", "LEASE: LIVE"),
                GateOutcome::Pending => (SemanticTone::Warn, "PENDING", "OK", "LEASE: 142s"),
                GateOutcome::RolledBack => (SemanticTone::Fail, "ROLLBACK", "FL", "LEASE: REV"),
                GateOutcome::Tainted => (SemanticTone::Fail, "TAINTED", "FL", "LEASE: HALT"),
            };
            (
                format!("CD · {}", cd.deployment_id),
                format!(
                    "SLOT {:?}->{:?} STG",
                    cd.active_slot,
                    cd.active_slot.other()
                ),
                format!("GATE: {gate_tag}"),
                format!("IT {}/{} CRIT:{crit_tag}", cd.it_passed, cd.it_total),
                ttl_tag.to_owned(),
                tone,
            )
        }
        None => (
            "CD · NO DEPLOY".to_owned(),
            "SLOT: NONE".to_owned(),
            "GATE: IDLE".to_owned(),
            "IT 0/0 CRIT:--".to_owned(),
            "LEASE: —".to_owned(),
            SemanticTone::Neutral,
        ),
    };
    let right_lines = [
        ScreenRow {
            text: &r0,
            tone: cd_tone,
            signal: None,
            critical: false,
            control_id: None,
            selected: false,
        },
        ScreenRow {
            text: &r1,
            tone: cd_tone,
            signal: None,
            critical: false,
            control_id: None,
            selected: false,
        },
        ScreenRow {
            text: &r2,
            tone: cd_tone,
            signal: None,
            critical: false,
            control_id: None,
            selected: false,
        },
        ScreenRow {
            text: &r3,
            tone: cd_tone,
            signal: None,
            critical: false,
            control_id: None,
            selected: false,
        },
        ScreenRow {
            text: &r4,
            tone: SemanticTone::Neutral,
            signal: None,
            critical: false,
            control_id: None,
            selected: false,
        },
    ];
    let right_id = format!("c4-card-{idx}-right");
    skin.screen(
        w,
        &Screen {
            control_id: &right_id,
            title: &l0,
            rect: right_rect,
            font_family,
            font_px: card.cd_grid.font_px,
            cols: card.cd_grid.cols,
            rows: card.cd_grid.rows,
            lines: &right_lines,
            stations: &[],
            tracks: &[],
            page: 0,
            total_pages: 1,
        },
    );
}

/// Dispatch panel rendering based on solved [`Frames::panels`] on page 0.
fn render_area_panels(skin: &dyn Skin, w: &mut SvgWriter, state: &ConsoleState, frames: &Frames) {
    // On Compact/Phone where Trainway is page 0 and Delivery is page 1,
    // choose page 1 when Area::Cd is selected so the Delivery panel is active.
    let target_page = if state.route.area == Area::Cd && frames.pages > 1 {
        1
    } else {
        0
    };
    for p in frames.page(target_page) {
        match p.kind {
            PanelKind::Trainway => render_c5_trainway(skin, w, state, frames, p),
            PanelKind::Delivery => render_c6_delivery_gate(skin, w, state, frames, p),
            PanelKind::Fleet => render_fleet_panel(skin, w, state, frames, p),
        }
    }
}

/// Render `C5` CI Branch & PR Trainway Screen + `STOP` lamp + `FABRIC REV` + `YARD +N` + `RESTART` switch.
#[allow(clippy::too_many_lines)]
fn render_c5_trainway(
    skin: &dyn Skin,
    w: &mut SvgWriter,
    state: &ConsoleState,
    frames: &Frames,
    panel: &LayoutPanel,
) {
    let rect = clamp_rect(panel.plate, frames.canvas);
    let screen_rect = clamp_rect(panel.crt, rect);
    if rect.w <= 0.0 || rect.h <= 0.0 {
        return;
    }
    skin.panel(
        w,
        &Panel {
            id: "c5-trainway-panel",
            title: if rect.w < 500.0 {
                "CI TRAINWAY"
            } else {
                "CI BRANCH & PR TRAINWAY"
            },
            rect,
            role: Some("region"),
            aria_label: Some("CI Branch and PR Trainway"),
        },
    );

    let view = state.client.view();
    let topo = view.and_then(|v| v.topology.as_ref());
    let stop = topo.is_some_and(RepoTopology::stop);

    // Header strip controls inside `plate` above `crt`.
    let hdr_y = rect.y + 4.0;
    let hdr_h = ((screen_rect.y - rect.y) - 8.0).clamp(16.0, 24.0);
    let right = rect.right() - 6.0;

    // Guarded PullToTurn RESTART switch.
    let rst_w = if rect.w < 500.0 {
        116.0_f64.min(rect.w * 0.30)
    } else {
        136.0_f64.min(rect.w * 0.32)
    };
    let rst_rect = clamp_rect(
        Rect::new((right - rst_w).max(rect.x), hdr_y, rst_w, hdr_h),
        rect,
    );
    let rst_opts = [
        SelectorOption {
            id: "norm",
            label: "READY",
            badge: None,
            tone: SemanticTone::Neutral,
        },
        SelectorOption {
            id: "rst",
            label: if state.restart_armed_until_ms.is_some() {
                "CONFIRM!"
            } else if rst_w < 124.0 {
                "RESTART"
            } else {
                "HOLD RST"
            },
            badge: None,
            tone: SemanticTone::Warn,
        },
    ];
    skin.selector(
        w,
        &Selector {
            control_id: "c5-restart-switch",
            label: "Guarded Trainway Restart Switch (Hold 300ms to arm)",
            rect: rst_rect,
            action: SwitchAction::PullToTurn,
            options: &rst_opts,
            selected: state.restart_switch.current(),
            pulled: state.restart_armed_until_ms.is_some(),
            horizontal: true,
        },
    );

    // LIVE toggle.
    let live_w = 78.0_f64.min(rect.w * 0.20);
    let live_rect = clamp_rect(
        Rect::new(
            (rst_rect.x - live_w - 4.0).max(rect.x),
            hdr_y,
            live_w,
            hdr_h,
        ),
        rect,
    );
    skin.readout(
        w,
        &Readout {
            control_id: "c5-live-toggle",
            label: "",
            value: if state.trainway_live {
                "● LIVE"
            } else {
                "○ PAUSED"
            },
            rect: live_rect,
            tone: if state.trainway_live {
                SemanticTone::Pass
            } else {
                SemanticTone::Warn
            },
            interactive: true,
        },
    );

    // STOP lamp.
    let stop_w = 58.0_f64.min(rect.w * 0.16);
    let stop_rect = clamp_rect(
        Rect::new(
            (live_rect.x - stop_w - 4.0).max(rect.x),
            hdr_y,
            stop_w,
            hdr_h,
        ),
        rect,
    );
    skin.lamp(
        w,
        &Lamp {
            control_id: "c5-stop-lamp",
            label: "STOP",
            rect: stop_rect,
            lit: stop,
            tone: SemanticTone::Fail,
        },
    );

    let max_sidings = match frames.breakpoint {
        Breakpoint::Wide | Breakpoint::Medium => 4,
        Breakpoint::CompactLandscape => 3,
        Breakpoint::Phone => 2,
    };

    let mut row_strings: Vec<(String, SemanticTone, Option<SignalGlyph>, Option<String>)> =
        Vec::new();
    let mut station_labels: Vec<&str> = Vec::new();
    let mut track_signals: Vec<Vec<(u32, SignalGlyph, SemanticTone)>> = Vec::new();
    type TrackMeta = (String, u32, u32, bool, bool, Option<String>, bool);
    let mut track_refs: Vec<TrackMeta> = Vec::new();

    if let Some(t) = topo {
        for s in &t.stations {
            station_labels.push(s.label.as_str());
        }
        let layout = allocate_lanes(t, &state.trainway_lanes, max_sidings, state.yard_page);
        let sub_count = view
            .and_then(|v| {
                v.repos
                    .iter()
                    .find(|r| Some(&r.id) == state.route.repo.as_ref())
            })
            .map_or(t.submodules.len(), |r| r.submodules.len());
        let st_header = format!(
            "FABRIC REV {:04} · SUBMODULES: {} · INTERCHANGE ⇄ {}",
            t.fabric_rev,
            sub_count,
            if t.submodule_bump_pending {
                "BUMP PENDING"
            } else {
                "SYNCED"
            }
        );
        row_strings.push((
            st_header,
            SemanticTone::Info,
            None,
            Some("c5-fabric-rev".into()),
        ));

        // Main trunk (lane 0).
        if let Some(mt) = layout.main_train.as_ref() {
            let mut sigs = Vec::with_capacity(mt.stage_states.len());
            for (idx, st) in mt.stage_states.iter().enumerate() {
                let (g, tone) = stage_glyph_and_tone(*st);
                sigs.push((u32::try_from(idx).unwrap_or(0), g, tone));
            }
            let (glyph, tone) = if mt.failed() {
                (SignalGlyph::BlockFailed, SemanticTone::Fail)
            } else {
                (SignalGlyph::BlockRunning, SemanticTone::Pass)
            };
            let node_str = mt.node_id.as_deref().unwrap_or("QUEUE");
            let stage_idx = (mt.stage as usize + 1).min(mt.stage_states.len().max(1));
            let line = format!(
                "L0 {:<14} [{}] {} · STAGE {}/{}",
                mt.ref_name,
                node_str,
                if mt.node_lost { "NODE LOST" } else { "TRUNK" },
                stage_idx,
                mt.stage_states.len()
            );
            row_strings.push((
                line,
                if mt.node_lost {
                    SemanticTone::Fail
                } else {
                    tone
                },
                Some(glyph),
                Some(format!("c5-track-{}", mt.ref_name)),
            ));
            track_signals.push(sigs);
            track_refs.push((
                mt.ref_name.clone(),
                0,
                0,
                false,
                false,
                Some(node_str.to_owned()),
                mt.node_lost,
            ));
        }

        // Visible sidings (lanes 1..).
        for al in &layout.visible_sidings {
            let mut sigs = Vec::new();
            let mut node_str = "QUEUE";
            let mut lost = false;
            let mut failed = false;
            let mut stage_progress = String::new();
            if let Some(tr) = al.train.as_ref() {
                for (idx, st) in tr.stage_states.iter().enumerate() {
                    let (g, tone) = stage_glyph_and_tone(*st);
                    sigs.push((u32::try_from(idx).unwrap_or(0), g, tone));
                }
                node_str = tr.node_id.as_deref().unwrap_or("QUEUE");
                lost = tr.node_lost;
                failed = tr.failed();
                let st_idx = (tr.stage as usize + 1).min(tr.stage_states.len().max(1));
                stage_progress = format!(" · STAGE {st_idx}/{}", tr.stage_states.len());
            }
            let fading = matches!(al.branch.phase, SidingPhase::Fading { .. });
            let under_construction = al.branch.phase == SidingPhase::UnderConstruction;
            let (glyph, tone) = if failed || lost {
                (SignalGlyph::BlockFailed, SemanticTone::Fail)
            } else if under_construction {
                (SignalGlyph::BlockPending, SemanticTone::Warn)
            } else {
                (SignalGlyph::BlockRunning, SemanticTone::Info)
            };
            let phase_tag = if lost {
                "NODE LOST"
            } else if under_construction {
                "NEW SIDING UNDER CONSTRUCTION"
            } else if fading {
                "MERGED"
            } else {
                "ACTIVE"
            };
            let line = format!(
                "L{} {:<14} {:?} [{}] {phase_tag}{stage_progress}",
                al.lane, al.branch.ref_name, al.branch.pr_state, node_str
            );
            row_strings.push((
                line,
                tone,
                Some(glyph),
                Some(format!("c5-track-{}", al.branch.ref_name)),
            ));
            track_signals.push(sigs);
            track_refs.push((
                al.branch.ref_name.clone(),
                al.lane,
                al.branch.fork_stage,
                under_construction || al.branch.pr_state == PrState::Open,
                fading,
                Some(node_str.to_owned()),
                lost,
            ));
        }

        if layout.yard_overflow > 0 {
            let yard_line = format!(
                "YARD +{} OVERFLOW SIDINGS (PG {}/{})",
                layout.yard_overflow,
                layout.page + 1,
                layout.total_pages
            );
            row_strings.push((
                yard_line,
                SemanticTone::Warn,
                Some(SignalGlyph::BlockPending),
                Some("c5-yard-overflow".into()),
            ));
        }

        if t.submodule_bump_pending || !t.submodules.is_empty() {
            let bump = if t.submodule_bump_pending {
                "SUBMODULE BUMP PENDING".to_owned()
            } else {
                format!("ALL {sub_count} SUBMODULES SYNCED")
            };
            let sub_line = format!("INTERCHANGE ⇄ {bump}");
            row_strings.push((
                sub_line,
                if t.submodule_bump_pending {
                    SemanticTone::Warn
                } else {
                    SemanticTone::Info
                },
                None,
                Some("c5-submodule-interchange".into()),
            ));
        }
    } else {
        row_strings.push((
            "AWAITING TOPOLOGY STREAM…".into(),
            SemanticTone::Neutral,
            None,
            None,
        ));
    }

    // Also include parent repo submodule interchange when viewing a superproject.
    if row_strings
        .iter()
        .all(|(_, _, _, cid)| cid.as_deref() != Some("c5-submodule-interchange"))
    {
        if let Some(v) = view {
            if let Some(parent) = v
                .repos
                .iter()
                .find(|r| Some(&r.id) == state.route.repo.as_ref())
            {
                if let Some(first_sub) = parent.submodules.first() {
                    let sub_line = format!(
                        "INTERCHANGE ⇄ {} ({}) @ {}",
                        first_sub.path, first_sub.repo.repo, first_sub.pinned_sha
                    );
                    row_strings.push((
                        sub_line,
                        SemanticTone::Info,
                        None,
                        Some("c5-submodule-interchange".into()),
                    ));
                }
            }
        }
    }

    let screen_rows: Vec<ScreenRow<'_>> = row_strings
        .iter()
        .map(|(txt, tone, sig, cid)| ScreenRow {
            text: txt.as_str(),
            tone: *tone,
            signal: *sig,
            critical: false,
            control_id: cid.as_deref(),
            selected: false,
        })
        .collect();

    let screen_tracks: Vec<ScreenTrack<'_>> = track_refs
        .iter()
        .zip(track_signals.iter())
        .map(
            |((ref_name, lane, fork, dashed, fading, node_tag, lost), sigs)| ScreenTrack {
                ref_name: ref_name.as_str(),
                lane: *lane,
                fork_stage: *fork,
                dashed: *dashed,
                fading: *fading,
                signals: sigs.as_slice(),
                node_tag: node_tag.as_deref(),
                node_lost: *lost,
            },
        )
        .collect();

    let font = state.font_metrics();
    skin.screen(
        w,
        &Screen {
            control_id: "c5-trainway-screen",
            title: "CI Branch and PR Trainway Screen",
            rect: screen_rect,
            font_family: font.family,
            font_px: panel.grid.font_px,
            cols: panel.grid.cols,
            rows: panel.grid.rows,
            lines: &screen_rows,
            stations: &station_labels,
            tracks: &screen_tracks,
            page: state.yard_page,
            total_pages: 1,
        },
    );
}

/// Render `C6` CD Delivery Integration-Test Gate Screen + `PROMOTE`/`ROLLBACK` junction + history strip + log pager.
#[allow(clippy::too_many_lines)]
fn render_c6_delivery_gate(
    skin: &dyn Skin,
    w: &mut SvgWriter,
    state: &ConsoleState,
    frames: &Frames,
    panel: &LayoutPanel,
) {
    let rect = clamp_rect(panel.plate, frames.canvas);
    let screen_rect = clamp_rect(panel.crt, rect);
    if rect.w <= 0.0 || rect.h <= 0.0 {
        return;
    }
    skin.panel(
        w,
        &Panel {
            id: "c6-delivery-panel",
            title: "CD DELIVERY INTEGRATION-TEST GATE",
            rect,
            role: Some("region"),
            aria_label: Some("CD Delivery Integration-Test Gate"),
        },
    );

    let view = state.client.view();
    let gate = view.and_then(|v| v.gate.as_ref());

    let hdr_y = rect.y + 4.0;
    let hdr_h = ((screen_rect.y - rect.y) - 8.0).clamp(16.0, 24.0);
    let junc_w = 180.0_f64.min(rect.w * 0.45);
    let junc_rect = clamp_rect(
        Rect::new(
            (rect.right() - junc_w - 6.0).max(rect.x),
            hdr_y,
            junc_w,
            hdr_h,
        ),
        rect,
    );
    let (junc_val, junc_tone) = match gate {
        Some(g) if g.tainted || g.outcome == GateOutcome::Tainted => {
            ("TAINTED ✕", SemanticTone::Fail)
        }
        Some(g) if g.rollback_lit() => ("◀ ROLLBACK", SemanticTone::Fail),
        Some(g) if g.promote_lit() => ("PROMOTE ▶", SemanticTone::Pass),
        Some(_) => ("GATE PENDING", SemanticTone::Warn),
        None => ("AWAITING GATE", SemanticTone::Neutral),
    };
    skin.readout(
        w,
        &Readout {
            control_id: "c6-junction-status",
            label: "GATE",
            value: junc_val,
            rect: junc_rect,
            tone: junc_tone,
            interactive: false,
        },
    );

    type GateRowData = (
        String,
        SemanticTone,
        Option<SignalGlyph>,
        bool,
        Option<String>,
        bool,
    );
    let mut rows_data: Vec<GateRowData> = Vec::new();
    let mut total_log_pages = 1_u32;

    if let Some(g) = gate {
        // If user selected a historical deployment from `view.history`, show its summary banner.
        let active_summary = view.and_then(|v| {
            state
                .selected_history_idx
                .checked_sub(1)
                .and_then(|hi| v.history.get(hi))
        });
        let hdr = if let Some(hs) = active_summary {
            format!(
                "REPLAY {} · {} · IT {}/{} · {:?}",
                hs.deployment_id, hs.artifact_digest, hs.it_passed, hs.it_total, hs.outcome
            )
        } else {
            format!(
                "DEPLOY {} · SLOT {:?}->{:?} · TTL 142s{}",
                g.deployment_id,
                g.restore_slot,
                g.staged_slot,
                g.rollback_reason
                    .map_or_else(String::new, |r| format!(" ({r:?})"))
            )
        };
        rows_data.push((hdr, junc_tone, None, false, None, false));

        let passed_it = g
            .tests
            .iter()
            .filter(|t| t.state == TestState::Passed)
            .count();
        let total_it = g.tests.len().max(1);
        let filled = ((passed_it * 10) / total_it).min(10);
        let bar = format!("{}{}", "█".repeat(filled), "░".repeat(10 - filled));
        let crit_failed = g
            .tests
            .iter()
            .any(|t| t.critical && t.state == TestState::Failed);
        rows_data.push((
            format!(
                "GATE [{bar}] {passed_it}/{} IT · CRIT: {}",
                g.tests.len(),
                if crit_failed { "FAIL" } else { "OK" }
            ),
            junc_tone,
            None,
            false,
            None,
            false,
        ));

        for (idx, tc) in g.tests.iter().enumerate() {
            let (glyph, tone) = test_glyph_and_tone(tc.state);
            let selected = idx == state.selected_test_idx;
            let crate_tag = tc.crate_name.as_deref().unwrap_or("core");
            let line = format!("{:<21} [{:<6}] {:>5}ms", tc.name, crate_tag, tc.duration_ms);
            rows_data.push((
                line,
                tone,
                Some(glyph),
                tc.critical,
                Some(format!("c6-test-{idx}")),
                selected,
            ));
        }

        // Schematic Two-Branch PROMOTE / ROLLBACK Junction.
        let promote_tone = if g.promote_lit() {
            SemanticTone::Pass
        } else {
            SemanticTone::Neutral
        };
        let rollback_tone = if g.rollback_lit() {
            SemanticTone::Fail
        } else {
            SemanticTone::Neutral
        };
        rows_data.push((
            format!(
                "══╦══▶ PROMOTE SLOT {:?} ({})",
                g.staged_slot,
                if g.promote_lit() { "ACTIVE" } else { "STAGED" }
            ),
            promote_tone,
            Some(if g.promote_lit() {
                SignalGlyph::BlockPassed
            } else {
                SignalGlyph::BlockPending
            }),
            false,
            None,
            g.promote_lit(),
        ));
        rows_data.push((
            format!(
                "  ╚══▶ ROLLBACK SLOT {:?} ({})",
                g.restore_slot,
                if g.rollback_lit() {
                    "RESTORED"
                } else {
                    "STANDBY"
                }
            ),
            rollback_tone,
            Some(if g.rollback_lit() {
                SignalGlyph::BlockFailed
            } else {
                SignalGlyph::BlockPending
            }),
            false,
            None,
            g.rollback_lit(),
        ));

        // Selected test's log excerpt (paged, zero scrollbars).
        if let Some(sel_tc) = g
            .tests
            .get(state.selected_test_idx)
            .or_else(|| g.tests.first())
        {
            rows_data.push((
                format!(
                    "── LOG EXCERPT [{}] ({}ms) ──",
                    sel_tc.name, sel_tc.duration_ms
                ),
                SemanticTone::Info,
                None,
                false,
                None,
                false,
            ));
            let lines = extract_log_lines(sel_tc);
            let per_page = 1_usize;
            total_log_pages = u32::try_from(lines.len().div_ceil(per_page).max(1)).unwrap_or(1);
            let pg = (state.log_page as usize).min((total_log_pages as usize).saturating_sub(1));
            for (li, l) in lines.iter().skip(pg * per_page).take(per_page).enumerate() {
                rows_data.push((
                    format!("LOG[{}]: {l}", pg * per_page + li + 1),
                    SemanticTone::Info,
                    None,
                    false,
                    Some("c6-log-pager".into()),
                    false,
                ));
            }
        }

        if let Some(v) = view {
            if !v.history.is_empty() {
                let hist_summary: Vec<String> = v
                    .history
                    .iter()
                    .enumerate()
                    .take(3)
                    .map(|(hi, hg)| {
                        let sym = match hg.outcome {
                            GateOutcome::Promoted => "■",
                            GateOutcome::Pending => "▢",
                            GateOutcome::RolledBack | GateOutcome::Tainted => "✕",
                        };
                        let sel = if state.selected_history_idx == hi + 1 {
                            "*"
                        } else {
                            ""
                        };
                        format!("{}{sym}{sel}", hg.deployment_id)
                    })
                    .collect();
                rows_data.push((
                    format!("HISTORY: {}", hist_summary.join(" · ")),
                    SemanticTone::Neutral,
                    None,
                    false,
                    Some("c6-history-strip".into()),
                    state.selected_history_idx > 0,
                ));
            }
        }
    } else {
        rows_data.push((
            "AWAITING DELIVERY GATE STREAM…".into(),
            SemanticTone::Neutral,
            None,
            false,
            None,
            false,
        ));
    }

    let screen_rows: Vec<ScreenRow<'_>> = rows_data
        .iter()
        .map(|(txt, tone, sig, crit, cid, sel)| ScreenRow {
            text: txt.as_str(),
            tone: *tone,
            signal: *sig,
            critical: *crit,
            control_id: cid.as_deref(),
            selected: *sel,
        })
        .collect();

    let font = state.font_metrics();
    skin.screen(
        w,
        &Screen {
            control_id: "c6-delivery-screen",
            title: "CD Delivery Integration-Test Gate Screen",
            rect: screen_rect,
            font_family: font.family,
            font_px: panel.grid.font_px,
            cols: panel.grid.cols,
            rows: panel.grid.rows,
            lines: &screen_rows,
            stations: &[],
            tracks: &[],
            page: state.log_page,
            total_pages: total_log_pages,
        },
    );
}

fn extract_log_lines(tc: &GateTest) -> Vec<&str> {
    match tc.log_excerpt.as_deref() {
        Some(raw) if !raw.is_empty() => raw.lines().collect(),
        _ => vec!["status: ok (no failure excerpt)"],
    }
}

/// Render `Area::Fleet` Panel: Node locks (`node-a`, `worker-riscv64`) and `MuIsolation` sandbox telemetry.
fn render_fleet_panel(
    skin: &dyn Skin,
    w: &mut SvgWriter,
    state: &ConsoleState,
    frames: &Frames,
    panel: &LayoutPanel,
) {
    let rect = clamp_rect(panel.plate, frames.canvas);
    let screen_rect = clamp_rect(panel.crt, rect);
    if rect.w <= 0.0 || rect.h <= 0.0 {
        return;
    }
    skin.panel(
        w,
        &Panel {
            id: "flt-nodes-panel",
            title: "FLEET NODE LOCKS & MUISOLATION TELEMETRY",
            rect,
            role: Some("region"),
            aria_label: Some("Fleet Node Locks and MuIsolation Sandbox Telemetry"),
        },
    );

    let mut lines_owned: Vec<(String, SemanticTone)> = Vec::new();
    lines_owned.push((
        "PROFILE: max_pids=256  max_slots=1  KernelSandbox/Landlock".into(),
        SemanticTone::Info,
    ));
    if let Some(v) = state.client.view() {
        for n in &v.nodes {
            let tone = if n.online {
                SemanticTone::Pass
            } else {
                SemanticTone::Fail
            };
            lines_owned.push((
                format!(
                    "NODE {:<16} STATUS: {}  SLOTS: 1/1",
                    n.id.to_ascii_uppercase(),
                    if n.online { "ONLINE" } else { "OFFLINE / LOST" }
                ),
                tone,
            ));
        }
        if let Some(topo) = v.topology.as_ref() {
            for tr in &topo.trains {
                lines_owned.push((
                    format!(
                        "TRAIN #{} {:<14} STAGE {}  NODE {}",
                        tr.job_id,
                        tr.ref_name,
                        tr.stage,
                        tr.node_id.as_deref().unwrap_or("QUEUED")
                    ),
                    if tr.node_lost || tr.failed() {
                        SemanticTone::Fail
                    } else {
                        SemanticTone::Info
                    },
                ));
            }
        }
    } else {
        lines_owned.push(("AWAITING FLEET TELEMETRY…".into(), SemanticTone::Neutral));
    }

    let rows: Vec<ScreenRow<'_>> = lines_owned
        .iter()
        .map(|(txt, tone)| ScreenRow {
            text: txt.as_str(),
            tone: *tone,
            signal: None,
            critical: false,
            control_id: None,
            selected: false,
        })
        .collect();

    let font = state.font_metrics();
    skin.screen(
        w,
        &Screen {
            control_id: "flt-nodes-screen",
            title: "Fleet Node Locks Screen",
            rect: screen_rect,
            font_family: font.family,
            font_px: panel.grid.font_px,
            cols: panel.grid.cols,
            rows: panel.grid.rows,
            lines: &rows,
            stations: &[],
            tracks: &[],
            page: 0,
            total_pages: 1,
        },
    );
}

/// Render the accessible HTML shell wrapping `svg` with:
/// - Skip-to-dashboard link (`#muconsole-dashboard`);
/// - Polite and assertive `aria-live` regions;
/// - `font-size: 16px` `/` type-ahead repository filter `<input>` (preventing iOS Safari auto-zoom);
/// - `data-crt-font` and `data-lazy-dyslexic` attributes.
#[must_use]
pub fn render_html_shell(svg: &str, state: &ConsoleState) -> String {
    let font_attr = font_slug(state.font);
    let dyslexic_face = if state.lazy_dyslexic_loaded {
        "@font-face{font-family:'OpenDyslexicM Nerd Font Mono';src:url('./fonts/OpenDyslexicMNerdFontMono-Regular.woff2') format('woff2');font-display:swap;}"
    } else {
        ""
    };
    format!(
        "<div id=\"muconsole-shell\" data-crt-font=\"{font_attr}\" data-lazy-dyslexic=\"{}\" data-area=\"{}\">\
         <style>{dyslexic_face}#muconsole-filter{{font-size:16px;}}</style>\
         <a href=\"#muconsole-dashboard\" class=\"skip-link\">Skip to dashboard</a>\
         <div id=\"muconsole-aria-polite\" role=\"status\" aria-live=\"polite\" aria-atomic=\"true\">{}</div>\
         <div id=\"muconsole-aria-assertive\" role=\"alert\" aria-live=\"assertive\" aria-atomic=\"true\">{}</div>\
         <input id=\"muconsole-filter\" type=\"search\" aria-label=\"Type-ahead repository filter\" style=\"font-size:16px\" value=\"{}\"/>\
         <main id=\"muconsole-dashboard\" role=\"main\" aria-label=\"MuConsole Unified CI/CD Dashboard\">{svg}</main>\
         </div>",
        state.lazy_dyslexic_loaded,
        area_label(state.route.area),
        state.aria_polite,
        state.aria_assertive,
        state.filter_query,
    )
}
