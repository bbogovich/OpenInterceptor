//! # Widgets for the strategy and menu screens (Stage 6.1)
//!
//! Small *immediate-mode* widgets: a screen calls [`button`] every frame with where the
//! button is and what it says, and gets back whether it was clicked this frame — no
//! widget objects are kept between frames, so there is nothing to keep in sync with
//! the game's own state. The few things that must persist (a list's scroll position and
//! selection) live in a plain struct the screen owns ([`ListView`]).
//!
//! Everything that is arithmetic — word wrapping, list scrolling, hit tests — is a pure
//! function, unit-tested; the drawing goes through the campaign's skin ([`Ui`]).

use super::ui::Ui;
use macroquad::prelude::*;

/// A rectangle: x, y, width, height (screen pixels unless said otherwise).
pub type Rect4 = [f32; 4];

/// Whether `p` is inside `r`.
pub fn contains(r: Rect4, p: Vec2) -> bool {
    p.x >= r[0] && p.y >= r[1] && p.x < r[0] + r[2] && p.y < r[1] + r[3]
}

/// The mouse and keys a screen reads once per frame.
#[derive(Debug, Clone, Copy, Default)]
pub struct Input {
    pub mouse: Vec2,
    /// The left button went down this frame.
    pub clicked: bool,
    /// Wheel movement this frame (positive: away from the user).
    pub wheel: f32,
    pub enter: bool,
    pub escape: bool,
}

impl Input {
    /// This frame's input.
    pub fn read() -> Input {
        let (mx, my) = mouse_position();
        Input {
            mouse: vec2(mx, my),
            clicked: is_mouse_button_pressed(MouseButton::Left),
            wheel: mouse_wheel().1,
            enter: is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::KpEnter),
            escape: is_key_pressed(KeyCode::Escape),
        }
    }

    /// A click inside `r`.
    pub fn clicked_in(&self, r: Rect4) -> bool {
        self.clicked && contains(r, self.mouse)
    }
}

/// Splits `text` into lines no wider than `width`, breaking between words (a word wider
/// than the line is cut). `measure` gives a string's width. Explicit newlines are kept.
pub fn wrap(text: &str, width: f32, measure: &dyn Fn(&str) -> f32) -> Vec<String> {
    let mut out = Vec::new();
    for para in text.split('\n') {
        let mut line = String::new();
        for word in para.split_whitespace() {
            let candidate = if line.is_empty() { word.to_string() } else { format!("{line} {word}") };
            if measure(&candidate) <= width || line.is_empty() && measure(word) <= width {
                line = candidate;
                continue;
            }
            if !line.is_empty() {
                out.push(std::mem::take(&mut line));
            }
            // A single word too wide for a line: cut it to fit.
            let mut w = String::new();
            for ch in word.chars() {
                w.push(ch);
                if measure(&w) > width && w.chars().count() > 1 {
                    w.pop();
                    out.push(std::mem::take(&mut w));
                    w.push(ch);
                }
            }
            line = w;
        }
        out.push(line);
    }
    out
}

/// A scrolling list's position: the first row shown and the selected row.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ListView {
    pub first: usize,
    pub selected: usize,
}

impl ListView {
    /// Keeps the selection inside `n` rows and the view showing it, `visible` rows tall.
    pub fn fit(&mut self, n: usize, visible: usize) {
        let visible = visible.max(1);
        self.selected = self.selected.min(n.saturating_sub(1));
        if self.selected < self.first {
            self.first = self.selected;
        }
        if self.selected >= self.first + visible {
            self.first = self.selected + 1 - visible;
        }
        self.first = self.first.min(n.saturating_sub(visible));
    }

    /// Scrolls the view by `rows` (negative: up) without moving the selection.
    pub fn scroll(&mut self, rows: isize, n: usize, visible: usize) {
        let max = n.saturating_sub(visible.max(1));
        self.first = (self.first as isize + rows).clamp(0, max as isize) as usize;
    }

    /// Moves the selection by `by` rows and brings it into view.
    pub fn step(&mut self, by: isize, n: usize, visible: usize) {
        if n == 0 {
            return;
        }
        self.selected = (self.selected as isize + by).clamp(0, n as isize - 1) as usize;
        self.fit(n, visible);
    }
}

/// A push button: drawn in the skin, highlighted under the mouse, dimmed when disabled.
/// Returns whether it was clicked this frame.
pub fn button(ui: &Ui, input: &Input, r: Rect4, label: &str, enabled: bool) -> bool {
    let hover = enabled && contains(r, input.mouse);
    let fill = if hover { ui.colour("highlight") } else { ui.colour("panel") };
    draw_rectangle(r[0], r[1], r[2], r[3], fill);
    draw_rectangle_lines(r[0], r[1], r[2], r[3], 1.5, if enabled { ui.colour("border") } else { ui.colour("dim") });
    let size = (r[3] * 0.55).clamp(11.0, 22.0);
    let tw = ui.measure(label, size);
    let colour = if enabled { ui.colour("text") } else { ui.colour("dim") };
    ui.text_in(label, r[0] + (r[2] - tw).max(4.0) * 0.5, r[1] + r[3] * 0.5 + size * 0.35, size, colour);
    enabled && input.clicked_in(r)
}

/// A row of buttons filling `r` from the left, each `w` wide with `gap` between; returns
/// the index clicked. Buttons that do not fit are left out (the caller sizes the row).
pub fn button_row(ui: &Ui, input: &Input, r: Rect4, labels: &[(&str, bool)], w: f32, gap: f32) -> Option<usize> {
    let mut hit = None;
    for (i, (label, enabled)) in labels.iter().enumerate() {
        let x = r[0] + i as f32 * (w + gap);
        if x + w > r[0] + r[2] + 0.5 {
            break;
        }
        if button(ui, input, [x, r[1], w, r[3]], label, *enabled) {
            hit = Some(i);
        }
    }
    hit
}

/// The rectangles of `n` buttons laid out in a grid of `cols` columns inside `r`.
pub fn grid(r: Rect4, n: usize, cols: usize, gap: f32) -> Vec<Rect4> {
    let cols = cols.max(1);
    let rows = n.div_ceil(cols).max(1);
    let w = (r[2] - gap * (cols - 1) as f32) / cols as f32;
    let h = (r[3] - gap * (rows - 1) as f32) / rows as f32;
    (0..n).map(|i| [r[0] + (i % cols) as f32 * (w + gap), r[1] + (i / cols) as f32 * (h + gap), w, h]).collect()
}

/// A modal prompt over the whole screen: a title, wrapped text and a row of buttons.
/// Returns the button clicked; Enter chooses the first, Esc the last.
pub fn modal(ui: &Ui, input: &Input, title: &str, body: &str, buttons: &[&str]) -> Option<usize> {
    let (sw, sh) = (screen_width(), screen_height());
    draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.0, 0.0, 0.0, 0.55));
    let w = (sw * 0.5).clamp(360.0, 760.0);
    let size = (sh * 0.026).clamp(15.0, 24.0);
    let lines = wrap(body, w - 48.0, &|s| ui.measure(s, size));
    let bh = (size * 1.8).max(30.0);
    let h = 70.0 + lines.len() as f32 * size * 1.3 + bh + 30.0;
    let (x, y) = ((sw - w) * 0.5, (sh - h) * 0.45);
    ui.panel(x, y, w, h);
    ui.text_in(title, x + 24.0, y + 38.0, size * 1.25, ui.colour("border"));
    for (i, l) in lines.iter().enumerate() {
        ui.text_in(l, x + 24.0, y + 70.0 + i as f32 * size * 1.3, size, ui.colour("text"));
    }
    let n = buttons.len().max(1) as f32;
    let bw = ((w - 48.0 - 12.0 * (n - 1.0)) / n).min(220.0);
    let row = [x + w - 24.0 - (bw * n + 12.0 * (n - 1.0)), y + h - bh - 20.0, bw * n + 12.0 * (n - 1.0), bh];
    let labels: Vec<(&str, bool)> = buttons.iter().map(|b| (*b, true)).collect();
    let clicked = button_row(ui, input, row, &labels, bw, 12.0);
    if clicked.is_some() {
        return clicked;
    }
    if input.enter && !buttons.is_empty() {
        return Some(0);
    }
    if input.escape && !buttons.is_empty() {
        return Some(buttons.len() - 1);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every character 10 pixels wide.
    fn m(s: &str) -> f32 {
        s.chars().count() as f32 * 10.0
    }

    #[test]
    fn text_wraps_between_words_and_cuts_only_giant_words() {
        assert_eq!(wrap("one two three", 80.0, &m), vec!["one two", "three"]);
        assert_eq!(wrap("a\nb", 80.0, &m), vec!["a", "b"], "newlines kept");
        assert_eq!(wrap("abcdefghijkl", 50.0, &m), vec!["abcde", "fghij", "kl"], "a giant word is cut");
        assert_eq!(wrap("", 50.0, &m), vec![""]);
    }

    #[test]
    fn lists_keep_the_selection_in_view() {
        let mut v = ListView::default();
        v.step(7, 20, 5);
        assert_eq!((v.selected, v.first), (7, 3), "scrolled just enough to show row 7");
        v.step(-6, 20, 5);
        assert_eq!((v.selected, v.first), (1, 1));
        v.scroll(100, 20, 5);
        assert_eq!(v.first, 15, "no further than the last screenful");
        v.selected = 50;
        v.fit(10, 5);
        assert_eq!((v.selected, v.first), (9, 5), "a shrinking list pulls the selection in");
        v.step(1, 0, 5);
        assert_eq!(v.selected, 9, "an empty list does nothing");
    }

    #[test]
    fn grids_and_hits() {
        let g = grid([0.0, 0.0, 300.0, 60.0], 6, 3, 10.0);
        assert_eq!(g.len(), 6);
        assert_eq!(g[0], [0.0, 0.0, 280.0 / 3.0, 25.0]);
        assert_eq!(g[4][1], 35.0, "second row");
        assert!(contains([10.0, 10.0, 5.0, 5.0], vec2(12.0, 14.0)));
        assert!(!contains([10.0, 10.0, 5.0, 5.0], vec2(15.0, 12.0)), "the far edge is outside");
    }
}
