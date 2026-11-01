// SPDX-License-Identifier: AGPL-3.0-only
//! Print the `pyftsubset --unicodes=` list derived from the glyph table.
//!
//! ```text
//! muconsole-subset-unicodes crt            # U+0020-007E,...,U+F418
//! muconsole-subset-unicodes etch
//! muconsole-subset-unicodes crt --required # one hex code point per line
//! ```

use muconsole_theme::glyphs::{pyftsubset_unicodes, required_codepoints, SubsetProfile};
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(profile) = args.first().and_then(|a| SubsetProfile::parse(a)) else {
        eprintln!("usage: muconsole-subset-unicodes <crt|etch> [--required]");
        return ExitCode::from(2);
    };
    if args.iter().any(|a| a == "--required") {
        for cp in required_codepoints(profile) {
            println!("{cp:04X}");
        }
    } else {
        println!("{}", pyftsubset_unicodes(profile));
    }
    ExitCode::SUCCESS
}
