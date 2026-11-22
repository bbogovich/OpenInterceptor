//! # The original's compact list box (RQ-UI-13)
//!
//! The rules of the original's `ListBox` (Interceptor.exe 0x51b7c6, ListBox.cpp;
//! verified unless said otherwise), as pure functions the strategy screen draws with:
//!
//! * **Rows** are one row-font line plus 2 px apart; a box shows
//!   `round((height − 7) / (line height + 2))` of them ([`visible_rows`]).
//! * **Columns** come from a header template such as `"PURCHASE      ~SIZE   ~COST"`:
//!   the text is split at `~` (and tabs; and newlines, which the engine's text tables
//!   make of the original's `~`), and each column starts where the previous
//!   column's header text — trailing spaces included — ends, measured in the header font
//!   (0x51c46b). A `#` in front of a header marks the editable column and is not shown
//!   ([`columns`]). A cell wider than its column is cut with "...".
//! * **The selection bar** is a full-row fill whose green level pulses: a counter `n`
//!   steps every 2 ticks (1/60 s) through 0..74; the level is `150·(1 − n/74)` for
//!   `n < 37` and `150·(0.5 + (n − 37)/74)` after — about 75 to 153 of 255, every 2.5 s
//!   ([`pulse_level`]). A pressed row flashes amber, RGB(200, 150, 0), on alternate frames.
//! * **Scroll arrows** are separate buttons that fire **on press** and then, while held,
//!   repeat after 20 ticks every 5 ticks ([`Repeat`]); each step moves one row; they are
//!   disabled at the ends.

use super::text::{self, Face};

/// How many rows a list box of height `h` shows with a row font of line height `lh`:
/// `round((h − 7) / (lh + 2))`, at least one.
pub fn visible_rows(h: i32, lh: i32) -> usize {
    let pitch = (lh + 2).max(1);
    let num = (h - 7).max(0);
    // Rounded to nearest in integers: (2·num + pitch) / (2·pitch).
    ((2 * num + pitch) / (2 * pitch)).max(1) as usize
}

/// One column of a list: its left edge (from the list's left), its header text and
/// whether it is the editable column (`#`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Column {
    pub x: i32,
    pub title: String,
    pub editable: bool,
}

/// The columns of a header template, measured in the header face. Each column's width is
/// its header token's width (spaces count), so the template's padding sets the layout.
pub fn columns(header: &Face, template: &str) -> Vec<Column> {
    let mut out = Vec::new();
    let mut x = 0;
    // The engine's text tables turn the original's `~` line breaks into newlines, so a
    // newline separates columns too.
    for token in template.split(['~', '\t', '\n']).filter(|t| !t.is_empty()) {
        let (editable, title) = match token.strip_prefix('#') {
            Some(rest) => (true, rest),
            None => (false, token),
        };
        out.push(Column { x, title: title.trim_end().to_string(), editable });
        // The exe measures the token as written (with its `#`, and its trailing spaces).
        x += text::width(header, token, false);
    }
    out
}

/// The width of column `i` in a list `w` wide: up to the next column, the last to the edge.
pub fn column_width(cols: &[Column], i: usize, w: i32) -> i32 {
    let end = cols.get(i + 1).map_or(w, |c| c.x);
    (end - cols[i].x).max(0)
}

/// The pulse counter at time `ticks` (1/60 s): one step every 2 ticks, 0..74.
pub fn pulse_step(ticks: u64) -> u32 {
    ((ticks / 2) % 75) as u32
}

/// The selection bar's level (0-255) at counter `n` (0..74).
pub fn pulse_level(n: u32) -> f32 {
    let n = n.min(74) as f32;
    if n < 37.0 {
        150.0 * (1.0 - n / 74.0)
    } else {
        150.0 * (0.5 + (n - 37.0) / 74.0)
    }
}

/// The amber of a pressed row (shown on alternate frames).
pub const PRESSED: [f32; 3] = [200.0 / 255.0, 150.0 / 255.0, 0.0];

/// A held button's auto-repeat: fires when pressed, then after `delay` ticks every
/// `interval` ticks while it stays held (the exe's 0x41767b; scroll arrows use 20, 5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Repeat {
    /// When it was pressed (ticks), `None` while not held.
    pressed_at: Option<u64>,
    /// The last time it fired.
    last: u64,
}

impl Repeat {
    /// Advances by one frame at `now` (ticks): `down` is whether the button is held with
    /// the cursor inside it, `pressed` whether it went down this frame. Returns whether
    /// the button's action fires this frame.
    pub fn update(&mut self, now: u64, pressed: bool, down: bool, delay: u64, interval: u64) -> bool {
        if pressed {
            self.pressed_at = Some(now);
            self.last = now;
            return true;
        }
        let Some(at) = self.pressed_at.filter(|_| down) else {
            self.pressed_at = None;
            return false;
        };
        if now >= at + delay && now >= self.last + interval {
            self.last = now;
            return true;
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formats::sheetfont::{tests::synthetic_sheet, SheetFont};

    #[test]
    fn rows_that_show_follow_the_exes_rounding() {
        // 23011: line height 14, pitch 16. E&P's box is 188 tall: (188 - 7) / 16 = 11.3 → 11.
        assert_eq!(visible_rows(188, 14), 11);
        assert_eq!(visible_rows(60, 14), 3, "Equip Ships' 60 px box: 3.3 → 3");
        assert_eq!(visible_rows(31, 14), 2, "1.5 rounds up");
        assert_eq!(visible_rows(5, 14), 1, "never none");
    }

    #[test]
    fn columns_start_where_the_previous_header_ends() {
        // Every glyph 5 px wide, space = width of 'e' = 5.
        let (w, h, img) = synthetic_sheet(10, 26, 154, |_| 5, |_| 7);
        let f = SheetFont::parse(w, h, &img).unwrap();
        let face = Face { font: &f, top: 3, wrap_offset: 0, spacing: 0, centre_raise: 0 };
        let cols = columns(&face, "ITEM  ~SIZE ~#QTY");
        assert_eq!(cols.len(), 3);
        assert_eq!((cols[0].x, cols[1].x, cols[2].x), (0, 30, 55), "ITEM + 2 spaces = 30; SIZE + 1 = 25");
        assert_eq!((cols[1].title.as_str(), cols[2].title.as_str(), cols[2].editable), ("SIZE", "QTY", true));
        assert_eq!(column_width(&cols, 0, 200), 30);
        assert_eq!(column_width(&cols, 2, 200), 145, "the last column runs to the edge");
        assert!(columns(&face, "").is_empty());
        assert_eq!(columns(&face, "ITEM  \nSIZE ").len(), 2, "a text table's newline is a column break");
    }

    #[test]
    fn the_selection_bar_pulses_between_75_and_150() {
        assert_eq!(pulse_level(0), 150.0);
        assert!((pulse_level(36) - 77.03).abs() < 0.01);
        assert_eq!(pulse_level(37), 75.0);
        assert!((pulse_level(74) - 150.0).abs() < 0.01);
        assert_eq!((pulse_step(0), pulse_step(3), pulse_step(149), pulse_step(150)), (0, 1, 74, 0), "period 150 ticks");
    }

    #[test]
    fn a_held_arrow_fires_on_press_then_repeats_after_20_ticks_every_5() {
        let mut r = Repeat::default();
        let mut fired = Vec::new();
        for t in 100..140u64 {
            if r.update(t, t == 100, true, 20, 5) {
                fired.push(t - 100);
            }
        }
        assert_eq!(fired, vec![0, 20, 25, 30, 35]);
        // Released: no more, and a new press fires at once.
        assert!(!r.update(141, false, false, 20, 5));
        assert!(!r.update(170, false, true, 20, 5), "held again without a new press does nothing");
        assert!(r.update(171, true, true, 20, 5));
    }
}
