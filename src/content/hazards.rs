//! # Battle hazards (`hazards:` in missions and scenarios)
//!
//! A battle can take place in an asteroid belt, inside a minefield, next to a black hole
//! or in an ion storm. Each hazard is optional data; the simulation applies them
//! ([`crate::sim::hazards`]). R-MSN-009.
//!
//! ```yaml
//! hazards:
//!   asteroids:  {class: cosmium_asteroid, count: 40, radius: 5km, center: [0, 0, -4000], spin: 6}
//!   minefield:  {class: space_mine, count: 20, radius: 2km, center: [0, 0, -3000]}
//!   black_hole: {position: [0, 0, -9000], pull: 15, radius: 400}
//!   ion_storm:  {sensors: 40%}
//! ```

use super::schema::{f, Get, Ty, Unit};
use glam::Vec3;

/// Objects scattered in a sphere: asteroids or mines.
#[derive(Debug, Clone, PartialEq)]
pub struct Scatter {
    /// The object class id (checked against the rules when the battle starts).
    pub class: String,
    pub count: u32,
    pub radius: f32,
    pub center: Vec3,
    /// Largest turn rate, degrees per second (asteroids tumble slowly).
    pub spin: f32,
    /// Faction id, or `None` for the class's own.
    pub faction: Option<String>,
}

/// A black hole: pulls every craft and missile in, and destroys what crosses `radius`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BlackHole {
    pub position: Vec3,
    /// Acceleration towards it at 1 km, m/s²; it grows with 1/d² (capped at 20×).
    pub pull: f32,
    /// The point of no return, metres.
    pub radius: f32,
}

/// Every hazard a battle may have.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Hazards {
    pub asteroids: Option<Scatter>,
    pub minefield: Option<Scatter>,
    pub black_hole: Option<BlackHole>,
    /// Ion storm: radar range is multiplied by this (0..1); locks build at half speed.
    pub ion_storm: Option<f32>,
}

impl Hazards {
    /// Reads `hazards:` from a mission or scenario. Missing keys take defaults; a bad
    /// value is an error naming the key.
    pub fn parse(g: Get) -> Result<Hazards, String> {
        let vec3 = |g: Get, k: &str| -> Result<Vec3, String> {
            match g.nums(k) {
                None => Ok(Vec3::ZERO),
                Some(v) if v.len() == 3 => Ok(Vec3::new(v[0], v[1], v[2])),
                Some(_) => Err(format!("hazards: {k} needs [x, y, z]")),
            }
        };
        let scatter = |k: &str, count: u32, radius: f32| -> Result<Option<Scatter>, String> {
            let Some(n) = g.node(k) else { return Ok(None) };
            let s = Get(n);
            let class = s.str("class").ok_or(format!("hazards.{k}: needs a class"))?.to_string();
            Ok(Some(Scatter {
                class,
                count: s.num("count").map(|c| c.max(0.0) as u32).unwrap_or(count),
                radius: s.qty("radius", Unit::Distance).unwrap_or(radius).max(1.0),
                center: vec3(s, "center")?,
                spin: s.num("spin").unwrap_or(4.0).max(0.0),
                faction: s.str("faction").map(str::to_string),
            }))
        };
        let black_hole = match g.node("black_hole").map(Get) {
            Some(b) => Some(BlackHole {
                position: vec3(b, "position")?,
                pull: b.num("pull").unwrap_or(15.0).max(0.0),
                radius: b.qty("radius", Unit::Distance).unwrap_or(400.0).max(1.0),
            }),
            None => None,
        };
        let ion_storm =
            g.node("ion_storm").map(Get).map(|s| s.qty("sensors", Unit::Fraction).unwrap_or(0.4).clamp(0.05, 1.0));
        Ok(Hazards {
            asteroids: scatter("asteroids", 40, 5000.0)?,
            minefield: scatter("minefield", 20, 2000.0)?,
            black_hole,
            ion_storm,
        })
    }

    /// The class ids the hazards place (for checking against the rules).
    pub fn classes(&self) -> impl Iterator<Item = &str> {
        self.asteroids.iter().chain(self.minefield.iter()).map(|s| s.class.as_str())
    }
}

/// The schema of `hazards:` (for the per-file schemas and the JSON Schema).
pub fn schema() -> Ty {
    let scatter = |what: &'static str| {
        Ty::Struct(vec![
            f("class", Ty::Name, what),
            f("count", Ty::Num, "how many"),
            f("radius", Ty::Qty(Unit::Distance), "the sphere they are scattered in"),
            f("center", Ty::Numbers(3), "the sphere's centre [x, y, z], metres"),
            f("spin", Ty::Num, "largest turn rate, degrees per second"),
            f("faction", Ty::Name, "their faction (default: the class's own)"),
        ])
    };
    Ty::Struct(vec![
        f("asteroids", scatter("the asteroid object class"), "an asteroid field"),
        f("minefield", scatter("the mine class (kind: mine)"), "a minefield"),
        f(
            "black_hole",
            Ty::Struct(vec![
                f("position", Ty::Numbers(3), "where it is [x, y, z], metres"),
                f("pull", Ty::Num, "acceleration towards it at 1 km, m/s² (grows with 1/d²)"),
                f("radius", Ty::Qty(Unit::Distance), "craft closer than this are destroyed"),
            ]),
            "a black hole",
        ),
        f(
            "ion_storm",
            Ty::Struct(vec![f("sensors", Ty::Qty(Unit::Fraction), "radar range factor, e.g. 40%")]),
            "an ion storm",
        ),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formats::yaml;

    #[test]
    fn hazards_parse_with_defaults_and_report_mistakes() {
        let doc = yaml::parse(
            "asteroids: {class: rock, count: 12, radius: 3km, center: [0, 0, -2000]}\n\
             black_hole: {position: [0, 0, -9000], pull: 20}\nion_storm: {sensors: 30%}\n",
        )
        .unwrap();
        let h = Hazards::parse(Get(&doc)).unwrap();
        let a = h.asteroids.as_ref().unwrap();
        assert_eq!((a.class.as_str(), a.count, a.radius, a.center), ("rock", 12, 3000.0, Vec3::new(0.0, 0.0, -2000.0)));
        assert_eq!(h.black_hole.unwrap().radius, 400.0, "default radius");
        assert_eq!(h.ion_storm, Some(0.3));
        assert!(h.minefield.is_none());
        assert_eq!(h.classes().collect::<Vec<_>>(), vec!["rock"]);
        let bad = yaml::parse("minefield: {count: 3}\n").unwrap();
        assert!(Hazards::parse(Get(&bad)).unwrap_err().contains("needs a class"));
        let bad = yaml::parse("black_hole: {position: [1, 2]}\n").unwrap();
        assert!(Hazards::parse(Get(&bad)).is_err());
    }
}
