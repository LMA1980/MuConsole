// SPDX-License-Identifier: AGPL-3.0-only
//! Print the generated `tokens.css`.
//!
//! ```text
//! cargo run -q -p muconsole-theme --bin muconsole-tokens-css > crates/muconsole-theme/assets/tokens.css
//! ```

fn main() {
    print!("{}", muconsole_theme::css::tokens_css());
}
