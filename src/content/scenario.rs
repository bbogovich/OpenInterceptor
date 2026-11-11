//! # Scenarios
//!
//! A scenario is a playable space arena described in a pack's `scenarios/*.yaml`. The
//! testbed's live test environments are scenarios, and Stage 3 missions build on the
//! same format. Being plain data, a modder can add a new test arena without writing Rust.
//!
//! ```yaml
//! id: flight-test
//! title: "Flight test: station patrol"
//! description: Fly around an X-COM base with two wingmen…
//! player: choose                # or a ship id
//! sun: [-0.4, 0.5, -0.8]        # direction *towards* the sun
//! starfield_seed: 7
//! objects:
//!   base:
//!     class: xcom_base
//!     position: [0, 0, -3000]   # metres; -z is "ahead" of the player's start
//!     rotation: [0, 0, 0]       # yaw, pitch, roll in degrees
//!     ai: idle                  # wingman | enemy | idle | target
//!   wing1: {class: lightning, ai: wingman, leader: player, offset: [-60, 0, 30]}
//!   wreck:
//!     class: wraith
//!     destroyed_systems: [engines, manoeuvring]   # start disabled (a capture target)
//!     hull: 40%
//!     behaviour: wraith_ambush  # fly this behaviour instead of the class's
//! nav:                          # optional navigation points (engine extension)
//!   alpha: {position: [0, 0, -3000]}
//! ```

use super::schema::{Get, Unit};
use crate::formats::yaml::Node;
use glam::Vec3;

/// How an object in a scenario is controlled.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AiRole {
    /// The human player (exactly one object, or `player = choose`).
    Player,
    /// Follows a leader in formation and attacks the leader's enemies.
    Wingman,
    /// Hunts hostile ships.
    Enemy,
    /// Holds position and does nothing (stations, parked ships).
    Idle,
    /// Flies a slow circle; a practice target that does not shoot back.
    Target,
}

impl AiRole {
    fn parse(s: &str) -> Option<AiRole> {
        Some(match s.to_ascii_lowercase().as_str() {
            "player" => AiRole::Player,
            "wingman" => AiRole::Wingman,
            "enemy" => AiRole::Enemy,
            "idle" => AiRole::Idle,
            "target" => AiRole::Target,
            _ => return None,
        })
    }
}

/// One object placed in a scenario.
#[derive(Debug, Clone, PartialEq)]
pub struct Placement {
    /// The `<name>` from `[object.<name>]`.
    pub name: String,
    /// Ship class id (resolved against the ruleset when the scenario starts).
    pub class: String,
    pub faction: Option<String>,
    pub position: Vec3,
    /// Yaw, pitch, roll in degrees.
    pub rotation: Vec3,
    pub ai: AiRole,
    pub leader: Option<String>,
    pub offset: Vec3,
    /// Initial speed, m/s.
    pub speed: f32,
    /// Systems (kinds or system ids) that start destroyed.
    pub destroyed_systems: Vec<String>,
    /// Starting hull as a fraction of the class's hit points.
    pub hull: f32,
    /// A behaviour id overriding the class's (`behaviour = scout_saucer`).
    pub behaviour: Option<String>,
}

/// A navigation point.
#[derive(Debug, Clone, PartialEq)]
pub struct NavPoint {
    pub name: String,
    pub position: Vec3,
}

/// A whole scenario.
#[derive(Debug, Clone, PartialEq)]
pub struct Scenario {
    pub id: String,
    pub title: String,
    pub description: String,
    /// `None` means the player picks a ship in the launcher.
    pub player_class: Option<String>,
    /// Where the player starts.
    pub player_position: Vec3,
    pub player_rotation: Vec3,
    pub sun_direction: Vec3,
    pub starfield_seed: u64,
    /// Music asset reference.
    pub music: Option<String>,
    pub objects: Vec<Placement>,
    pub nav_points: Vec<NavPoint>,
    /// Index of the pack that defined it.
    pub pack: usize,
    /// Sort key in the launcher (lower first).
    pub order: i64,
}

fn vec3(g: Get, key: &str, default: Vec3) -> Vec3 {
    match g.nums(key) {
        Some(v) if v.len() == 3 => Vec3::new(v[0], v[1], v[2]),
        _ => default,
    }
}

impl Scenario {
    /// Parses scenario YAML text (`pack` is recorded for asset resolution).
    pub fn parse(text: &str, pack: usize) -> Result<Scenario, String> {
        let doc = crate::formats::yaml::parse(text).map_err(|e| format!("{}: {}", e.line, e.message))?;
        Scenario::from_yaml(&doc, pack)
    }

    /// Builds a scenario from a parsed file, checking it against the schema first. Errors
    /// read `<line>: <path>: <message>`.
    pub fn from_yaml(doc: &Node, pack: usize) -> Result<Scenario, String> {
        let mut problems = Vec::new();
        super::schema::validate(doc, &super::files::scenario(), "", &mut problems);
        if let Some(p) = problems.first() {
            return Err(format!("{}: {}: {}", p.line, p.path, p.message));
        }
        let g = Get(doc);
        let id = g.str("id").ok_or(format!("{}: a scenario needs an id", doc.line))?.to_string();
        let player = g.str("player").unwrap_or("choose");
        let mut objects = Vec::new();
        for (name, body) in g.map("objects") {
            let o = Get(body);
            let class = o.str("class").ok_or(format!("{}: objects.{name}: no class", body.line))?.to_string();
            let ai = match o.str("ai") {
                Some(a) => AiRole::parse(a).ok_or(format!("{}: objects.{name}: unknown ai '{a}'", body.line))?,
                None => AiRole::Idle,
            };
            objects.push(Placement {
                name: name.clone(),
                class,
                faction: o.str("faction").map(str::to_string),
                position: vec3(o, "position", Vec3::ZERO),
                rotation: vec3(o, "rotation", Vec3::ZERO),
                ai,
                leader: o.str("leader").map(str::to_string),
                offset: vec3(o, "offset", Vec3::new(-60.0, 0.0, 30.0)),
                speed: o.num("speed").unwrap_or(0.0),
                destroyed_systems: o.list("destroyed_systems").iter().map(|x| x.to_ascii_lowercase()).collect(),
                hull: o.qty("hull", Unit::Fraction).unwrap_or(1.0).clamp(0.01, 1.0),
                behaviour: o.str("behaviour").map(str::to_string),
            });
        }
        let nav_points = g
            .map("nav")
            .iter()
            .map(|(name, body)| NavPoint { name: name.clone(), position: vec3(Get(body), "position", Vec3::ZERO) })
            .collect();
        Ok(Scenario {
            title: g.str("title").unwrap_or(&id).to_string(),
            description: g.str("description").unwrap_or("").trim().to_string(),
            player_class: (!player.eq_ignore_ascii_case("choose")).then(|| player.to_string()),
            player_position: vec3(g, "player_position", Vec3::ZERO),
            player_rotation: vec3(g, "player_rotation", Vec3::ZERO),
            sun_direction: vec3(g, "sun", Vec3::new(-0.4, 0.5, -0.8)).normalize_or(Vec3::Y),
            starfield_seed: g.num("starfield_seed").unwrap_or(1.0) as u64,
            music: g.str("music").map(str::to_string),
            order: g.num("order").unwrap_or(100.0) as i64,
            id,
            objects,
            nav_points,
            pack,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_objects_and_nav_points() {
        let s = Scenario::parse(
            "id: t\ntitle: Test\nplayer: lightning\nsun: [0, 1, 0]\norder: 5\nobjects:\n  base: {class: xcom_base, position: [0, 0, -3000]}\n  \
             w1: {class: lightning, ai: wingman, leader: player, offset: [10, 0, 5], speed: 100, destroyed_systems: [engines, vct], hull: 40%}\n\
             nav:\n  alpha: {position: [1, 2, 3]}\n",
            2,
        )
        .unwrap();
        assert_eq!((s.id.as_str(), s.title.as_str(), s.pack, s.order), ("t", "Test", 2, 5));
        assert_eq!(s.player_class.as_deref(), Some("lightning"));
        assert_eq!(s.sun_direction, Vec3::Y);
        assert_eq!(s.objects.len(), 2);
        assert_eq!(s.objects[0].ai, AiRole::Idle, "default role");
        assert_eq!(s.objects[0].position, Vec3::new(0.0, 0.0, -3000.0));
        assert_eq!(s.objects[1].ai, AiRole::Wingman);
        assert_eq!(s.objects[1].leader.as_deref(), Some("player"));
        assert_eq!(s.objects[1].offset, Vec3::new(10.0, 0.0, 5.0));
        assert_eq!(s.objects[1].destroyed_systems, vec!["engines", "vct"]);
        assert_eq!((s.objects[0].hull, s.objects[1].hull), (1.0, 0.4));
        assert_eq!(s.nav_points[0].position, Vec3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn choose_means_the_launcher_picks() {
        let s = Scenario::parse("id: x\n", 0).unwrap();
        assert_eq!(s.player_class, None);
        assert_eq!(s.title, "x");
    }

    #[test]
    fn errors_name_the_line_and_path() {
        assert!(Scenario::parse("", 0).is_err());
        assert!(Scenario::parse("title: x\n", 0).is_err());
        for (text, needle) in [
            ("id: x\nobjects:\n  a: {class: c, ai: dance}\n", "objects.a.ai: 'dance' is not one of"),
            ("id: x\nobjects:\n  a: {}\n", "objects.a: no class"),
            ("id: x\nobjects:\n  a: {class: c, position: [1, 2]}\n", "objects.a.position: expected a list of 3"),
            ("id: x\nobjects:\n  a: {class: c, positon: [1, 2, 3]}\n", "did you mean 'position'"),
            ("id: x\nnav:\n  a: {position: 1}\n", "nav.a.position"),
        ] {
            let e = Scenario::parse(text, 0).unwrap_err();
            assert!(e.contains(needle), "{text}: {e}");
        }
    }
}
