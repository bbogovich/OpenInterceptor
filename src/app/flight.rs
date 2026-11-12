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
use crate::content::rules::Anchor;
use crate::content::scenario::NavPoint;
use crate::hud::{self, draw::HudFrame, layout, Layout, MessageLog, MfdPage};
use crate::input::{ChaseInput, Command, View};
use crate::render::camera::SpaceCamera;
use crate::render::effects::Effects;
use crate::render::gpu::GpuModel;
use crate::sim::{ai, collision, targeting, Condition, ShipId, SimEvent, World, TICK};
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

/// Vertical field of view (radians) for a cockpit designed at `design_aspect` (width /
/// height) shown on a screen of `aspect`. The engine's normal view has a 60° vertical
/// field; on a screen narrower than the design that would cut off the cockpit's sides
/// and misplace its instruments, so the field widens until the design's full width fits
/// — the 3D equivalent of the 2D layout fitting the design inside the screen.
pub fn cockpit_fov(design_aspect: f32, aspect: f32) -> f32 {
    let base = 60f32.to_radians();
    if aspect >= design_aspect || aspect <= 0.0 {
        base
    } else {
        2.0 * ((base * 0.5).tan() * design_aspect / aspect).atan()
    }
}

/// Whether a view shows the player's own ship model.
pub fn shows_own_ship(view: View) -> bool {
    matches!(view, View::Chase | View::ChaseManual)
}

/// The wingman menu's entries, in the original's order: `(label, command)`.
pub const WING_MENU: [(&str, Command); 6] = [
    ("Fire at will", Command::WingFireAtWill),
    ("Attack my target", Command::WingAttackMyTarget),
    ("Ignore my target", Command::WingFormation),
    ("Evasive!", Command::WingEvasive),
    ("Retreat!", Command::WingRetreat),
    ("Status report", Command::WingStatus),
];

/// Applies a targeting or wingman command to the world. Returns lines for the message
/// line (a status report has one per wingman).
/// A clip of `line` from the voice of one of the player's wingmen (`except` excluded),
/// for the radio: the first wingman with a voice set that has the line. `pick` chooses
/// Whether the debrief is due: `delay` seconds after the result was decided at
/// `ended_at` (a result with no recorded time shows at once).
pub fn debrief_due(ended_at: Option<f64>, now: f64, delay: f32) -> bool {
    ended_at.is_none_or(|t| now >= t + delay as f64)
}

/// Applies the campaign's relations to a battle (Stage 5.5): each `(faction, hostile)`
/// makes that faction hostile to the player's faction, or not. In the battle's own copy
/// of the rules, like the loadout.
pub fn apply_stances(world: &mut World, player: ShipId, stances: &[(String, bool)]) {
    let Some(me) = world.get(player).map(|s| s.faction) else { return };
    let mine = world.rules.faction(me).id.clone();
    let rules = std::sync::Arc::make_mut(&mut world.rules);
    for (f, hostile) in stances {
        if *f != mine {
            rules.set_hostile(&mine, f, *hostile);
        }
    }
}

/// The campaign loadout on a battle's world (see [`FlightSession::apply_loadout`]).
pub fn apply_loadout(world: &mut World, player: ShipId, beams: &[String], missiles: &[(String, u32)]) {
    let Some(class) = world.get(player).map(|s| s.class) else { return };
    let rules = std::sync::Arc::make_mut(&mut world.rules);
    let beam_ids: Vec<_> = beams
        .iter()
        .filter_map(|b| rules.beams.iter().position(|x| &x.id == b))
        .map(|i| crate::content::rules::BeamId(i as u16))
        .collect();
    let racks: Vec<_> = missiles
        .iter()
        .filter_map(|(m, n)| {
            rules.missiles.iter().position(|x| &x.id == m).map(|i| (crate::content::rules::MissileId(i as u16), *n))
        })
        .collect();
    if !beam_ids.is_empty() {
        rules.ships[class.0 as usize].beams = beam_ids;
    }
    let mounts = rules.ships[class.0 as usize].beams.len();
    for s in world.ships.iter_mut().filter(|s| s.alive && s.class == class) {
        s.beam_cooldown = vec![0.0; mounts];
        s.next_mount = 0;
    }
    if let Some(me) = world.get_mut(player) {
        if !racks.is_empty() {
            me.missiles = racks;
            me.selected_rack = 0;
        }
    }
}

/// among its clips.
pub fn wingman_voice(world: &World, player: ShipId, except: Option<ShipId>, line: &str, pick: usize) -> Option<u32> {
    world.ships.iter().enumerate().find_map(|(i, s)| {
        let id = world.id_of(i);
        let ours = s.alive
            && Some(id) != except
            && matches!(&s.pilot, crate::sim::Pilot::Ai(a) if a.behaviour == crate::sim::ai::Behaviour::Wingman && a.leader == Some(player));
        let clips = world.rules.voices.get(s.voice? as usize)?.clips(line);
        (ours && !clips.is_empty()).then(|| clips[pick % clips.len()])
    })
}

pub fn apply_command(world: &mut World, player: ShipId, cmd: Command) -> Vec<String> {
    let Some(current) = world.get(player).map(|s| s.target) else { return Vec::new() };
    let new_target = match cmd {
        Command::NextTarget => targeting::cycle(world, player, current, true),
        Command::PrevTarget => targeting::cycle(world, player, current, false),
        Command::NearestEnemy => targeting::nearest_hostile(world, player),
        Command::TargetInSights => targeting::in_sights(world, player, 15.0),
        Command::WingFireAtWill => return order(world, player, ai::WingOrder::FireAtWill, "WINGMEN: FIRE AT WILL"),
        Command::WingAttackMyTarget => {
            return order(world, player, ai::WingOrder::AttackMyTarget, "WINGMEN: ATTACK MY TARGET")
        }
        Command::WingFormation => return order(world, player, ai::WingOrder::Formation, "WINGMEN: IGNORE MY TARGET"),
        Command::WingEvasive => return order(world, player, ai::WingOrder::Evasive, "WINGMEN: EVASIVE!"),
        Command::WingRetreat => return order(world, player, ai::WingOrder::Retreat, "WINGMEN: RETREAT!"),
        Command::WingNextFormation => {
            return vec![match ai::next_formation(world, player) {
                Some(f) => format!("WINGMEN: FORM {}", f.replace('_', " ").to_uppercase()),
                None => "NO WINGMEN".into(),
            }]
        }
        Command::WingStatus => {
            let lines = ai::wing_status(world, player);
            return if lines.is_empty() { vec!["NO WINGMEN".into()] } else { lines };
        }
        _ => return Vec::new(),
    };
    let Some(s) = world.get_mut(player) else { return Vec::new() };
    if new_target != s.target {
        s.lock = Default::default();
    }
    s.target = new_target;
    vec![match new_target {
        Some(t) => {
            let name = world.rules.ship(world.ships[t.index as usize].class).name.clone();
            format!("TARGET: {}", name.to_uppercase())
        }
        None => "NO TARGET".into(),
    }]
}

fn order(world: &mut World, player: ShipId, o: ai::WingOrder, text: &str) -> Vec<String> {
    vec![match ai::order_wingmen(world, player, o) {
        0 => "NO WINGMEN".into(),
        _ => text.into(),
    }]
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
            SimEvent::SystemChanged { ship, system, condition, repaired } if *ship == player => {
                let s = &world.ships[player.index as usize];
                let name = world.rules.system(s.systems[*system].id).name.to_uppercase();
                out.push(match (condition, repaired) {
                    (Condition::Ok, _) => format!("{name} REPAIRED"),
                    (Condition::Damaged, true) => format!("{name} BACK ON-LINE"),
                    (Condition::Damaged, false) => format!("{name} DAMAGED: HALF EFFICIENCY"),
                    (Condition::Inoperative, _) => format!("{name} INOPERATIVE: REPAIRS UNDERWAY"),
                    (Condition::Destroyed, _) => format!("{name} DESTROYED"),
                });
            }
            SimEvent::Disabled { ship } if *ship == player => {
                out.push("ENGINES AND THRUSTERS OUT: SHIP DISABLED".into())
            }
            SimEvent::Disabled { ship } if Some(*ship) == target => out.push("TARGET DISABLED".into()),
            SimEvent::Departed { ship, .. } if *ship == player => out.push("HYPERJUMP COMPLETE".into()),
            SimEvent::Departed { ship, .. } => {
                let s = &world.ships[ship.index as usize];
                if world.rules.hostile(world.ships[player.index as usize].faction, s.faction) || Some(*ship) == target {
                    out.push(format!("{} JUMPED OUT", world.rules.ship(s.class).name.to_uppercase()));
                }
            }
            SimEvent::Captured { prize, .. } => {
                let s = &world.ships[prize.index as usize];
                out.push(format!("{} CAPTURED", world.rules.ship(s.class).name.to_uppercase()));
            }
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
    /// Seconds since the last telemetry message to the console.
    console_clock: f32,
    /// A campaign battle: the debrief returns to the strategy screen, with the result.
    pub campaign_battle: bool,
    /// World time the mission's result was decided (the debrief waits
    /// `flight.debrief_delay` after it, so the battle can be watched to its end).
    ended_at: Option<f64>,
    /// The player asked for the debrief now (Enter during the wait).
    skip_wait: bool,
    /// A simulator mission (a modified copy), kept so "again" flies the same setup.
    custom: Option<crate::content::mission::MissionDef>,
    /// Hides the keys still held from the menu that started this battle (see StartGuard).
    start_guard: crate::input::StartGuard,
    /// Hits on the player's craft so far, and how many reached the hull: the console plays
    /// a burst of static when either count rises.
    console_hits: [u32; 2],
    /// The craft class the console's catalogue was built for (resent when it changes).
    console_class: Option<crate::content::rules::ShipClassId>,
    /// The target class whose model the console has (C3), and the encoded `model`
    /// messages by class, built once each.
    console_target: Option<crate::content::rules::ShipClassId>,
    /// Seconds since the last `contacts` message (5 Hz), and how many log messages the
    /// console has been sent as events.
    console_contacts_clock: f32,
    console_messages_seen: u64,
    console_models: std::collections::HashMap<crate::content::rules::ShipClassId, std::rc::Rc<String>>,
    /// The keys help overlay (`?` or Shift+F1) is showing; the battle pauses under it.
    help_open: bool,
    mouse_flight: bool,
    accumulator: f32,
    /// 2D cockpit art, if the player's cockpit has an image.
    cockpit: Option<Texture2D>,
    /// The cockpit's wide-screen side bands (R-HUD-013) and their extension art.
    cockpit_wide: Option<crate::content::rules::WideFill>,
    wide_art: [Option<Texture2D>; 2],
    /// 3D cockpit model and its offset from the pilot's eye, if any.
    cockpit_model: Option<(Rc<GpuModel>, Vec3)>,
    /// The cockpit's design space (instrument coordinates).
    design: Vec2,
    instruments: HashMap<String, [f32; 4]>,
    scenario_id: String,
    ship_choice: Option<String>,
    /// The wingman menu, while open: the highlighted entry.
    wing_menu: Option<usize>,
    /// The controls menu, while open over the battle (from the pause menu).
    controls: Option<super::controls::ControlsMenu>,
    /// Prizes captured by the player's tractor beam when jumping out.
    captured: Vec<String>,
    /// The cockpit's id (for saving its layout).
    cockpit_id: Option<String>,
    /// A 3D cockpit's instruments placed on its surface (drawn in perspective).
    anchors: HashMap<String, Anchor>,
    /// One texture per anchored instrument, drawn into each frame.
    instrument_targets: HashMap<String, RenderTarget>,
    /// The cockpit layout editor, while open (F12).
    editor: Option<LayoutEditor>,
    /// The mission refereeing this battle, if it is one.
    mission: Option<crate::mission::MissionRun>,
}

/// The cockpit layout editor's state (see `hud::layout` for the maths).
struct LayoutEditor {
    /// Index into `hud::draw::INSTRUMENTS`.
    selected: usize,
    /// A 2D drag in progress: the grip and the last mouse position (design units).
    drag: Option<(layout::Grip, Vec2)>,
    message: String,
    changed: bool,
}

impl FlightSession {
    /// Builds the world for scenario `id`, with `ship` as the player's class if the
    /// scenario lets the player choose.
    pub fn start(ctx: &mut AppContext, id: &str, ship: Option<&str>) -> Result<FlightSession, String> {
        Self::start_battle(ctx, id, ship, false)
    }

    /// Like [`FlightSession::start`] for mission `id` (refereed by a
    /// [`crate::mission::MissionRun`]).
    pub fn start_mission(ctx: &mut AppContext, id: &str, ship: Option<&str>) -> Result<FlightSession, String> {
        Self::start_battle(ctx, id, ship, true)
    }

    /// Fits the player's craft with a campaign loadout (Stage 4.7): `beams` are beam ids
    /// (one per mount) and `missiles` missile ids with rounds. Ids that are not beams or
    /// missiles of this campaign are skipped; with no beams the class's own are kept.
    ///
    /// Beams belong to the ship *class*, so the class is changed in this battle's own
    /// copy of the rules (`Arc::make_mut` copies the shared ruleset the first time it is
    /// changed) — the content everyone else uses is untouched. Other craft of the same
    /// class in this battle (wingmen) fly the same fit.
    pub fn apply_loadout(&mut self, beams: &[String], missiles: &[(String, u32)]) {
        apply_loadout(&mut self.world, self.player, beams, missiles);
    }

    /// Seconds still to wait before the debrief (0 once it shows).
    fn debrief_wait_left(&self) -> f64 {
        match self.ended_at {
            Some(t) => (t + self.world.rules.flight.debrief_delay as f64 - self.world.time).max(0.0),
            None => 0.0,
        }
    }

    /// Whether the debrief panel is up: the mission has a result, and the wait after it
    /// is over (or was skipped, or the player left the battle, when there is nothing
    /// left to watch).
    fn debrief_shown(&self) -> bool {
        let Some(d) = self.mission.as_ref().and_then(|m| m.debrief.as_ref()) else { return false };
        self.skip_wait
            || d.result == "retreat"
            || debrief_due(self.ended_at, self.world.time, self.world.rules.flight.debrief_delay)
    }

    /// Starts a mission from a ready-made definition: the simulator's modified copy.
    pub fn start_custom(
        ctx: &mut AppContext,
        def: crate::content::mission::MissionDef,
        ship: Option<&str>,
    ) -> Result<FlightSession, String> {
        let id = def.id.clone();
        let mut session = Self::start_with(ctx, &id, ship, Some(def.clone()))?;
        session.custom = Some(def);
        Ok(session)
    }

    fn start_battle(
        ctx: &mut AppContext,
        id: &str,
        ship: Option<&str>,
        is_mission: bool,
    ) -> Result<FlightSession, String> {
        let mission_def = match is_mission {
            true => Some(
                ctx.content.rules.missions.iter().find(|m| m.id == id).ok_or(format!("no mission '{id}'"))?.clone(),
            ),
            false => None,
        };
        Self::start_with(ctx, id, ship, mission_def)
    }

    fn start_with(
        ctx: &mut AppContext,
        id: &str,
        ship: Option<&str>,
        mission_def: Option<crate::content::mission::MissionDef>,
    ) -> Result<FlightSession, String> {
        let content = &ctx.content;
        // A mission shows its name and briefing; its battle is built by the mission run.
        let scenario = match &mission_def {
            Some(m) => crate::content::scenario::Scenario {
                id: m.id.clone(),
                title: m.name.clone(),
                description: m.briefing.clone(),
                player_class: m.player_class.clone(),
                player_position: m.player_position,
                player_rotation: Vec3::ZERO,
                hazards: Default::default(), // missions place theirs after the roles
                environment: None,           // set by the mission run
                sun_direction: m.sun,
                starfield_seed: m.starfield_seed,
                music: None,
                objects: Vec::new(),
                nav_points: Vec::new(),
                pack: m.pack,
                order: m.order,
            },
            None => content.scenario(id).ok_or(format!("no scenario '{id}'"))?.clone(),
        };
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
                Some(m) => collision::Shape::from_model(&m.model, class.kind.is_static()),
                None => collision::Shape::sphere(10.0),
            };
            shapes.push(shape);
        }
        let (world, player, mission) = match &mission_def {
            Some(def) => {
                let (w, run) = crate::mission::MissionRun::start(def, rules.clone(), Arc::new(shapes), choice)?;
                let player = run.player;
                (w, player, Some(run))
            }
            None => {
                let (w, p) = World::from_scenario(&scenario, rules.clone(), Arc::new(shapes), choice)?;
                (w, p, None)
            }
        };
        let mut world = world;
        // Simple physics for the player, when chosen and allowed at this level (R-FLT-013).
        if let Some(s) = world.get_mut(player) {
            s.simple_physics = ctx.simple_physics && rules.simple_physics_allowed();
        }
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
        let cockpit_id = cockpit_def.map(|c| c.id.clone());
        // Wide screens (R-HUD-013): the side bands and their optional extension art.
        let cockpit_wide = cockpit_def.map(|c| c.wide.clone());
        let load_art = |name: Option<&String>, pack: usize| {
            let img = content.load_image(name?, pack, true).map_err(|e| eprintln!("cockpit wide art: {e}")).ok()?;
            let t = Texture2D::from_rgba8(img.width as u16, img.height as u16, &img.rgba);
            t.set_filter(FilterMode::Linear);
            Some(t)
        };
        let wide_art = match cockpit_def {
            Some(c) => [load_art(c.wide.left.as_ref(), c.image_pack), load_art(c.wide.right.as_ref(), c.image_pack)],
            None => [None, None],
        };
        let cockpit_model = cockpit_def.and_then(|c| {
            let model = c.model.as_ref()?;
            let gpu = ctx.renderer.models.get(content, model, c.image_pack, c.model_scale)?;
            Some((gpu, Vec3::from(c.model_offset)))
        });
        let has_cockpit = cockpit.is_some() || cockpit_model.is_some();
        // Anchors apply to a 3D cockpit without 2D art.
        let anchors = match (cockpit_def, &cockpit_model, &cockpit) {
            (Some(c), Some(_), None) => c.anchors.clone(),
            _ => HashMap::new(),
        };
        let (instruments, design) = match cockpit_def.filter(|_| has_cockpit) {
            Some(c) => (c.instruments.clone(), Vec2::from(c.design)),
            None => (hud::default_instruments(), hud::DESIGN),
        };
        let mut messages = MessageLog::default();
        messages.push(0.0, scenario.title.to_uppercase());
        // Tell the pilot about an ion storm (it shortens their radar).
        if let Some(f) = world.hazards.ion_storm {
            messages.push(0.0, format!("ION STORM: SENSORS AT {:.0}%", f * 100.0));
        }
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
            help_open: false,
            console_clock: 0.0,
            start_guard: Default::default(),
            custom: None,
            campaign_battle: false,
            ended_at: None,
            skip_wait: false,
            console_hits: [0, 0],
            console_class: None,
            console_target: None,
            console_contacts_clock: 0.0,
            console_messages_seen: 0,
            console_models: Default::default(),
            mouse_flight: false,
            accumulator: 0.0,
            cockpit,
            cockpit_wide,
            wide_art,
            cockpit_model,
            design,
            instruments,
            scenario_id: scenario.id.clone(),
            ship_choice: ship.map(str::to_string),
            wing_menu: None,
            controls: None,
            captured: Vec::new(),
            cockpit_id,
            anchors,
            instrument_targets: HashMap::new(),
            editor: None,
            mission,
        })
    }

    /// Where "fly again" goes: the same scenario or mission with the same ship.
    fn again(&self) -> Next {
        if let Some(def) = &self.custom {
            return Next::Simulate { mission: Box::new(def.clone()), ship: self.ship_choice.clone() };
        }
        match &self.mission {
            Some(_) => Next::Mission { mission: self.scenario_id.clone(), ship: self.ship_choice.clone() },
            None => Next::Fly { scenario: self.scenario_id.clone(), ship: self.ship_choice.clone() },
        }
    }

    pub fn frame(&mut self, ctx: &mut AppContext) -> Next {
        // The controls menu, opened from the pause menu, takes the whole frame.
        if let Some(menu) = &mut self.controls {
            if menu.frame(ctx) {
                self.controls = None;
            }
            return Next::Stay;
        }
        let mut raw = ctx.devices.read();
        // The Enter (or pad A) that started this battle must not also fire the guns.
        self.start_guard.filter(&mut raw);
        self.console_frame(ctx, &mut raw);
        // F12 in the cockpit view opens the layout editor (the battle pauses).
        let in_custom_cockpit = self.view == View::Cockpit && (self.cockpit.is_some() || self.cockpit_model.is_some());
        if raw.pressed.contains("f12") && in_custom_cockpit {
            self.editor = match self.editor {
                Some(_) => None,
                None => Some(LayoutEditor {
                    selected: 0,
                    drag: None,
                    message: "Tab: next instrument   click/drag: place   wheel or +/-: size   S: save   F12/Esc: done"
                        .into(),
                    changed: false,
                }),
            };
        }
        if self.editor.is_some() {
            self.edit_layout(ctx, &raw);
        }
        let (mut controls, mut commands, chase_input) = ctx.bindings.translate(&raw, self.mouse_flight);
        if self.editor.is_some() {
            // The editor owns the keyboard and mouse; the battle waits.
            controls = Default::default();
            commands.clear();
        }
        let dt = get_frame_time().min(MAX_FRAME);
        let alive = self.world.get(self.player).is_some();
        let departed = self.world.ships[self.player.index as usize].departed;

        // --- Wingman menu: digits (or the d-pad and A) choose, the menu key closes ----
        if let Some(sel) = self.wing_menu {
            let pressed = |k: &str| raw.pressed.contains(k);
            let n = WING_MENU.len();
            let mut choice = (1..=n).find(|d| pressed(&format!("key{d}")) || pressed(&format!("kp{d}")));
            let mut highlight = sel;
            if raw.pad_pressed(crate::input::gamepad::Button::DpadDown) {
                highlight = (sel + 1) % n;
            }
            if raw.pad_pressed(crate::input::gamepad::Button::DpadUp) {
                highlight = (sel + n - 1) % n;
            }
            if raw.pad_pressed(crate::input::gamepad::Button::A) {
                choice = Some(sel + 1);
            }
            let close = raw.pad_pressed(crate::input::gamepad::Button::B) || commands.contains(&Command::WingMenu);
            self.wing_menu = Some(highlight);
            // While the menu is open the d-pad and A belong to it, not to the throttle,
            // the MFD or the afterburner.
            controls.throttle_delta = 0.0;
            controls.afterburner = false;
            commands.retain(|c| !matches!(c, Command::MfdNext | Command::MfdPrev | Command::WingMenu));
            if let Some(k) = choice {
                commands.push(WING_MENU[k - 1].1);
                self.wing_menu = None;
            } else if close {
                self.wing_menu = None;
            }
        }

        // --- Debrief: the mission is over ---------------------------------------------
        // Between the result and the debrief, Enter skips the wait.
        if self.mission.as_ref().is_some_and(|m| m.debrief.is_some()) && !self.debrief_shown() {
            if raw.pressed.contains("enter") {
                self.skip_wait = true;
                return Next::Stay; // this press only opens the debrief
            }
        } else if self.debrief_shown() && self.campaign_battle {
            // A campaign battle: any key returns to the campaign with the result.
            if raw.pressed.contains("enter") || raw.pressed.contains("escape") {
                if let Some(d) = self.mission.as_ref().and_then(|m| m.debrief.as_ref()) {
                    ctx.battle_report = Some(super::strategy::BattleReport {
                        result: d.result.clone(),
                        actions: d.campaign.clone(),
                        kills: d.kills,
                        cargo: d.cargo,
                    });
                }
                return Next::Strategy;
            }
        } else if self.debrief_shown() {
            if raw.pressed.contains("enter") || raw.pad.is_some_and(|p| p.button(crate::input::gamepad::Button::A)) {
                return self.again();
            }
            if raw.pressed.contains("escape") || raw.pressed.contains("l") {
                return Next::Launcher;
            }
        }

        // --- Pause menu -------------------------------------------------------------
        if self.help_open {
            if raw.pressed.contains("escape") || commands.contains(&Command::Help) {
                self.help_open = false;
            }
        } else if self.menu_open {
            if raw.pressed.contains("escape") {
                self.menu_open = false;
            } else if raw.pressed.contains("r") && !self.campaign_battle {
                return self.again();
            } else if (raw.pressed.contains("l") || raw.pressed.contains("q")) && self.campaign_battle {
                // Leaving a campaign battle early is a retreat.
                ctx.battle_report = Some(super::strategy::BattleReport {
                    result: "retreat".into(),
                    actions: Vec::new(),
                    kills: 0,
                    cargo: None,
                });
                return Next::Strategy;
            } else if raw.pressed.contains("l") || raw.pressed.contains("q") {
                return Next::Launcher;
            } else if raw.pressed.contains("k") {
                self.controls = Some(super::controls::ControlsMenu::new(ctx));
                return Next::Stay;
            }
        } else {
            for cmd in &commands {
                match cmd {
                    Command::Menu if self.wing_menu.is_some() => self.wing_menu = None,
                    Command::Menu => self.menu_open = true,
                    Command::WingMenu => self.wing_menu = Some(0),
                    Command::RepeatMessage => {
                        if let Some(m) = self.messages.last().map(str::to_string) {
                            self.messages.push(self.world.time, m);
                        }
                    }
                    Command::Pause => self.paused = !self.paused,
                    Command::Help => self.help_open = true,
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
                        let lines = apply_command(&mut self.world, self.player, *other);
                        if !lines.is_empty() {
                            ctx.sounds.play(&crate::audio::SoundKey::Named("target"), 0.5);
                        }
                        // A wingman answers on the radio (R-MSN-007).
                        let line = match other {
                            Command::WingStatus => Some("status"),
                            Command::WingFireAtWill
                            | Command::WingAttackMyTarget
                            | Command::WingFormation
                            | Command::WingEvasive
                            | Command::WingRetreat => Some("acknowledge"),
                            _ => None,
                        };
                        let pick = (self.world.time * 7.0) as usize;
                        if let Some(clip) = line.and_then(|l| wingman_voice(&self.world, self.player, None, l, pick)) {
                            ctx.sounds.play(&crate::audio::SoundKey::Voice(clip), 0.9);
                        }
                        for text in lines {
                            self.messages.push(self.world.time, text);
                        }
                    }
                }
            }
            if !alive
                && (raw.pressed.contains("enter")
                    || raw.pad.is_some_and(|p| p.button(crate::input::gamepad::Button::A)))
            {
                return self.again();
            }
        }
        self.chase.apply(&chase_input, dt);

        // --- Simulation -------------------------------------------------------------
        let over = self.debrief_shown();
        if !self.paused && !self.menu_open && !self.help_open && self.editor.is_none() && !over {
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
                if let Some(m) = &mut self.mission {
                    let ended = m.debrief.is_some();
                    for line in m.update(&mut self.world) {
                        self.messages.push(self.world.time, line);
                    }
                    // Chatter spoken on the radio, and the computer's verdict at the end.
                    for clip in m.voices.drain(..) {
                        ctx.sounds.play(&crate::audio::SoundKey::Voice(clip), 0.9);
                    }
                    if let (false, Some(d)) = (ended, &m.debrief) {
                        self.ended_at = Some(self.world.time);
                        let name = if d.result == "win" { "mission_accomplished" } else { "mission_failed" };
                        ctx.sounds.play(&crate::audio::SoundKey::Named(name), 0.9);
                    }
                }
                for e in &self.world.events {
                    if let SimEvent::Hit { ship, shield, .. } = e {
                        if *ship == self.player {
                            self.console_hits[0] += 1;
                            self.console_hits[1] += u32::from(!*shield);
                        }
                    }
                    // A wingman lost: another says so on the radio.
                    if let SimEvent::Destroyed { ship, .. } = e {
                        let was_wingman = matches!(&self.world.ships[ship.index as usize].pilot,
                            crate::sim::Pilot::Ai(a) if a.behaviour == crate::sim::ai::Behaviour::Wingman && a.leader == Some(self.player));
                        if was_wingman {
                            let pick = (self.world.time * 11.0) as usize;
                            if let Some(clip) = wingman_voice(&self.world, self.player, Some(*ship), "down", pick) {
                                ctx.sounds.play(&crate::audio::SoundKey::Voice(clip), 0.9);
                            }
                        }
                    }
                    if let SimEvent::Captured { captor, prize } = e {
                        if *captor == self.player {
                            let class = self.world.ships[prize.index as usize].class;
                            self.captured.push(self.world.rules.ship(class).name.clone());
                        }
                    }
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
        let aspect = screen_width() / screen_height();
        // A 3D cockpit (no 2D art) keeps its whole design width on narrow screens.
        let fov = if alive && view == View::Cockpit && self.cockpit_model.is_some() && self.cockpit.is_none() {
            cockpit_fov(self.design.x / self.design.y, aspect)
        } else {
            60f32.to_radians()
        };
        let cam = SpaceCamera::new(pos, rot, fov, aspect).with_velocity(me.velocity);
        ctx.renderer.lighting = self.world.rules.flight.lighting;
        let hide = if shows_own_ship(view) { None } else { Some(self.player) };

        // --- Drawing ----------------------------------------------------------------
        // The battle's environment: a sky and fog (atmosphere), a planet (orbit).
        let env = self.world.env.id.as_ref().and_then(|id| self.world.rules.environments.iter().find(|e| &e.id == id));
        ctx.renderer.env_sky = env.and_then(|e| e.sky);
        ctx.renderer.planet = env.and_then(|e| e.planet);
        ctx.renderer.nebula = env.and_then(|e| e.nebula);
        clear_background(match ctx.renderer.env_sky {
            Some(s) => Color::new(s.horizon[0], s.horizon[1], s.horizon[2], 1.0),
            None => BLACK,
        });
        ctx.renderer.draw_sky(&cam, self.seed, self.sun);
        ctx.renderer.draw_world(&ctx.content, &self.world, &self.effects, &cam, self.sun, hide);
        ctx.renderer.draw_glows(&ctx.content, &self.world, &self.effects, &cam, hide, Some(self.player));
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
        // A 3D cockpit's anchored instruments are drawn on its surface instead.
        let anchored: std::collections::HashSet<String> =
            if in_cockpit && !with_art { self.anchors.keys().cloned().collect() } else { Default::default() };
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
            backdrops: !custom,
            forward_view: alive && matches!(view, View::Cockpit | View::Forward | View::Chase),
            anchored: &anchored,
            text_scale: 1.0,
        };
        if alive && !anchored.is_empty() {
            let offset = self.cockpit_model.as_ref().map(|(_, o)| *o).unwrap_or_default();
            draw_anchored(&frame, &self.anchors, &mut self.instrument_targets, &cam, offset, screen);
        }
        hud::draw::draw_targeting(&frame);
        if let (true, Some(tex)) = (with_art, &self.cockpit) {
            hud::draw::draw_cockpit(tex, layout);
            if let Some(def) = &self.cockpit_wide {
                hud::draw::draw_side_bands(layout, def, self.wide_art[0].as_ref(), self.wide_art[1].as_ref());
            }
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
        if departed {
            draw_centered("JUMPED TO HYPERSPACE", screen.y * 0.4, 40.0, SKYBLUE);
            if !self.captured.is_empty() {
                draw_centered(&format!("Captured: {}", self.captured.join(", ")), screen.y * 0.4 + 36.0, 24.0, GOLD);
            }
            draw_centered("Enter: fly again    Esc: menu", screen.y * 0.4 + 70.0, 22.0, WHITE);
        } else if !alive {
            draw_centered("SHIP DESTROYED", screen.y * 0.4, 40.0, RED);
            draw_centered("Enter: try again    Esc: menu", screen.y * 0.4 + 40.0, 22.0, WHITE);
        }
        if let Some(sel) = self.wing_menu {
            let (w, h) = (300.0, 34.0 + WING_MENU.len() as f32 * 26.0);
            let (x, y) = (screen.x * 0.5 - w * 0.5, screen.y * 0.18);
            draw_rectangle(x, y, w, h, Color::new(0.0, 0.06, 0.02, 0.85));
            draw_rectangle_lines(x, y, w, h, 1.5, Color::new(0.35, 1.0, 0.45, 0.8));
            draw_text("WINGMEN", x + 12.0, y + 24.0, 22.0, Color::new(0.35, 1.0, 0.45, 1.0));
            for (i, (label, _)) in WING_MENU.iter().enumerate() {
                let ly = y + 50.0 + i as f32 * 26.0;
                let c = if i == sel { WHITE } else { Color::new(0.35, 1.0, 0.45, 0.9) };
                draw_text(&format!("{}  {label}", i + 1), x + 16.0, ly, 20.0, c);
            }
        }
        if self.paused {
            draw_centered("PAUSED (P)", screen.y * 0.3, 36.0, YELLOW);
        }
        if let Some(d) = self.mission.as_ref().and_then(|m| m.debrief.as_ref()) {
            if self.debrief_shown() {
                draw_debrief(d, screen);
            } else {
                // The result is in; the battle plays on until the debrief.
                let left = self.debrief_wait_left().ceil();
                let verdict = match d.result.as_str() {
                    "win" => "MISSION ACCOMPLISHED",
                    "lose" => "MISSION FAILED",
                    "killed" => "YOUR CRAFT WAS DESTROYED",
                    _ => "MISSION ENDED",
                };
                draw_centered(verdict, screen.y * 0.22, 36.0, YELLOW);
                draw_centered(
                    &format!("Debrief in {left:.0} s  (Enter: now)"),
                    screen.y * 0.22 + 34.0,
                    22.0,
                    LIGHTGRAY,
                );
            }
        }
        if let Some(ed) = &self.editor {
            let name = hud::draw::INSTRUMENTS[ed.selected];
            let gold = Color::new(1.0, 0.8, 0.2, 1.0);
            match self.anchors.get(name) {
                Some(a) => {
                    let offset = self.cockpit_model.as_ref().map(|(_, o)| *o).unwrap_or_default();
                    let c = layout::anchor_corners(a, offset);
                    let pts: Vec<Option<Vec2>> =
                        c.iter().map(|p| cam.project(cam.position + cam.rotation * *p, screen)).collect();
                    for k in 0..4 {
                        if let (Some(p), Some(q)) = (pts[k], pts[(k + 1) % 4]) {
                            draw_line(p.x, p.y, q.x, q.y, 2.0, gold);
                        }
                    }
                }
                None => {
                    if let Some(r) = self.instruments.get(name) {
                        let r = Layout::new(screen, self.design).rect(*r);
                        draw_rectangle_lines(r[0], r[1], r[2], r[3], 2.0, gold);
                        draw_rectangle(r[0] + r[2] - 6.0, r[1] + r[3] - 6.0, 12.0, 12.0, gold);
                    }
                }
            }
            draw_rectangle(0.0, 0.0, screen.x, 54.0, Color::new(0.0, 0.0, 0.0, 0.7));
            draw_text(&format!("COCKPIT LAYOUT: {name}"), 12.0, 22.0, 22.0, gold);
            draw_text(&ed.message, 12.0, 44.0, 18.0, WHITE);
        }
        // Until a tablet pairs, show where to point it and the code to type.
        if let Some(server) = ctx.console.as_ref().filter(|s| s.paired() == 0 || self.menu_open) {
            let hint = format!("CONSOLE  {}   CODE {}", server.url, server.code);
            // One line above the view caption in the bottom-left corner.
            draw_text(&hint, 12.0, screen.y - 36.0, 18.0, Color::new(0.5, 0.9, 0.6, 0.85));
        }
        if self.menu_open {
            draw_rectangle(0.0, 0.0, screen.x, screen.y, Color::new(0.0, 0.0, 0.0, 0.6));
            draw_centered("MENU", screen.y * 0.35, 40.0, WHITE);
            draw_centered(
                "Esc  resume      R  restart      K  controls      L  back to launcher",
                screen.y * 0.35 + 44.0,
                22.0,
                LIGHTGRAY,
            );
            draw_centered("?  or  Shift+F1   every key, grouped", screen.y * 0.35 + 90.0, 18.0, GRAY);
        }
        if self.help_open {
            draw_help(&ctx.bindings, screen);
        }
        // Other screens draw plain space: the sky belongs to this battle only.
        ctx.renderer.env_sky = None;
        ctx.renderer.planet = None;
        ctx.renderer.nebula = None;
        Next::Stay
    }
}

impl FlightSession {
    /// One frame of the tablet console: its input joins this frame's `raw`; paired
    /// consoles get the catalogue (again when the craft changes) and 20 Hz telemetry.
    fn console_frame(&mut self, ctx: &mut AppContext, raw: &mut crate::input::RawInput) {
        use crate::remote::{protocol, server::Event};
        let Some(server) = &ctx.console else { return };
        let class = self.world.get(self.player).map(|s| s.class);
        let mut resend = class != self.console_class;
        for e in server.poll() {
            if let Event::Paired(id) = e {
                let welcome = crate::remote::json::Obj::new()
                    .str("type", "welcome")
                    .num("version", protocol::VERSION as f64)
                    .str("campaign", ctx.content.campaign.as_deref().unwrap_or(""))
                    .end();
                server.send(id, &welcome);
                resend = true;
            }
            // Tapping a contact on the map targets it (if the player can see it).
            if let Event::Input(_, protocol::Inbound::Target { index, generation }) = e {
                let id = crate::sim::ShipId { index, generation };
                let visible = match (self.world.get(self.player), self.world.get(id)) {
                    (Some(me), Some(t)) => crate::sim::systems::on_radar(&self.world, me, t),
                    _ => false,
                };
                if visible {
                    if let Some(me) = self.world.get_mut(self.player) {
                        me.target = Some(id);
                    }
                }
            }
            ctx.remote.apply(&e, get_time());
        }
        if resend {
            server.broadcast(&protocol::catalogue(&self.world.rules, &self.world, self.player));
            // The console takes the look of the campaign and of this craft's cockpit.
            let cockpit = class.and_then(|c| self.world.rules.ship(c).cockpit.clone());
            server.broadcast(&protocol::theme(&self.world.rules.console.for_cockpit(cockpit.as_deref())));
            self.console_class = class;
            self.console_target = None; // a new console needs the target model too
        }
        // The target viewport's model (C3): sent when the target's class changes.
        let target_class =
            self.world.get(self.player).and_then(|s| s.target).and_then(|t| self.world.get(t)).map(|t| t.class);
        if let Some(tc) = target_class.filter(|tc| Some(*tc) != self.console_target) {
            let content = &ctx.content;
            let rules = &self.world.rules;
            let msg = self.console_models.entry(tc).or_insert_with(|| {
                let c = rules.ship(tc);
                let text = match content.load_model(&c.model, c.model_pack, c.model_scale) {
                    Ok(m) => {
                        let lod = m.lods.iter().find(|l| l.triangle_count() <= protocol::MODEL_TRIANGLE_BUDGET);
                        let lod = lod.unwrap_or_else(|| m.lods.last().expect("a model always has a LOD"));
                        // Each textured group: its average colour (shown until the
                        // image loads) and the image itself, shrunk, as a PNG.
                        let (colors, textures): (Vec<[f32; 3]>, Vec<Option<String>>) = lod
                            .groups
                            .iter()
                            .map(|g| {
                                match g.texture.as_ref().and_then(|t| content.load_texture(t, c.model_pack).ok()) {
                                    Some(img) => (
                                        protocol::average_color(&img.rgba),
                                        Some(protocol::texture_url(img.width, img.height, &img.rgba)),
                                    ),
                                    None => ([0.7, 0.7, 0.7], None),
                                }
                            })
                            .unzip();
                        protocol::model(&c.name, &m, &colors, &textures)
                    }
                    Err(_) => crate::remote::json::Obj::new().str("type", "model").str("name", &c.name).end(),
                };
                std::rc::Rc::new(text)
            });
            server.broadcast(msg);
            self.console_target = Some(tc);
        }
        // The map's contacts, 5 times a second, and new log lines as events.
        self.console_contacts_clock += get_frame_time();
        if self.console_contacts_clock >= 0.2 {
            self.console_contacts_clock = 0.0;
            server.broadcast(&protocol::contacts(&self.world, self.player));
        }
        for line in self.messages.since(self.console_messages_seen) {
            server.broadcast(&protocol::event(line));
        }
        self.console_messages_seen = self.messages.total();
        self.console_clock += get_frame_time();
        if self.console_clock >= 0.05 {
            self.console_clock = 0.0;
            server.broadcast(&protocol::telemetry(&self.world, self.player, self.world.time, self.console_hits));
        }
        ctx.remote.merge_into(raw, get_time());
    }

    /// One frame of the cockpit layout editor's input (see `hud::layout`).
    fn edit_layout(&mut self, ctx: &mut AppContext, raw: &crate::input::RawInput) {
        let Some(ed) = &mut self.editor else { return };
        let pressed = |k: &str| raw.pressed.contains(k);
        if pressed("escape") {
            self.editor = None;
            return;
        }
        let n = hud::draw::INSTRUMENTS.len();
        if pressed("tab") {
            ed.selected = if raw.shift { (ed.selected + n - 1) % n } else { (ed.selected + 1) % n };
            ed.drag = None;
        }
        let name = hud::draw::INSTRUMENTS[ed.selected].to_string();
        let screen = Vec2::new(screen_width(), screen_height());
        let (mx, my) = mouse_position();
        let mouse = Vec2::new(mx, my);
        let (_, wheel) = mouse_wheel();
        let grow = if pressed("equal") || pressed("kpadd") || wheel > 0.0 {
            1.05
        } else if pressed("minus") || pressed("kpsubtract") || wheel < 0.0 {
            1.0 / 1.05
        } else {
            1.0
        };
        if let Some(a) = self.anchors.get(&name).copied() {
            // 3D: click the dashboard to place; size with the wheel.
            if is_mouse_button_pressed(MouseButton::Left) {
                if let Some((model, offset)) = &self.cockpit_model {
                    let aspect = screen.x / screen.y;
                    let fov = cockpit_fov(self.design.x / self.design.y, aspect);
                    let dir = layout::cursor_ray(mouse, screen, fov, aspect);
                    match layout::pick(model.model.lods[0].triangles(), *offset, dir) {
                        Some((at, normal)) => {
                            self.anchors.insert(name.clone(), Anchor { at: at.into(), normal: normal.into(), ..a });
                            ed.changed = true;
                            ed.message = format!("{name} placed at {at:.3}");
                        }
                        None => ed.message = "No cockpit surface under the cursor".into(),
                    }
                }
            }
            if grow != 1.0 {
                self.anchors.insert(name.clone(), layout::scale_anchor(&a, grow));
                ed.changed = true;
            }
        } else if let Some(r) = self.instruments.get(&name).copied() {
            // 2D: drag to move, drag the corner to resize.
            let l = Layout::new(screen, self.design);
            let design_point = (mouse - l.offset) / l.scale;
            if is_mouse_button_pressed(MouseButton::Left) {
                ed.drag = layout::grip(r, design_point, 8.0 / l.scale.max(0.1)).map(|g| (g, design_point));
            }
            if !is_mouse_button_down(MouseButton::Left) {
                ed.drag = None;
            }
            let mut r2 = r;
            if let Some((g, last)) = ed.drag {
                r2 = layout::drag_rect(r2, g, design_point - last, 8.0);
                ed.drag = Some((g, design_point));
            }
            let step = if raw.shift { 10.0 } else { 1.0 };
            let nudge = Vec2::new(
                (pressed("right") as i32 - pressed("left") as i32) as f32,
                (pressed("down") as i32 - pressed("up") as i32) as f32,
            ) * step;
            r2 = layout::drag_rect(r2, layout::Grip::Move, nudge, 8.0);
            if grow != 1.0 {
                let (cx, cy) = (r2[0] + r2[2] * 0.5, r2[1] + r2[3] * 0.5);
                let (w, h) = (r2[2] * grow, r2[3] * grow);
                r2 = [cx - w * 0.5, cy - h * 0.5, w, h];
            }
            if r2 != r {
                self.instruments.insert(name.clone(), r2);
                ed.changed = true;
            }
        }
        if pressed("s") {
            let ed = self.editor.as_mut().expect("checked above");
            ed.message = match save_layout(ctx, self.cockpit_id.as_deref(), &self.instruments, &self.anchors) {
                Ok(path) => format!("Saved as an addon: {}", path.display()),
                Err(e) => format!("Not saved: {e}"),
            };
        }
    }
}

/// Saves a cockpit layout as a small addon in the user's mods folder, so it loads with
/// the campaign from now on (and can be copied into a mod's own rules).
fn save_layout(
    ctx: &AppContext,
    cockpit: Option<&str>,
    instruments: &HashMap<String, [f32; 4]>,
    anchors: &HashMap<String, Anchor>,
) -> Result<std::path::PathBuf, String> {
    let cockpit = cockpit.ok_or("this ship has no cockpit definition")?;
    let campaign = ctx.content.campaign.clone().unwrap_or_else(|| "all".into());
    let dir = crate::cli::user_data_dir()
        .ok_or("no user data folder")?
        .join("mods")
        .join(format!("{campaign}-cockpit-{cockpit}"));
    std::fs::create_dir_all(dir.join("rules")).map_err(|e| e.to_string())?;
    let pack = format!(
        "# yaml-language-server: $schema=../../schemas/pack.schema.json\n\
         # Written by the cockpit layout editor. Copy rules/cockpit-layout.yaml into your own\n\
         # mod to ship the layout with it.\n\
         id: {campaign}-cockpit-{cockpit}\nname: Cockpit layout ({cockpit})\nkind: addon\ntargets: [{campaign}]\nformat: 2\n"
    );
    std::fs::write(dir.join("pack.yaml"), pack).map_err(|e| e.to_string())?;
    let path = dir.join("rules").join("cockpit-layout.yaml");
    std::fs::write(&path, layout::layout_yaml(cockpit, instruments, anchors)).map_err(|e| e.to_string())?;
    Ok(path)
}

/// Draws each anchored instrument into its own texture, then maps the texture onto the
/// instrument's rectangle on the cockpit's surface, in perspective. The rectangle is
/// split into a grid so the texture follows the perspective closely (a single quad of
/// two triangles would bend straight lines at the diagonal).
fn draw_anchored(
    frame: &HudFrame,
    anchors: &HashMap<String, Anchor>,
    targets: &mut HashMap<String, RenderTarget>,
    cam: &SpaceCamera,
    offset: Vec3,
    screen: Vec2,
) {
    let Some(me) = frame.world.get(frame.player) else { return };
    // Texture pixels per screen pixel: drawing at twice the on-screen size keeps lines
    // and text crisp after the texture is mapped (and filtered) onto the dashboard.
    const DENSITY: f32 = 2.0;
    for (name, a) in anchors {
        let corners = layout::anchor_corners(a, offset);
        let on_screen = |p: Vec3| cam.project(cam.position + cam.rotation * p, screen);
        let (Some(tl), Some(tr), Some(bl)) = (on_screen(corners[0]), on_screen(corners[1]), on_screen(corners[3]))
        else {
            continue;
        };
        let w = ((tr - tl).length() * DENSITY).clamp(32.0, 2048.0).round();
        let h = ((bl - tl).length() * DENSITY).clamp(16.0, 2048.0).round();
        // Recreate the texture only when its size changes noticeably (window resizes).
        let stale = targets.get(name).is_none_or(|rt| {
            let (tw, th) = (rt.texture.width(), rt.texture.height());
            (tw - w).abs() > tw * 0.2 || (th - h).abs() > th * 0.2
        });
        if stale {
            let rt = render_target(w as u32, h as u32);
            rt.texture.set_filter(FilterMode::Linear);
            targets.insert(name.clone(), rt);
        }
        let rt = targets[name].clone();
        let (w, h) = (rt.texture.width(), rt.texture.height());
        let mut camera = Camera2D::from_display_rect(Rect::new(0.0, 0.0, w, h));
        camera.render_target = Some(rt);
        set_camera(&camera);
        clear_background(Color::new(0.0, 0.0, 0.0, 0.0));
        let inner = HudFrame { layout: Layout::new(Vec2::new(w, h), Vec2::new(w, h)), text_scale: DENSITY, ..*frame };
        hud::draw::draw_instrument(&inner, me, name, [0.0, 0.0, w, h]);
    }
    set_default_camera();
    for (name, a) in anchors {
        let Some(rt) = targets.get(name) else { continue };
        let corners = layout::anchor_corners(a, offset);
        const GRID: u16 = 6;
        let mut vertices = Vec::new();
        for j in 0..=GRID {
            for i in 0..=GRID {
                let (u, v) = (i as f32 / GRID as f32, j as f32 / GRID as f32);
                let p = layout::anchor_point(&corners, u, v);
                let Some(sp) = cam.project(cam.position + cam.rotation * p, screen) else { return };
                // Render targets come out upside down: sample with v flipped.
                vertices.push(macroquad::models::Vertex::new(sp.x, sp.y, 0.0, u, 1.0 - v, WHITE));
            }
        }
        let mut indices = Vec::new();
        for j in 0..GRID {
            for i in 0..GRID {
                let k = j * (GRID + 1) + i;
                indices.extend([k, k + 1, k + GRID + 1, k + 1, k + GRID + 2, k + GRID + 1]);
            }
        }
        draw_mesh(&macroquad::models::Mesh { vertices, indices, texture: Some(rt.texture.clone()) });
    }
}

/// The keys help overlay: every action, grouped, with the player's current bindings,
/// laid out in as many columns as the screen needs.
fn draw_help(bindings: &crate::input::Bindings, screen: Vec2) {
    draw_rectangle(0.0, 0.0, screen.x, screen.y, Color::new(0.0, 0.02, 0.0, 0.9));
    let size = (screen.y * 0.021).clamp(13.0, 20.0);
    let line = size * 1.25;
    draw_centered("KEYS   (? or Shift+F1 or Esc to close)", size * 2.2, size * 1.4, Color::new(0.4, 1.0, 0.5, 1.0));
    let groups = crate::input::help_rows(bindings);
    let rows: usize = groups.iter().map(|(_, r)| r.len() + 2).sum();
    let top = size * 4.0;
    let per_column = (((screen.y - top - size) / line) as usize).max(8);
    let columns = rows.div_ceil(per_column).max(1);
    let width = screen.x / columns as f32;
    let (mut col, mut row) = (0usize, 0usize);
    for (group, items) in &groups {
        // Keep a group's title with at least its first two rows.
        if row > 0 && row + 3 > per_column {
            (col, row) = (col + 1, 0);
        }
        let x = col as f32 * width + size;
        draw_text(group.title(), x, top + row as f32 * line, size, Color::new(0.4, 1.0, 0.5, 1.0));
        row += 1;
        for (label, keys) in items {
            if row >= per_column {
                (col, row) = (col + 1, 0);
            }
            let x = col as f32 * width + size;
            let y = top + row as f32 * line;
            draw_text(label, x, y, size, WHITE);
            draw_text(keys, x + width * 0.48, y, size, Color::new(0.75, 0.85, 0.75, 1.0));
            row += 1;
        }
        row += 1;
    }
}

/// The debrief panel over the frozen battle.
fn draw_debrief(d: &crate::mission::Debrief, screen: Vec2) {
    let (w, h) = ((screen.x * 0.6).min(760.0), (screen.y * 0.6).min(520.0));
    let (x, y) = ((screen.x - w) * 0.5, (screen.y - h) * 0.5);
    draw_rectangle(x, y, w, h, Color::new(0.0, 0.04, 0.0, 0.88));
    draw_rectangle_lines(x, y, w, h, 2.0, Color::new(0.35, 1.0, 0.45, 0.8));
    let (title, color) = match d.result.as_str() {
        "win" => ("MISSION ACCOMPLISHED", Color::new(0.4, 1.0, 0.5, 1.0)),
        "lose" => ("MISSION FAILED", Color::new(1.0, 0.35, 0.3, 1.0)),
        "killed" => ("YOUR CRAFT WAS DESTROYED", Color::new(1.0, 0.35, 0.3, 1.0)),
        "draw" => ("MISSION ENDED", YELLOW),
        _ => ("LEFT THE BATTLE", YELLOW),
    };
    draw_centered(title, y + 48.0, 34.0, color);
    let mut ly = y + 90.0;
    let green = Color::new(0.7, 0.95, 0.75, 1.0);
    for line in wrap_text(&d.text, (w / 11.0) as usize) {
        draw_text(&line, x + 24.0, ly, 20.0, green);
        ly += 24.0;
    }
    ly += 10.0;
    let mins = (d.seconds / 60.0) as u32;
    let stats = format!("Kills {}   Losses {}   Time {}:{:02}", d.kills, d.losses, mins, d.seconds as u32 % 60);
    draw_text(&stats, x + 24.0, ly, 20.0, WHITE);
    ly += 30.0;
    for (k, met) in &d.optional {
        let mark = if *met { "[x]" } else { "[ ]" };
        draw_text(&format!("{mark} {}", k.replace('_', " ")), x + 24.0, ly, 20.0, if *met { green } else { GRAY });
        ly += 24.0;
    }
    if !d.campaign.is_empty() {
        ly += 6.0;
        draw_text("Campaign effects (applied by the strategy layer):", x + 24.0, ly, 18.0, GRAY);
        ly += 22.0;
        for c in &d.campaign {
            draw_text(&format!("  {c}"), x + 24.0, ly, 18.0, LIGHTGRAY);
            ly += 20.0;
        }
    }
    draw_centered("Enter: fly again     L / Esc: launcher", y + h - 20.0, 20.0, YELLOW);
}

/// Splits text into lines of at most `width` characters at spaces.
fn wrap_text(text: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut line = String::new();
    for word in text.split_whitespace() {
        if !line.is_empty() && line.len() + 1 + word.len() > width.max(10) {
            lines.push(std::mem::take(&mut line));
        }
        if !line.is_empty() {
            line.push(' ');
        }
        line.push_str(word);
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines
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
    fn cockpit_fov_widens_only_on_narrow_screens() {
        let base = 60f32.to_radians();
        assert_eq!(cockpit_fov(16.0 / 9.0, 16.0 / 9.0), base);
        assert_eq!(cockpit_fov(16.0 / 9.0, 21.0 / 9.0), base, "wider screens keep 60°");
        let f = cockpit_fov(16.0 / 9.0, 4.0 / 3.0);
        // The horizontal half-angle must match the design's at 60° vertical.
        let horizontal = |fov: f32, a: f32| ((fov * 0.5).tan() * a).atan();
        assert!((horizontal(f, 4.0 / 3.0) - horizontal(base, 16.0 / 9.0)).abs() < 1e-5);
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
    fn the_debrief_waits_for_the_delay_after_the_result() {
        assert!(!debrief_due(Some(100.0), 110.0, 30.0), "still watching");
        assert!(debrief_due(Some(100.0), 130.0, 30.0));
        assert!(debrief_due(Some(100.0), 100.0, 0.0), "no delay: at once");
        assert!(debrief_due(None, 0.0, 30.0));
    }

    #[test]
    fn campaign_stances_decide_who_fights_whom() {
        let mut w = crate::sim::test_support::world();
        let f = w.rules.ship_id("fighter").unwrap();
        let me = w.spawn(f, FactionId(0), "me", Vec3::ZERO, glam::Quat::IDENTITY, crate::sim::Pilot::External);
        let (a, b) = (w.rules.factions[0].id.clone(), w.rules.factions[1].id.clone());
        assert!(w.rules.hostile(FactionId(0), FactionId(1)), "the test rules start hostile");
        apply_stances(&mut w, me, &[(b.clone(), false)]);
        assert!(!w.rules.hostile(FactionId(0), FactionId(1)), "made friends");
        apply_stances(&mut w, me, &[(b, true), (a, false)]);
        assert!(w.rules.hostile(FactionId(0), FactionId(1)), "hostile again; its own faction is left alone");
    }

    #[test]
    fn a_campaign_loadout_refits_the_players_class_in_this_battle_only() {
        let mut w = crate::sim::test_support::world();
        let f = w.rules.ship_id("fighter").unwrap();
        let me = w.spawn(f, FactionId(0), "me", Vec3::ZERO, glam::Quat::IDENTITY, crate::sim::Pilot::External);
        let shared = w.rules.clone();
        let beam = w.rules.beams[0].id.clone();
        let missile = w.rules.missiles[0].id.clone();
        apply_loadout(&mut w, me, &[beam.clone(), beam.clone(), beam], &[(missile, 7), ("nope".into(), 3)]);
        assert_eq!(w.rules.ship(f).beams.len(), 3, "three beam mounts now");
        assert_eq!(w.get(me).unwrap().beam_cooldown.len(), 3);
        assert_eq!(w.get(me).unwrap().missiles, vec![(crate::content::rules::MissileId(0), 7)], "unknown ids skipped");
        assert_eq!(shared.ship(f).beams.len(), 2, "the shared rules are untouched");
        // An empty loadout keeps the class's own weapons.
        let mut w2 = crate::sim::test_support::world();
        let me2 = w2.spawn(f, FactionId(0), "me", Vec3::ZERO, glam::Quat::IDENTITY, crate::sim::Pilot::External);
        apply_loadout(&mut w2, me2, &[], &[]);
        assert_eq!(w2.rules.ship(f).beams.len(), 2);
    }

    #[test]
    fn a_wingman_answers_in_its_own_voice() {
        use crate::content::rules::{FactionId, VoiceSet};
        let mut w = crate::sim::test_support::world();
        let mut rules = (*w.rules).clone();
        rules.voices = vec![VoiceSet {
            id: "v".into(),
            lines: vec![("acknowledge".into(), vec![4, 5]), ("down".into(), vec![9])],
        }];
        w.rules = std::sync::Arc::new(rules);
        let f = w.rules.ship_id("fighter").unwrap();
        let me = w.spawn(f, FactionId(0), "me", glam::Vec3::ZERO, glam::Quat::IDENTITY, crate::sim::Pilot::External);
        assert_eq!(wingman_voice(&w, me, None, "acknowledge", 0), None, "no wingmen yet");
        let ai = crate::sim::ai::AiState::new(crate::sim::ai::Behaviour::Wingman, Some(me), glam::Vec3::X * 60.0);
        let wing =
            w.spawn(f, FactionId(0), "wing", glam::Vec3::X * 60.0, glam::Quat::IDENTITY, crate::sim::Pilot::Ai(ai));
        assert_eq!(wingman_voice(&w, me, None, "acknowledge", 0), None, "a wingman without a voice says nothing");
        w.get_mut(wing).unwrap().voice = Some(0);
        assert_eq!(wingman_voice(&w, me, None, "acknowledge", 1), Some(5));
        assert_eq!(wingman_voice(&w, me, None, "status", 0), None, "a line the voice lacks");
        assert_eq!(wingman_voice(&w, me, Some(wing), "down", 0), None, "the lost wingman is not the speaker");
    }

    #[test]
    fn commands_target_and_order_wingmen() {
        let mut w = world();
        let f = w.rules.ship_id("fighter").unwrap();
        let r = w.rules.ship_id("raider").unwrap();
        let me = w.spawn(f, FactionId(0), "me", Vec3::ZERO, Quat::IDENTITY, Pilot::External);
        let foe = w.spawn(r, FactionId(1), "foe", Vec3::new(0.0, 0.0, -500.0), Quat::IDENTITY, Pilot::External);
        assert_eq!(apply_command(&mut w, me, Command::NearestEnemy), vec!["TARGET: RAIDER"]);
        assert_eq!(w.get(me).unwrap().target, Some(foe));
        assert_eq!(apply_command(&mut w, me, Command::WingFireAtWill), vec!["NO WINGMEN"]);
        assert_eq!(apply_command(&mut w, me, Command::WingStatus), vec!["NO WINGMEN"]);
        w.spawn(
            f,
            FactionId(0),
            "wing",
            Vec3::X * 50.0,
            Quat::IDENTITY,
            Pilot::Ai(ai::AiState::new(ai::Behaviour::Wingman, Some(me), Vec3::X * 50.0)),
        );
        assert_eq!(apply_command(&mut w, me, Command::WingAttackMyTarget), vec!["WINGMEN: ATTACK MY TARGET"]);
        assert_eq!(apply_command(&mut w, me, Command::WingFormation), vec!["WINGMEN: IGNORE MY TARGET"]);
        assert_eq!(apply_command(&mut w, me, Command::WingEvasive), vec!["WINGMEN: EVASIVE!"]);
        assert_eq!(apply_command(&mut w, me, Command::WingRetreat), vec!["WINGMEN: RETREAT!"]);
        assert_eq!(apply_command(&mut w, me, Command::WingStatus).len(), 1, "one line per wingman");
        assert!(apply_command(&mut w, me, Command::Pause).is_empty(), "not a world command");
        w.get_mut(foe).unwrap().alive = false;
        w.get_mut(me).unwrap().target = None;
        assert_eq!(apply_command(&mut w, me, Command::NearestEnemy), vec!["NO TARGET"]);
        assert!(apply_command(&mut w, me, Command::NextTarget)[0].starts_with("TARGET:"));
        assert!(!apply_command(&mut w, me, Command::PrevTarget).is_empty());
        assert!(!apply_command(&mut w, me, Command::TargetInSights).is_empty());
        assert_eq!(WING_MENU[5].1, Command::WingStatus);
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
