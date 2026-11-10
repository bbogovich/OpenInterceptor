//! # Painting the HUD
//!
//! macroquad 2D drawing of what the parent module computes. These functions need a
//! window, so they are verified by the manual test cases (docs site, *Testing → Manual
//! tests*) rather than unit tests; all the decisions they paint are tested in `hud`.

use super::{gauge_angle, gauge_values, mfd_lines, radar_point, Layout, MessageLog, MfdPage};
use crate::render::camera::SpaceCamera;
use crate::sim::{ai, ShipId, World};
use glam::{Vec2, Vec3};
use macroquad::prelude::*;
use std::collections::HashMap;

/// Everything the HUD needs that is not in the world.
pub struct HudFrame<'a> {
    pub world: &'a World,
    pub player: ShipId,
    pub camera: &'a SpaceCamera,
    pub layout: Layout,
    pub instruments: &'a HashMap<String, [f32; 4]>,
    pub mfd: MfdPage,
    pub nav: Option<(&'a str, Vec3)>,
    pub messages: &'a MessageLog,
    pub now: f64,
    /// Glass instruments (no cockpit art): draw backgrounds behind each instrument.
    pub glass: bool,
    /// Draw target brackets and the reticle (only when looking forward).
    pub forward_view: bool,
}

fn col(c: [f32; 4]) -> Color {
    Color::new(c[0], c[1], c[2], c[3])
}

const HUD_GREEN: Color = Color::new(0.35, 1.0, 0.45, 0.9);
const HUD_DIM: Color = Color::new(0.35, 1.0, 0.45, 0.35);

/// Draws text with its top-left at `(x, y)` scaled to `size` pixels.
fn text(s: &str, x: f32, y: f32, size: f32, color: Color) {
    draw_text(s, x, y + size * 0.8, size, color);
}

/// Draws all instruments.
pub fn draw_instruments(f: &HudFrame) {
    let Some(me) = f.world.get(f.player) else { return };
    let rules = &f.world.rules;
    let get = |name: &str| f.instruments.get(name).map(|r| f.layout.rect(*r));
    if f.glass {
        for r in f.instruments.values().map(|r| f.layout.rect(*r)) {
            draw_rectangle(r[0], r[1], r[2], r[3], Color::new(0.0, 0.05, 0.0, 0.45));
        }
    }
    // Radar scopes.
    for (name, front) in [("radar_front", true), ("radar_rear", false)] {
        let Some(r) = get(name) else { continue };
        let c = Vec2::new(r[0] + r[2] * 0.5, r[1] + r[3] * 0.5);
        let radius = r[2].min(r[3]) * 0.5;
        draw_circle_lines(c.x, c.y, radius, 1.0, HUD_DIM);
        draw_circle_lines(c.x, c.y, radius * 0.5, 1.0, HUD_DIM);
        for id in f.world.live_ids() {
            if id == f.player {
                continue;
            }
            let s = &f.world.ships[id.index as usize];
            let dist = s.position.distance(me.position);
            if dist > rules.flight.radar_range && rules.ship(s.class).max_speed > 0.0 {
                continue;
            }
            let local = me.rotation.inverse() * (s.position - me.position);
            if let Some(p) = radar_point(local, front) {
                let at = c + p * radius * 0.92;
                let color = col(super::faction_color(rules, s.faction));
                let size = if rules.ship(s.class).kind.is_static() { 3.0 } else { 1.8 };
                draw_rectangle(at.x - size, at.y - size, size * 2.0, size * 2.0, color);
                if Some(id) == me.target {
                    draw_rectangle_lines(at.x - 4.0, at.y - 4.0, 8.0, 8.0, 1.0, WHITE);
                }
            }
        }
        text(if front { "F" } else { "R" }, r[0], r[1], 10.0 * f.layout.scale.max(1.0), HUD_DIM);
    }
    // Gauges: speed, energy, shields, hull.
    let values = gauge_values(rules, me);
    let labels = ["SPD", "NRG", "SHD", "HUL"];
    for (k, (v, label)) in values.iter().zip(labels).enumerate() {
        let Some(r) = get(&format!("gauge{}", k + 1)) else { continue };
        let c = Vec2::new(r[0] + r[2] * 0.5, r[1] + r[3] * 0.5);
        let radius = r[2].min(r[3]) * 0.45;
        let color = if *v < 0.25 && k > 0 { RED } else { HUD_GREEN };
        let a = gauge_angle(*v);
        draw_line(c.x, c.y, c.x + a.cos() * radius, c.y + a.sin() * radius, 2.0, color);
        draw_circle(c.x, c.y, 1.5, color);
        if f.glass {
            draw_circle_lines(c.x, c.y, radius, 1.0, HUD_DIM);
            text(label, c.x - radius, c.y + radius * 0.6, 9.0 * f.layout.scale.max(1.0), HUD_DIM);
        }
    }
    // MFD.
    if let Some(r) = get("mfd") {
        let lines = mfd_lines(f.world, f.player, f.mfd, f.nav);
        let size = (r[3] / 5.5).max(10.0);
        for (i, l) in lines.iter().enumerate() {
            text(l, r[0] + 4.0, r[1] + 3.0 + i as f32 * size * 1.05, size, HUD_GREEN);
        }
    }
    // Message line.
    if let Some(r) = get("message") {
        let size = (r[3] * 0.8).max(11.0);
        if let Some(m) = f.messages.visible(f.now, 1).last() {
            let w = measure_text(m, None, size as u16, 1.0).width;
            text(m, r[0] + (r[2] - w) * 0.5, r[1] + (r[3] - size) * 0.5, size, HUD_GREEN);
        }
    }
    // Throttle and status strip under the message line.
    let class = rules.ship(me.class);
    let speed = me.forward_speed();
    let mut status = format!("THR {:>3.0}%  {:>4.0} m/s", me.throttle * 100.0, speed);
    if me.reverse {
        status += "  REV";
    }
    if me.afterburning {
        status += "  BURN";
    }
    if me.braking {
        status += "  BRAKE";
    }
    if let Some(h) = me.speed_hold {
        status += &format!("  HOLD {h:.0}");
    }
    if class.vct {
        status += "  VCT";
    }
    let size = 10.0 * f.layout.scale.max(1.0);
    text(&status, f.layout.offset.x + 8.0, 40.0 * f.layout.scale, size, HUD_GREEN);
}

/// Reticle, target brackets, lead indicator and off-screen arrow.
pub fn draw_targeting(f: &HudFrame) {
    if !f.forward_view {
        return;
    }
    let Some(me) = f.world.get(f.player) else { return };
    let screen = Vec2::new(screen_width(), screen_height());
    let centre = screen * 0.5;
    let s = 10.0 * f.layout.scale.max(1.0);
    draw_line(centre.x - s, centre.y, centre.x - s * 0.4, centre.y, 1.5, HUD_GREEN);
    draw_line(centre.x + s * 0.4, centre.y, centre.x + s, centre.y, 1.5, HUD_GREEN);
    draw_line(centre.x, centre.y - s, centre.x, centre.y - s * 0.4, 1.5, HUD_GREEN);
    draw_line(centre.x, centre.y + s * 0.4, centre.x, centre.y + s, 1.5, HUD_GREEN);
    let Some(target) = me.target.and_then(|t| f.world.get(t)) else { return };
    let rules = &f.world.rules;
    let color = col(super::faction_color(rules, target.faction));
    let radius = f.world.shapes[target.class.0 as usize].radius;
    match f.camera.project(target.position, screen) {
        Some(p) if p.x >= 0.0 && p.y >= 0.0 && p.x <= screen.x && p.y <= screen.y => {
            // Bracket size from the target's angular size, with a minimum.
            let dist = target.position.distance(f.camera.position).max(1.0);
            let px = (radius / dist) / (f.camera.fov_y * 0.5).tan() * screen.y * 0.5;
            let h = px.max(10.0);
            let k = h * 0.4;
            for (sx, sy) in [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)] {
                let corner = Vec2::new(p.x + sx * h, p.y + sy * h);
                draw_line(corner.x, corner.y, corner.x - sx * k, corner.y, 1.5, color);
                draw_line(corner.x, corner.y, corner.x, corner.y - sy * k, 1.5, color);
            }
            if me.lock.locked {
                let d = h * 1.3;
                draw_triangle_lines(
                    Vec2::new(p.x, p.y - d),
                    Vec2::new(p.x + d, p.y),
                    Vec2::new(p.x, p.y + d),
                    1.5,
                    RED,
                );
                draw_triangle_lines(
                    Vec2::new(p.x, p.y - d),
                    Vec2::new(p.x - d, p.y),
                    Vec2::new(p.x, p.y + d),
                    1.5,
                    RED,
                );
                text("LOCK", p.x + d + 4.0, p.y - 6.0, 12.0, RED);
            } else if me.lock.progress > 0.0 {
                let t = me.missiles.get(me.selected_rack).map(|(m, _)| rules.missile(*m).lock_time).unwrap_or(1.0);
                let frac = (me.lock.progress / t).min(1.0);
                draw_circle_lines(p.x, p.y, h * (2.2 - 1.2 * frac), 1.0, Color::new(1.0, 0.3, 0.3, 0.8));
            }
            let label = format!("{} {}", rules.ship(target.class).name, super::format_distance(dist));
            text(&label, p.x - h, p.y + h + 2.0, 11.0 * f.layout.scale.max(1.0), color);
            // Lead indicator: where to aim the beams.
            let speed = rules.ship(me.class).beams.first().map(|b| rules.beam(*b).speed).unwrap_or(2000.0);
            let lead = ai::lead_point(me.position, target.position, target.velocity - me.velocity, speed);
            if let Some(l) = f.camera.project(lead, screen) {
                draw_circle_lines(l.x, l.y, 4.0, 1.0, color);
            }
        }
        _ => {
            // Off screen: an arrow on a ring around the centre, pointing at the target.
            let local = f.camera.local_direction(target.position);
            let dir = Vec2::new(local.x, -local.y).normalize_or(Vec2::Y);
            let ring = screen.y * 0.3;
            let tip = centre + dir * ring;
            let back = centre + dir * (ring - 14.0);
            let side = Vec2::new(-dir.y, dir.x) * 7.0;
            draw_triangle(tip, back + side, back - side, color);
        }
    }
}

/// Paints the cockpit art over the whole design space.
pub fn draw_cockpit(tex: &Texture2D, layout: Layout) {
    let r = layout.full();
    draw_texture_ex(
        tex,
        r[0],
        r[1],
        WHITE,
        DrawTextureParams { dest_size: Some(Vec2::new(r[2], r[3])), ..Default::default() },
    );
}

/// Small caption in a corner (view name, pause state, help).
pub fn draw_caption(s: &str, corner_right: bool) {
    let size = 16.0;
    let w = measure_text(s, None, size as u16, 1.0).width;
    let x = if corner_right { screen_width() - w - 10.0 } else { 10.0 };
    text(s, x, screen_height() - size - 8.0, size, Color::new(1.0, 1.0, 1.0, 0.7));
}
