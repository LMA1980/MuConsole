// SPDX-License-Identifier: AGPL-3.0-only
//! Nerd Font icon table and font subset profiles.
//!
//! This is the single source of truth for every icon the CRTs draw. The same
//! table produces the `pyftsubset --unicodes` list (see the
//! `muconsole-subset-unicodes` binary), and `scripts/fetch-fonts.sh` checks
//! that every code point listed here is present in each subset, so a missing
//! icon fails the build instead of rendering tofu.

/// One Nerd Font icon.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Icon {
    pub name: &'static str,
    pub ch: char,
    pub usage: &'static str,
}

impl Icon {
    /// Unicode scalar value.
    #[must_use]
    pub const fn codepoint(self) -> u32 {
        self.ch as u32
    }
}

pub const BRANCH: char = '\u{e0a0}';
pub const SERVER: char = '\u{f233}';
pub const GIT_REPO: char = '\u{f418}';
pub const BOLT: char = '\u{f0e7}';
pub const CHECK: char = '\u{f00c}';
pub const CROSS: char = '\u{f00d}';
pub const WARNING: char = '\u{f071}';

/// Every icon used by the console.
pub const ICONS: &[Icon] = &[
    Icon {
        name: "branch",
        ch: BRANCH,
        usage: "siding labels",
    },
    Icon {
        name: "server",
        ch: SERVER,
        usage: "train node tags",
    },
    Icon {
        name: "git-repo",
        ch: GIT_REPO,
        usage: "repo terminal",
    },
    Icon {
        name: "bolt",
        ch: BOLT,
        usage: "integration gate header",
    },
    Icon {
        name: "check",
        ch: CHECK,
        usage: "promote / passed",
    },
    Icon {
        name: "cross",
        ch: CROSS,
        usage: "failed",
    },
    Icon {
        name: "warning",
        ch: WARNING,
        usage: "rollback / caution",
    },
    Icon {
        name: "code-fork",
        ch: '\u{f126}',
        usage: "pull request rejoin",
    },
    Icon {
        name: "clock",
        ch: '\u{f017}',
        usage: "durations, lease TTL",
    },
    Icon {
        name: "lock",
        ch: '\u{f023}',
        usage: "exclusive node lock",
    },
    Icon {
        name: "unlock",
        ch: '\u{f09c}',
        usage: "node lock released",
    },
    Icon {
        name: "refresh",
        ch: '\u{f021}',
        usage: "retry / reconnecting",
    },
    Icon {
        name: "play",
        ch: '\u{f04b}',
        usage: "running",
    },
    Icon {
        name: "pause",
        ch: '\u{f04c}',
        usage: "paused / throttled",
    },
    Icon {
        name: "info",
        ch: '\u{f05a}',
        usage: "notes",
    },
    Icon {
        name: "database",
        ch: '\u{f1c0}',
        usage: "registry / cache",
    },
    Icon {
        name: "cog",
        ch: '\u{f013}',
        usage: "settings",
    },
    Icon {
        name: "plug",
        ch: '\u{f1e6}',
        usage: "connection state",
    },
    Icon {
        name: "cloud",
        ch: '\u{f0c2}',
        usage: "upstream",
    },
    Icon {
        name: "check-circle",
        ch: '\u{f058}',
        usage: "gate passed",
    },
    Icon {
        name: "times-circle",
        ch: '\u{f057}',
        usage: "gate failed",
    },
];

/// Look up an icon by name.
#[must_use]
pub fn icon(name: &str) -> Option<Icon> {
    ICONS.iter().copied().find(|i| i.name == name)
}

/// Font subset profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SubsetProfile {
    /// CRT terminal fonts (Hurmit, OpenDyslexicM): Latin-1, punctuation,
    /// box drawing, block elements, geometric shapes, state glyphs and icons.
    Crt,
    /// Etched / embossed plate labels (HeavyData): uppercase, digits,
    /// punctuation and ▲▼•.
    Etch,
}

impl SubsetProfile {
    /// Parse `crt` / `etch`.
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "crt" => Some(Self::Crt),
            "etch" => Some(Self::Etch),
            _ => None,
        }
    }
}

/// Inclusive code point ranges of the CRT text subset (icons are added on top).
pub const CRT_RANGES: &[(u32, u32)] = &[
    (0x0020, 0x007E), // Basic Latin
    (0x00A0, 0x00FF), // Latin-1 Supplement
    (0x2010, 0x2027), // dashes, quotes, bullet, ellipsis
    (0x2500, 0x257F), // box drawing
    (0x2580, 0x259F), // block elements
    (0x25A0, 0x25FF), // geometric shapes (where the font has them)
];

/// Inclusive code point ranges of the HeavyData etch subset.
pub const ETCH_RANGES: &[(u32, u32)] = &[
    (0x0020, 0x0040), // space, punctuation, digits, @
    (0x0041, 0x005A), // A-Z
    (0x005B, 0x0060), // [ \ ] ^ _ `
    (0x2022, 0x2022), // •
    (0x25B2, 0x25B2), // ▲
    (0x25BC, 0x25BC), // ▼
];

/// Expand the profile into a sorted, de-duplicated list of code points.
#[must_use]
pub fn subset_codepoints(profile: SubsetProfile) -> Vec<u32> {
    let mut out: Vec<u32> = Vec::new();
    let ranges = match profile {
        SubsetProfile::Crt => CRT_RANGES,
        SubsetProfile::Etch => ETCH_RANGES,
    };
    for &(lo, hi) in ranges {
        out.extend(lo..=hi);
    }
    if profile == SubsetProfile::Crt {
        out.extend(ICONS.iter().map(|i| i.codepoint()));
    }
    out.sort_unstable();
    out.dedup();
    out
}

/// Compact `pyftsubset --unicodes=` value: `U+0020-007E,U+00A0-00FF,...`.
#[must_use]
pub fn pyftsubset_unicodes(profile: SubsetProfile) -> String {
    let cps = subset_codepoints(profile);
    let mut parts: Vec<String> = Vec::new();
    let mut i = 0usize;
    while i < cps.len() {
        let start = cps[i];
        let mut end = start;
        while i + 1 < cps.len() && cps[i + 1] == end + 1 {
            i += 1;
            end = cps[i];
        }
        if start == end {
            parts.push(format!("U+{start:04X}"));
        } else {
            parts.push(format!("U+{start:04X}-{end:04X}"));
        }
        i += 1;
    }
    parts.join(",")
}

/// Code points that must exist in every subset of the profile (checked by
/// `scripts/fetch-fonts.sh` after subsetting): the Nerd icons, the ellipsis
/// used for truncation and the box-drawing track pieces.
///
/// Block-signal states (■ passed, ▢ running, ✕ failed, ▫ pending) are drawn
/// as SVG primitives by the instrument library, never as font glyphs:
/// OpenDyslexicM has almost no Geometric Shapes coverage and neither CRT
/// font has U+2715.
#[must_use]
pub fn required_codepoints(profile: SubsetProfile) -> Vec<u32> {
    match profile {
        SubsetProfile::Crt => {
            let mut v: Vec<u32> = ICONS.iter().map(|i| i.codepoint()).collect();
            // … and the box-drawing pieces used by the Trainway.
            v.extend([0x2026, 0x2500, 0x2550, 0x256D, 0x2570]);
            v
        }
        SubsetProfile::Etch => {
            let mut v: Vec<u32> = (u32::from('A')..=u32::from('Z')).collect();
            v.extend(u32::from('0')..=u32::from('9'));
            v
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spec_icons_present() {
        for (name, cp) in [
            ("branch", 0xE0A0),
            ("server", 0xF233),
            ("git-repo", 0xF418),
            ("bolt", 0xF0E7),
            ("check", 0xF00C),
            ("cross", 0xF00D),
            ("warning", 0xF071),
        ] {
            assert_eq!(icon(name).map(Icon::codepoint), Some(cp), "{name}");
        }
        assert!(icon("nope").is_none());
    }

    #[test]
    fn icons_unique_and_private_use() {
        let mut cps: Vec<u32> = ICONS.iter().map(|i| i.codepoint()).collect();
        let n = cps.len();
        cps.sort_unstable();
        cps.dedup();
        assert_eq!(cps.len(), n, "duplicate icon codepoint");
        assert!(cps.iter().all(|cp| (0xE000..=0xF8FF).contains(cp)));
        assert!(n >= 20, "about 20 icons expected");
    }

    #[test]
    fn subsets_contain_required() {
        for p in [SubsetProfile::Crt, SubsetProfile::Etch] {
            let all = subset_codepoints(p);
            for cp in required_codepoints(p) {
                assert!(all.binary_search(&cp).is_ok(), "{p:?} missing U+{cp:04X}");
            }
        }
    }

    #[test]
    fn unicode_list_is_compact_ranges() {
        let s = pyftsubset_unicodes(SubsetProfile::Etch);
        assert_eq!(s, "U+0020-0060,U+2022,U+25B2,U+25BC");
        let crt = pyftsubset_unicodes(SubsetProfile::Crt);
        assert!(crt.starts_with("U+0020-007E,U+00A0-00FF,"));
        assert!(crt.contains("U+E0A0"));
        assert!(crt.contains("U+F418"));
        assert!(!crt.contains(' '));
        assert_eq!(SubsetProfile::parse("crt"), Some(SubsetProfile::Crt));
        assert_eq!(SubsetProfile::parse("x"), None);
    }
}
