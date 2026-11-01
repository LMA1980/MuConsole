// SPDX-License-Identifier: AGPL-3.0-only
//! Built-in 5×7 dot-matrix glyph table for STN LCD windows.
//!
//! LCD windows need no font: each glyph is seven rows of five dots. Row bits
//! are MSB-left (`0b10000` is the leftmost dot). The table is sorted by code
//! point so lookups are a binary search. Lowercase input is shown uppercase;
//! characters without a glyph render as an empty (ghost-only) cell.

/// Glyph width in dots.
pub const GLYPH_COLS: usize = 5;
/// Glyph height in dots.
pub const GLYPH_ROWS: usize = 7;
/// Horizontal cell advance in dots (glyph + 1 dot gap).
pub const CELL_ADVANCE: usize = 6;

/// One glyph: seven 5-bit rows.
pub type Glyph = [u8; GLYPH_ROWS];

/// Glyph table sorted by code point.
pub const GLYPHS: &[(char, Glyph)] = &[
    (
        ' ',
        [
            0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000,
        ],
    ),
    (
        '#',
        [
            0b01010, 0b01010, 0b11111, 0b01010, 0b11111, 0b01010, 0b01010,
        ],
    ),
    (
        '%',
        [
            0b11000, 0b11001, 0b00010, 0b00100, 0b01000, 0b10011, 0b00011,
        ],
    ),
    (
        '+',
        [
            0b00000, 0b00100, 0b00100, 0b11111, 0b00100, 0b00100, 0b00000,
        ],
    ),
    (
        '-',
        [
            0b00000, 0b00000, 0b00000, 0b11111, 0b00000, 0b00000, 0b00000,
        ],
    ),
    (
        '.',
        [
            0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b01100, 0b01100,
        ],
    ),
    (
        '/',
        [
            0b00001, 0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b10000,
        ],
    ),
    (
        '0',
        [
            0b01110, 0b10001, 0b10011, 0b10101, 0b11001, 0b10001, 0b01110,
        ],
    ),
    (
        '1',
        [
            0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110,
        ],
    ),
    (
        '2',
        [
            0b01110, 0b10001, 0b00001, 0b00010, 0b00100, 0b01000, 0b11111,
        ],
    ),
    (
        '3',
        [
            0b11111, 0b00010, 0b00100, 0b00010, 0b00001, 0b10001, 0b01110,
        ],
    ),
    (
        '4',
        [
            0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010,
        ],
    ),
    (
        '5',
        [
            0b11111, 0b10000, 0b11110, 0b00001, 0b00001, 0b10001, 0b01110,
        ],
    ),
    (
        '6',
        [
            0b00110, 0b01000, 0b10000, 0b11110, 0b10001, 0b10001, 0b01110,
        ],
    ),
    (
        '7',
        [
            0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000,
        ],
    ),
    (
        '8',
        [
            0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110,
        ],
    ),
    (
        '9',
        [
            0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00010, 0b01100,
        ],
    ),
    (
        ':',
        [
            0b00000, 0b01100, 0b01100, 0b00000, 0b01100, 0b01100, 0b00000,
        ],
    ),
    (
        '>',
        [
            0b01000, 0b00100, 0b00010, 0b00001, 0b00010, 0b00100, 0b01000,
        ],
    ),
    (
        '@',
        [
            0b01110, 0b10001, 0b10111, 0b10101, 0b10111, 0b10000, 0b01110,
        ],
    ),
    (
        'A',
        [
            0b01110, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001,
        ],
    ),
    (
        'B',
        [
            0b11110, 0b10001, 0b10001, 0b11110, 0b10001, 0b10001, 0b11110,
        ],
    ),
    (
        'C',
        [
            0b01110, 0b10001, 0b10000, 0b10000, 0b10000, 0b10001, 0b01110,
        ],
    ),
    (
        'D',
        [
            0b11110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b11110,
        ],
    ),
    (
        'E',
        [
            0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b11111,
        ],
    ),
    (
        'F',
        [
            0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b10000,
        ],
    ),
    (
        'G',
        [
            0b01110, 0b10001, 0b10000, 0b10111, 0b10001, 0b10001, 0b01111,
        ],
    ),
    (
        'H',
        [
            0b10001, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001,
        ],
    ),
    (
        'I',
        [
            0b01110, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110,
        ],
    ),
    (
        'J',
        [
            0b00111, 0b00010, 0b00010, 0b00010, 0b00010, 0b10010, 0b01100,
        ],
    ),
    (
        'K',
        [
            0b10001, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010, 0b10001,
        ],
    ),
    (
        'L',
        [
            0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b11111,
        ],
    ),
    (
        'M',
        [
            0b10001, 0b11011, 0b10101, 0b10101, 0b10001, 0b10001, 0b10001,
        ],
    ),
    (
        'N',
        [
            0b10001, 0b10001, 0b11001, 0b10101, 0b10011, 0b10001, 0b10001,
        ],
    ),
    (
        'O',
        [
            0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110,
        ],
    ),
    (
        'P',
        [
            0b11110, 0b10001, 0b10001, 0b11110, 0b10000, 0b10000, 0b10000,
        ],
    ),
    (
        'Q',
        [
            0b01110, 0b10001, 0b10001, 0b10001, 0b10101, 0b10010, 0b01101,
        ],
    ),
    (
        'R',
        [
            0b11110, 0b10001, 0b10001, 0b11110, 0b10100, 0b10010, 0b10001,
        ],
    ),
    (
        'S',
        [
            0b01111, 0b10000, 0b10000, 0b01110, 0b00001, 0b00001, 0b11110,
        ],
    ),
    (
        'T',
        [
            0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100,
        ],
    ),
    (
        'U',
        [
            0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110,
        ],
    ),
    (
        'V',
        [
            0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01010, 0b00100,
        ],
    ),
    (
        'W',
        [
            0b10001, 0b10001, 0b10001, 0b10101, 0b10101, 0b10101, 0b01010,
        ],
    ),
    (
        'X',
        [
            0b10001, 0b10001, 0b01010, 0b00100, 0b01010, 0b10001, 0b10001,
        ],
    ),
    (
        'Y',
        [
            0b10001, 0b10001, 0b01010, 0b00100, 0b00100, 0b00100, 0b00100,
        ],
    ),
    (
        'Z',
        [
            0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b11111,
        ],
    ),
    (
        '_',
        [
            0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b11111,
        ],
    ),
];

/// Normalise a character for LCD display (uppercase ASCII).
#[must_use]
pub fn normalize(c: char) -> char {
    c.to_ascii_uppercase()
}

/// Look up the glyph for `c` (after [`normalize`]).
#[must_use]
pub fn glyph(c: char) -> Option<&'static Glyph> {
    let c = normalize(c);
    GLYPHS
        .binary_search_by(|(k, _)| k.cmp(&c))
        .ok()
        .map(|i| &GLYPHS[i].1)
}

/// Whether `c` has a glyph.
#[must_use]
pub fn has_glyph(c: char) -> bool {
    glyph(c).is_some()
}

/// Lit dot coordinates `(col, row)` of a glyph, row-major.
pub fn dots(g: &Glyph) -> impl Iterator<Item = (usize, usize)> + '_ {
    g.iter().enumerate().flat_map(|(row, bits)| {
        (0..GLYPH_COLS)
            .filter(move |col| bits & (1 << (GLYPH_COLS - 1 - col)) != 0)
            .map(move |col| (col, row))
    })
}

/// Characters of `text` that have no LCD glyph (useful for tests and lints).
#[must_use]
pub fn missing_glyphs(text: &str) -> Vec<char> {
    let mut out: Vec<char> = text.chars().filter(|c| !has_glyph(*c)).collect();
    out.sort_unstable();
    out.dedup();
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_is_sorted_and_unique() {
        for pair in GLYPHS.windows(2) {
            assert!(pair[0].0 < pair[1].0, "{:?} !< {:?}", pair[0].0, pair[1].0);
        }
    }

    #[test]
    fn rows_fit_five_columns() {
        for (c, g) in GLYPHS {
            for row in g {
                assert!(*row < (1 << GLYPH_COLS), "{c:?} row {row:#b}");
            }
        }
    }

    #[test]
    fn coverage_matches_reference() {
        let expected = "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789 /-.:#>+%@_";
        assert_eq!(GLYPHS.len(), expected.chars().count());
        assert!(missing_glyphs(expected).is_empty());
        // Labels used by the concept render must be fully covered.
        for label in [
            "LMA1980/MODEL-RUNNER",
            "K3-ENGINE @ MODEL-RUNNER",
            "FEAT/RVV-GEMM",
            "#1234",
            "MODEL-RUNNER 7F3A B",
            "212S",
            " -- NONE --",
            ">MUREGIST",
            "83.0%",
            "+1.2",
            "SNAKE_CASE",
        ] {
            assert!(missing_glyphs(label).is_empty(), "{label}");
        }
    }

    #[test]
    fn lowercase_maps_to_uppercase() {
        assert_eq!(glyph('a'), glyph('A'));
        assert!(glyph('~').is_none());
        assert_eq!(missing_glyphs("a~b~"), vec!['~']);
    }

    #[test]
    fn at_sign_and_space() {
        let at = glyph('@').expect("@ glyph");
        assert_eq!(at[2], 0b10111);
        assert_eq!(dots(glyph(' ').expect("space")).count(), 0);
        // 'A' has 18 lit dots in the reference font.
        assert_eq!(dots(glyph('A').expect("A")).count(), 18);
    }
}
