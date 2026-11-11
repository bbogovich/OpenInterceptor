//! # The simulator: set up a mission, then fly it (Stage 3.9)
//!
//! A setup screen in front of any mission: choose the mission, your ship, the battle
//! environment, hazards to add and the size of the enemy force, then fly it. The choices
//! become a modified copy of the mission ([`apply`]), so the battle runs through the
//! ordinary mission code. R-MSN-010; the testbed's mission browser (issue #20).
//!
//! ```text
//!  Mission       < Defend the station >
//!  Ship          < Comet >
//!  Environment   < open space | orbit | moon_surface … >
//!  Asteroids     < off | on >        (the campaign's asteroid class)
//!  Minefield     < off | on >        (the campaign's mine class)
//!  Black hole    < off | on >
//!  Ion storm     < off | on >
//!  Enemy force   < ×0.5 | ×1 | ×1.5 | ×2 >
//!  Fly
//! ```

use super::{AppContext, Next};
use crate::content::hazards::{BlackHole, Hazards, Scatter};
use crate::content::mission::MissionDef;
use crate::content::rules::{Ruleset, ShipKind};
use crate::content::scenario::AiRole;
use glam::Vec3;
use macroquad::prelude::*;

/// Enemy force multipliers offered.
pub const FORCES: [f32; 4] = [0.5, 1.0, 1.5, 2.0];

/// The player's choices (indices into the lists the screen offers).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SimSetup {
    pub mission: usize,
    pub ship: usize,
    /// 0 = the mission's own environment; k = `rules.environments[k - 1]`.
    pub environment: usize,
    pub asteroids: bool,
    pub minefield: bool,
    pub black_hole: bool,
    pub ion_storm: bool,
    /// Index into [`FORCES`] (default ×1).
    pub force: usize,
}

/// The campaign's asteroid class (an object whose id names an asteroid) and mine class
/// (`kind: mine`), if it has them; hazards without a class are not offered.
pub fn hazard_classes(rules: &Ruleset) -> (Option<String>, Option<String>) {
    let asteroid = rules.ships.iter().find(|c| c.id.contains("asteroid")).map(|c| c.id.clone());
    let mine = rules.ships.iter().find(|c| c.kind == ShipKind::Mine).map(|c| c.id.clone());
    (asteroid, mine)
}

/// The mission as the setup asks for it: environment and hazards replaced or added, and
/// every enemy role and wave scaled by the force multiplier (at least one craft each).
pub fn apply(def: &MissionDef, setup: &SimSetup, rules: &Ruleset) -> MissionDef {
    let mut m = def.clone();
    if setup.environment > 0 {
        m.environment = rules.environments.get(setup.environment - 1).map(|e| e.id.clone());
    }
    let (asteroid, mine) = hazard_classes(rules);
    let mut h: Hazards = m.hazards.clone();
    let ahead = Vec3::new(0.0, 0.0, -4500.0);
    if let (true, Some(class)) = (setup.asteroids, asteroid) {
        h.asteroids = Some(Scatter { class, count: 40, radius: 4500.0, center: ahead, spin: 6.0, faction: None });
    }
    if let (true, Some(class)) = (setup.minefield, mine) {
        h.minefield = Some(Scatter { class, count: 20, radius: 2500.0, center: ahead, spin: 0.0, faction: None });
    }
    if setup.black_hole {
        h.black_hole = Some(BlackHole { position: Vec3::new(0.0, -800.0, -9000.0), pull: 12.0, radius: 450.0 });
    }
    if setup.ion_storm {
        h.ion_storm = Some(0.4);
    }
    m.hazards = h;
    let k = FORCES.get(setup.force).copied().unwrap_or(1.0);
    let scale =
        |(lo, hi): (u32, u32)| (((lo as f32) * k).round().max(1.0) as u32, ((hi as f32) * k).round().max(1.0) as u32);
    let enemy: Vec<bool> = m.roles.iter().map(|r| r.ai == AiRole::Enemy).collect();
    for (r, is_enemy) in m.roles.iter_mut().zip(&enemy) {
        if *is_enemy {
            r.count = scale(r.count);
        }
    }
    for w in &mut m.waves {
        if enemy.get(w.role).copied().unwrap_or(false) {
            w.count = w.count.map(scale);
        }
    }
    m
}

/// The simulator screen.
pub struct Simulator {
    pub setup: SimSetup,
    row: usize,
    /// Why the last Fly did not start (shown until the next change).
    note: Option<String>,
}

/// Whether `ship` may fly in the environment `env` (a ship with `atmospheric: false`
/// cannot enter an atmosphere). `None` when it may; otherwise the reason.
pub fn ship_allowed(rules: &Ruleset, ship: crate::content::rules::ShipClassId, env: Option<&str>) -> Option<String> {
    let e = rules.environments.iter().find(|e| Some(e.id.as_str()) == env)?;
    let c = rules.ship(ship);
    (e.physics == crate::content::environment::Physics::Atmosphere && !c.atmospheric)
        .then(|| format!("The {} cannot enter an atmosphere: choose another ship or environment.", c.name))
}

/// The rows, top to bottom.
const ROWS: [&str; 9] =
    ["Mission", "Ship", "Environment", "Asteroids", "Minefield", "Black hole", "Ion storm", "Enemy force", "Fly"];

impl Default for Simulator {
    fn default() -> Self {
        Self::new()
    }
}

impl Simulator {
    pub fn new() -> Simulator {
        Simulator { setup: SimSetup { force: 1, ..Default::default() }, row: 0, note: None }
    }

    /// One frame: input, then drawing. Returns where to go next.
    pub fn frame(&mut self, ctx: &mut AppContext) -> Next {
        super::console_idle(ctx, "simulator");
        let rules = &ctx.content.rules;
        let missions = rules.missions.len();
        let ships = rules.playable_ships();
        let pressed = |k: KeyCode| is_key_pressed(k);
        if pressed(KeyCode::Escape) {
            return Next::Launcher;
        }
        if missions == 0 {
            clear_background(BLACK);
            draw_text("This campaign has no missions to simulate. Esc: back", 40.0, 80.0, 24.0, WHITE);
            return Next::Stay;
        }
        if pressed(KeyCode::Down) {
            self.row = (self.row + 1) % ROWS.len();
        }
        if pressed(KeyCode::Up) {
            self.row = (self.row + ROWS.len() - 1) % ROWS.len();
        }
        let step = pressed(KeyCode::Right) as i32 - pressed(KeyCode::Left) as i32;
        let cycle = |v: usize, n: usize| ((v as i32 + step).rem_euclid(n.max(1) as i32)) as usize;
        let (asteroid, mine) = hazard_classes(rules);
        let s = &mut self.setup;
        if step != 0 {
            self.note = None;
            match self.row {
                0 => s.mission = cycle(s.mission, missions),
                1 => s.ship = cycle(s.ship, ships.len()),
                2 => s.environment = cycle(s.environment, rules.environments.len() + 1),
                3 if asteroid.is_some() => s.asteroids = !s.asteroids,
                4 if mine.is_some() => s.minefield = !s.minefield,
                5 => s.black_hole = !s.black_hole,
                6 => s.ion_storm = !s.ion_storm,
                7 => s.force = cycle(s.force, FORCES.len()),
                _ => {}
            }
        }
        let def = &rules.missions[s.mission.min(missions - 1)];
        if pressed(KeyCode::Enter) || pressed(KeyCode::KpEnter) {
            let m = apply(def, s, rules);
            let class = match &def.player_class {
                Some(c) => rules.ship_id(c),
                None => ships.get(s.ship).copied(),
            };
            match class.and_then(|c| ship_allowed(rules, c, m.environment.as_deref())) {
                Some(reason) => self.note = Some(reason),
                None => {
                    let ship = ships.get(s.ship).map(|c| rules.ship(*c).id.clone());
                    return Next::Simulate {
                        mission: Box::new(m),
                        ship: if def.player_class.is_some() { None } else { ship },
                    };
                }
            }
        }
        // Drawing.
        clear_background(Color::new(0.02, 0.03, 0.05, 1.0));
        draw_text("SIMULATOR", 40.0, 60.0, 40.0, WHITE);
        draw_text("Up/Down choose   Left/Right change   Enter fly   Esc back", 40.0, 92.0, 18.0, GRAY);
        let onoff = |b: bool| if b { "on" } else { "off" }.to_string();
        let values = [
            def.name.clone(),
            if def.player_class.is_some() {
                "(the mission's own)".into()
            } else {
                ships.get(s.ship).map(|c| rules.ship(*c).name.clone()).unwrap_or_default()
            },
            match s.environment {
                0 => def.environment.clone().map(|e| format!("{e} (the mission's)")).unwrap_or("open space".into()),
                k => rules.environments[k - 1].id.clone(),
            },
            if asteroid.is_some() { onoff(s.asteroids) } else { "(none in this campaign)".into() },
            if mine.is_some() { onoff(s.minefield) } else { "(none in this campaign)".into() },
            onoff(s.black_hole),
            onoff(s.ion_storm),
            format!("x{}", FORCES[s.force]),
            String::new(),
        ];
        for (i, (label, value)) in ROWS.iter().zip(values.iter()).enumerate() {
            let y = 150.0 + i as f32 * 36.0;
            let on = i == self.row;
            let c = if on { Color::new(1.0, 0.85, 0.4, 1.0) } else { LIGHTGRAY };
            draw_text(if on { ">" } else { " " }, 40.0, y, 26.0, c);
            draw_text(label, 70.0, y, 26.0, c);
            if !value.is_empty() {
                draw_text(&format!("<  {value}  >"), 300.0, y, 26.0, if on { WHITE } else { GRAY });
            }
        }
        let y = 150.0 + ROWS.len() as f32 * 36.0 + 20.0;
        if let Some(n) = &self.note {
            draw_text(n, 40.0, y - 8.0, 20.0, Color::new(1.0, 0.5, 0.4, 1.0));
        }
        let y = y + 20.0;
        for (i, line) in wrap(&def.briefing, 90).iter().take(6).enumerate() {
            draw_text(line, 40.0, y + i as f32 * 22.0, 18.0, Color::new(0.7, 0.9, 0.7, 1.0));
        }
        Next::Stay
    }
}

/// Splits `text` into lines of at most `width` characters at spaces.
fn wrap(text: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut line = String::new();
    for word in text.split_whitespace() {
        if !line.is_empty() && line.len() + word.len() + 1 > width {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::ContentConfig;

    fn default_rules() -> Ruleset {
        crate::content::Content::load(&ContentConfig {
            pack_dirs: crate::content::checkout_dirs(),
            no_original: true,
            ..Default::default()
        })
        .unwrap()
        .rules
    }

    #[test]
    fn ships_that_cannot_enter_an_atmosphere_are_refused_there() {
        let mut rules = default_rules();
        let comet = rules.ship_id("comet").unwrap();
        assert_eq!(ship_allowed(&rules, comet, Some("moon_surface")), None);
        rules.ships[comet.0 as usize].atmospheric = false;
        assert!(ship_allowed(&rules, comet, Some("moon_surface")).unwrap().contains("cannot enter an atmosphere"));
        assert_eq!(ship_allowed(&rules, comet, Some("orbit")), None, "space is fine");
        assert_eq!(ship_allowed(&rules, comet, None), None);
    }

    #[test]
    fn a_setup_changes_environment_hazards_and_enemy_force_only() {
        let rules = default_rules();
        let def = &rules.missions[0];
        let same = apply(def, &SimSetup { force: 1, ..Default::default() }, &rules);
        assert_eq!(&same, def, "the defaults change nothing");
        let setup = SimSetup {
            environment: 1 + rules.environments.iter().position(|e| e.id == "moon_surface").unwrap(),
            asteroids: true,
            minefield: true,
            black_hole: true,
            ion_storm: true,
            force: 3, // ×2
            ..Default::default()
        };
        let m = apply(def, &setup, &rules);
        assert_eq!(m.environment.as_deref(), Some("moon_surface"));
        assert_eq!(m.hazards.asteroids.as_ref().map(|a| a.class.as_str()), Some("cosmium_asteroid"));
        assert_eq!(m.hazards.minefield.as_ref().map(|a| a.class.as_str()), Some("space_mine"));
        assert!(m.hazards.black_hole.is_some() && m.hazards.ion_storm == Some(0.4));
        for (a, b) in m.roles.iter().zip(&def.roles) {
            if a.ai == AiRole::Enemy {
                assert_eq!(a.count, (b.count.0 * 2, b.count.1 * 2), "enemies doubled");
            } else {
                assert_eq!(a.count, b.count, "friends and objectives unchanged");
            }
        }
        let half = apply(def, &SimSetup { force: 0, ..Default::default() }, &rules);
        assert!(half.roles.iter().all(|r| r.count.0 >= 1), "never fewer than one");
    }
}
