//! # Grid sheet fonts: bitmap fonts drawn in a grid of square cells
//!
//! The original strategy screens draw their text with **bitmap sheet fonts**: one image
//! per font (a `TARG` resource in `SpritesDLL.dll`, read at run time from the player's
//! own data), its glyphs laid out in a grid. The colour is painted into the image —
//! there is no tint. This module reads such a sheet into metrics: where each glyph is,
//! how wide it is and how far the pen moves after it. It knows nothing about drawing;
//! `app::text` lays text out with these metrics and `app::ui` draws the glyphs.
//!
//! The same layout is the engine's open **grid sheet** format, so a mod (and the
//! original-free `default` campaign, whose sheet `tools/default_assets/fonts.py`
//! generates) can draw its own font as a PNG in the same grid.
//!
//! ## The layout (the original's font loader, Interceptor.exe 0x5fd750; verified)
//!
//! * Pixel (0, 0) has the **grid colour**; the bottom-right pixel has the **key colour**
//!   (transparent). Both are see-through when drawn.
//! * The **cell pitch** is the first row `y >= 1` whose column-1 pixel is the grid colour
//!   (the first horizontal grid line). Cells are square; the **glyph box** is
//!   `pitch - 2` pixels, at `(col·pitch + 1, row·pitch + 1)`, inside a 1 px grid line.
//! * Glyphs are numbered from 1 (up to 154), left to right, top to bottom.
//! * A glyph's **width** is its rightmost inked column + 1 (inked = neither grid nor key):
//!   the glyph is trimmed on the right (a right/bottom trim is *derived*). Its width is also
//!   its **advance** — how far the pen moves after it.
//! * The **space** advances by the width of glyph 31 ('e').
//! * The **line height** is the lowest inked row + 1 over all glyphs (derived).
//! * Characters map to glyphs through a fixed table ([`CHAR_MAP`]): A-Z are 1-26, a-z
//!   27-52, the digits 53-62, punctuation 63-92, accented Latin-1 letters 93-141.
//!
//! ## Rust idea: `[T; N]` arrays
//!
//! The char → glyph map is a fixed `[u8; 256]` array (one entry per Latin-1 code) stored
//! inline in [`SheetFont`]: no heap allocation, and indexing with a `u8` can never go
//! out of bounds — the compiler knows the array has exactly 256 entries.

use super::{FResult, FormatError};

/// The most glyphs a sheet holds.
pub const MAX_GLYPHS: usize = 154;

/// The first glyph of the accented-letter block. The original draws these glyphs one
/// pixel lower than the others (glyph top = y − 4 instead of y − 5, verified in DrawText
/// 0x5fe4ad); [`crate::app::text`] applies that rule.
pub const ACCENTED_FIRST: u8 = 93;

/// The glyph whose width is the space's advance ('e').
pub const SPACE_WIDTH_GLYPH: u8 = 31;

/// The char → glyph table of the grid format: `(glyph, the characters drawn with it)`.
/// Read from the original exe's table at VA 0x6af488 (verified); several characters can
/// share a glyph (an accented capital is drawn with its lower-case glyph). Glyphs not
/// listed (68, 96-99, …) are never drawn. This is a layout fact of the format, not art.
pub const CHAR_MAP: &[(u8, &str)] = &[
    (1, "A"),
    (2, "B"),
    (3, "C"),
    (4, "D"),
    (5, "E"),
    (6, "F"),
    (7, "G"),
    (8, "H"),
    (9, "I"),
    (10, "J"),
    (11, "K"),
    (12, "L"),
    (13, "M"),
    (14, "N"),
    (15, "O"),
    (16, "P"),
    (17, "Q"),
    (18, "R"),
    (19, "S"),
    (20, "T"),
    (21, "U"),
    (22, "V"),
    (23, "W"),
    (24, "X"),
    (25, "Y"),
    (26, "Z"),
    (27, "a"),
    (28, "b"),
    (29, "c"),
    (30, "d"),
    (31, "e"),
    (32, "f"),
    (33, "g"),
    (34, "h"),
    (35, "i"),
    (36, "j"),
    (37, "k"),
    (38, "l"),
    (39, "m"),
    (40, "n"),
    (41, "o"),
    (42, "p"),
    (43, "q"),
    (44, "r"),
    (45, "s"),
    (46, "t"),
    (47, "u"),
    (48, "v"),
    (49, "w"),
    (50, "x"),
    (51, "y"),
    (52, "z"),
    (53, "1"),
    (54, "2"),
    (55, "3"),
    (56, "4"),
    (57, "5"),
    (58, "6"),
    (59, "7"),
    (60, "8"),
    (61, "9"),
    (62, "0"),
    (63, ","),
    (64, "."),
    (65, "/"),
    (66, ";"),
    (67, "'`"),
    (69, "<"),
    (70, ">"),
    (71, "?"),
    (72, ":"),
    (73, "\""),
    (74, "~"),
    (75, "!"),
    (76, "@"),
    (77, "#"),
    (78, "$"),
    (79, "%"),
    (80, "^"),
    (81, "&"),
    (82, "*"),
    (83, "("),
    (84, ")"),
    (85, "-"),
    (86, "="),
    (87, "+"),
    (88, "["),
    (89, "]"),
    (90, "\\"),
    (91, "{"),
    (92, "}"),
    (93, "¿"),
    (94, "Âà"),
    (95, "Áá"),
    (100, "Æ"),
    (101, "Ç"),
    (102, "Èè"),
    (103, "Éé"),
    (104, "Êê"),
    (105, "Ëë"),
    (106, "Ìì"),
    (107, "Íí"),
    (109, "Ïï"),
    (112, "Òò"),
    (113, "Óó"),
    (114, "Ôô"),
    (115, "Õõ"),
    (116, "Öö"),
    (117, "Ùù"),
    (118, "Úú"),
    (119, "Ûû"),
    (120, "Üü"),
    (123, "ß"),
    (124, "À"),
    (126, "â"),
    (127, "Ãã"),
    (128, "Ää"),
    (131, "ç"),
    (138, "Îî"),
    (141, "Ññ"),
];

/// [`CHAR_MAP`] as a lookup table: Latin-1 code → glyph (0 = not drawn).
pub fn char_table() -> [u8; 256] {
    let mut t = [0u8; 256];
    for (glyph, chars) in CHAR_MAP {
        for c in chars.chars() {
            // Every character in the table is Latin-1 (below 256): the cast is exact.
            t[c as usize] = *glyph;
        }
    }
    t
}

/// One glyph's place in the sheet: the top-left of its box and its trimmed width.
/// Its height is the font's box size.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Glyph {
    pub x: u32,
    pub y: u32,
    pub width: u32,
}

/// A grid sheet font's metrics (the image itself stays with the caller).
#[derive(Debug, Clone, PartialEq)]
pub struct SheetFont {
    /// Cell pitch, pixels (cell size including the 1 px grid line).
    pub pitch: u32,
    /// Glyph box size, pixels (`pitch - 2`).
    pub size: u32,
    /// Cells per row.
    pub cols: u32,
    /// Glyph `g` is `glyphs[g - 1]`.
    pub glyphs: Vec<Glyph>,
    /// The lowest inked row + 1 over all glyphs.
    pub line_height: u32,
    /// The widest glyph a character maps to (the monospace cell).
    pub max_width: u32,
    /// The grid and key colours (RGBA), see-through when drawn.
    pub grid: [u8; 4],
    pub key: [u8; 4],
    /// Latin-1 code → glyph (0 = not drawn).
    pub map: [u8; 256],
}

impl SheetFont {
    /// Reads the metrics of a sheet given as RGBA pixels, rows top to bottom.
    pub fn parse(width: u32, height: u32, rgba: &[u8]) -> FResult<SheetFont> {
        let (w, h) = (width as usize, height as usize);
        if w < 4 || h < 4 || rgba.len() < w * h * 4 {
            return Err(FormatError::Invalid(format!("font sheet {width}x{height} is too small")));
        }
        let px = |x: usize, y: usize| -> [u8; 4] {
            let i = (y * w + x) * 4;
            [rgba[i], rgba[i + 1], rgba[i + 2], rgba[i + 3]]
        };
        let grid = px(0, 0);
        let key = px(w - 1, h - 1);
        if grid == key {
            return Err(FormatError::Invalid("font sheet: the grid and key colours are the same".into()));
        }
        // The first horizontal grid line below the top one gives the pitch.
        let pitch = (1..h).find(|&y| px(1, y) == grid).ok_or_else(|| {
            FormatError::Invalid("font sheet: no grid line found in column 1 (pixel (0,0) is the grid colour)".into())
        })?;
        if pitch < 3 {
            return Err(FormatError::Invalid(format!("font sheet: cell pitch {pitch} is too small")));
        }
        let size = pitch - 2;
        let cols = w / pitch;
        let mut glyphs = Vec::with_capacity(MAX_GLYPHS);
        let mut line_height = 0;
        for g in 0..MAX_GLYPHS {
            let (cx, cy) = ((g % cols) * pitch + 1, (g / cols) * pitch + 1);
            if cy + size > h {
                break;
            }
            let mut right = 0;
            for y in 0..size {
                for x in 0..size {
                    let p = px(cx + x, cy + y);
                    if p != key && p != grid {
                        right = right.max(x + 1);
                        line_height = line_height.max(y + 1);
                    }
                }
            }
            glyphs.push(Glyph { x: cx as u32, y: cy as u32, width: right as u32 });
        }
        if glyphs.len() < SPACE_WIDTH_GLYPH as usize {
            return Err(FormatError::Invalid(format!(
                "font sheet: only {} glyphs fit; the grid format needs at least the letters (52)",
                glyphs.len()
            )));
        }
        // The widest *mapped* glyph (the loader takes the maximum over the char table).
        let map = char_table();
        let max_width = map
            .iter()
            .filter_map(|&g| (g as usize).checked_sub(1).and_then(|i| glyphs.get(i)))
            .map(|g| g.width)
            .max()
            .unwrap_or(0);
        Ok(SheetFont {
            pitch: pitch as u32,
            size: size as u32,
            cols: cols as u32,
            glyphs,
            line_height: line_height as u32,
            max_width,
            grid,
            key,
            map,
        })
    }

    /// The glyph a character is drawn with (`None`: not drawn — the space, an unmapped
    /// character, or a glyph beyond the sheet).
    pub fn glyph_of(&self, c: char) -> Option<u8> {
        let code = c as u32;
        if code >= 256 {
            return None;
        }
        let g = self.map[code as usize];
        (g != 0 && (g as usize) <= self.glyphs.len()).then_some(g)
    }

    /// A glyph by number (1-based).
    pub fn glyph(&self, g: u8) -> Option<&Glyph> {
        (g as usize).checked_sub(1).and_then(|i| self.glyphs.get(i))
    }

    /// How far the pen moves after `c`: its glyph's width; the space moves by the width
    /// of glyph 31; anything not drawn moves 0 (as in the original's width table).
    pub fn advance(&self, c: char) -> u32 {
        if c == ' ' {
            return self.glyph(SPACE_WIDTH_GLYPH).map_or(0, |g| g.width);
        }
        self.glyph_of(c).and_then(|g| self.glyph(g)).map_or(0, |g| g.width)
    }

    /// Makes the grid and key colours transparent in the sheet's pixels (for drawing).
    pub fn clear_background(&self, rgba: &mut [u8]) {
        // `chunks_exact_mut(4)` walks the buffer one pixel at a time, each a mutable
        // 4-byte slice borrowed from `rgba` — no copy, no index arithmetic.
        for p in rgba.chunks_exact_mut(4) {
            if p == self.grid || p == self.key {
                p[3] = 0;
            }
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    /// Builds a synthetic sheet in the grid layout: `cols` cells per row, `pitch` px
    /// cells, `glyphs` glyphs, glyph `g` inked as a `width(g)` × `height(g)` block from
    /// its box's top-left. Grid = blue, key = magenta, ink = white. Used by `app::text`'s
    /// tests too.
    pub fn synthetic_sheet(
        pitch: usize,
        cols: usize,
        glyphs: usize,
        width: impl Fn(usize) -> usize,
        height: impl Fn(usize) -> usize,
    ) -> (u32, u32, Vec<u8>) {
        const GRID: [u8; 4] = [0, 0, 255, 255];
        const KEY: [u8; 4] = [255, 0, 255, 255];
        const INK: [u8; 4] = [255, 255, 255, 255];
        let rows = glyphs.div_ceil(cols);
        let (w, h) = (cols * pitch + 1, rows * pitch + 2);
        let mut img = vec![0u8; w * h * 4];
        let mut put = |x: usize, y: usize, c: [u8; 4]| img[(y * w + x) * 4..(y * w + x) * 4 + 4].copy_from_slice(&c);
        for y in 0..h {
            for x in 0..w {
                let on_line = x % pitch == 0 || y % pitch == 0;
                put(x, y, if on_line && y < rows * pitch + 1 { GRID } else { KEY });
            }
        }
        for g in 0..glyphs {
            let (cx, cy) = ((g % cols) * pitch + 1, (g / cols) * pitch + 1);
            for y in 0..height(g + 1) {
                for x in 0..width(g + 1) {
                    put(cx + x, cy + y, INK);
                }
            }
        }
        // The bottom-right pixel must be the key (it is: the last row is below the grid).
        (w as u32, h as u32, img)
    }

    #[test]
    fn the_pitch_comes_from_the_first_grid_line_and_glyphs_are_trimmed() {
        let (w, h, img) = synthetic_sheet(16, 26, 154, |g| 1 + g % 12, |g| if g == 7 { 14 } else { 10 });
        let f = SheetFont::parse(w, h, &img).unwrap();
        assert_eq!((f.pitch, f.size, f.cols), (16, 14, 26));
        assert_eq!(f.glyphs.len(), 154);
        assert_eq!(f.glyph(1), Some(&Glyph { x: 1, y: 1, width: 2 }));
        assert_eq!(f.glyph(27).unwrap(), &Glyph { x: 1, y: 17, width: 4 }, "glyph 27 starts row 2");
        assert_eq!(f.line_height, 14, "the tallest glyph sets the line height");
        assert_eq!(f.max_width, 12);
        assert!(SheetFont::parse(2, 2, &[0; 16]).is_err());
        // Glyph 68 is mapped to no character: it does not widen the monospace cell.
        let (w, h, img) = synthetic_sheet(16, 26, 154, |g| if g == 68 { 13 } else { 5 }, |_| 9);
        assert_eq!(SheetFont::parse(w, h, &img).unwrap().max_width, 5);
    }

    #[test]
    fn characters_map_to_glyphs_and_space_takes_the_width_of_e() {
        let (w, h, img) = synthetic_sheet(10, 20, 154, |g| g % 8 + 1, |_| 6);
        let f = SheetFont::parse(w, h, &img).unwrap();
        assert_eq!(f.glyph_of('A'), Some(1));
        assert_eq!(f.glyph_of('a'), Some(27));
        assert_eq!(f.glyph_of('1'), Some(53));
        assert_eq!(f.glyph_of('0'), Some(62));
        assert_eq!(f.glyph_of('$'), Some(78));
        assert_eq!(f.glyph_of('É'), f.glyph_of('é'), "accented capitals share the small letter's glyph");
        assert_eq!(f.glyph_of(' '), None, "the space is not drawn");
        assert_eq!(f.glyph_of('€'), None, "nothing beyond Latin-1");
        assert_eq!(f.advance(' '), f.glyph(31).unwrap().width);
        assert_eq!(f.advance('A'), 2);
        assert_eq!(f.advance('\u{1}'), 0, "unmapped characters do not move the pen");
    }

    #[test]
    fn a_short_sheet_stops_at_its_last_row_and_clear_background_keys_out() {
        let (w, h, mut img) = synthetic_sheet(8, 10, 60, |_| 3, |_| 3);
        let f = SheetFont::parse(w, h, &img).unwrap();
        assert_eq!(f.glyphs.len(), 60);
        assert_eq!(f.glyph_of('9'), None, "glyph 61 is beyond this sheet");
        f.clear_background(&mut img);
        assert_eq!(img[3], 0, "grid pixel is see-through");
        assert_eq!(img[((w * h) as usize - 1) * 4 + 3], 0, "key pixel is see-through");
        let ink = ((1 + w) * 4) as usize; // pixel (1, 1): glyph 1's ink
        assert_eq!(img[ink + 3], 255);
    }
}
