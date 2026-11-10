//! # Model viewer
//!
//! Browse every model: each ship class's model (so modders see their pack exactly as the
//! game will), and — with original data — every raw mesh on the CD, including debris
//! chunks and unused objects. An orbit camera circles the model; attachment points are
//! shown as small axes; LODs can be forced to compare detail levels.

use super::{AppContext, Next};
use crate::render::camera::SpaceCamera;
use glam::{Quat, Vec3};
use macroquad::prelude::*;

/// An orbiting camera around the origin (pure maths, tested).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Orbit {
    pub yaw: f32,
    pub pitch: f32,
    pub distance: f32,
}

impl Orbit {
    /// Camera position and rotation looking at the origin.
    pub fn pose(&self) -> (Vec3, Quat) {
        let rot = Quat::from_rotation_y(self.yaw) * Quat::from_rotation_x(-self.pitch);
        (rot * Vec3::new(0.0, 0.0, self.distance), rot)
    }

    /// Applies input (radians per second / zoom factor per second) for `dt` seconds.
    pub fn update(&mut self, yaw: f32, pitch: f32, zoom: f32, dt: f32) {
        self.yaw += yaw * dt;
        self.pitch = (self.pitch + pitch * dt).clamp(-1.5, 1.5);
        self.distance *= (1.0 - zoom * dt).clamp(0.5, 1.5);
    }
}

/// Direction of a mouse-wheel movement: −1, 0 or +1.
///
/// Beware: Rust's `f32::signum(0.0)` is `1.0`, not `0.0` (IEEE-754 zero has a sign), so
/// `wheel.signum()` would report "scrolling up" every frame the wheel is still.
pub fn wheel_steps(wheel: f32) -> f32 {
    if wheel == 0.0 {
        0.0
    } else {
        wheel.signum()
    }
}

/// One model in the list.
#[derive(Debug, Clone, PartialEq)]
pub struct ViewerEntry {
    pub label: String,
    pub reference: String,
    pub pack: usize,
    pub scale: f32,
}

/// The viewer screen.
pub struct ModelViewer {
    pub entries: Vec<ViewerEntry>,
    pub selected: usize,
    orbit: Orbit,
    /// `None` = automatic LOD by distance; `Some(i)` forces LOD `i`.
    forced_lod: Option<usize>,
    /// The entry the camera distance was last fitted to (fit again when it changes).
    fitted: Option<usize>,
    spin: bool,
    angle: f32,
}

/// Builds the list: ship classes first, then raw original meshes.
pub fn entries(content: &crate::content::Content) -> Vec<ViewerEntry> {
    let mut v: Vec<ViewerEntry> = content
        .rules
        .ships
        .iter()
        .map(|s| ViewerEntry {
            label: format!("{} ({})", s.name, s.id),
            reference: s.model.clone(),
            pack: s.model_pack,
            scale: s.model_scale,
        })
        .collect();
    v.extend(content.original_mesh_names().into_iter().map(|n| ViewerEntry {
        label: format!("mesh {n}"),
        reference: format!("original:{n}"),
        pack: 0,
        scale: 1.0,
    }));
    v
}

impl ModelViewer {
    pub fn new(ctx: &AppContext) -> ModelViewer {
        ModelViewer {
            entries: entries(&ctx.content),
            selected: 0,
            orbit: Orbit { yaw: 0.6, pitch: 0.35, distance: 30.0 },
            forced_lod: None,
            fitted: None,
            spin: true,
            angle: 0.0,
        }
    }

    pub fn frame(&mut self, ctx: &mut AppContext) -> Next {
        let raw = ctx.devices.read();
        let pressed = |k: &str| raw.pressed.contains(k);
        let held = |k: &str| raw.held.contains(k);
        let n = self.entries.len().max(1);
        let mut changed = false;
        if pressed("pagedown") || pressed("n") {
            self.selected = (self.selected + 1) % n;
            changed = true;
        }
        if pressed("pageup") || pressed("b") {
            self.selected = (self.selected + n - 1) % n;
            changed = true;
        }
        if pressed("escape") {
            return Next::Launcher;
        }
        if pressed("l") {
            self.forced_lod = match self.forced_lod {
                None => Some(0),
                Some(i) if i < 3 => Some(i + 1),
                Some(_) => None,
            };
        }
        if pressed("space") {
            self.spin = !self.spin;
        }
        let dt = get_frame_time();
        let axis = |a: &str, b: &str| (held(b) as i32 - held(a) as i32) as f32;
        let zoom = axis("minus", "equal") + wheel_steps(mouse_wheel().1) * 4.0;
        self.orbit.update(axis("left", "right") * 1.5, axis("down", "up") * 1.0, zoom * 1.5, dt);
        if is_mouse_button_down(MouseButton::Left) {
            let d = mouse_delta_position();
            self.orbit.yaw += d.x * 3.0;
            self.orbit.pitch = (self.orbit.pitch - d.y * 3.0).clamp(-1.5, 1.5);
        }
        if self.spin {
            self.angle += dt * 0.3;
        }

        clear_background(Color::new(0.02, 0.02, 0.05, 1.0));
        let Some(entry) = self.entries.get(self.selected).cloned() else {
            draw_text("No models.", 40.0, 60.0, 30.0, WHITE);
            return Next::Stay;
        };
        let model = ctx.renderer.models.get(&ctx.content, &entry.reference, entry.pack, entry.scale);
        if let Some(m) = &model {
            if changed || self.fitted != Some(self.selected) {
                self.orbit.distance = (m.radius * 2.8).max(5.0);
                self.fitted = Some(self.selected);
            }
            // Never let the camera enter the model.
            self.orbit.distance = self.orbit.distance.max(m.radius * 1.05);
            let (pos, rot) = self.orbit.pose();
            let cam =
                SpaceCamera { position: pos, rotation: rot, fov_y: 0.8, aspect: screen_width() / screen_height() };
            let sun = Vec3::new(-0.4, 0.7, 0.6);
            ctx.renderer.draw_sky(&cam, 9, sun);
            ctx.renderer.begin_solid(&cam, sun);
            let model_rot = Quat::from_rotation_y(self.angle);
            ctx.renderer.draw_model_lod(&cam, m, Vec3::ZERO, model_rot, Vec3::ZERO, self.forced_lod);
            gl_use_default_material();
            for a in &m.model.attachments {
                ctx.renderer.draw_axes(&cam, model_rot * a.position, model_rot, m.radius * 0.15);
            }
            set_default_camera();
            let lod_info: Vec<String> = m
                .model
                .lods
                .iter()
                .map(|l| format!("{} tris < {:.0} m", l.triangle_count(), l.switch_distance.min(1e7)))
                .collect();
            let lines = [
                entry.label.clone(),
                format!("radius {:.1} m   attachments {}", m.radius, m.model.attachments.len()),
                format!("LODs: {}", lod_info.join("  |  ")),
                format!(
                    "showing: {}   distance {:.0} m",
                    self.forced_lod.map(|i| format!("LOD {i}")).unwrap_or("auto LOD".into()),
                    self.orbit.distance
                ),
            ];
            for (i, l) in lines.iter().enumerate() {
                draw_text(l, 20.0, 34.0 + i as f32 * 24.0, if i == 0 { 28.0 } else { 20.0 }, WHITE);
            }
        } else {
            set_default_camera();
            draw_text(&format!("{}: could not load", entry.label), 20.0, 40.0, 26.0, ORANGE);
            if let Some(e) = ctx.renderer.models.errors.last() {
                draw_text(e, 20.0, 70.0, 18.0, ORANGE);
            }
        }
        draw_text(
            &format!(
                "{}/{}   PgUp/PgDn (B/N) model   arrows/drag orbit   +/- wheel zoom   L LOD   Space spin   Esc back",
                self.selected + 1,
                n
            ),
            20.0,
            screen_height() - 20.0,
            18.0,
            GRAY,
        );
        Next::Stay
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn orbit_looks_at_the_origin_from_its_distance() {
        let o = Orbit { yaw: 0.0, pitch: 0.0, distance: 10.0 };
        let (p, r) = o.pose();
        assert!((p - Vec3::new(0.0, 0.0, 10.0)).length() < 1e-5);
        assert!(((r * Vec3::NEG_Z) - Vec3::NEG_Z).length() < 1e-5, "looking towards the origin");
        let o = Orbit { yaw: 0.0, pitch: 0.5, distance: 10.0 };
        let (p, r) = o.pose();
        assert!(p.y > 0.0, "positive pitch is above");
        assert!(((r * Vec3::NEG_Z) - (-p.normalize())).length() < 1e-5, "still aimed at the origin");
    }

    #[test]
    fn a_still_wheel_does_not_zoom() {
        assert_eq!(wheel_steps(0.0), 0.0);
        assert_eq!(wheel_steps(3.5), 1.0);
        assert_eq!(wheel_steps(-0.1), -1.0);
    }

    #[test]
    fn orbit_input_clamps() {
        let mut o = Orbit { yaw: 0.0, pitch: 0.0, distance: 10.0 };
        o.update(1.0, 100.0, 1.0, 1.0);
        assert_eq!(o.pitch, 1.5);
        assert_eq!(o.distance, 5.0, "zoom limited per step");
        o.update(0.0, 0.0, -10.0, 1.0);
        assert_eq!(o.distance, 7.5);
        assert_eq!(o.yaw, 1.0);
    }

    #[test]
    fn sample_entries_list_every_class() {
        let packs = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("packs");
        let c = crate::content::Content::load(&crate::content::ContentConfig {
            pack_dirs: vec![packs],
            no_original: true,
            ..Default::default()
        })
        .unwrap();
        let e = entries(&c);
        assert_eq!(e.len(), c.rules.ships.len());
        assert!(e.iter().any(|x| x.label.starts_with("Dart")));
    }
}
