//! # The credits roll (Stage 6.2)
//!
//! The campaign's credit lines (`campaign.credits.lines`) and then the engine's scroll
//! up over a starfield, headings (lines starting `#`) in the skin's accent colour. It
//! ends when the last line has scrolled off, or on Esc/Enter. A campaign that names a
//! credits film (`credits.video`) plays that instead (the menu sends it to the video
//! player).

use super::AppContext;
use crate::render::camera::SpaceCamera;
use glam::{Quat, Vec3};
use macroquad::prelude::*;

/// Scroll speed, in screen heights per second.
const SPEED: f32 = 0.06;

/// The credits screen.
pub struct Credits {
    lines: Vec<String>,
    time: f32,
}

/// Where line `i` of `n` is (its baseline, pixels from the top) after `t` seconds on a
/// screen `h` high with lines `line` apart: the roll starts just below the screen and
/// moves up. Pure, so it is tested.
pub fn line_y(i: usize, t: f32, h: f32, line: f32) -> f32 {
    h + line + i as f32 * line - t * SPEED * h
}

/// Whether the roll is over: its last line has passed the top.
pub fn finished(n: usize, t: f32, h: f32, line: f32) -> bool {
    line_y(n.saturating_sub(1), t, h, line) < -line
}

impl Credits {
    pub fn new(ctx: &AppContext) -> Credits {
        Credits { lines: ctx.content.rules.credits.roll(), time: 0.0 }
    }

    /// One frame; `true` when over.
    pub fn frame(&mut self, ctx: &mut AppContext) -> bool {
        if is_key_pressed(KeyCode::Escape)
            || is_key_pressed(KeyCode::Enter)
            || is_mouse_button_pressed(MouseButton::Left)
        {
            return true;
        }
        self.time += get_frame_time().min(0.1);
        let (sw, sh) = (screen_width(), screen_height());
        let line = (sh * 0.05).clamp(24.0, 48.0);
        clear_background(ctx.ui.colour("background"));
        let cam = SpaceCamera::new(Vec3::ZERO, Quat::from_rotation_x(self.time * 0.004), 1.0, sw / sh.max(1.0));
        ctx.renderer.draw_sky(&cam, ctx.content.rules.menu.starfield_seed, Vec3::new(-0.5, 0.6, 0.6));
        gl_use_default_material();
        set_default_camera();
        let ui = &ctx.ui;
        for (i, l) in self.lines.iter().enumerate() {
            let y = line_y(i, self.time, sh, line);
            if y < -line || y > sh + line {
                continue;
            }
            let (text, size, colour) = match l.strip_prefix('#') {
                Some(h) => (h.trim(), line * 0.75, ui.colour("accent")),
                None => (l.as_str(), line * 0.6, ui.colour("text")),
            };
            let w = ui.measure(text, size);
            ui.text_in(text, (sw - w) * 0.5, y, size, colour);
        }
        finished(self.lines.len(), self.time, sh, line)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_roll_rises_from_below_and_ends_off_the_top() {
        assert!(line_y(0, 0.0, 600.0, 30.0) > 600.0, "starts below the screen");
        assert!(line_y(0, 10.0, 600.0, 30.0) < line_y(0, 5.0, 600.0, 30.0), "and moves up");
        assert!(!finished(10, 1.0, 600.0, 30.0));
        // Ten lines: the last starts 630 + 270 below the top, and leaves after
        // (900 + 30) / (0.06 × 600) ≈ 25.8 s.
        assert!(!finished(10, 25.0, 600.0, 30.0));
        assert!(finished(10, 26.5, 600.0, 30.0));
    }
}
