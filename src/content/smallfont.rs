//! # The built-in 6-pixel caption font (`builtin:small`)
//!
//! The original game draws its text-button captions, report graphs and small labels with
//! a tiny font compiled into the program, not a sheet in a DLL (Interceptor.exe: glyph
//! routines at 0x614b8c, drawn by `PortDrawString` 0x5ffa81 in any colour). Copying its
//! glyphs would copy original art, so this module carries **our own** 6-pixel design —
//! drawn for this project — whose **advances** (how far the pen moves after each
//! character) are the exe's, read from its width routine 0x617487 (a jump table at
//! 0x6174a4 whose per-character handlers each `add eax, n`; **verified**). With the same
//! advances a caption is exactly as wide as the original's, so it centres on the same
//! pixels.
//!
//! The font is produced as an in-memory **grid sheet** (see [`crate::formats::sheetfont`])
//! so it goes through the same reader, texture upload and layout as any other sheet font.
//! A skin names it with `sheet: builtin:small`; every campaign has it, the original-free
//! `default` included.
//!
//! Glyphs are 6 rows: capitals and digits use rows 0-4, lower case rows 1-4 with
//! ascenders on row 0 and descenders on row 5. Ink is white, so a widget's colour tints
//! it — as the original draws this font in any colour.

use crate::formats::sheetfont::{char_table, CHAR_MAP, MAX_GLYPHS};
use crate::formats::tga::Image;

/// The asset reference that names this font.
pub const REFERENCE: &str = "builtin:small";

/// Glyph rows of the cell (the exe clips this font to 6 rows: `y + 6`).
pub const HEIGHT: usize = 6;

/// The exe's advance per Latin-1 code (0x617487's table; 0 = not drawn, no advance).
/// `/` really is 0 in the exe.
pub fn advances() -> [u8; 256] {
    // (first code, advances from there on) — runs copied from the exe's handlers.
    const RUNS: &[(usize, &[u8])] = &[
        (32, &[3, 2, 3, 5, 5, 8, 6, 2, 3, 3, 4, 4, 2, 3, 2, 0]),
        (48, &[5, 3, 5, 5, 5, 5, 5, 5, 5, 5, 2, 2, 4, 4, 4, 5]),
        (64, &[8, 6, 6, 6, 6, 6, 5, 7, 6, 2, 4, 6, 5, 7, 6, 7]),
        (80, &[6, 7, 6, 6, 6, 6, 6, 8, 6, 6, 5, 3, 3, 3, 4, 7]),
        (96, &[3, 5, 5, 5, 5, 5, 3, 5, 5, 3, 3, 4, 3, 6, 5, 5]),
        (112, &[5, 5, 3, 4, 3, 5, 4, 6, 4, 4, 4, 3, 3, 3, 5, 0]),
        (129, &[5, 5, 5, 5, 5, 5, 0, 5, 5, 5, 5, 5, 5]),
        (153, &[9]),
        (160, &[5, 5]),
        (191, &[5, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 3, 3, 4, 4, 6, 6, 7, 7, 7, 7, 7]),
        (217, &[6, 6, 6, 6, 6, 4, 5, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 3, 3, 4, 4, 6, 6, 7, 7, 7, 7, 7]),
        (249, &[6, 6, 6, 6, 6, 4, 5]),
    ];
    let mut a = [0u8; 256];
    for (first, run) in RUNS {
        a[*first..*first + run.len()].copy_from_slice(run);
    }
    a
}

/// Our own glyph designs: 6 rows each, `#` = ink. Width = the longest row.
const GLYPHS: &[(char, [&str; HEIGHT])] = &[
    ('A', [".##.", "#..#", "####", "#..#", "#..#", ""]),
    ('B', ["###.", "#..#", "###.", "#..#", "###.", ""]),
    ('C', [".###", "#...", "#...", "#...", ".###", ""]),
    ('D', ["###.", "#..#", "#..#", "#..#", "###.", ""]),
    ('E', ["####", "#...", "###.", "#...", "####", ""]),
    ('F', ["###", "#..", "##.", "#..", "#..", ""]),
    ('G', [".###.", "#....", "#.##.", "#..#.", ".###.", ""]),
    ('H', ["#..#", "#..#", "####", "#..#", "#..#", ""]),
    ('I', ["#", "#", "#", "#", "#", ""]),
    ('J', ["..#", "..#", "..#", "#.#", ".#.", ""]),
    ('K', ["#..#", "#.#.", "##..", "#.#.", "#..#", ""]),
    ('L', ["#..", "#..", "#..", "#..", "###", ""]),
    ('M', ["#...#", "##.##", "#.#.#", "#...#", "#...#", ""]),
    ('N', ["#..#", "##.#", "#.##", "#..#", "#..#", ""]),
    ('O', [".###.", "#...#", "#...#", "#...#", ".###.", ""]),
    ('P', ["###.", "#..#", "###.", "#...", "#...", ""]),
    ('Q', [".###.", "#...#", "#...#", "#..#.", ".##.#", ""]),
    ('R', ["###.", "#..#", "###.", "#.#.", "#..#", ""]),
    ('S', [".###", "#...", ".##.", "...#", "###.", ""]),
    ('T', ["#####", "..#..", "..#..", "..#..", "..#..", ""]),
    ('U', ["#..#", "#..#", "#..#", "#..#", ".##.", ""]),
    ('V', ["#...#", "#...#", ".#.#.", ".#.#.", "..#..", ""]),
    ('W', ["#.....#", "#.....#", "#..#..#", ".#.#.#.", "..#.#..", ""]),
    ('X', ["#...#", ".#.#.", "..#..", ".#.#.", "#...#", ""]),
    ('Y', ["#...#", ".#.#.", "..#..", "..#..", "..#..", ""]),
    ('Z', ["####", "..#.", ".#..", "#...", "####", ""]),
    ('a', ["", ".##.", "...#", "#..#", ".###", ""]),
    ('b', ["#...", "###.", "#..#", "#..#", "###.", ""]),
    ('c', ["", ".###", "#...", "#...", ".###", ""]),
    ('d', ["...#", ".###", "#..#", "#..#", ".###", ""]),
    ('e', ["", ".##.", "####", "#...", ".###", ""]),
    ('f', [".#", "##", "#.", "#.", "#.", ""]),
    ('g', ["", ".###", "#..#", ".###", "...#", ".##."]),
    ('h', ["#...", "###.", "#..#", "#..#", "#..#", ""]),
    ('i', ["#", "", "#", "#", "#", ""]),
    ('j', [".#", "", ".#", ".#", ".#", "#."]),
    ('k', ["#..", "#.#", "##.", "#.#", "#.#", ""]),
    ('l', ["#", "#", "#", "#", "#", ""]),
    ('m', ["", "####.", "#.#.#", "#.#.#", "#.#.#", ""]),
    ('n', ["", "###.", "#..#", "#..#", "#..#", ""]),
    ('o', ["", ".##.", "#..#", "#..#", ".##.", ""]),
    ('p', ["", "###.", "#..#", "###.", "#...", "#..."]),
    ('q', ["", ".###", "#..#", ".###", "...#", "...#"]),
    ('r', ["", "##", "#.", "#.", "#.", ""]),
    ('s', ["", ".##", "#..", "..#", "##.", ""]),
    ('t', ["#.", "##", "#.", "#.", ".#", ""]),
    ('u', ["", "#..#", "#..#", "#..#", ".###", ""]),
    ('v', ["", "#.#", "#.#", "#.#", ".#.", ""]),
    ('w', ["", "#...#", "#.#.#", "#.#.#", ".#.#.", ""]),
    ('x', ["", "#.#", ".#.", ".#.", "#.#", ""]),
    ('y', ["", "#.#", "#.#", ".##", "..#", "##."]),
    ('z', ["", "###", "..#", ".#.", "###", ""]),
    ('0', [".##.", "#..#", "#..#", "#..#", ".##.", ""]),
    ('1', [".#", "##", ".#", ".#", ".#", ""]),
    ('2', ["###.", "...#", ".##.", "#...", "####", ""]),
    ('3', ["###.", "...#", ".##.", "...#", "###.", ""]),
    ('4', ["#..#", "#..#", "####", "...#", "...#", ""]),
    ('5', ["####", "#...", "###.", "...#", "###.", ""]),
    ('6', [".##.", "#...", "###.", "#..#", ".##.", ""]),
    ('7', ["####", "...#", "..#.", ".#..", ".#..", ""]),
    ('8', [".##.", "#..#", ".##.", "#..#", ".##.", ""]),
    ('9', [".##.", "#..#", ".###", "...#", ".##.", ""]),
    (',', ["", "", "", "", "#", "#"]),
    ('.', ["", "", "", "", "#", ""]),
    (';', ["", "#", "", "", "#", "#"]),
    ('\'', ["#", "#", "", "", "", ""]),
    ('`', ["#.", ".#", "", "", "", ""]),
    ('<', ["", "..#", ".#.", "#..", ".#.", "..#"]),
    ('>', ["", "#..", ".#.", "..#", ".#.", "#.."]),
    ('?', ["###.", "...#", ".##.", "....", ".#..", ""]),
    (':', ["", "#", "", "", "#", ""]),
    ('"', ["##", "##", "", "", "", ""]),
    ('~', ["", "", ".#.#", "#.#.", "", ""]),
    ('!', ["#", "#", "#", "", "#", ""]),
    ('@', [".#####.", "#.....#", "#.###.#", "#.#####", "#......", ".#####."]),
    ('#', [".#.#", "####", ".#.#", "####", ".#.#", ""]),
    ('$', [".###", "##..", ".##.", "..##", "###.", ""]),
    ('%', ["##...#.", "##..#..", "...#...", "..#..##", ".#...##", ""]),
    ('^', [".#.", "#.#", "", "", "", ""]),
    ('&', [".##..", "#..#.", ".##.#", "#..#.", ".##.#", ""]),
    ('*', ["#.#", ".#.", "#.#", "", "", ""]),
    ('(', [".#", "#.", "#.", "#.", ".#", ""]),
    (')', ["#.", ".#", ".#", ".#", "#.", ""]),
    ('-', ["", "", "##", "", "", ""]),
    ('=', ["", "###", "", "###", "", ""]),
    ('+', ["", ".#.", "###", ".#.", "", ""]),
    ('[', ["##", "#.", "#.", "#.", "##", ""]),
    (']', ["##", ".#", ".#", ".#", "##", ""]),
    ('\\', ["#.", "#.", ".#", ".#", ".#", ""]),
    ('{', [".#", "#.", "##", "#.", ".#", ""]),
    ('}', ["#.", ".#", "##", ".#", "#.", ""]),
];

/// Draws the font as a grid sheet (RGBA): grid lines blue, background magenta (the key),
/// ink white; glyph `g` in cell `g - 1` as [`CHAR_MAP`] numbers them. The accented
/// glyphs reuse their plain letter's design (the advance stays the exe's per character).
pub fn sheet() -> Image {
    const PITCH: usize = 10; // box 8: room for the widest glyph (7) and 6 rows
    const COLS: usize = 26;
    const GRID: [u8; 4] = [0, 0, 255, 255];
    const KEY: [u8; 4] = [255, 0, 255, 255];
    let rows = MAX_GLYPHS.div_ceil(COLS);
    let (w, h) = (COLS * PITCH + 1, rows * PITCH + 2);
    let mut rgba = vec![0u8; w * h * 4];
    for y in 0..h {
        for x in 0..w {
            let on_grid = (x % PITCH == 0 || y % PITCH == 0) && y <= rows * PITCH;
            rgba[(y * w + x) * 4..(y * w + x) * 4 + 4].copy_from_slice(if on_grid { &GRID } else { &KEY });
        }
    }
    let design = |c: char| GLYPHS.iter().find(|(g, _)| *g == c).map(|(_, rows)| rows);
    for (glyph, chars) in CHAR_MAP {
        // The first character of the glyph's entry; an accented letter borrows the
        // design of its plain letter (é → e) so it at least reads.
        let Some(c) = chars.chars().next() else { continue };
        let Some(rows_of) = design(c).or_else(|| design(plain(c))) else { continue };
        let g = *glyph as usize - 1;
        let (cx, cy) = ((g % COLS) * PITCH + 1, (g / COLS) * PITCH + 1);
        for (ry, row) in rows_of.iter().enumerate() {
            for (rx, dot) in row.chars().enumerate() {
                if dot == '#' {
                    let i = ((cy + ry) * w + cx + rx) * 4;
                    rgba[i..i + 4].copy_from_slice(&[255, 255, 255, 255]);
                }
            }
        }
    }
    Image { width: w as u32, height: h as u32, rgba }
}

/// The plain letter an accented Latin-1 letter is built on (only for borrowing a design).
fn plain(c: char) -> char {
    match c {
        'À'..='Å' | 'à'..='å' => 'a',
        'Æ' | 'æ' => 'a',
        'Ç' | 'ç' => 'c',
        'È'..='Ë' | 'è'..='ë' => 'e',
        'Ì'..='Ï' | 'ì'..='ï' => 'i',
        'Ñ' | 'ñ' => 'n',
        'Ò'..='Ö' | 'ò'..='ö' => 'o',
        'Ù'..='Ü' | 'ù'..='ü' => 'u',
        'ß' => 's',
        '¿' => '?',
        other => other,
    }
}

/// Every drawn character's glyph fits inside its advance with a gap (checked by a test;
/// a design wider than the exe's advance would run into the next character).
pub fn design_width(c: char) -> Option<usize> {
    GLYPHS.iter().find(|(g, _)| *g == c).map(|(_, rows)| rows.iter().map(|r| r.len()).max().unwrap_or(0))
}

/// The char table is the grid format's (kept here for the tests' convenience).
pub fn map() -> [u8; 256] {
    char_table()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formats::sheetfont::SheetFont;

    #[test]
    fn the_exes_advances_are_the_fonts() {
        let a = advances();
        assert_eq!((a[b' ' as usize], a[b'I' as usize], a[b'A' as usize], a[b'W' as usize]), (3, 2, 6, 8));
        assert_eq!((a[b'/' as usize], a[b'm' as usize], a[b'%' as usize], a[153]), (0, 6, 8, 9));
        assert_eq!((a[0xe9], a[0xff]), (6, 5));
    }

    #[test]
    fn every_design_fits_its_advance_and_the_sheet_reads() {
        let a = advances();
        for (c, rows) in GLYPHS {
            let w = design_width(*c).unwrap();
            let adv = a[*c as usize] as usize;
            assert!(w < adv, "'{c}' is {w} px wide but advances {adv}");
            assert_eq!(rows.len(), HEIGHT);
        }
        let img = sheet();
        let f = SheetFont::parse(img.width, img.height, &img.rgba).unwrap();
        assert_eq!((f.pitch, f.size), (10, 8));
        assert!(f.line_height <= HEIGHT as u32);
        assert_eq!(f.glyph(f.glyph_of('W').unwrap()).unwrap().width, 7);
        assert_eq!(map()[b'A' as usize], 1);
    }
}
