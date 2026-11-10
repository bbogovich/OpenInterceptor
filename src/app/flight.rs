//! # A flight session: playing one scenario
//!
//! Each frame:
//!
//! 1. read devices → [`Controls`](crate::sim::Controls) for the player + [`Command`]s;
//! 2. apply commands (targeting, views, wingman orders, pause…);
//! 3. run as many fixed 60 Hz simulation ticks as real time requires (the
//!    *accumulator* pattern), feeding each tick's events to effects, sound and messages;
//! 4. place the camera for the current view and draw: sky, world, glows, cockpit, HUD.
//!
//! ## The accumulator
//!
//! Frames arrive at irregular intervals (16.7 ms at 60 Hz, 8.3 ms at 120 Hz, 50 ms on a
//! hiccup). The simulation must always step by exactly [`TICK`]. So real elapsed time
//! is added to an accumulator, and whole ticks are taken out of it; the remainder
//! carries over. Capping the frame time stops a long stall (a breakpoint, a window
//! drag) from triggering hundreds of catch-up ticks.

use super::{AppContext, Next};
use crate::content::scenario::NavPoint;
use crate::hud::{self, draw::HudFrame, Layout, MessageLog, MfdPage};
use crate::input::{ChaseInput, Command, View};
use crate::render::camera::SpaceCamera;
use crate::render::effects::Effects;
use crate::render::gpu::GpuModel;
use crate::sim::{ai, collision, targeting, ShipId, SimEvent, World, TICK};
use glam::{Quat, Vec2, Vec3};
use macroquad::prelude::*;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

/// Longest frame time fed to the simulation, seconds.
pub const MAX_FRAME: f32 = 0.25;

/// Manual chase-camera orbit around the player.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChaseOrbit {
    pub yaw: f32,
    pub pitch: f32,
    /// Distance in multiples of the ship's radius.
    pub distance: f32,
}

impl Default for ChaseOrbit {
    fn default() -> Self {
        ChaseOrbit { yaw: 0.0, pitch: 0.25, distance: 3.2 }
    }
}

impl ChaseOrbit {
    pub fn apply(&mut self, input: &ChaseInput, dt: f32) {
        if input.reset {
            *self = ChaseOrbit::default();
        }
        self.yaw += input.yaw * dt * 1.5;
        self.pitch = (self.pitch + input.pitch * dt * 1.0).clamp(-1.4, 1.4);
        self.distance = (self.distance * (1.0 - input.zoom * dt)).clamp(1.2, 20.0);
    }
}

/// Camera position and rotation for a view of a ship at `pos`/`rot` with bounding
/// `radius`. Pure, so every view is unit-tested.
pub fn view_pose(view: View, pos: Vec3, rot: Quat, radius: f32, chase: &ChaseOrbit) -> (Vec3, Quat) {
    // The pilot's eye: slightly up and forward of the centre.
    let eye = pos + rot * Vec3::new(0.0, radius * 0.08, -radius * 0.15);
    let look = |deg: f32| rot * Quat::from_rotation_y(deg.to_radians());
    match view {
        View::Cockpit | View::Forward => (eye, rot),
        View::Left => (eye, look(90.0)),
        View::Right => (eye, look(-90.0)),
        View::RearLeft => (eye, look(135.0)),
        View::RearRight => (eye, look(-135.0)),
        View::Chase => (pos + rot * Vec3::new(0.0, radius * 0.7, radius * 3.2), rot * Quat::from_rotation_x(-0.12)),
        View::ChaseManual => {
            let orbit = rot * Quat::from_rotation_y(chase.yaw) * Quat::from_rotation_x(-chase.pitch);
            (pos + orbit * Vec3::new(0.0, 0.0, radius * chase.distance), orbit)
        }
    }
}

/// Whether a view shows the player's own ship model.
pub fn shows_own_ship(view: View) -> bool {
    matches!(view, View::Chase | View::ChaseManual)
}

/// Applies a targeting or wingman command to the world. Returns a message for the
/// message line, if the command produces one.
pub fn apply_command(world: &mut World, player: ShipId, cmd: Command) -> Option<String> {
    let current = world.get(player)?.target;
    let new_target = match cmd {
        Command::NextTarget => targeting::cycle(world, player, current, true),
        Command::PrevTarget => targeting::cycle(world, player, current, false),
        Command::NearestEnemy => targeting::nearest_hostile(world, player),
        Command::TargetInSights => targeting::in_sights(world, player, 15.0),
        Command::WingFireAtWill => {
            return Some(order(world, player, ai::WingOrder::FireAtWill, "WINGMEN: FIRE AT WILL"))
        }
        Command::WingAttackMyTarget => {
            return Some(order(world, player, ai::WingOrder::AttackMyTarget, "WINGMEN: ATTACK MY TARGET"))
        }
        Command::WingFormation => return Some(order(world, player, ai::WingOrder::Formation, "WINGMEN: FORM ON ME")),
        _ => return None,
    };
    let s = world.get_mut(player)?;
    if new_target != s.target {
        s.lock = Default::default();
    }
    s.target = new_target;
    match new_target {
        Some(t) => {
            let name = world.rules.ship(world.ships[t.index as usize].class).name.clone();
            Some(format!("TARGET: {}", name.to_uppercase()))
        }
        None => Some("NO TARGET".into()),
    }
}

fn order(world: &mut World, player: ShipId, o: ai::WingOrder, text: &str) -> String {
    match ai::order_wingmen(world, player, o) {
        0 => "NO WINGMEN".into(),
        _ => text.into(),
    }
}

/// Message-line texts for this tick's events, from the player's point of view.
pub fn event_messages(world: &World, player: ShipId) -> Vec<String> {
    let target = world.ships.get(player.index as usize).and_then(|s| s.target);
    let mut out = Vec::new();
    for e in &world.events {
        match e {
            SimEvent::Message { ship, text } if *ship == player => out.push(text.clone()),
            SimEvent::LockAcquired { ship } if *ship == player => out.push("MISSILE LOCK".into()),
            SimEvent::NoEnergy { ship } if *ship == player => out.push("INSUFFICIENT POWER".into()),
            SimEvent::Destroyed { ship, .. } if *ship == player => out.push("YOUR SHIP HAS BEEN DESTROYED".into()),
            SimEvent::Destroyed { ship, .. } => {
                let name = world.rules.ship(world.ships[ship.index as usize].class).name.to_uppercase();
                // The target was cleared on death, so compare with last frame's target.
                if Some(*ship) == target
                    || world
                        .rules
                        .hostile(world.ships[player.index as usize].faction, world.ships[ship.index as usize].faction)
                {
                    out.push(format!("{name} DESTROYED"));
                }
            }
            _ => {}
        }
    }
    out
}

/// A running scenario.
pub struct FlightSession {
    pub world: World,
    pub player: ShipId,
    scenario_title: String,
    sun: Vec3,
    seed: u64,
    effects: Effects,
    messages: MessageLog,
    /// The current camera view (public so `--view` can set the starting view).
    pub view: View,
    mfd: MfdPage,
    chase: ChaseOrbit,
    nav: Vec<NavPoint>,
    nav_index: usize,
    paused: bool,
    menu_open: bool,
    mouse_flight: bool,
    accumulator: f32,
    /// 2D cockpit art, if the player's cockpit has an image.
    cockpit: Option<Texture2D>,
    /// 3D cockpit model and its offset from the pilot's eye, if any.
    cockpit_model: Option<(Rc<GpuModel>, Vec3)>,
    /// The cockpit's design space (instrument coordinates).
    design: Vec2,
    instruments: HashMap<String, [f32; 4]>,
    scenario_id: String,
    ship_choice: Option<String>,
}

impl FlightSession {
    /// Builds the world for scenario `id`, with `ship` as the player's class if the
    /// scenario lets the player choose.
    pub fn start(ctx: &mut AppContext, id: &str, ship: Option<&str>) -> Result<FlightSession, String> {
        let content = &ctx.content;
        let scenario = content.scenario(id).ok_or(format!("no scenario '{id}'"))?.clone();
        let rules = Arc::new(content.rules.clone());
        let choice = match ship {
            Some(s) => Some(rules.ship_id(s).ok_or(format!("no ship class '{s}'"))?),
            None => None,
        };
        // Collision shapes: spheres for craft, triangle meshes (cheapest LOD) for
        // stations and other big static objects.
        let mut shapes = Vec::with_capacity(rules.ships.len());
        for class in &rules.ships {
            let shape = match ctx.renderer.models.get(content, &class.model, class.model_pack, class.model_scale) {
                Some(m) => collision::Shape {
                    radius: m.radius,
                    mesh: class
                        .kind
                        .is_static()
                        .then(|| collision::TriMesh { triangles: m.model.collision_lod().triangles().collect() }),
                },
                None => collision::Shape { radius: 10.0, mesh: None },
            };
            shapes.push(shape);
        }
        let (world, player) = World::from_scenario(&scenario, rules.clone(), Arc::new(shapes), choice)?;
        let player_class = rules.ship(world.get(player).expect("just spawned").class);
        let cockpit_def = player_class.cockpit.as_deref().and_then(|c| rules.cockpit(c));
        let cockpit = cockpit_def.and_then(|c| c.image.as_ref().map(|i| (i, c.image_pack))).and_then(
            |(image, pack)| match content.load_image(image, pack, true) {
                Ok(img) => {
                    let t = Texture2D::from_rgba8(img.width as u16, img.height as u16, &img.rgba);
                    t.set_filter(FilterMode::Linear);
                    Some(t)
                }
                Err(e) => {
                    eprintln!("cockpit: {e}");
                    None
                }
            },
        );
        let cockpit_model = cockpit_def.and_then(|c| {
            let model = c.model.as_ref()?;
            let gpu = ctx.renderer.models.get(content, model, c.image_pack, c.model_scale)?;
            Some((gpu, Vec3::from(c.model_offset)))
        });
        let has_cockpit = cockpit.is_some() || cockpit_model.is_some();
        let (instruments, design) = match cockpit_def.filter(|_| has_cockpit) {
            Some(c) => (c.instruments.clone(), Vec2::from(c.design)),
            None => (hud::default_instruments(), hud::DESIGN),
        };
        let mut messages = MessageLog::default();
        messages.push(0.0, scenario.title.to_uppercase());
        Ok(FlightSession {
            world,
            player,
            scenario_title: scenario.title.clone(),
            sun: scenario.sun_direction,
            seed: scenario.starfield_seed,
            effects: Effects::default(),
            messages,
            view: if has_cockpit { View::Cockpit } else { View::Forward },
            mfd: MfdPage::Target,
            chase: ChaseOrbit::default(),
            nav: scenario.nav_points.clone(),
            nav_index: 0,
            paused: false,
            menu_open: false,
            mouse_flight: false,
            accumulator: 0.0,
            cockpit,
            cockpit_model,
            design,
            instruments,
            scenario_id: scenario.id.clone(),
            ship_choice: ship.map(str::to_string),
        })
    }

    pub fn frame(&mut self, ctx: &mut AppContext) -> Next {
        let raw = ctx.devices.read();
        let (controls, commands, chase_input) = ctx.bindings.translate(&raw, self.mouse_flight);
        let dt = get_frame_time().min(MAX_FRAME);
        let alive = self.world.get(self.player).is_some();

        // --- Pause menu -------------------------------------------------------------
        if self.menu_open {
            if raw.pressed.contains("escape") {
                self.menu_open = false;
            } else if raw.pressed.contains("r") {
                return Next::Fly { scenario: self.scenario_id.clone(), ship: self.ship_choice.clone() };
            } else if raw.pressed.contains("l") || raw.pressed.contains("q") {
                return Next::Launcher;
            }
        } else {
            for cmd in &commands {
                match cmd {
                    Command::Menu => self.menu_open = true,
                    Command::Pause => self.paused = !self.paused,
                    Command::View(v) => self.view = *v,
                    Command::MfdNext => self.mfd = self.mfd.cycle(true),
                    Command::MfdPrev => self.mfd = self.mfd.cycle(false),
                    Command::NextNav if !self.nav.is_empty() => {
                        self.nav_index = (self.nav_index + 1) % self.nav.len();
                        self.messages
                            .push(self.world.time, format!("NAV {}", self.nav[self.nav_index].name.to_uppercase()));
                    }
                    Command::MouseFlight => {
                        self.mouse_flight = !self.mouse_flight;
                        show_mouse(!self.mouse_flight);
                        set_cursor_grab(self.mouse_flight);
                    }
                    other => {
                        if let Some(text) = apply_command(&mut self.world, self.player, *other) {
                            ctx.sounds.play(&crate::audio::SoundKey::Named("target"), 0.5);
                            self.messages.push(self.world.time, text);
                        }
                    }
                }
            }
            if !alive
                && (raw.pressed.contains("enter")
                    || raw.pad.is_some_and(|p| p.button(crate::input::gamepad::Button::A)))
            {
                return Next::Fly { scenario: self.scenario_id.clone(), ship: self.ship_choice.clone() };
            }
        }
        self.chase.apply(&chase_input, dt);

        // --- Simulation -------------------------------------------------------------
        if !self.paused && !self.menu_open {
            if let Some(s) = self.world.get_mut(self.player) {
                // Whatever is still set in `s.controls` was not consumed by a tick yet.
                s.controls = controls.carry_unconsumed(&s.controls);
            }
            self.accumulator += dt;
            while self.accumulator >= TICK {
                self.world.step();
                self.effects.on_events(&self.world);
                let ear = self.world.ships[self.player.index as usize].position;
                ctx.sounds.play_events(&self.world, self.player, ear);
                for m in event_messages(&self.world, self.player) {
                    self.messages.push(self.world.time, m);
                }
                self.accumulator -= TICK;
            }
            self.effects.update(dt);
        }

        // --- Camera -----------------------------------------------------------------
        let me = &self.world.ships[self.player.index as usize];
        let radius = self.world.shapes[me.class.0 as usize].radius;
        // After death, keep watching the wreck from outside.
        let view = if alive { self.view } else { View::ChaseManual };
        let (pos, rot) = view_pose(view, me.position, me.rotation, radius, &self.chase);
        let cam = SpaceCamera::new(pos, rot, 60f32.to_radians(), screen_width() / screen_height());
        let hide = if shows_own_ship(view) { None } else { Some(self.player) };

        // --- Drawing ----------------------------------------------------------------
        clear_background(BLACK);
        ctx.renderer.draw_sky(&cam, self.seed, self.sun);
        ctx.renderer.draw_world(&ctx.content, &self.world, &self.effects, &cam, self.sun, hide);
        ctx.renderer.draw_glows(&ctx.content, &self.world, &self.effects, &cam, hide);
        if let (true, View::Cockpit, Some((model, offset))) = (alive, view, &self.cockpit_model) {
            ctx.renderer.draw_cockpit_model(&cam, model, *offset, self.sun);
        }
        set_default_camera();
        let screen = Vec2::new(screen_width(), screen_height());
        let in_cockpit = alive && view == View::Cockpit;
        let with_art = in_cockpit && self.cockpit.is_some();
        let custom = in_cockpit && (self.cockpit.is_some() || self.cockpit_model.is_some());
        let nav = self.nav.get(self.nav_index).map(|n| (n.name.as_str(), n.position));
        let default_instruments;
        let (instruments, layout) = if custom {
            (&self.instruments, Layout::new(screen, self.design))
        } else {
            default_instruments = hud::default_instruments();
            (&default_instruments, Layout::new(screen, hud::DESIGN))
        };
        let frame = HudFrame {
            world: &self.world,
            player: self.player,
            camera: &cam,
            layout,
            instruments,
            mfd: self.mfd,
            nav,
            messages: &self.messages,
            now: self.world.time,
            glass: !with_art,
            forward_view: alive && matches!(view, View::Cockpit | View::Forward | View::Chase),
        };
        hud::draw::draw_targeting(&frame);
        if let (true, Some(tex)) = (with_art, &self.cockpit) {
            hud::draw::draw_cockpit(tex, layout);
        }
        if alive {
            hud::draw::draw_instruments(&frame);
        }
        // Nav point marker.
        if let Some((name, p)) = nav {
            if let Some(s) = cam.project(p, screen) {
                draw_poly_lines(s.x, s.y, 4, 8.0, 45.0, 1.5, SKYBLUE);
                draw_text(&name.to_uppercase(), s.x + 10.0, s.y, 16.0, SKYBLUE);
            }
        }
        let caption = format!(
            "{} - {:?} view{}",
            self.scenario_title,
            view,
            if self.mouse_flight { " - mouse flight (C)" } else { "" }
        );
        hud::draw::draw_caption(&caption, false);
        if !alive {
            draw_centered("SHIP DESTROYED", screen.y * 0.4, 40.0, RED);
            draw_centered("Enter: try again    Esc: menu", screen.y * 0.4 + 40.0, 22.0, WHITE);
        }
        if self.paused {
            draw_centered("PAUSED (P)", screen.y * 0.3, 36.0, YELLOW);
        }
        if self.menu_open {
            draw_rectangle(0.0, 0.0, screen.x, screen.y, Color::new(0.0, 0.0, 0.0, 0.6));
            draw_centered("MENU", screen.y * 0.35, 40.0, WHITE);
            draw_centered(
                "Esc  resume      R  restart      L  back to launcher",
                screen.y * 0.35 + 44.0,
                22.0,
                LIGHTGRAY,
            );
            draw_centered(
                "Keys: arrows pitch/yaw  Q/E or Num7/9 roll  +/- throttle  Tab burner",
                screen.y * 0.35 + 90.0,
                18.0,
                GRAY,
            );
            draw_centered(
                "Enter/LCtrl beams  Space missile  T/Y/R/G target  X link  F1-F8 views  C mouse",
                screen.y * 0.35 + 114.0,
                18.0,
                GRAY,
            );
        }
        Next::Stay
    }
}

fn draw_centered(s: &str, y: f32, size: f32, color: Color) {
    let w = measure_text(s, None, size as u16, 1.0).width;
    draw_text(s, (screen_width() - w) * 0.5, y, size, color);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::rules::FactionId;
    use crate::sim::test_support::world;
    use crate::sim::Pilot;

    #[test]
    fn views_place_the_camera() {
        let (pos, rot, r) = (Vec3::new(100.0, 0.0, 0.0), Quat::IDENTITY, 10.0);
        let chase = ChaseOrbit::default();
        let (p, q) = view_pose(View::Cockpit, pos, rot, r, &chase);
        assert!(p.distance(pos) < r && q == rot);
        let (_, q) = view_pose(View::Left, pos, rot, r, &chase);
        assert!(((q * Vec3::NEG_Z) - Vec3::NEG_X).length() < 1e-5, "left view looks left");
        let (_, q) = view_pose(View::RearRight, pos, rot, r, &chase);
        let look = q * Vec3::NEG_Z;
        assert!(look.z > 0.5 && look.x > 0.5, "rear-right looks back and right");
        let (p, _) = view_pose(View::Chase, pos, rot, r, &chase);
        assert!(p.z > pos.z + 2.0 * r && p.y > pos.y, "chase sits behind and above");
        let (p, q) = view_pose(View::ChaseManual, pos, rot, r, &chase);
        assert!(((q * Vec3::NEG_Z) - (pos - p).normalize()).length() < 1e-4, "orbit looks at the ship");
        assert!(shows_own_ship(View::Chase) && !shows_own_ship(View::Cockpit));
    }

    #[test]
    fn chase_orbit_responds_and_resets() {
        let mut c = ChaseOrbit::default();
        c.apply(&ChaseInput { yaw: 1.0, pitch: 1.0, zoom: 1.0, reset: false }, 0.5);
        assert!(c.yaw > 0.0 && c.pitch > 0.25 && c.distance < 3.2);
        c.apply(&ChaseInput { reset: true, ..Default::default() }, 0.0);
        assert_eq!(c, ChaseOrbit::default());
    }

    #[test]
    fn commands_target_and_order_wingmen() {
        let mut w = world();
        let f = w.rules.ship_id("fighter").unwrap();
        let r = w.rules.ship_id("raider").unwrap();
        let me = w.spawn(f, FactionId(0), "me", Vec3::ZERO, Quat::IDENTITY, Pilot::External);
        let foe = w.spawn(r, FactionId(1), "foe", Vec3::new(0.0, 0.0, -500.0), Quat::IDENTITY, Pilot::External);
        assert_eq!(apply_command(&mut w, me, Command::NearestEnemy), Some("TARGET: RAIDER".into()));
        assert_eq!(w.get(me).unwrap().target, Some(foe));
        assert_eq!(apply_command(&mut w, me, Command::WingFireAtWill), Some("NO WINGMEN".into()));
        w.spawn(
            f,
            FactionId(0),
            "wing",
            Vec3::X * 50.0,
            Quat::IDENTITY,
            Pilot::Ai(ai::AiState::new(ai::Behaviour::Wingman, Some(me), Vec3::X * 50.0)),
        );
        assert_eq!(apply_command(&mut w, me, Command::WingAttackMyTarget), Some("WINGMEN: ATTACK MY TARGET".into()));
        assert_eq!(apply_command(&mut w, me, Command::WingFormation), Some("WINGMEN: FORM ON ME".into()));
        assert_eq!(apply_command(&mut w, me, Command::Pause), None, "not a world command");
        w.get_mut(foe).unwrap().alive = false;
        w.get_mut(me).unwrap().target = None;
        assert_eq!(apply_command(&mut w, me, Command::NearestEnemy), Some("NO TARGET".into()));
        assert!(apply_command(&mut w, me, Command::NextTarget).unwrap().starts_with("TARGET:"));
        assert!(apply_command(&mut w, me, Command::PrevTarget).is_some());
        assert!(apply_command(&mut w, me, Command::TargetInSights).is_some());
    }

    #[test]
    fn events_become_pilot_messages() {
        let mut w = world();
        let f = w.rules.ship_id("fighter").unwrap();
        let r = w.rules.ship_id("raider").unwrap();
        let me = w.spawn(f, FactionId(0), "me", Vec3::ZERO, Quat::IDENTITY, Pilot::External);
        let foe = w.spawn(r, FactionId(1), "foe", Vec3::new(0.0, 0.0, -500.0), Quat::IDENTITY, Pilot::External);
        w.events = vec![
            SimEvent::Message { ship: me, text: "WEAPONS FULL".into() },
            SimEvent::Message { ship: foe, text: "not mine".into() },
            SimEvent::LockAcquired { ship: me },
            SimEvent::NoEnergy { ship: me },
            SimEvent::Destroyed { ship: foe, position: Vec3::ZERO, radius: 5.0 },
            SimEvent::Destroyed { ship: me, position: Vec3::ZERO, radius: 5.0 },
        ];
        assert_eq!(
            event_messages(&w, me),
            vec![
                "WEAPONS FULL",
                "MISSILE LOCK",
                "INSUFFICIENT POWER",
                "RAIDER DESTROYED",
                "YOUR SHIP HAS BEEN DESTROYED"
            ]
        );
    }
}
