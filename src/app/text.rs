//! # Text layout with sheet fonts: the original's anchor rules (RQ-UI-03/04)
//!
//! The original strategy screens place every string with one of two routines, and the
//! playtest's "text is mis-sized and not aligned with its control" came from guessing
//! TrueType baselines instead of following them. This module re-implements both as
//! **pure functions**: they take a font's metrics ([`Face`]) and return where each glyph
//! goes ([`Placed`], in design-space pixels). `app::ui` draws the result; the tests
//! below check the positions against the exe's own numbers.
//!
//! * [`draw_text`] — `DrawText(x, y, flags)` (Interceptor.exe 0x5fe4ad): `x` is the left
//!   edge, the centre (flag 1) or the right edge (flag 4) of the line. **`y` is not the
//!   top**: the glyph top is `y − top`, where `top` is 5, or 7 for the four small fonts
//!   23010/23011/23013/23015 — and accented glyphs sit one pixel lower.
//! * [`draw_text_rect`] — `DrawTextRect(rect, flags)` (0x5fed2b): a line that fits and
//!   has no `~` is drawn at the rect's left (or centre, flag 1) and at **y = top + h/2**,
//!   i.e. vertically centred. Otherwise the text wraps at spaces (`~` forces a break),
//!   lines step by `lh + lh/3`; centred text is centred as a block, left text starts at
//!   `top + lh/2 + lh`; the four small fonts add a further 4 px (`wrap_offset`).
//!
//! All arithmetic is integer, as in the original: `/` on `i32` truncates toward zero,
//! exactly like the exe's `cdq; sub eax, edx; sar eax, 1` idiom, so positions match
//! pixel for pixel.

use crate::formats::sheetfont::{SheetFont, ACCENTED_FIRST};

/// A sheet font with its placement rules.
#[derive(Debug, Clone, Copy)]
pub struct Face<'a> {
    pub font: &'a SheetFont,
    /// Glyph top = anchor y − `top` (5 in the original, 7 for its four small fonts).
    pub top: i32,
    /// Added to the first line's y of wrapped text (4 for the four small fonts).
    pub wrap_offset: i32,
    /// Extra pixels after every character (0 in the original; for mod fonts drawn
    /// without a built-in gap).
    pub spacing: i32,
    /// A centred line moves up this much (the original's 6 px font: `y −= 2` with flag 1,
    /// PortDrawString 0x5ffa81).
    pub centre_raise: i32,
}

/// Horizontal alignment: the DrawText flags (none, 1, 4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Align {
    Left,
    Centre,
    Right,
}

impl Align {
    /// `left`, `centre` (or `center`), `right`; anything else is left.
    pub fn parse(s: &str) -> Align {
        match s {
            "centre" | "center" => Align::Centre,
            "right" => Align::Right,
            _ => Align::Left,
        }
    }
}

/// One glyph to draw: its number in the sheet and the top-left of its box.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Placed {
    pub glyph: u8,
    pub x: i32,
    pub y: i32,
}

/// How far the pen moves after `c` (monospace: every character takes the widest glyph).
fn advance(face: &Face, c: char, mono: bool) -> i32 {
    if mono {
        face.font.max_width as i32 + face.spacing
    } else {
        face.font.advance(c) as i32 + face.spacing
    }
}

/// The width of a line, pixels (the original's 0x5fe260: the sum of the advances).
pub fn width(face: &Face, s: &str, mono: bool) -> i32 {
    s.chars().map(|c| advance(face, c, mono)).sum()
}

/// `DrawText`: one line anchored at (x, y). Appends its glyphs to `out` (a caller's
/// buffer, so a screen can reuse one allocation every frame).
pub fn draw_text(face: &Face, s: &str, x: i32, y: i32, align: Align, mono: bool, out: &mut Vec<Placed>) {
    let w = width(face, s, mono);
    let mut pen = match align {
        Align::Left => x,
        Align::Centre => x - w / 2,
        Align::Right => x - w,
    };
    let y = if align == Align::Centre { y - face.centre_raise } else { y };
    for c in s.chars() {
        if let Some(g) = face.font.glyph_of(c) {
            let gw = face.font.glyph(g).map_or(0, |g| g.width as i32);
            // Monospace centres each glyph in a max-width cell (the exe's flag 0x44c420).
            let gx = if mono { pen + (face.font.max_width as i32 - gw) / 2 } else { pen };
            let drop = if g >= ACCENTED_FIRST { 1 } else { 0 };
            out.push(Placed { glyph: g, x: gx, y: y - face.top + drop });
        }
        pen += advance(face, c, mono);
    }
}

/// Splits text into lines no wider than `max` the way the original does: words are
/// separated by spaces and `~`; a word goes to a new line when the line would get too
/// wide, or when the character just before it is `~`. Lines carry no trailing space.
pub fn wrap(face: &Face, s: &str, max: i32, mono: bool) -> Vec<String> {
    let mut lines = Vec::new();
    let mut line = String::new();
    let mut prev_delim = None;
    let mut word = String::new();
    // Walk the characters, cutting words at delimiters (runs of delimiters count as one,
    // as with C's `strtok`, but the last one before a word decides a forced break).
    let mut words: Vec<(String, Option<char>)> = Vec::new();
    for c in s.chars() {
        if c == ' ' || c == '~' {
            if !word.is_empty() {
                words.push((std::mem::take(&mut word), prev_delim));
            }
            prev_delim = Some(c);
        } else {
            word.push(c);
        }
    }
    if !word.is_empty() {
        words.push((word, prev_delim));
    }
    for (i, (w, before)) in words.into_iter().enumerate() {
        let candidate = if line.is_empty() { w.clone() } else { format!("{line} {w}") };
        let forced = i > 0 && before == Some('~');
        if (width(face, &candidate, mono) > max || forced) && !line.is_empty() {
            lines.push(std::mem::replace(&mut line, w));
        } else {
            line = candidate;
        }
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines
}

/// `DrawTextRect`: text in the rectangle `[x, y, w, h]` (one line centred vertically,
/// or wrapped). `Align::Right` is an engine extension for a single line (the original
/// has no right-aligned rect text): the line ends at the rect's right edge.
pub fn draw_text_rect(face: &Face, s: &str, rect: [i32; 4], align: Align, mono: bool, out: &mut Vec<Placed>) {
    let [left, top, w, h] = rect;
    let x = match align {
        Align::Left => left,
        Align::Centre => left + w / 2,
        Align::Right => left + w,
    };
    if width(face, s, mono) <= w && !s.contains('~') {
        draw_text(face, s, x, top + h / 2, align, mono, out);
        return;
    }
    let lines = wrap(face, s, w, mono);
    let lh = face.font.line_height as i32;
    let step = lh + lh / 3;
    let mut y =
        if align == Align::Centre { (top + h / 2) - (step * lines.len() as i32) / 2 } else { top + lh / 2 + lh };
    y += face.wrap_offset;
    // Wrapped lines keep the flag's horizontal alignment (right falls back to left: the
    // original's flag 4 is not used with wrapped text).
    let line_align = if align == Align::Centre { Align::Centre } else { Align::Left };
    let lx = if line_align == Align::Centre { x } else { left };
    for l in &lines {
        draw_text(face, l, lx, y, line_align, mono, out);
        y += step;
    }
}

/// Wrapped text from the **top** of a rectangle (an engine layout, not one of the exe's
/// routines: used for a list's detail box, which the original screens do not have):
/// glyph tops of the first line at the rect's top, then one line every `lh + lh/3`;
/// lines that would end below the rect are left out.
pub fn draw_text_top(face: &Face, s: &str, rect: [i32; 4], out: &mut Vec<Placed>) {
    let lh = face.font.line_height as i32;
    let step = lh + lh / 3;
    let bottom = rect[1] + rect[3];
    for (i, l) in wrap(face, s, rect[2], false).iter().enumerate() {
        let top = rect[1] + i as i32 * step;
        if top + lh > bottom {
            break;
        }
        draw_text(face, l, rect[0], top + face.top, Align::Left, false, out);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formats::sheetfont::tests::synthetic_sheet;

    /// A synthetic font shaped like 23010: 16 px cells, 14 px line height, widths
    /// 6 px for most glyphs and 12 for 'W' (glyph 23).
    fn font() -> SheetFont {
        let (w, h, img) = synthetic_sheet(16, 26, 154, |g| if g == 23 { 12 } else { 6 }, |_| 14);
        SheetFont::parse(w, h, &img).unwrap()
    }

    fn face(f: &SheetFont) -> Face<'_> {
        Face { font: f, top: 7, wrap_offset: 4, spacing: 0, centre_raise: 0 }
    }

    #[test]
    fn draw_text_anchors_glyph_tops_at_y_minus_top() {
        let f = font();
        let mut out = Vec::new();
        // The CSD time: centred on x 561 at y 370 → glyph tops at 363 (ui-framework §3.3).
        draw_text(&face(&f), "12:00", 561, 370, Align::Centre, false, &mut out);
        assert!(out.iter().all(|p| p.y == 363));
        assert_eq!(width(&face(&f), "12:00", false), 30);
        assert_eq!(out[0].x, 561 - 15, "centred: x − width/2");
        let mut right = Vec::new();
        draw_text(&face(&f), "AB", 100, 20, Align::Right, false, &mut right);
        assert_eq!((right[0].x, right[1].x), (88, 94), "flag 4: the line ends at x");
        // A font with the plain rule (top = y − 5) and an accented glyph one lower.
        let mut acc = Vec::new();
        draw_text(&Face { top: 5, ..face(&f) }, "Eé", 0, 10, Align::Left, false, &mut acc);
        assert_eq!((acc[0].y, acc[1].y), (5, 6));
        // The small font: y is the glyph top, and a centred line moves up 2 (flag 1).
        let small = Face { top: 0, centre_raise: 2, ..face(&f) };
        let mut c = Vec::new();
        draw_text(&small, "A", 50, 10, Align::Centre, false, &mut c);
        draw_text(&small, "A", 50, 10, Align::Left, false, &mut c);
        assert_eq!((c[0].y, c[1].y), (8, 10));
    }

    #[test]
    fn spaces_move_the_pen_without_a_glyph_and_monospace_centres_in_cells() {
        let f = font();
        let mut out = Vec::new();
        draw_text(&face(&f), "A W", 0, 7, Align::Left, false, &mut out);
        assert_eq!(out.len(), 2, "the space draws nothing");
        assert_eq!(out[1].x, 12, "A (6) + space (width of 'e', 6)");
        let mut mono = Vec::new();
        draw_text(&face(&f), "AW", 0, 7, Align::Left, true, &mut mono);
        assert_eq!(f.max_width, 12);
        assert_eq!((mono[0].x, mono[1].x), (3, 12), "each glyph centred in a 12 px cell");
        assert_eq!(width(&face(&f), "AW", true), 24);
    }

    #[test]
    fn a_line_that_fits_is_centred_vertically_in_its_rect() {
        let f = font();
        let mut out = Vec::new();
        // CSD status bar [344, 457, 276, 15], centred: y = 457 + 7 = 464 → top 457.
        draw_text_rect(&face(&f), "Sol", [344, 457, 276, 15], Align::Centre, false, &mut out);
        assert_eq!(out[0].y, 457);
        assert_eq!(out[0].x, 344 + 138 - 9);
        let mut left = Vec::new();
        draw_text_rect(&face(&f), "Sol", [10, 0, 100, 20], Align::Left, false, &mut left);
        assert_eq!((left[0].x, left[0].y), (10, 3));
    }

    #[test]
    fn long_text_wraps_at_spaces_and_tildes_force_breaks() {
        let f = font();
        let fc = face(&f);
        // Each letter 6 px, space 6 px: "aaa bbb" = 42 px.
        assert_eq!(wrap(&fc, "aaa bbb ccc", 42, false), vec!["aaa bbb", "ccc"]);
        assert_eq!(wrap(&fc, "aaa~bbb", 200, false), vec!["aaa", "bbb"]);
        assert_eq!(wrap(&fc, "aaa~ bbb", 200, false), vec!["aaa bbb"], "the last delimiter decides");
        assert_eq!(wrap(&fc, "aaaaaaaaaa b", 30, false), vec!["aaaaaaaaaa", "b"], "a long word overflows");
        // Left: first y = top + lh/2 + lh + wrap_offset = 0 + 7 + 14 + 4 = 25; step 14 + 4.
        let mut out = Vec::new();
        draw_text_rect(&fc, "aaa~bbb", [0, 0, 200, 100], Align::Left, false, &mut out);
        let tops: Vec<i32> = out.iter().map(|p| p.y).collect();
        assert_eq!(tops, vec![18, 18, 18, 36, 36, 36]);
        // Centred: the block of 2 lines (2 × 18) is centred on y 50: 50 − 18 + 4 = 36.
        let mut c = Vec::new();
        draw_text_rect(&fc, "aaa~bbb", [0, 0, 200, 100], Align::Centre, false, &mut c);
        assert_eq!((c[0].y, c[3].y), (36 - 7, 54 - 7));
        assert_eq!(c[0].x, 100 - 9);
        // From the top: tops at 0 and 18; a third line would end below 40 and is left out.
        let mut t = Vec::new();
        draw_text_top(&fc, "aaa~bbb~ccc", [0, 0, 200, 40], &mut t);
        let tops: Vec<i32> = t.iter().map(|p| p.y).collect();
        assert_eq!(tops, vec![0, 0, 0, 18, 18, 18]);
    }
}
