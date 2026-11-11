//! # Running a mission
//!
//! [`MissionRun`] turns a [`MissionDef`] into a battle and referees it:
//!
//! 1. **Start** — roles that arrive at the start are placed (relative to the player's
//!    start or to another role, at a distance and bearing, spread out and facing the
//!    player) and the [`World`] is built.
//! 2. **Every tick** (after [`World::step`]) — the battle's events are counted per role
//!    (destroyed, retreated, captured); roles whose `arrive:` condition now holds come
//!    in; waves and radio chatter fire once when their condition first holds; optional
//!    objectives are latched.
//! 3. **The end** — `lose` is checked before `win`, then `draw`, then the timer; the
//!    player jumping out is a `retreat`. The result's battle actions run (messages) and
//!    the [`Debrief`] lists the rest for the strategy layer.
//!
//! ## Layering
//!
//! `mission` sits between `sim` and `app`: it reads and spawns into a `World`, but the
//! simulation knows nothing about missions, so `sim` stays testable on its own. The
//! battle conditions are evaluated through [`BattleCx`], which implements the script
//! language's [`Context`] trait for this battle.

use crate::content::mission::{Arrival, MissionDef, RoleDef};
use crate::content::scenario::{Placement, Scenario};
use crate::script::action::Scope;
use crate::script::battle::id;
use crate::script::{Context, Value};
use crate::sim::{collision::Shape, Pilot, ShipId, SimEvent, World};
use glam::{Quat, Vec3};
use std::cell::Cell;
use std::sync::Arc;

/// One role during the battle.
#[derive(Debug, Clone, Default)]
pub struct RoleRun {
    /// Every craft that has entered for this role (alive or not).
    pub members: Vec<ShipId>,
    pub arrived: bool,
    pub destroyed: u32,
    pub retreated: u32,
    pub captured: u32,
}

/// What the debrief screen shows.
#[derive(Debug, Clone, PartialEq)]
pub struct Debrief {
    /// `win`, `lose`, `draw`, `retreat` or `killed` (the player's craft destroyed).
    pub result: String,
    pub text: String,
    /// `(optional objective, achieved)`.
    pub optional: Vec<(String, bool)>,
    /// Campaign actions of the outcome, for the strategy layer (Stage 4).
    pub campaign: Vec<String>,
    /// Enemy craft destroyed, the player's side's craft lost, seconds.
    pub kills: u32,
    pub losses: u32,
    pub seconds: f32,
}

/// A mission in progress.
pub struct MissionRun {
    pub def: MissionDef,
    pub player: ShipId,
    pub roles: Vec<RoleRun>,
    waves_done: Vec<bool>,
    chatter_done: Vec<bool>,
    optional_met: Vec<bool>,
    start: f64,
    rng: u64,
    kills: u32,
    losses: u32,
    /// Set when the battle ends.
    pub debrief: Option<Debrief>,
}

/// xorshift64: the mission's own deterministic random numbers (counts, bearings).
fn next(rng: &mut u64) -> f32 {
    *rng ^= *rng << 13;
    *rng ^= *rng >> 7;
    *rng ^= *rng << 17;
    (*rng >> 40) as f32 / (1u64 << 24) as f32
}

/// The battle condition context: the world plus this mission's bookkeeping.
pub struct BattleCx<'a> {
    pub world: &'a World,
    pub run: &'a MissionRun,
    /// `random(p)` draws from a copy of the mission's generator (a `Cell`, because
    /// [`Context::query`] takes `&self`), written back after the check.
    rng: Cell<u64>,
}

impl Context for BattleCx<'_> {
    fn query(&self, q: u16, args: &[f32]) -> Value {
        let w = self.world;
        let role = |i: f32| self.run.roles.get(i as usize);
        let alive = |r: &RoleRun| r.members.iter().filter(|m| w.get(**m).is_some()).count();
        let player = w.get(self.run.player);
        let n = Value::Num;
        let b = Value::Bool;
        match q {
            id::TIME => n((w.time - self.run.start) as f32),
            id::ALIVE => b(role(args[0]).is_some_and(|r| alive(r) > 0)),
            id::COUNT => n(role(args[0]).map(|r| alive(r) as f32).unwrap_or(0.0)),
            id::DESTROYED_ALL | id::DESTROYED => b(role(args[0])
                .is_some_and(|r| r.arrived && !r.members.is_empty() && r.destroyed as usize == r.members.len())),
            id::DESTROYED_ANY => b(role(args[0]).is_some_and(|r| r.destroyed > 0)),
            id::RETREATED_ALL => b(role(args[0])
                .is_some_and(|r| r.arrived && !r.members.is_empty() && r.retreated as usize == r.members.len())),
            id::CAPTURED_ANY => b(role(args[0]).is_some_and(|r| r.captured > 0)),
            id::PLAYER_ALIVE => b(player.is_some()),
            id::PLAYER_HULL => n(player.map(|p| p.hull / w.rules.ship(p.class).hull.max(1.0)).unwrap_or(0.0)),
            id::PLAYER_RETREATED => b(w.ships[self.run.player.index as usize].departed),
            id::DIFFICULTY => n(w.rules.difficulty as f32),
            id::RANDOM => {
                let mut r = self.rng.get();
                let x = next(&mut r);
                self.rng.set(r);
                b(x < args[0])
            }
            id::HULL => n(role(args[0])
                .map(|r| {
                    r.members
                        .iter()
                        .filter_map(|m| w.get(*m))
                        .map(|s| s.hull / w.rules.ship(s.class).hull.max(1.0))
                        .fold(1.0f32, f32::min)
                })
                .unwrap_or(1.0)),
            id::NEAR => b(match (role(args[0]), player) {
                (Some(r), Some(p)) => {
                    r.members.iter().filter_map(|m| w.get(*m)).any(|s| s.position.distance(p.position) <= args[1])
                }
                _ => false,
            }),
            id::ARRIVED => b(role(args[0]).is_some_and(|r| r.arrived)),
            _ => b(false),
        }
    }
}

impl MissionRun {
    /// Builds the battle for `def`: the player (class `player_class` when the mission
    /// lets the player choose) and every role that arrives at the start.
    pub fn start(
        def: &MissionDef,
        rules: Arc<crate::content::rules::Ruleset>,
        shapes: Arc<Vec<Shape>>,
        player_class: Option<crate::content::rules::ShipClassId>,
    ) -> Result<(World, MissionRun), String> {
        let mut run = MissionRun {
            def: def.clone(),
            player: ShipId { index: 0, generation: 0 },
            roles: vec![RoleRun::default(); def.roles.len()],
            waves_done: vec![false; def.waves.len()],
            chatter_done: vec![false; def.chatter.len()],
            optional_met: vec![false; def.optional.len()],
            start: 0.0,
            rng: def.starfield_seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1,
            kills: 0,
            losses: 0,
            debrief: None,
        };
        // An empty scenario carrying the player and the battle's look; roles go in
        // through `spawn_role`, the same path waves use.
        let scenario = Scenario {
            id: def.id.clone(),
            title: def.name.clone(),
            description: def.briefing.clone(),
            player_class: def.player_class.clone(),
            player_position: def.player_position,
            player_rotation: Vec3::ZERO,
            sun_direction: def.sun,
            starfield_seed: def.starfield_seed,
            music: None,
            objects: Vec::new(),
            nav_points: Vec::new(),
            pack: def.pack,
            order: def.order,
        };
        let (mut w, player) = World::from_scenario(&scenario, rules, shapes, player_class)?;
        run.player = player;
        run.start = w.time;
        for i in 0..def.roles.len() {
            if def.roles[i].arrive.is_none() {
                run.spawn_role(&mut w, i, None, None)?;
            }
        }
        Ok((w, run))
    }

    /// Where a role's craft enter: the reference point (the player's start, or another
    /// role's survivors' centre), then the bearing and distance.
    fn arrival_point(&mut self, w: &World, at: &Arrival) -> Vec3 {
        let origin = match at.from {
            Some(r) => {
                let alive: Vec<Vec3> =
                    self.roles[r].members.iter().filter_map(|m| w.get(*m)).map(|s| s.position).collect();
                if alive.is_empty() {
                    self.def.player_position
                } else {
                    alive.iter().copied().sum::<Vec3>() / alive.len() as f32
                }
            }
            None => self.def.player_position,
        };
        let [yaw, pitch] = match at.bearing {
            Some(b) => b,
            None => [next(&mut self.rng) * 360.0 - 180.0, next(&mut self.rng) * 40.0 - 20.0],
        };
        // Bearing (0, 0) is straight ahead of the player's start: −Z.
        let dir = Quat::from_euler(glam::EulerRot::YXZ, yaw.to_radians(), pitch.to_radians(), 0.0) * Vec3::NEG_Z;
        origin + dir * at.distance
    }

    /// Spawns `count` (or the role's own count) members of role `i` at `at` (or the
    /// role's own arrival). Returns how many entered.
    pub fn spawn_role(
        &mut self,
        w: &mut World,
        i: usize,
        count: Option<(u32, u32)>,
        at: Option<&Arrival>,
    ) -> Result<usize, String> {
        let role: RoleDef = self.def.roles[i].clone();
        let (lo, hi) = count.unwrap_or(role.count);
        let n = lo + ((next(&mut self.rng) * (hi - lo + 1) as f32) as u32).min(hi - lo);
        let at = at.cloned().unwrap_or_else(|| role.at.clone());
        let centre = self.arrival_point(w, &at);
        // Face the player's start (or, if placed on it, straight ahead).
        let look = (self.def.player_position - centre).normalize_or(Vec3::NEG_Z);
        let yaw = (-look.x).atan2(-look.z).to_degrees();
        let pitch = look.y.clamp(-1.0, 1.0).asin().to_degrees();
        // Members side by side across the line of sight.
        let side = Quat::from_euler(glam::EulerRot::YXZ, yaw.to_radians(), 0.0, 0.0) * Vec3::X;
        for k in 0..n {
            let offset = (k as f32 - (n as f32 - 1.0) * 0.5) * at.spread;
            let class = role.classes
                [((next(&mut self.rng) * role.classes.len() as f32) as usize).min(role.classes.len() - 1)]
            .clone();
            let p = Placement {
                name: format!("{}.{}", role.name, self.roles[i].members.len() + 1),
                class,
                faction: role.faction.clone(),
                position: centre + side * offset,
                rotation: Vec3::new(yaw, pitch, 0.0),
                ai: role.ai,
                leader: None,
                offset: Vec3::new(offset, 0.0, 40.0),
                speed: role.speed,
                destroyed_systems: role.destroyed_systems.clone(),
                hull: role.hull,
                protect: role.protect,
                state: role.state.clone(),
                behaviour: role.behaviour.clone(),
            };
            let id = w.spawn_placement(&p, self.player)?;
            // Wingmen fly with the player.
            if role.ai == crate::content::scenario::AiRole::Wingman {
                if let Some(crate::sim::Ship { pilot: Pilot::Ai(state), .. }) = w.get_mut(id) {
                    state.leader = Some(self.player);
                }
            }
            self.roles[i].members.push(id);
        }
        self.roles[i].arrived = true;
        Ok(n as usize)
    }

    /// Referees one tick (call after `World::step`). Returns lines for the pilot's
    /// message display (chatter and battle actions).
    pub fn update(&mut self, w: &mut World) -> Vec<String> {
        let mut lines = Vec::new();
        if self.debrief.is_some() {
            return lines;
        }
        let me = w.ships[self.player.index as usize].faction;
        // 1. Losses. Captures come from this tick's events; destroyed and retreated
        //    are read from each member's state, so damage dealt outside a tick counts too.
        for e in &w.events {
            if let SimEvent::Captured { prize, .. } = e {
                if let Some(r) = self.roles.iter_mut().find(|r| r.members.contains(prize)) {
                    r.captured += 1;
                }
            }
        }
        let (mut kills, mut losses) = (0, 0);
        for r in &mut self.roles {
            let (mut destroyed, mut departed) = (0u32, 0u32);
            for m in &r.members {
                let s = &w.ships[m.index as usize];
                // A slot's generation changes when it is reused: then the member is long gone.
                let gone = w.get(*m).is_none();
                if !gone {
                    continue;
                }
                if w.same_slot(*m) && s.departed {
                    departed += 1;
                } else {
                    destroyed += 1;
                    if w.rules.hostile(me, s.faction) {
                        kills += 1;
                    } else if s.faction == me {
                        losses += 1;
                    }
                }
            }
            r.destroyed = destroyed;
            r.retreated = departed.saturating_sub(r.captured);
        }
        (self.kills, self.losses) = (kills, losses);
        // 2. Arrivals, waves, chatter and optional objectives.
        let check = |run: &mut MissionRun, w: &World, e: &crate::script::Expr| {
            let cx = BattleCx { world: w, run, rng: Cell::new(run.rng) };
            let ok = e.test(&cx);
            let r = cx.rng.get();
            run.rng = r;
            ok
        };
        for i in 0..self.def.roles.len() {
            let arrive = self.def.roles[i].arrive.clone();
            if let Some(cond) = arrive.filter(|_| !self.roles[i].arrived) {
                if check(self, w, &cond) {
                    if let Err(e) = self.spawn_role(w, i, None, None) {
                        lines.push(format!("MISSION ERROR: {e}"));
                    }
                }
            }
        }
        for k in 0..self.def.waves.len() {
            if self.waves_done[k] {
                continue;
            }
            let wave = self.def.waves[k].clone();
            if check(self, w, &wave.when) {
                self.waves_done[k] = true;
                if let Err(e) = self.spawn_role(w, wave.role, wave.count, wave.at.as_ref()) {
                    lines.push(format!("MISSION ERROR: {e}"));
                }
            }
        }
        for k in 0..self.def.chatter.len() {
            let c = self.def.chatter[k].clone();
            if !self.chatter_done[k] && check(self, w, &c.when) {
                self.chatter_done[k] = true;
                lines.push(c.line);
            }
        }
        for k in 0..self.def.optional.len() {
            let cond = self.def.optional[k].1.clone();
            if !self.optional_met[k] && check(self, w, &cond) {
                self.optional_met[k] = true;
            }
        }
        // 3. The end.
        let time = (w.time - self.start) as f32;
        let me = &w.ships[self.player.index as usize];
        let player_gone = me.departed;
        // `same_slot` guards against a reused slot: a destroyed player's slot may hold a
        // new ship later, and that ship's state says nothing about the player.
        let player_killed = !w.same_slot(self.player) || !me.alive;
        let result = if player_gone {
            Some("retreat")
        } else if player_killed {
            // Checked before the mission's own conditions, so the debrief says what
            // actually happened ("your craft was destroyed"), not the lose text, which
            // usually describes the objective failing (e.g. the base falling).
            Some("killed")
        } else if self.def.lose.clone().is_some_and(|c| check(self, w, &c)) {
            Some("lose")
        } else if self.def.win.clone().is_some_and(|c| check(self, w, &c)) {
            Some("win")
        } else if self.def.draw.clone().is_some_and(|c| check(self, w, &c)) {
            Some("draw")
        } else {
            self.def.timer.as_ref().filter(|(after, _)| time >= *after).map(|(_, then)| then.as_str())
        };
        if let Some(result) = result {
            let result = result.to_string();
            let mut campaign = Vec::new();
            // A mission without its own `killed` outcomes counts it as a loss.
            let has_own = self.def.outcomes.iter().any(|(r, _)| r == "killed");
            let key = if result == "killed" && !has_own { "lose" } else { result.as_str() };
            for a in self.def.outcome(key) {
                match a.scope {
                    Scope::Battle => lines.extend(a.args.first().cloned()),
                    Scope::Campaign => campaign.push(a.display()),
                }
            }
            let text = self.def.debrief.iter().find(|(r, _)| *r == result).map(|(_, t)| t.clone()).unwrap_or_default();
            lines.push(match result.as_str() {
                "win" => "MISSION ACCOMPLISHED".into(),
                "lose" => "MISSION FAILED".into(),
                "draw" => "MISSION ENDED".into(),
                "killed" => "YOUR CRAFT WAS DESTROYED".into(),
                _ => "LEFT THE BATTLE".into(),
            });
            self.debrief = Some(Debrief {
                result,
                text,
                optional: self.def.optional.iter().zip(&self.optional_met).map(|((k, _), m)| (k.clone(), *m)).collect(),
                campaign,
                kills: self.kills,
                losses: self.losses,
                seconds: time,
            });
        }
        lines
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::mission::MissionDef;
    use crate::content::schema::Get;
    use crate::formats::yaml;
    use crate::sim::test_support::{rules, shapes};

    fn mission(text: &str) -> (World, MissionRun) {
        let r = rules();
        let classes: Vec<String> = r.ships.iter().map(|c| c.id.clone()).collect();
        let doc = yaml::parse(text).unwrap();
        let def = MissionDef::parse("t", Get(&doc), 0, &classes, &r.difficulty_levels).unwrap();
        let s = shapes(&r);
        MissionRun::start(&def, r, s, None).unwrap()
    }

    /// Destroys a ship outright (shields and armour first, so the hit lands on the hull).
    fn kill(w: &mut World, id: ShipId) {
        let s = w.get_mut(id).unwrap();
        (s.shield_fore, s.shield_aft, s.armor) = (0.0, 0.0, 0.0);
        crate::sim::combat::apply_damage(w, id, 1e9, Vec3::ZERO);
    }

    fn run(w: &mut World, m: &mut MissionRun, secs: f32) -> Vec<String> {
        let mut out = Vec::new();
        for _ in 0..(secs / crate::sim::TICK) as usize {
            w.step();
            out.extend(m.update(w));
            if m.debrief.is_some() {
                break;
            }
        }
        out
    }

    #[test]
    fn roles_are_placed_counted_and_faced_at_the_player() {
        let (w, m) = mission(
            "player: {class: fighter}\nroles:\n  foes: {class: raider, count: 3, ai: idle, at: {distance: 2km, bearing: [90, 0], spread: 100m}}\n  later: {class: raider, arrive: time > 1s}\nobjectives: {win: destroyed_all(foes)}\n",
        );
        assert_eq!(m.roles[0].members.len(), 3);
        assert!(!m.roles[1].arrived, "arrives later");
        let s = w.get(m.roles[0].members[1]).unwrap();
        // Bearing 90° (yaw left): the middle member is 2 km along −X.
        assert!((s.position - Vec3::new(-2000.0, 0.0, 0.0)).length() < 1.0, "{:?}", s.position);
        assert!(s.forward().dot(Vec3::X) > 0.99, "facing the player");
    }

    #[test]
    fn arrivals_waves_chatter_and_a_win() {
        let (mut w, mut m) = mission(
            "player: {class: fighter}\nroles:\n  foes: {class: raider, count: 1, ai: idle, at: {distance: 3km}}\n  late: {class: raider, ai: idle, arrive: time > 0.5s, at: {distance: 4km}}\n\
             waves:\n  - {role: foes, when: destroyed_all(foes), count: 2}\n\
             chatter:\n  - {when: arrived(late), line: Here they come}\n\
             objectives: {win: count(foes) == 0 and arrived(late) and destroyed_any(foes) and time > 2s, lose: not player_alive}\n\
             optional: {quick: time < 100s}\n\
             outcomes: {win: [message(Well flown), score(won), money(+500)]}\ndebrief: {win: Good show}\n",
        );
        let lines = run(&mut w, &mut m, 1.0);
        assert!(m.roles[1].arrived && lines.contains(&"Here they come".to_string()));
        // Destroy the first foe: the wave brings two more.
        kill(&mut w, m.roles[0].members[0]);
        run(&mut w, &mut m, 0.2);
        assert_eq!((m.roles[0].members.len(), m.roles[0].destroyed), (3, 1), "a wave of two");
        for k in 1..3 {
            kill(&mut w, m.roles[0].members[k]);
        }
        let lines = run(&mut w, &mut m, 3.0);
        let d = m.debrief.clone().expect("won");
        assert_eq!((d.result.as_str(), d.text.as_str()), ("win", "Good show"));
        assert_eq!(d.campaign, vec!["score(won)".to_string(), "money(+500)".to_string()]);
        assert_eq!(d.optional, vec![("quick".to_string(), true)]);
        assert!(lines.contains(&"Well flown".to_string()) && lines.contains(&"MISSION ACCOMPLISHED".to_string()));
        assert_eq!(d.kills, 3);
    }

    #[test]
    fn lose_beats_win_and_the_timer_ends_it() {
        let (mut w, mut m) = mission(
            "player: {class: fighter}\nroles:\n  ward: {class: fighter, ai: idle, protect: true, at: {distance: 1km}}\nobjectives: {lose: destroyed(ward), win: destroyed(ward)}\n",
        );
        kill(&mut w, m.roles[0].members[0]);
        run(&mut w, &mut m, 0.5);
        assert_eq!(m.debrief.as_ref().unwrap().result, "lose", "lose is checked first");
        let (mut w, mut m) =
            mission("player: {class: fighter}\nroles:\n  x: {class: raider, ai: idle, at: {distance: 5km}}\ntimer: {after: 1s, then: draw}\n");
        run(&mut w, &mut m, 2.0);
        assert_eq!(m.debrief.as_ref().unwrap().result, "draw");
    }

    #[test]
    fn the_players_death_is_its_own_result_with_the_lose_outcomes() {
        let text = "player: {class: fighter}\nroles:\n  ward: {class: fighter, ai: idle, protect: true, at: {distance: 1km}}\n\
                    objectives: {lose: destroyed(ward) or not player_alive}\noutcomes: {lose: [lose(ward)]}\n\
                    debrief: {lose: The ward fell}\n";
        let (mut w, mut m) = mission(text);
        let me = m.player;
        kill(&mut w, me);
        run(&mut w, &mut m, 0.5);
        let d = m.debrief.as_ref().unwrap();
        assert_eq!(d.result, "killed", "not 'lose': the ward is fine");
        assert_eq!(d.text, "", "the lose text describes the ward, so it is not shown");
        assert_eq!(d.campaign, vec!["lose(ward)".to_string()], "the lose outcomes apply");
    }
}
