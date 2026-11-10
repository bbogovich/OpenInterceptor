//! # Scenarios
//!
//! A scenario is a playable space arena described in a pack's `scenarios/*.ini`. The
//! testbed's live test environments are scenarios, and Stage 3 missions build on the
//! same format. Being plain data, a modder can add a new test arena without writing Rust.
//!
//! ```ini
//! [scenario]
//! id = flight-test
//! title = Flight test: station patrol
//! description = Fly around an X-COM base with two wingmen…
//! player = choose               ; or a ship id
//! sun = -0.4, 0.5, -0.8         ; direction *towards* the sun
//! starfield_seed = 7
//!
//! [object.base]
//! class = xcom_base
//! position = 0, 0, -3000        ; metres; -Z is "ahead" of the player's start
//! rotation = 0, 0, 0            ; yaw, pitch, roll in degrees
//! ai = idle                     ; player | wingman | enemy | idle | target
//!
//! [object.wing1]
//! class = lightning
//! ai = wingman
//! leader = player
//! offset = -60, 0, 30           ; formation slot relative to the leader
//!
//! [nav.alpha]                   ; optional navigation points (engine extension)
//! position = 0, 0, -3000
//! ```

use crate::formats::ini::{Ini, Section};
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

fn vec3(s: &Section, key: &str, default: Vec3) -> Result<Vec3, String> {
    match s.get_floats(key) {
        Some(v) if v.len() == 3 => Ok(Vec3::new(v[0], v[1], v[2])),
        Some(_) => Err(format!("{key} needs three numbers")),
        None if s.get_nonempty(key).is_some() => Err(format!("{key} is not a list of numbers")),
        None => Ok(default),
    }
}

impl Scenario {
    /// Parses scenario text. `pack` is recorded for asset resolution.
    pub fn parse(text: &str, pack: usize) -> Result<Scenario, String> {
        let ini = Ini::parse(text);
        let s = ini.section("scenario").ok_or("no [scenario] section")?;
        let id = s.get_nonempty("id").ok_or("scenario has no id")?.to_string();
        let player = s.get_nonempty("player").unwrap_or("choose");
        let mut objects = Vec::new();
        let mut nav_points = Vec::new();
        for sec in ini.sections() {
            if let Some(name) = sec.name.strip_prefix("object.") {
                let ctx = |e: String| format!("[{}]: {e}", sec.name);
                let class = sec.get_nonempty("class").ok_or_else(|| ctx("no class".into()))?.to_string();
                let ai = match sec.get_nonempty("ai") {
                    Some(a) => AiRole::parse(a).ok_or_else(|| ctx(format!("unknown ai '{a}'")))?,
                    None => AiRole::Idle,
                };
                objects.push(Placement {
                    name: name.to_string(),
                    class,
                    faction: sec.get_nonempty("faction").map(str::to_string),
                    position: vec3(sec, "position", Vec3::ZERO).map_err(ctx)?,
                    rotation: vec3(sec, "rotation", Vec3::ZERO).map_err(ctx)?,
                    ai,
                    leader: sec.get_nonempty("leader").map(str::to_string),
                    offset: vec3(sec, "offset", Vec3::new(-60.0, 0.0, 30.0)).map_err(ctx)?,
                    speed: sec.get_f32("speed").unwrap_or(0.0),
                });
            } else if let Some(name) = sec.name.strip_prefix("nav.") {
                nav_points.push(NavPoint {
                    name: name.to_string(),
                    position: vec3(sec, "position", Vec3::ZERO).map_err(|e| format!("[{}]: {e}", sec.name))?,
                });
            }
        }
        if objects.iter().filter(|o| o.ai == AiRole::Player).count() > 0 {
            return Err("use `player =` in [scenario] rather than ai = player".into());
        }
        Ok(Scenario {
            title: s.get_nonempty("title").unwrap_or(&id).to_string(),
            description: s.get("description").unwrap_or("").to_string(),
            player_class: (!player.eq_ignore_ascii_case("choose")).then(|| player.to_string()),
            player_position: vec3(s, "player_position", Vec3::ZERO)?,
            player_rotation: vec3(s, "player_rotation", Vec3::ZERO)?,
            sun_direction: vec3(s, "sun", Vec3::new(-0.4, 0.5, -0.8))?.normalize_or(Vec3::Y),
            starfield_seed: s.get_i64("starfield_seed").unwrap_or(1) as u64,
            music: s.get_nonempty("music").map(str::to_string),
            order: s.get_i64("order").unwrap_or(100),
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
            "[scenario]\nid=t\ntitle=Test\nplayer=lightning\nsun=0,1,0\norder=5\n\
             [object.base]\nclass=xcom_base\nposition=0,0,-3000\n\
             [object.w1]\nclass=lightning\nai=wingman\nleader=player\noffset=10,0,5\nspeed=100\n\
             [nav.alpha]\nposition=1,2,3\n",
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
        assert_eq!(s.nav_points[0].position, Vec3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn choose_means_the_launcher_picks() {
        let s = Scenario::parse("[scenario]\nid=x\n", 0).unwrap();
        assert_eq!(s.player_class, None);
        assert_eq!(s.title, "x");
    }

    #[test]
    fn errors_name_the_section() {
        assert!(Scenario::parse("", 0).is_err());
        assert!(Scenario::parse("[scenario]\n", 0).is_err());
        let e = Scenario::parse("[scenario]\nid=x\n[object.a]\nclass=c\nai=dance\n", 0).unwrap_err();
        assert!(e.contains("[object.a]") && e.contains("dance"));
        assert!(Scenario::parse("[scenario]\nid=x\n[object.a]\n", 0).is_err());
        assert!(Scenario::parse("[scenario]\nid=x\n[object.a]\nclass=c\nposition=1,2\n", 0).is_err());
        assert!(Scenario::parse("[scenario]\nid=x\n[object.a]\nclass=c\nposition=a,b,c\n", 0).is_err());
        assert!(Scenario::parse("[scenario]\nid=x\n[object.a]\nclass=c\nai=player\n", 0).is_err());
        assert!(Scenario::parse("[scenario]\nid=x\n[nav.a]\nposition=1\n", 0).is_err());
    }
}
