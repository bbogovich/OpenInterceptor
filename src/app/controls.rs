//! # The controls menu screen
//!
//! Lets the player rebind every action to keys and gamepad buttons (including
//! modifier chords), and change the stick, dead-zone and mouse options. All decisions
//! are made by [`crate::input::editor::Editor`]; this module only feeds it the devices
//! and paints it. Leaving the menu applies the new bindings at once and saves them to
//! the user's `bindings.yaml`.
//!
//! The screen is used both from the launcher and from the in-flight pause menu, so it is
//! a component with a `frame` method rather than a top-level screen of its own.

use super::AppContext;
use crate::input::editor::{Capture, Editor, Row, COLUMNS};
use crate::input::gamepad::Button;
use crate::input::{canonical, info};
use macroquad::prelude::*;

/// Seconds a capture waits for input before giving up.
const CAPTURE_TIMEOUT: f64 = 8.0;

/// The menu component.
pub struct ControlsMenu {
    pub editor: Editor,
    /// First visible row.
    scroll: usize,
    capture_started: f64,
}

impl ControlsMenu {
    pub fn new(ctx: &AppContext) -> ControlsMenu {
        ControlsMenu { editor: Editor::new(&ctx.bindings), scroll: 0, capture_started: 0.0 }
    }

    /// Runs one frame: input, then drawing. Returns true when the player leaves the menu
    /// (the bindings have then been applied and saved).
    pub fn frame(&mut self, ctx: &mut AppContext) -> bool {
        let raw = ctx.devices.read();
        let e = &mut self.editor;
        let pad_pressed = |b| raw.pad_pressed(b);
        match e.capture {
            Some(Capture::Key) => {
                // Prefer a real key over a lone Shift if several arrived in one frame.
                let key = raw.pressed.iter().find(|k| !k.ends_with("shift")).or(raw.pressed.iter().next());
                if let Some(k) = key {
                    e.capture_key(&canonical(k), raw.shift);
                }
            }
            Some(Capture::Pad { .. }) => {
                if raw.pressed.contains("escape") {
                    e.cancel();
                } else if let Some(now) = raw.pad {
                    e.capture_pad(&now, &raw.pad_previous.unwrap_or_default());
                }
            }
            None => {
                let pressed = |k: &str| raw.pressed.contains(k);
                if pressed("escape") || pad_pressed(Button::B) {
                    self.leave(ctx);
                    return true;
                }
                if pressed("up") || pad_pressed(Button::DpadUp) {
                    e.move_row(-1);
                }
                if pressed("down") || pad_pressed(Button::DpadDown) {
                    e.move_row(1);
                }
                if pressed("pageup") || pad_pressed(Button::LeftShoulder) {
                    e.move_row(-12);
                }
                if pressed("pagedown") || pad_pressed(Button::RightShoulder) {
                    e.move_row(12);
                }
                let horizontal = (pressed("right") || pad_pressed(Button::DpadRight)) as i32
                    - (pressed("left") || pad_pressed(Button::DpadLeft)) as i32;
                if horizontal != 0 {
                    match e.row() {
                        Row::Setting(_) => e.adjust(horizontal),
                        _ => e.move_column(horizontal),
                    }
                }
                if pressed("tab") {
                    e.move_column(if raw.shift { -1 } else { 1 });
                }
                if pressed("enter") || pressed("kpenter") || pad_pressed(Button::A) {
                    e.activate();
                }
                if pressed("delete") || pressed("backspace") || pad_pressed(Button::X) {
                    e.clear();
                }
                if pressed("d") || pad_pressed(Button::Y) {
                    if raw.shift {
                        e.reset_all();
                    } else {
                        e.reset_selected();
                    }
                }
                self.mouse(ctx);
            }
        }
        let now = get_time();
        if self.editor.capture.is_none() {
            self.capture_started = now;
        } else if now - self.capture_started > CAPTURE_TIMEOUT {
            self.editor.cancel();
        }
        self.draw();
        false
    }

    /// Applies the edited bindings to the game and saves them.
    fn leave(&mut self, ctx: &mut AppContext) {
        if self.editor.dirty {
            ctx.bindings = self.editor.bindings.clone();
            match super::save_bindings(&ctx.bindings) {
                Ok(path) => eprintln!("controls saved to {}", path.display()),
                Err(err) => eprintln!("controls not saved: {err}"),
            }
        }
    }

    /// Geometry shared by drawing and mouse hit-testing.
    fn layout() -> Layout {
        let w = screen_width();
        let left = 40.0f32.min(w * 0.03);
        let label_w = (w * 0.34).clamp(220.0, 420.0);
        let cell_w = ((w - left * 2.0 - label_w) / COLUMNS.len() as f32).max(80.0);
        Layout {
            left,
            label_w,
            cell_w,
            top: 110.0,
            line: 26.0,
            visible: ((screen_height() - 190.0) / 26.0).max(5.0) as usize,
        }
    }

    fn mouse(&mut self, _ctx: &AppContext) {
        let l = Self::layout();
        let (_, wheel) = mouse_wheel();
        if wheel != 0.0 {
            self.editor.move_row(if wheel > 0.0 { -3 } else { 3 });
        }
        if is_mouse_button_pressed(MouseButton::Left) {
            let (x, y) = mouse_position();
            let row = ((y - l.top) / l.line).floor();
            if row < 0.0 {
                return;
            }
            let index = self.scroll + row as usize;
            if index >= self.editor.rows.len() || matches!(self.editor.rows[index], Row::Header(_)) {
                return;
            }
            let col = ((x - l.left - l.label_w) / l.cell_w).floor();
            let same = self.editor.selected == index;
            self.editor.selected = index;
            if col >= 0.0 && (col as usize) < COLUMNS.len() {
                let same_cell = same && self.editor.column == col as usize;
                self.editor.column = col as usize;
                if same_cell {
                    self.editor.activate();
                }
            } else if same {
                self.editor.activate();
            }
        }
    }

    fn draw(&mut self) {
        let l = Self::layout();
        let e = &self.editor;
        // Keep the selection on screen.
        if e.selected < self.scroll + 1 {
            self.scroll = e.selected.saturating_sub(1);
        } else if e.selected >= self.scroll + l.visible {
            self.scroll = e.selected + 1 - l.visible;
        }
        clear_background(Color::new(0.02, 0.03, 0.06, 1.0));
        draw_text("CONTROLS", l.left, 50.0, 36.0, WHITE);
        draw_text(
            "Up/Down select  Left/Right column  Enter change  Delete clear  D default  Shift+D all defaults  Esc save and leave",
            l.left,
            78.0,
            17.0,
            GRAY,
        );
        draw_text(
            "Gamepad: D-pad select  A change  X clear  Y default  LB/RB page  B save and leave",
            l.left,
            98.0,
            17.0,
            GRAY,
        );
        let accent = Color::new(1.0, 0.75, 0.3, 1.0);
        for (k, name) in COLUMNS.iter().enumerate() {
            draw_text(name, l.left + l.label_w + l.cell_w * k as f32 + 6.0, l.top - 6.0, 18.0, LIGHTGRAY);
        }
        for (vis, i) in (self.scroll..e.rows.len()).take(l.visible).enumerate() {
            let y = l.top + vis as f32 * l.line;
            let selected = i == e.selected;
            match e.rows[i] {
                Row::Header(title) => {
                    draw_text(title, l.left, y + 19.0, 20.0, accent);
                    draw_line(
                        l.left,
                        y + l.line - 2.0,
                        screen_width() - l.left,
                        y + l.line - 2.0,
                        1.0,
                        Color::new(1.0, 0.75, 0.3, 0.3),
                    );
                }
                Row::Action(a) => {
                    if selected {
                        draw_rectangle(
                            l.left - 4.0,
                            y,
                            screen_width() - l.left * 2.0 + 8.0,
                            l.line,
                            Color::new(0.2, 0.3, 0.5, 0.35),
                        );
                    }
                    draw_text(info(a).label, l.left + 12.0, y + 18.0, 18.0, if selected { WHITE } else { LIGHTGRAY });
                    for c in 0..COLUMNS.len() {
                        let x = l.left + l.label_w + l.cell_w * c as f32;
                        let cell_selected = selected && c == e.column;
                        if cell_selected {
                            let waiting = e.capture.is_some();
                            let color =
                                if waiting { Color::new(1.0, 0.6, 0.1, 0.8) } else { Color::new(0.4, 0.7, 1.0, 0.9) };
                            draw_rectangle_lines(x + 2.0, y + 2.0, l.cell_w - 4.0, l.line - 4.0, 2.0, color);
                        }
                        let text = if cell_selected && e.capture.is_some() { "...".to_string() } else { e.cell(a, c) };
                        draw_text(
                            &text,
                            x + 8.0,
                            y + 18.0,
                            18.0,
                            if c < 2 { WHITE } else { Color::new(0.6, 0.9, 1.0, 1.0) },
                        );
                    }
                }
                Row::Setting(s) => {
                    if selected {
                        draw_rectangle(
                            l.left - 4.0,
                            y,
                            screen_width() - l.left * 2.0 + 8.0,
                            l.line,
                            Color::new(0.2, 0.3, 0.5, 0.35),
                        );
                    }
                    draw_text(s.label(), l.left + 12.0, y + 18.0, 18.0, if selected { WHITE } else { LIGHTGRAY });
                    let v = format!("<  {}  >", e.setting_text(s));
                    draw_text(&v, l.left + l.label_w + 8.0, y + 18.0, 18.0, WHITE);
                }
            }
        }
        // Scroll indicator and status line.
        if e.rows.len() > l.visible {
            let track = l.visible as f32 * l.line;
            let h = track * l.visible as f32 / e.rows.len() as f32;
            let y = l.top + track * self.scroll as f32 / e.rows.len() as f32;
            draw_rectangle(screen_width() - 14.0, y, 6.0, h, Color::new(1.0, 1.0, 1.0, 0.3));
        }
        let status_y = screen_height() - 30.0;
        draw_text(&e.message, l.left, status_y, 20.0, if e.capture.is_some() { accent } else { WHITE });
    }
}

/// Column and row geometry.
struct Layout {
    left: f32,
    label_w: f32,
    cell_w: f32,
    top: f32,
    line: f32,
    visible: usize,
}
