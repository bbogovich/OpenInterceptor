//! # Playing a cutscene (MD2)
//!
//! A [`CutscenePlayer`] runs a [`Cutscene`] on an ordinary battle [`World`]: each entity
//! is a ship spawned when it enters, placed every tick on its keyframed path (so the
//! simulation never steers it), and its timed actions call the battle's own code —
//! firing beams, launching missiles, being destroyed — so bolts, explosions, debris and
//! sounds are exactly a battle's. The camera follows the cuts; captions are drawn at the
//! bottom. Esc (or Enter) skips. The world is stepped at the fixed 60 Hz tick with a
//! fixed seed, so a cutscene plays the same every time.

use super::AppContext;
use crate::content::cutscene::{catmull_rom, path_rotation, scalar, Action, Camera, Cutscene, Target};
use crate::render::camera::SpaceCamera;
use crate::render::effects::Effects;
use crate::sim::{collision, Pilot, ShipId, World, TICK};
use glam::{Quat, Vec3};
use macroquad::prelude::*;
use std::sync::Arc;

/// A cutscene being played.
pub struct CutscenePlayer {
    scene: Cutscene,
    world: World,
    effects: Effects,
    /// Per entity: its ship once it has entered.
    ships: Vec<Option<ShipId>>,
    /// Per entity: the next action to run.
    next_action: Vec<usize>,
    /// Per entity: firing beams until this time.
    firing_until: Vec<f32>,
    /// Sounds started so far.
    sounds_done: usize,
    time: f32,
    accumulator: f32,
}

impl CutscenePlayer {
    /// Prepares a cutscene: the world with its environment, nothing spawned yet.
    pub fn new(ctx: &mut AppContext, id: &str) -> Result<CutscenePlayer, String> {
        let scene = ctx.content.cutscenes.iter().find(|c| c.id == id).cloned().ok_or(format!("no cutscene '{id}'"))?;
        let content = &ctx.content;
        let rules = Arc::new(content.rules.clone());
        let mut shapes = Vec::with_capacity(rules.ships.len());
        for class in &rules.ships {
            shapes.push(match ctx.renderer.models.get(content, &class.model, class.model_pack, class.model_scale) {
                Some(m) => collision::Shape::from_model(&m.model, class.kind.is_static()),
                None => collision::Shape::sphere(10.0),
            });
        }
        let mut world = World::new(rules, Arc::new(shapes), scene.starfield_seed);
        if let Some(env) = &scene.environment {
            world.set_environment(env)?;
        }
        let n = scene.entities.len();
        Ok(CutscenePlayer {
            scene,
            world,
            effects: Effects::default(),
            ships: vec![None; n],
            next_action: vec![0; n],
            firing_until: vec![0.0; n],
            sounds_done: 0,
            time: 0.0,
            accumulator: 0.0,
        })
    }

    /// One tick of the scene at its current time.
    fn tick(&mut self, ctx: &AppContext) {
        let t = self.time;
        for (k, e) in self.scene.entities.iter().enumerate() {
            let inside = t >= e.enter && e.leave.is_none_or(|l| t < l);
            match (self.ships[k], inside) {
                (None, true) => {
                    let Some(class) = self.world.rules.ship_id(&e.class) else { continue };
                    let faction = e
                        .faction
                        .as_ref()
                        .and_then(|f| self.world.rules.faction_id(f))
                        .unwrap_or(self.world.rules.ship(class).faction);
                    let pos = catmull_rom(&e.path, t);
                    let id = self.world.spawn(class, faction, &e.id, pos, path_rotation(&e.path, t), Pilot::External);
                    self.ships[k] = Some(id);
                    if e.harmless {
                        self.world.harmless.push(id);
                    }
                }
                (Some(id), false) => {
                    if let Some(s) = self.world.get_mut(id) {
                        s.alive = false; // left the scene quietly
                    }
                }
                _ => {}
            }
            // Keyframed: placed on its path, moving at the path's speed (for effects
            // and bolts fired from it).
            if let Some(s) = self.ships[k].and_then(|id| self.world.get_mut(id)) {
                let (p, ahead) = (catmull_rom(&e.path, t), catmull_rom(&e.path, t + TICK));
                s.position = p;
                s.velocity = (ahead - p) / TICK;
                s.rotation = path_rotation(&e.path, t);
                s.angular_velocity = Vec3::ZERO;
                s.controls = Default::default();
                s.controls.fire_beams = t < self.firing_until[k];
            }
            // Timed actions.
            while let Some((at, action)) = e.actions.get(self.next_action[k]) {
                if *at > t {
                    break;
                }
                self.next_action[k] += 1;
                let Some(id) = self.ships[k] else { continue };
                match action {
                    Action::FireBeams(secs) => self.firing_until[k] = t + secs,
                    Action::Missile(target) => {
                        let tgt = self.scene.entities.iter().position(|x| &x.id == target).and_then(|j| self.ships[j]);
                        if let Some(s) = self.world.get_mut(id) {
                            s.target = tgt;
                            s.controls.fire_missile = true;
                        }
                    }
                    Action::Destroy => {
                        if let Some(s) = self.world.get_mut(id) {
                            (s.shield_fore, s.shield_aft, s.armor) = (0.0, 0.0, 0.0);
                            let at = s.position;
                            crate::sim::combat::apply_damage(&mut self.world, id, f32::MAX / 4.0, at);
                        }
                    }
                }
            }
        }
        // Sounds as their time comes.
        while let Some((at, sound, vol)) = self.scene.sounds.get(self.sounds_done) {
            if *at > t {
                break;
            }
            self.sounds_done += 1;
            // A named event sound of the campaign (`sounds:`), as battles use.
            ctx.sounds.play_named(sound, *vol);
        }
        self.world.step();
        self.effects.on_events(&self.world);
        let ear = self.camera(ctx).position;
        let listener = self.ships.iter().flatten().next().copied().unwrap_or(ShipId { index: u32::MAX, generation: 0 });
        ctx.sounds.play_events(&self.world, listener, ear);
        self.time += TICK;
    }

    /// The camera at the current time.
    fn camera(&self, _ctx: &AppContext) -> SpaceCamera {
        let t = self.time;
        let aspect = screen_width() / screen_height().max(1.0);
        let entity_pos = |id: &str| {
            self.scene.entities.iter().position(|e| e.id == id).map(|k| {
                let e = &self.scene.entities[k];
                (catmull_rom(&e.path, t), path_rotation(&e.path, t))
            })
        };
        match self.scene.camera_at(t).map(|c| &self.scene.cameras[c].1) {
            Some(Camera::Fixed { position, look_at, fov }) => {
                let p = catmull_rom(position, t);
                let target = match look_at {
                    Target::Entity(e) => entity_pos(e).map(|(q, _)| q).unwrap_or(Vec3::ZERO),
                    Target::Point(q) => *q,
                };
                let rot = Quat::from_rotation_arc(Vec3::NEG_Z, (target - p).normalize_or(Vec3::NEG_Z));
                // Keep the horizon level: remove the roll the arc rotation introduced.
                let fwd = rot * Vec3::NEG_Z;
                let level = Quat::from_mat3(&glam::Mat3::from_cols(
                    fwd.cross(Vec3::Y).normalize_or(Vec3::X),
                    fwd.cross(Vec3::Y).normalize_or(Vec3::X).cross(fwd).normalize_or(Vec3::Y),
                    -fwd,
                ));
                SpaceCamera::new(p, level, fov.to_radians(), aspect)
            }
            Some(Camera::Attach { entity, distance, orbit, fov }) => {
                let (p, r) = entity_pos(entity).unwrap_or((Vec3::ZERO, Quat::IDENTITY));
                let d = scalar(distance, t).max(1.0);
                let yaw = scalar(&orbit.iter().map(|(k, v)| (*k, v[0])).collect::<Vec<_>>(), t).to_radians();
                let pitch = scalar(&orbit.iter().map(|(k, v)| (*k, v[1])).collect::<Vec<_>>(), t).to_radians();
                let rot = r * Quat::from_rotation_y(yaw) * Quat::from_rotation_x(-pitch);
                SpaceCamera::new(p + rot * Vec3::new(0.0, 0.0, d), rot, fov.to_radians(), aspect)
            }
            None => SpaceCamera::new(Vec3::new(0.0, 50.0, 300.0), Quat::IDENTITY, 1.0, aspect),
        }
    }

    /// One frame: advance by the frame's time, draw. `true` when over (or skipped).
    pub fn frame(&mut self, ctx: &mut AppContext) -> bool {
        if super::skip_pressed(ctx) {
            return true;
        }
        self.accumulator += get_frame_time().min(0.1);
        while self.accumulator >= TICK {
            self.accumulator -= TICK;
            self.tick(ctx);
            if self.time >= self.scene.length {
                return true;
            }
        }
        self.effects.update(get_frame_time().min(0.1));
        let cam = self.camera(ctx);
        let env = self.world.env.id.as_ref().and_then(|id| self.world.rules.environments.iter().find(|e| &e.id == id));
        ctx.renderer.env_sky = env.and_then(|e| e.sky);
        ctx.renderer.planet = env.and_then(|e| e.planet);
        ctx.renderer.nebula = env.and_then(|e| e.nebula);
        ctx.renderer.lighting = self.world.rules.flight.lighting;
        clear_background(match ctx.renderer.env_sky {
            Some(s) => Color::new(s.horizon[0], s.horizon[1], s.horizon[2], 1.0),
            None => BLACK,
        });
        ctx.renderer.draw_sky(&cam, self.scene.starfield_seed, self.scene.sun);
        ctx.renderer.draw_world(&ctx.content, &self.world, &self.effects, &cam, self.scene.sun, None);
        ctx.renderer.draw_glows(&ctx.content, &self.world, &self.effects, &cam, None, None);
        gl_use_default_material();
        set_default_camera();
        // Letterbox bars and captions.
        let (w, h) = (screen_width(), screen_height());
        let bar = h * 0.1;
        draw_rectangle(0.0, 0.0, w, bar, BLACK);
        draw_rectangle(0.0, h - bar, w, bar, BLACK);
        for (from, to, text) in &self.scene.captions {
            if self.time >= *from && self.time < *to {
                let size = (h * 0.035).max(18.0);
                let tw = ctx.ui.measure(text, size);
                ctx.ui.text_in(text, (w - tw) * 0.5, h - bar * 0.45, size, WHITE);
            }
        }
        false
    }

    /// Clears what the scene set on the renderer (the next screen draws its own sky).
    pub fn finish(&self, ctx: &mut AppContext) {
        ctx.renderer.env_sky = None;
        ctx.renderer.planet = None;
        ctx.renderer.nebula = None;
    }
}
