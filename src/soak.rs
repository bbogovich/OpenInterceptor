//! # Headless battle runs (`--soak`)
//!
//! Balancing a fight by playing it again and again is slow and subjective. A soak run
//! plays a scenario without a window, as fast as the computer allows, and reports what
//! happened to the player's ship. Two pilots stand in for the player:
//!
//! - `passive` — flies at the nearest enemy at 60 % throttle and never shoots: how long
//!   does an undefended rocket last against these enemies at this difficulty?
//! - `ai` — flown by the ship class's own behaviour, like a wingman: roughly, can an
//!   average pilot in this ship win?
//!
//! ```text
//! openinterceptor --campaign default --soak rocket-vs-saucer --ship comet --difficulty cadet
//! soak rocket-vs-saucer (comet, cadet, passive): destroyed at 41.2 s; 214 hits taken; 0/3 enemies destroyed
//! ```

use crate::content::Content;
use crate::sim::{ai, collision, Pilot, SimEvent, World, TICK};
use std::sync::Arc;

/// Who flies the player's ship during a soak run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoakPilot {
    Passive,
    Ai,
}

/// What a soak run found.
#[derive(Debug, Clone, PartialEq)]
pub struct SoakReport {
    /// Seconds until the player was destroyed, if it was.
    pub destroyed_at: Option<f32>,
    pub seconds: f32,
    pub hits_taken: usize,
    /// Hostile ships destroyed and present at the start.
    pub kills: usize,
    pub enemies: usize,
    /// The player's hull fraction at the end.
    pub hull: f32,
}

impl SoakReport {
    pub fn summary(&self) -> String {
        let fate = match self.destroyed_at {
            Some(t) => format!("destroyed at {t:.1} s"),
            None => format!("survived {:.0} s with {:.0}% hull", self.seconds, self.hull * 100.0),
        };
        format!("{fate}; {} hits taken; {}/{} enemies destroyed", self.hits_taken, self.kills, self.enemies)
    }
}

/// Collision shapes for every ship class, from the models (as the game builds them).
pub fn collision_shapes(content: &Content) -> Vec<collision::Shape> {
    content
        .rules
        .ships
        .iter()
        .map(|class| match content.load_model(&class.model, class.model_pack, class.model_scale) {
            Ok(m) => collision::Shape::from_model(&m, class.kind.is_static()),
            Err(_) => collision::Shape::sphere(10.0),
        })
        .collect()
}

/// Plays mission `id` for up to `seconds` with `pilot` in the player's seat, and returns
/// the report and the mission's result (`None` if it was still running).
pub fn run_mission(
    content: &Content,
    id: &str,
    ship: Option<&str>,
    seconds: f32,
    pilot: SoakPilot,
) -> Result<(SoakReport, Option<String>), String> {
    let def = content.rules.missions.iter().find(|m| m.id == id).ok_or(format!("no mission '{id}'"))?;
    let rules = Arc::new(content.rules.clone());
    let choice = match ship {
        Some(s) => Some(rules.ship_id(s).ok_or(format!("no ship class '{s}'"))?),
        None => None,
    };
    let (mut w, mut run) =
        crate::mission::MissionRun::start(def, rules.clone(), Arc::new(collision_shapes(content)), choice)?;
    let player = run.player;
    if pilot == SoakPilot::Ai {
        if let Some(s) = w.get_mut(player) {
            s.pilot = Pilot::Ai(ai::AiState::new(ai::Behaviour::Enemy, None, glam::Vec3::ZERO));
        }
    }
    let ticks = (seconds / TICK) as usize;
    for _ in 0..ticks {
        if pilot == SoakPilot::Passive {
            passive_controls(&mut w, player);
        }
        w.step();
        run.update(&mut w);
        if run.debrief.is_some() {
            break;
        }
    }
    let d = run.debrief.clone();
    let s = &w.ships[player.index as usize];
    let report = SoakReport {
        destroyed_at: (!s.alive && !s.departed).then_some((w.time - 0.0) as f32),
        seconds: d.as_ref().map(|d| d.seconds).unwrap_or(seconds),
        hits_taken: 0,
        kills: d.as_ref().map(|d| d.kills as usize).unwrap_or(0),
        enemies: 0,
        hull: (s.hull / rules.ship(s.class).hull.max(1.0)).clamp(0.0, 1.0),
    };
    Ok((report, d.map(|d| d.result)))
}

/// The passive pilot: 60 % throttle, nose on the nearest enemy, never shooting.
fn passive_controls(w: &mut World, player: crate::sim::ShipId) {
    let aim = crate::sim::targeting::nearest_hostile(w, player).map(|t| w.ships[t.index as usize].position);
    if let Some(s) = w.get_mut(player) {
        let mut c = crate::sim::Controls { throttle_set: Some(0.6), ..Default::default() };
        if let Some(p) = aim {
            let (pitch, yaw, roll, _) = ai::steer_towards(s, p);
            (c.pitch, c.yaw, c.roll) = (pitch, yaw, roll);
        }
        s.controls = c;
    }
}

/// Plays `scenario` for up to `seconds` with `pilot` in the player's seat.
pub fn run(
    content: &Content,
    scenario: &str,
    ship: Option<&str>,
    seconds: f32,
    pilot: SoakPilot,
) -> Result<SoakReport, String> {
    let sc = content.scenario(scenario).ok_or(format!("no scenario '{scenario}'"))?;
    let rules = Arc::new(content.rules.clone());
    let choice = match ship {
        Some(s) => Some(rules.ship_id(s).ok_or(format!("no ship class '{s}'"))?),
        None => None,
    };
    let (mut w, player) = World::from_scenario(sc, rules.clone(), Arc::new(collision_shapes(content)), choice)?;
    let me = w.get(player).expect("just spawned").faction;
    let enemies = w.ships.iter().filter(|s| s.alive && rules.hostile(me, s.faction)).count();
    if pilot == SoakPilot::Ai {
        if let Some(s) = w.get_mut(player) {
            s.pilot = Pilot::Ai(ai::AiState::new(ai::Behaviour::Enemy, None, glam::Vec3::ZERO));
        }
    }
    let mut report = SoakReport { destroyed_at: None, seconds, hits_taken: 0, kills: 0, enemies, hull: 1.0 };
    let ticks = (seconds / TICK) as usize;
    for k in 0..ticks {
        if pilot == SoakPilot::Passive {
            passive_controls(&mut w, player);
        }
        w.step();
        for e in &w.events {
            match e {
                SimEvent::Hit { ship, .. } if *ship == player => report.hits_taken += 1,
                SimEvent::Destroyed { ship, .. } if *ship == player => {
                    report.destroyed_at = Some((k + 1) as f32 * TICK);
                }
                SimEvent::Destroyed { ship, .. } if rules.hostile(me, w.ships[ship.index as usize].faction) => {
                    report.kills += 1
                }
                _ => {}
            }
        }
        if report.destroyed_at.is_some() {
            break;
        }
    }
    let s = &w.ships[player.index as usize];
    report.hull = (s.hull / rules.ship(s.class).hull.max(1.0)).clamp(0.0, 1.0);
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::ContentConfig;

    fn content(level: &str) -> Content {
        let packs = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("packs");
        Content::load(&ContentConfig {
            pack_dirs: vec![packs],
            no_original: true,
            difficulty: Some(level.into()),
            ..Default::default()
        })
        .unwrap()
    }

    /// The balance promise of the default campaign: at cadet, an undefended rocket is not
    /// shot down in the first minute of the Dominion raid, and the enemy is clearly
    /// deadlier at space_ace than at cadet.
    #[test]
    fn default_campaign_difficulty_scales_enemy_lethality() {
        let easy = run(&content("cadet"), "dominion-raid", Some("meteor"), 60.0, SoakPilot::Passive).unwrap();
        let hard = run(&content("space_ace"), "dominion-raid", Some("meteor"), 60.0, SoakPilot::Passive).unwrap();
        assert!(easy.destroyed_at.is_none(), "cadet: {}", easy.summary());
        assert!(
            hard.hits_taken > easy.hits_taken * 2 || hard.destroyed_at.is_some(),
            "cadet {} / ace {}",
            easy.summary(),
            hard.summary()
        );
    }
}
