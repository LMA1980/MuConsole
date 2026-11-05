// SPDX-License-Identifier: AGPL-3.0-only
//! Drift verification suite for `muconsole-theme` CSS token generation.
//! Ensures `tokens_css()` output matches both `assets/tokens.css` and `dist/tokens.css`.

use muconsole_theme::css::tokens_css;

#[test]
fn test_tokens_css_matches_committed_assets() {
    let generated = tokens_css();
    let assets = include_str!("../assets/tokens.css");
    assert_eq!(
        generated,
        assets,
        "crates/muconsole-theme/assets/tokens.css has drifted from tokens_css()!\n\
         Regenerate with:\n\
         cargo run -q -p muconsole-theme --bin muconsole-tokens-css > crates/muconsole-theme/assets/tokens.css"
    );
}

#[test]
fn test_tokens_css_matches_checked_in_dist() {
    let generated = tokens_css();
    let dist = include_str!("../../../dist/tokens.css");
    assert_eq!(
        generated,
        dist,
        "dist/tokens.css has drifted from tokens_css()!\n\
         Regenerate and copy with:\n\
         cargo run -q -p muconsole-theme --bin muconsole-tokens-css > crates/muconsole-theme/assets/tokens.css && \
         cp crates/muconsole-theme/assets/tokens.css dist/tokens.css"
    );
}

#[test]
fn test_tokens_css_contains_wireframe_skin_m67_rules() {
    let css = tokens_css();
    for selector in [
        "#muconsole-shell",
        "#muconsole-header",
        ".skip-link",
        ".skip-link:focus",
        "#muconsole-filter",
        "#muconsole-dashboard",
    ] {
        assert!(
            css.contains(selector),
            "tokens_css() must define selector '{selector}' for Milestone 6.7"
        );
    }
}
