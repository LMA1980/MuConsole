// SPDX-License-Identifier: AGPL-3.0-only
//! Generator tests: well-formed XML, resolvable references, glyph coverage
//! and deterministic output.

use super::annunciator::{annunciator, page_dot};
use super::crt::{crt, crt_text, reverse_video};
use super::lcd_window::{lcd, LcdSpec};
use super::rotary::{drum_lines, inner_angle, lamp_angle, rotary, RotarySpec};
use super::surface::{chevron, plate, vent_slots, wood};
use super::switch::{
    angle_distance, indicator_lamp, pistol_switch, LensColor, PistolSwitch, SwitchAction,
    SwitchPosition,
};
use super::text::{embossed, etched, Anchor, TextStyle};
use super::Rect;
use crate::defs::write_defs;
use crate::glyphs::BRANCH;
use crate::svg::SvgWriter;
use crate::tokens::{Effects, Lamp, CRT_CD, CRT_CI};
use std::collections::BTreeSet;

const AREAS: [SwitchPosition<'static>; 4] = [
    SwitchPosition::new(-60.0, "OVR"),
    SwitchPosition::new(-20.0, "CI"),
    SwitchPosition::new(20.0, "CD"),
    SwitchPosition::new(60.0, "FLT"),
];
const CAROUSEL: [SwitchPosition<'static>; 3] = [
    SwitchPosition::new(-45.0, "PREV"),
    SwitchPosition::new(0.0, "N"),
    SwitchPosition::new(45.0, "NEXT"),
];
const RESTART: [SwitchPosition<'static>; 2] = [
    SwitchPosition::new(-40.0, "NORM"),
    SwitchPosition::new(40.0, "RST"),
];
const REPOS: [&str; 4] = [
    "ACTIONRUNNER-RS",
    "MODEL-RUNNER",
    "MUCONDE-AGENT",
    "MUREGIST",
];
const SUBS: [&str; 3] = ["K3-MANAGER", "K3-ENGINE", "K3-CORE"];

/// A reduced concept desk using every primitive.
fn scene(effects: Effects) -> String {
    let mut w = SvgWriter::document(1512.0, 860.0);
    write_defs(&mut w, effects);
    wood(&mut w, 1512.0, 860.0);
    // Rail.
    plate(&mut w, Rect::new(6.0, 6.0, 80.0, 848.0), true);
    lcd(&mut w, 20.0, 18.0, "0516", LcdSpec::new(1.6, 3.0).cols(4));
    embossed(
        &mut w,
        46.0,
        62.0,
        "AREA",
        TextStyle::sized(11.0).anchor(Anchor::Middle),
    );
    pistol_switch(
        &mut w,
        &PistolSwitch::new(46.0, 112.0, &AREAS, 1).size(66.0),
    );
    for (i, lbl) in ["OVERVIEW", "CI", "CD", "FLEET"].iter().enumerate() {
        #[allow(clippy::cast_precision_loss)]
        let yy = 176.0 + i as f64 * 30.0;
        indicator_lamp(&mut w, 22.0, yy, LensColor::Green, i == 1, 5.0);
        etched(
            &mut w,
            34.0,
            yy + 4.0,
            lbl,
            TextStyle::sized(9.0).spacing(0.0),
        );
    }
    vent_slots(&mut w, 24.0, 330.0, 44.0, 6);
    // Top bar.
    plate(&mut w, Rect::new(94.0, 6.0, 1412.0, 48.0), true);
    etched(
        &mut w,
        112.0,
        20.0,
        "REPO",
        TextStyle::sized(9.0).spacing(2.0),
    );
    let (lw, _) = lcd(
        &mut w,
        112.0,
        24.0,
        "lma1980/model-runner",
        LcdSpec::new(1.6, 3.0),
    );
    chevron(&mut w, 112.0 + lw + 14.0, 36.0);
    annunciator(
        &mut w,
        Rect::new(1324.0, 16.0, 50.0, 28.0),
        "CI OK",
        Lamp::Ok,
    );
    annunciator(
        &mut w,
        Rect::new(1440.0, 16.0, 50.0, 28.0),
        "FAULT",
        Lamp::Off,
    );
    // Carousel switch + card CRTs.
    plate(&mut w, Rect::new(1102.0, 64.0, 92.0, 192.0), true);
    pistol_switch(
        &mut w,
        &PistolSwitch::new(1148.0, 150.0, &CAROUSEL, 1)
            .size(70.0)
            .action(SwitchAction::SpringReturn)
            .nameplate("SPRING RTN"),
    );
    crt(
        &mut w,
        Rect::new(110.0, 106.0, 140.0, 132.0),
        &CRT_CI,
        effects,
        |w| crt_text(w, 120.0, 126.0, "QUALITY", CRT_CI.dim, 14.0, Anchor::Start),
    );
    crt(
        &mut w,
        Rect::new(260.0, 106.0, 140.0, 132.0),
        &CRT_CD,
        effects,
        |w| {
            reverse_video(w, Rect::new(268.0, 210.0, 124.0, 18.0), &CRT_CD);
            crt_text(
                w,
                330.0,
                224.0,
                "ROLLBACK",
                CRT_CD.reverse_ink,
                17.0,
                Anchor::Middle,
            );
        },
    );
    page_dot(&mut w, 530.0, 268.0, true);
    page_dot(&mut w, 548.0, 268.0, false);
    rotary(
        &mut w,
        Rect::new(1204.0, 64.0, 302.0, 192.0),
        &RotarySpec {
            repos: &REPOS,
            selected: 1,
            submodules: &SUBS,
            selected_sub: Some(1),
        },
    );
    // Trainway header controls.
    indicator_lamp(&mut w, 806.0, 302.0, LensColor::Green, true, 5.0);
    indicator_lamp(&mut w, 806.0, 326.0, LensColor::Red, false, 5.0);
    pistol_switch(
        &mut w,
        &PistolSwitch::new(872.0, 315.0, &RESTART, 0)
            .size(50.0)
            .action(SwitchAction::PullToTurn)
            .nameplate("PULL"),
    );
    crt(
        &mut w,
        Rect::new(116.0, 346.0, 854.0, 484.0),
        &CRT_CI,
        effects,
        |w| {
            let label = format!("{BRANCH} feat/rvv-gemm & <main>");
            crt_text(w, 128.0, 400.0, &label, CRT_CI.lit, 14.0, Anchor::Start);
        },
    );
    w.close();
    w.finish().expect("balanced document")
}

fn parse(svg: &str) -> roxmltree::Document<'_> {
    roxmltree::Document::parse(svg).expect("well-formed SVG")
}

/// Every `url(#id)` / `href="#id"` must resolve to an element id.
fn assert_references_resolve(doc: &roxmltree::Document<'_>) {
    let defined: BTreeSet<&str> = doc
        .descendants()
        .filter_map(|n| n.attribute("id"))
        .collect();
    for n in doc.descendants().filter(roxmltree::Node::is_element) {
        for a in n.attributes() {
            let v = a.value();
            let target = if let Some(rest) = v.strip_prefix("url(#") {
                rest.strip_suffix(')')
            } else if a.name() == "href" {
                v.strip_prefix('#')
            } else {
                None
            };
            if let Some(id) = target {
                assert!(defined.contains(id), "unresolved reference #{id}");
            }
        }
    }
}

#[test]
fn full_scene_is_well_formed_and_resolvable() {
    for effects in [Effects::FULL, Effects::LITE] {
        let svg = scene(effects);
        let doc = parse(&svg);
        assert_eq!(doc.root_element().tag_name().name(), "svg");
        assert_references_resolve(&doc);
    }
}

#[test]
fn output_is_deterministic() {
    assert_eq!(scene(Effects::FULL), scene(Effects::FULL));
    assert_eq!(scene(Effects::LITE), scene(Effects::LITE));
    assert_ne!(scene(Effects::FULL), scene(Effects::LITE));
}

#[test]
fn lite_mode_drops_bloom_and_scanlines() {
    let lite = scene(Effects::LITE);
    assert!(!lite.contains("mu-phos"));
    assert!(!lite.contains("mu-scan"));
    let full = scene(Effects::FULL);
    assert!(full.contains("filter=\"url(#mu-phos)\""));
    assert!(full.contains("fill=\"url(#mu-scan)\""));
    // Lamp glow is the one filter kept in Lite.
    assert!(lite.contains("url(#mu-lamp)"));
}

#[test]
fn crt_text_is_escaped() {
    let svg = scene(Effects::FULL);
    assert!(svg.contains("feat/rvv-gemm &amp; &lt;main&gt;"));
    let doc = parse(&svg);
    assert!(doc
        .descendants()
        .any(|n| n.text() == Some("\u{e0a0} feat/rvv-gemm & <main>")));
}

#[test]
fn lcd_uses_one_glyph_per_lit_cell() {
    let mut w = SvgWriter::new();
    let (width, height) = lcd(&mut w, 0.0, 0.0, "A~ b", LcdSpec::new(2.0, 4.0).cols(6));
    let s = w.finish().expect("balanced");
    // 6 ghost cells; 'A' and 'B' lit; '~' (no glyph) and ' ' ghost-only.
    assert_eq!(s.matches("#mu-lcd-ghost").count(), 6);
    assert_eq!(s.matches("href=\"#mu-lcd-41\"").count(), 1);
    assert_eq!(s.matches("href=\"#mu-lcd-42\"").count(), 1);
    assert_eq!(s.matches("href=\"#mu-lcd-").count(), 8);
    assert!((width - (6.0 * 12.0 - 2.0 + 8.0)).abs() < 1e-9);
    assert!((height - (14.0 + 8.0)).abs() < 1e-9);
    assert_eq!(LcdSpec::new(2.0, 4.0).cols_fitting(width), 6);
}

#[test]
fn defs_cover_every_lcd_glyph() {
    let mut w = SvgWriter::new();
    write_defs(&mut w, Effects::FULL);
    let s = w.finish().expect("balanced");
    let doc = parse(&s);
    let ids: BTreeSet<&str> = doc
        .descendants()
        .filter_map(|n| n.attribute("id"))
        .collect();
    for (c, _) in crate::lcd::GLYPHS {
        if *c != ' ' {
            let id = crate::defs::lcd_symbol_id(*c);
            assert!(ids.contains(id.as_str()), "missing symbol for {c:?}");
        }
    }
    assert!(ids.contains("mu-lens-red"));
    assert!(ids.contains("mu-bakelite"));
    assert!(ids.contains("mu-esc"));
}

#[test]
fn switch_variants() {
    let render = |s: &PistolSwitch<'_>| {
        let mut w = SvgWriter::new();
        pistol_switch(&mut w, s);
        w.finish().expect("balanced")
    };
    let maintained = render(&PistolSwitch::new(50.0, 50.0, &AREAS, 2));
    assert!(maintained.contains("rotate(20)"), "grip at CD = 20°");
    assert!(!maintained.contains("#b3121c"), "no collar");
    assert_eq!(
        maintained.matches("<text").count(),
        4,
        "one legend per position"
    );
    let spring =
        render(&PistolSwitch::new(50.0, 50.0, &CAROUSEL, 1).action(SwitchAction::SpringReturn));
    assert_eq!(spring.matches(" A15 15 ").count(), 2, "two return arrows");
    let pull = render(
        &PistolSwitch::new(50.0, 50.0, &RESTART, 9)
            .action(SwitchAction::PullToTurn)
            .nameplate("PULL"),
    );
    assert!(pull.contains("#b3121c"), "red collar");
    assert!(pull.contains("rotate(40)"), "index clamps to last position");
    assert!(pull.contains(">PULL</text>"));
    let empty = render(&PistolSwitch::new(50.0, 50.0, &[], 0));
    assert!(!empty.contains("mu-bakelite"), "no grip without positions");
    // Scaling: a 32 px switch halves the tick radius.
    let small = render(&PistolSwitch::new(0.0, 0.0, &CAROUSEL[1..2], 0).size(32.0));
    assert!(small.contains("y1=\"-11.25\""));
}

#[test]
fn switch_geometry_helpers() {
    let s = PistolSwitch::new(0.0, 0.0, &AREAS, 0);
    assert_eq!(s.nearest_position(-55.0), Some(0));
    assert_eq!(s.nearest_position(5.0), Some(2));
    assert_eq!(s.nearest_position(80.0), Some(3));
    assert!((angle_distance(350.0, 10.0) - 20.0).abs() < 1e-9);
    assert!((angle_distance(-60.0, 60.0) - 120.0).abs() < 1e-9);
    assert!(SwitchAction::PullToTurn.springs_back());
    assert!(!SwitchAction::Maintained.springs_back());
    assert!(SwitchAction::PullToTurn.needs_pull());
}

#[test]
fn indicator_lamp_states() {
    let mut w = SvgWriter::new();
    indicator_lamp(&mut w, 10.0, 10.0, LensColor::Red, true, 6.0);
    let lit = w.finish().expect("balanced");
    assert!(lit.contains("url(#mu-lens-red)") && lit.contains("url(#mu-lamp)"));
    let mut w = SvgWriter::new();
    indicator_lamp(&mut w, 10.0, 10.0, LensColor::Green, false, 6.0);
    let off = w.finish().expect("balanced");
    assert!(off.contains("url(#mu-lens-off)") && off.contains("#123d1d"));
    assert!(!off.contains("url(#mu-lamp)"));
    assert_eq!(off.matches("<line").count(), 24, "knurled bezel");
}

#[test]
fn rotary_helpers() {
    assert!((lamp_angle(0, 9) - 135.0).abs() < 1e-9);
    assert!((lamp_angle(8, 9) - 405.0).abs() < 1e-9);
    assert!((lamp_angle(0, 1) - 135.0).abs() < 1e-9);
    assert!((inner_angle(None, 7) + 90.0).abs() < 1e-9);
    assert!((inner_angle(Some(0), 3) - 0.0).abs() < 1e-9);
    let lines = drum_lines(&REPOS, 0);
    assert_eq!(lines[0], " MUREGIST", "wraps backwards");
    assert_eq!(lines[1], ">ACTIONRUNNER-RS");
    assert_eq!(lines[2], " MODEL-RUNNER");
    assert_eq!(drum_lines(&REPOS, 5)[1], ">MODEL-RUNNER", "wraps forwards");
    assert_eq!(drum_lines(&[], 3)[1], ">");
}

#[test]
fn rotary_without_submodules() {
    let mut w = SvgWriter::new();
    rotary(
        &mut w,
        Rect::new(0.0, 0.0, 302.0, 192.0),
        &RotarySpec {
            repos: &REPOS,
            selected: 3,
            submodules: &[],
            selected_sub: None,
        },
    );
    let s = w.finish().expect("balanced");
    assert!(!s.contains("#ff3b3b"), "no inner index line");
    assert!(!s.contains("#ffb43a"), "no submodule lamps");
    // " -- NONE --" renders N, O, E glyphs.
    assert!(s.contains("#mu-lcd-4e"));
    let mut w = SvgWriter::new();
    rotary(
        &mut w,
        Rect::new(0.0, 0.0, 302.0, 192.0),
        &RotarySpec {
            repos: &REPOS,
            selected: 0,
            submodules: &SUBS,
            selected_sub: None,
        },
    );
    let parent = w.finish().expect("balanced");
    assert!(parent.contains("PARENT  3 SUBMODULES"));
    assert!(
        parent
            .matches("fill=\"#1b1d1f\" stroke=\"#000\" stroke-width=\"0.5\"/>")
            .count()
            >= 3,
        "one unlit lamp per submodule"
    );
}

#[test]
fn etch_respects_minimum_size() {
    assert!((TextStyle::sized(8.0).rendered_size() - 11.0).abs() < 1e-9);
    assert!((TextStyle::sized(17.0).rendered_size() - 17.85).abs() < 1e-9);
    let mut w = SvgWriter::new();
    etched(&mut w, 0.0, 0.0, "AREA", TextStyle::default());
    let s = w.finish().expect("balanced");
    assert_eq!(s.matches("<text").count(), 3, "three-pass etch");
    assert!(s.contains("font-family=\"MuEtch\""));
}

#[test]
fn wood_leaves_are_fixed_scale() {
    let count_seams = |width: f64| {
        let mut w = SvgWriter::new();
        wood(&mut w, width, 100.0);
        w.finish()
            .expect("balanced")
            .matches("stroke=\"#070403\"")
            .count()
    };
    assert_eq!(count_seams(1512.0), 3, "seams at 384, 768, 1152");
    assert_eq!(
        count_seams(402.0),
        1,
        "phone sees one seam of the same cabinet"
    );
}
