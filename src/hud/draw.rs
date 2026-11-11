//! # Painting the HUD
//!
//! macroquad 2D drawing of what the parent module computes. These functions need a
//! window, so they are verified by the manual test cases (docs site, *Testing → Manual
//! tests*) rather than unit tests; all the decisions they paint are tested in `hud`.

use super::{
    gauge_angle, gauge_values, mfd_lines, power_readout, radar_point, status_flags, system_lamps, Lamp, Layout,
    MessageLog, MfdPage,
};
use crate::content::equipment::PowerRules;
use crate::render::camera::SpaceCamera;
use crate::sim::ShieldBalance;
use crate::sim::{ai, ShipId, World};
use glam::{Vec2, Vec3};
use macroquad::prelude::*;
use std::collections::HashMap;

/// Everything the HUD needs that is not in the world.
#[derive(Clone, Copy)]
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
    /// Glass instruments (no cockpit art): draw scope rings and gauge dials.
    pub glass: bool,
    /// Draw a dark panel behind each instrument (the plain glass HUD; a 3D cockpit's
    /// bezels already frame its instruments).
    pub backdrops: bool,
    /// Draw target brackets and the reticle (only when looking forward).
    pub forward_view: bool,
    /// Instruments drawn elsewhere (in perspective on a 3D cockpit): skipped here.
    pub anchored: &'a std::collections::HashSet<String>,
    /// Text size limit multiplier (instrument textures are drawn larger than the screen).
    pub text_scale: f32,
}

fn col(c: [f32; 4]) -> Color {
    Color::new(c[0], c[1], c[2], c[3])
}

const HUD_GREEN: Color = Color::new(0.35, 1.0, 0.45, 0.9);
const HUD_DIM: Color = Color::new(0.35, 1.0, 0.45, 0.35);
/// Instrument labels: dimmer than readings, bright enough to read.
const HUD_LABEL: Color = Color::new(0.35, 1.0, 0.45, 0.75);

/// Smallest text an instrument draws, in screen pixels: fitted text never shrinks
/// below this (it is clipped instead), or it would be unreadable.
const MIN_TEXT: f32 = 14.0;

/// `size` raised to the legible minimum (scaled for instrument textures).
fn legible(size: f32, f: &HudFrame) -> f32 {
    size.max(MIN_TEXT * f.text_scale)
}

/// `s` shortened (ending in "..") so it fits `w` pixels at `size`.
fn clip(s: &str, size: f32, w: f32) -> String {
    if width_per_px(s) * size <= w {
        return s.to_string();
    }
    let mut out: String = s.to_string();
    while !out.is_empty() && width_per_px(&format!("{out}..")) * size > w {
        out.pop();
    }
    format!("{out}..")
}

/// Draws text with its top-left at `(x, y)` at about `size` pixels: the size is
/// snapped down to the glyph-cache ladder (see [`super::glyphs`]).
fn text(s: &str, x: f32, y: f32, size: f32, color: Color) {
    let size = super::glyphs::snap(size);
    draw_text(s, x, y + size * 0.8, size, color);
}

/// Width of `s` per pixel of font size (text width scales linearly with size, so one
/// measurement at a reference size gives every size).
fn width_per_px(s: &str) -> f32 {
    measure_text(s, None, 64, 1.0).width / 64.0
}

/// The largest font size, at most `max`, at which `rows` lines (the widest being the
/// longest of `lines`) fit a `w` × `h` box with a little padding.
fn fit_size<S: AsRef<str>>(lines: &[S], rows: usize, w: f32, h: f32, max: f32) -> f32 {
    let widest = lines.iter().map(|l| width_per_px(l.as_ref())).fold(0.0f32, f32::max);
    let by_h = h * 0.92 / (rows.max(1) as f32 * 1.12);
    let by_w = if widest > 0.0 { w * 0.92 / widest } else { max };
    by_h.min(by_w).min(max).max(5.0)
}

/// Draws all instruments.
pub fn draw_instruments(f: &HudFrame) {
    let Some(me) = f.world.get(f.player) else { return };
    let rules = &f.world.rules;
    let skip = |name: &str| f.anchored.contains(name);
    let get = |name: &str| f.instruments.get(name).filter(|_| !skip(name)).map(|r| f.layout.rect(*r));
    if f.backdrops {
        for (name, r) in f.instruments.iter().filter(|(n, _)| !skip(n)) {
            let r = f.layout.rect(*r);
            let _ = name;
            draw_rectangle(r[0], r[1], r[2], r[3], Color::new(0.0, 0.05, 0.0, 0.45));
        }
    }
    for name in INSTRUMENTS {
        if matches!(name, "power" | "systems") {
            continue; // placed below, with a fallback
        }
        if let Some(r) = get(name) {
            draw_instrument(f, me, name, r);
        }
    }
    // Throttle and status line: in the cockpit's `status` rectangle (drawn with the
    // instruments above), else plain screen text at the top left.
    if get("status").is_none() && !skip("status") {
        let status = status_line(rules, me);
        // Sized by the window, not the cockpit's design space: it is plain screen text.
        let size = (screen_height() * 0.022).clamp(13.0, 26.0);
        text(&status, f.layout.offset.x + 8.0, (40.0 * f.layout.scale).max(size * 1.5), size, HUD_GREEN);
    }

    // Power and systems: in the cockpit's own rectangles, or a corner panel.
    // A cockpit without these rectangles gets small panels in the top-right corner of
    // its design space, below where cockpit art usually has its message bar.
    let corner = |top: f32, h: f32| {
        let d = f.layout.design;
        f.layout.rect([d.x - d.x * 0.125, d.y * 0.09 + top, d.x * 0.115, h])
    };
    let d = f.layout.design;
    for (name, fallback) in [("power", corner(0.0, d.y * 0.1)), ("systems", corner(d.y * 0.11, d.y * 0.14))] {
        if skip(name) {
            continue;
        }
        let r = get(name).unwrap_or(fallback);
        if !f.glass && get(name).is_none() {
            draw_rectangle(r[0], r[1], r[2], r[3], Color::new(0.0, 0.05, 0.0, 0.55));
        }
        draw_instrument(f, me, name, r);
    }
}

/// Every instrument the HUD knows how to draw.
pub const INSTRUMENTS: [&str; 11] = [
    "radar_front",
    "radar_rear",
    "gauge1",
    "gauge2",
    "gauge3",
    "gauge4",
    "mfd",
    "message",
    "power",
    "systems",
    "status",
];

/// A strip (much wider than tall) gets the one-line form of an instrument.
fn is_strip(r: [f32; 4]) -> bool {
    r[2] > r[3] * 5.0
}

/// One line of text fitted to a strip: as large as the strip allows, clipped if long.
fn strip_text(f: &HudFrame, r: [f32; 4], s: &str, color: Color) {
    let size = legible(fit_size(&[s], 1, f32::INFINITY, r[3], 22.0 * f.text_scale), f).min(r[3] * 1.1);
    let pad = size * 0.3;
    text(&clip(s, size, r[2] - pad * 2.0), r[0] + pad, r[1] + (r[3] - size) * 0.5, size, color);
}

/// Message text: brighter than instrument green, readable on lit display bands.
const HUD_BRIGHT: Color = Color::new(0.75, 1.0, 0.8, 1.0);

/// The throttle and status line: throttle, speed, and the drive and system flags.
fn status_line(rules: &crate::content::rules::Ruleset, me: &crate::sim::Ship) -> String {
    let class = rules.ship(me.class);
    let v = me.body_velocity();
    let mut status = format!("THR {:>3.0}%  {:>4.0} m/s  {}", me.throttle * 100.0, v.z, drift_text(v));
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
    for flag in status_flags(me) {
        status += "  ";
        status += &flag;
    }
    status
}

/// Sideways (X, + right) and vertical (Y, + up) drift in whole m/s, signed: "X+45 Y-12".
fn drift_text(body_velocity: Vec3) -> String {
    format!("X{:+.0} Y{:+.0}", body_velocity.x, body_velocity.y)
}

/// Draws one instrument into the rectangle `r` (pixels of whatever is being drawn to:
/// the screen, or an instrument texture for a 3D cockpit).
pub fn draw_instrument(f: &HudFrame, me: &crate::sim::Ship, name: &str, r: [f32; 4]) {
    let rules = &f.world.rules;
    match name {
        "radar_front" | "radar_rear" => {
            let front = name == "radar_front";
            let c = Vec2::new(r[0] + r[2] * 0.5, r[1] + r[3] * 0.5);
            let radius = r[2].min(r[3]) * 0.5;
            // Shield ring: the front scope shows the fore shield, the rear scope the aft,
            // as a coloured ring just inside the scope's edge.
            let p = power_readout(&f.world.rules, me);
            let blink = (f.now * 4.0) as i64 % 2 == 0;
            if let Some(ring) = shield_ring_color(if front { p.fore } else { p.aft }, blink) {
                let thick = (radius * 0.07).max(1.5 * f.text_scale);
                draw_circle_lines(c.x, c.y, radius - thick * 0.5, thick, ring);
            }
            // Cockpit art has its own scope rings; the engine adds rings only on the
            // plain glass HUD, so they never sit slightly off the painted ones.
            if f.glass {
                draw_circle_lines(c.x, c.y, radius, f.text_scale, HUD_DIM);
                draw_circle_lines(c.x, c.y, radius * 0.5, f.text_scale, HUD_DIM);
            }
            let blip = (radius * 0.04).max(1.8);
            for id in f.world.live_ids() {
                if id == f.player {
                    continue;
                }
                let s = &f.world.ships[id.index as usize];
                if !crate::sim::systems::on_radar(f.world, me, s) {
                    continue;
                }
                let local = me.rotation.inverse() * (s.position - me.position);
                if let Some(p) = radar_point(local, front) {
                    let at = c + p * radius * 0.92;
                    let color = col(super::contact_color(rules, me.faction, s));
                    let size = if rules.ship(s.class).kind.is_static() { blip * 1.7 } else { blip };
                    draw_rectangle(at.x - size, at.y - size, size * 2.0, size * 2.0, color);
                    // An objective to protect also gets a ring, so it reads without colour.
                    if s.protected_by == Some(me.faction) {
                        draw_circle_lines(at.x, at.y, size * 2.6, f.text_scale.max(1.0), color);
                    }
                    if Some(id) == me.target {
                        let b = size * 2.2;
                        draw_rectangle_lines(at.x - b, at.y - b, b * 2.0, b * 2.0, f.text_scale, WHITE);
                    }
                }
            }
            if f.glass {
                text(if front { "F" } else { "R" }, r[0], r[1], (r[3] * 0.12).min(14.0 * f.text_scale), HUD_DIM);
            }
        }
        "gauge1" | "gauge2" | "gauge3" | "gauge4" => {
            let k = name.as_bytes()[5] as usize - b'1' as usize;
            let v = gauge_values(rules, me)[k];
            let label = ["SPD", "NRG", "SHD", "HUL"][k];
            let c = Vec2::new(r[0] + r[2] * 0.5, r[1] + r[3] * 0.5);
            let radius = r[2].min(r[3]) * 0.45;
            let color = if v < 0.25 && k > 0 { RED } else { HUD_GREEN };
            let a = gauge_angle(v);
            let thick = (radius * 0.05).max(2.0);
            draw_line(c.x, c.y, c.x + a.cos() * radius, c.y + a.sin() * radius, thick, color);
            draw_circle(c.x, c.y, thick * 0.8, color);
            if f.glass {
                draw_circle_lines(c.x, c.y, radius, f.text_scale, HUD_DIM);
            }
            // The label goes on every dial (cockpit art has no printed dial faces).
            let size = legible(fit_size(&[label], 1, radius * 1.3, radius * 0.6, 24.0 * f.text_scale), f);
            let w = width_per_px(label) * size;
            text(label, c.x - w * 0.5, c.y + radius * 0.35, size, HUD_LABEL);
        }
        "mfd" => {
            let lines = mfd_lines(f.world, f.player, f.mfd, f.nav);
            // One size for every page (fitted to the longest page the MFD can show), so the
            // text does not jump as pages change; long lines shrink it further.
            let pad = r[2] * 0.04;
            let (w, h) = (r[2] - pad * 2.0, r[3] - pad * 2.0);
            let size = legible(fit_size(&lines, lines.len().max(6), w, h, 22.0 * f.text_scale), f);
            // At the legible minimum a long line is clipped and extra lines are dropped.
            let rows = ((h / (size * 1.12)) as usize).max(1);
            for (i, l) in lines.iter().take(rows).enumerate() {
                text(&clip(l, size, w), r[0] + pad, r[1] + pad + i as f32 * size * 1.12, size, HUD_GREEN);
            }
        }
        "message" => {
            if let Some(m) = f.messages.visible(f.now, 1).last() {
                let size = legible(fit_size(&[m], 1, r[2], r[3], 20.0 * f.text_scale), f).min(r[3] * 1.1);
                let m = &clip(m, size, r[2]);
                let w = measure_text(m, None, size as u16, 1.0).width;
                text(m, r[0] + (r[2] - w) * 0.5, r[1] + (r[3] - size) * 0.5, size, HUD_BRIGHT);
            }
        }
        "power" => draw_power(f, me, r),
        "systems" => draw_systems(f, me, r),
        "status" => strip_text(f, r, &status_line(&f.world.rules, me), HUD_GREEN),
        _ => {}
    }
}

/// The `power` instrument: two level columns (beams, shields) with the budget, and the
/// fore/aft shield bars with the balance marker.
fn draw_power(f: &HudFrame, me: &crate::sim::Ship, r: [f32; 4]) {
    let p = power_readout(&f.world.rules, me);
    if is_strip(r) {
        let line = format!(
            "B-{} S-{}  F{:.0} A{:.0}",
            PowerRules::letter(p.beam),
            PowerRules::letter(p.shield),
            p.fore * 100.0,
            p.aft * 100.0
        );
        let low = p.fore < 0.25 || p.aft < 0.25;
        strip_text(f, r, &line, if low { RED } else { HUD_GREEN });
        return;
    }
    let col_w = r[2] / 4.0;
    let size = legible(fit_size(&["B-E"], 1, col_w * 0.95, r[3] * 0.2, 16.0 * f.text_scale), f);
    let label_h = size * 1.1;
    let bar_top = r[1] + label_h;
    let bar_h = r[3] - label_h - 2.0;
    let levels = p.levels.max(1) as f32;
    // Beam and shield charge columns: one cell per level, lit up to the setting.
    for (k, (level, label)) in [(p.beam, "B"), (p.shield, "S")].into_iter().enumerate() {
        let x = r[0] + col_w * k as f32 + col_w * 0.15;
        let w = col_w * 0.7;
        let cell = bar_h / levels;
        for l in 1..p.levels {
            let y = bar_top + bar_h - cell * l as f32;
            let lit = l <= level;
            let c = if lit { HUD_GREEN } else { HUD_DIM };
            draw_rectangle(x, y + 1.0, w, cell - 2.0, if lit { c } else { Color::new(c.r, c.g, c.b, 0.15) });
        }
        let t = format!("{}-{}", label, PowerRules::letter(level));
        text(&t, x, r[1], size, HUD_GREEN);
    }
    // Shield facings, with the favoured one marked.
    for (k, (v, label, favoured)) in
        [(p.fore, "F", p.balance == ShieldBalance::Forward), (p.aft, "A", p.balance == ShieldBalance::Aft)]
            .into_iter()
            .enumerate()
    {
        let x = r[0] + col_w * (2 + k) as f32 + col_w * 0.25;
        let w = col_w * 0.5;
        draw_rectangle_lines(x, bar_top, w, bar_h, f.text_scale, HUD_DIM);
        let c = if v < 0.25 { RED } else { SKYBLUE };
        draw_rectangle(x + 1.0, bar_top + bar_h * (1.0 - v), w - 2.0, bar_h * v, c);
        text(label, x, r[1], size, if favoured { WHITE } else { HUD_GREEN });
    }
    // Over budget (it never should be) or fully used: show the count.
    if p.used >= p.budget {
        text(&format!("{}/{}", p.used, p.budget), r[0], r[1] + r[3] - size, size * 0.8, YELLOW);
    }
}

/// The `systems` instrument: a grid of labelled lamps.
fn draw_systems(f: &HudFrame, me: &crate::sim::Ship, r: [f32; 4]) {
    let lamps = system_lamps(me);
    if lamps.is_empty() {
        return;
    }
    if is_strip(r) {
        // One line: only the systems that are not plain OK, in their lamp colour.
        let odd: Vec<_> = lamps.iter().filter(|(_, l, _)| *l != Lamp::Ok).collect();
        if odd.is_empty() {
            strip_text(f, r, "SYSTEMS OK", HUD_GREEN);
            return;
        }
        let size = legible(fit_size(&["ENG"], 1, f32::INFINITY, r[3], 22.0 * f.text_scale), f).min(r[3] * 1.1);
        let mut x = r[0] + size * 0.3;
        for (label, lamp, _) in odd {
            let w = width_per_px(label) * size;
            if x + w > r[0] + r[2] {
                break;
            }
            text(label, x, r[1] + (r[3] - size) * 0.5, size, lamp_color(*lamp, false, false));
            x += w + size * 0.6;
        }
        return;
    }
    let (cols, rows) = lamp_grid(lamps.len(), r[2], r[3]);
    let (cw, ch) = (r[2] / cols as f32, r[3] / rows as f32);
    let d = (ch * 0.45).min(cw * 0.25);
    let size = legible(fit_size(&["ENG"], 1, cw - d * 1.9, ch * 0.8, 16.0 * f.text_scale), f);
    let blink = (f.now * 3.0) as i64 % 2 == 0;
    for (i, (label, lamp, repairing)) in lamps.iter().enumerate() {
        let (cx, cy) = (r[0] + cw * (i % cols) as f32, r[1] + ch * (i / cols) as f32);
        let color = lamp_color(*lamp, *repairing, blink);
        draw_circle(cx + d * 0.8, cy + ch * 0.5, d * 0.5, color);
        text(
            label,
            cx + d * 1.6,
            cy + (ch - size) * 0.5,
            size,
            if *lamp == Lamp::Destroyed { GRAY } else { HUD_GREEN },
        );
    }
}

/// The shield ring's colour for a facing at `frac` of its capacity: green when strong,
/// amber when weakened, red when low, and blinking red (`None` on the off beat) when the
/// shield is down.
fn shield_ring_color(frac: f32, blink: bool) -> Option<Color> {
    let a = 0.85;
    match frac {
        f if f > 0.66 => Some(Color::new(0.2, 0.9, 0.35, a)),
        f if f > 0.33 => Some(Color::new(1.0, 0.75, 0.1, a)),
        f if f > 0.02 => Some(Color::new(1.0, 0.2, 0.1, a)),
        _ => blink.then_some(Color::new(1.0, 0.1, 0.05, a)),
    }
}

/// A system lamp's colour; a system out of action and being repaired blinks dark.
fn lamp_color(lamp: Lamp, repairing: bool, blink: bool) -> Color {
    match lamp {
        Lamp::Ok => Color::new(0.2, 0.85, 0.3, 0.9),
        Lamp::On => Color::new(0.3, 0.95, 1.0, 1.0),
        Lamp::Damaged => Color::new(1.0, 0.7, 0.1, 1.0),
        Lamp::Out if repairing && blink => Color::new(0.45, 0.05, 0.05, 1.0),
        Lamp::Out => Color::new(1.0, 0.15, 0.1, 1.0),
        Lamp::Destroyed => Color::new(0.5, 0.5, 0.5, 1.0),
    }
}

/// Columns and rows for `n` lamps in a `w` × `h` panel: the arrangement that gives the
/// biggest label. A cell holds a lamp and a three-letter label, about four characters
/// wide (a ProggyClean character is ~0.55 of the font size wide), and one line high.
fn lamp_grid(n: usize, w: f32, h: f32) -> (usize, usize) {
    let label_size = |cols: usize| {
        let rows = n.div_ceil(cols);
        let (cw, ch) = (w / cols as f32, h / rows as f32);
        (cw / (4.0 * 0.55)).min(ch * 0.75)
    };
    let cols = (1..=n.max(1)).max_by(|&a, &b| label_size(a).total_cmp(&label_size(b))).unwrap_or(1);
    (cols, n.div_ceil(cols).max(1))
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
    // Flight-path marker: where the ship is actually going. With no drift it sits on
    // the crosshair; sideways drift slides it off to the side.
    if me.velocity.length() > 5.0 {
        let ahead = f.camera.position + me.velocity.normalize() * 1000.0;
        if let Some(p) = f.camera.project(ahead, screen) {
            let r = s * 0.45;
            draw_circle_lines(p.x, p.y, r, 1.5, HUD_GREEN);
            draw_line(p.x - r * 2.2, p.y, p.x - r, p.y, 1.5, HUD_GREEN);
            draw_line(p.x + r, p.y, p.x + r * 2.2, p.y, 1.5, HUD_GREEN);
            draw_line(p.x, p.y - r, p.x, p.y - r * 2.0, 1.5, HUD_GREEN);
        }
    }
    let Some(target) = me.target.and_then(|t| f.world.get(t)) else { return };
    let rules = &f.world.rules;
    let color = col(super::contact_color(rules, me.faction, target));
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
            let protect = if target.protected_by == Some(me.faction) { "PROTECT " } else { "" };
            let label = format!("{protect}{} {}", rules.ship(target.class).name, super::format_distance(dist));
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shield_rings_go_green_amber_red_then_blink() {
        let g = |f, b| shield_ring_color(f, b).map(|c| (c.r > 0.5, c.g > 0.5));
        assert_eq!(g(1.0, true), Some((false, true)), "green");
        assert_eq!(g(0.5, true), Some((true, true)), "amber");
        assert_eq!(g(0.1, true), Some((true, false)), "red");
        assert_eq!(g(0.0, true), Some((true, false)));
        assert_eq!(g(0.0, false), None, "down: blinks");
    }

    #[test]
    fn drift_reads_signed_whole_metres_per_second() {
        assert_eq!(drift_text(Vec3::new(45.4, -12.0, 300.0)), "X+45 Y-12");
        assert_eq!(drift_text(Vec3::ZERO), "X+0 Y+0");
    }

    #[test]
    fn lamp_grids_follow_the_panel_shape() {
        assert_eq!(lamp_grid(10, 100.0, 300.0), (2, 5), "tall panel: two columns");
        let (cols, rows) = lamp_grid(10, 600.0, 60.0);
        assert!(cols >= 5 && rows <= 2, "wide, short panel: few rows ({cols}x{rows})");
        assert_eq!(lamp_grid(1, 50.0, 50.0), (1, 1));
    }
}
