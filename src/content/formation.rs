//! # Formations: named shapes a group of craft flies in (MD1)
//!
//! A formation is pack data (`formations: {<id>: …}`), shared by everything that flies in
//! a group: wingmen in battle first, cutscene formations later (the Cutscene Designer).
//! A mission role of wingmen names one with `formation: <id>`; without one, the
//! campaign's `flight: {formation: <id>}` applies (default `vee`).
//!
//! ```yaml
//! formations:
//!   vee:          {shape: vee, spacing: 60m}
//!   line:         {shape: line_abreast, spacing: 70m}
//!   staggered:    {slots: [[-1, 0, 1], [1, 0, 2], [-1, 0, 3]], spacing: 50m}  # custom
//! ```
//!
//! Slots are in the **leader's frame**: +X is the leader's right, +Y up, +Z *behind*
//! (the craft fly towards −Z). A slot is given in units of `spacing`, so one formation
//! can be opened up or closed in by changing a single number. A shape gives a slot for
//! any number of followers; a custom list continues as a vee after its last slot. The
//! flight rules still keep every slot at least `flight.formation_gap` × the two craft's
//! radii from the leader (see [`crate::sim::ai::formation_slot`]).

use super::schema::{f, Get, Ty, Unit};
use glam::Vec3;

/// The built-in shapes.
#[derive(Debug, Clone, PartialEq)]
pub enum Shape {
    /// Alternating left and right, each pair one rank further back.
    Vee,
    /// Alternating left and right, level with the leader.
    LineAbreast,
    /// A diagonal line back and to the right.
    Echelon,
    /// Rows of two behind the leader (the leader is the front-left corner).
    Box,
    /// Left, right and behind; any more trail behind the diamond.
    Diamond,
    /// The listed slots (in spacing units), then a vee behind them.
    Custom(Vec<Vec3>),
}

/// One formation.
#[derive(Debug, Clone, PartialEq)]
pub struct FormationDef {
    pub id: String,
    pub shape: Shape,
    /// Metres per slot unit.
    pub spacing: f32,
}

/// The default spacing, metres.
pub const SPACING: f32 = 60.0;

impl FormationDef {
    /// Parses `formations.<id>`: `shape:` or `slots:` (not both), and `spacing:`.
    pub fn parse(id: &str, g: Get) -> Result<FormationDef, String> {
        let shape = match (g.str("shape"), g.node("slots")) {
            (Some(_), Some(_)) => return Err("give shape or slots, not both".into()),
            (Some(s), None) => Shape::parse(s)
                .ok_or(format!("shape: '{s}' is not vee, line_abreast, echelon, box or diamond (or give slots)"))?,
            (None, Some(n)) => {
                let mut slots = Vec::new();
                for (k, s) in n.as_seq().ok_or("slots: expected a list of [x, y, z]")?.iter().enumerate() {
                    let v: Vec<f32> = s
                        .as_seq()
                        .unwrap_or(&[])
                        .iter()
                        .filter_map(|x| x.as_str().and_then(|t| t.trim().parse().ok()))
                        .collect();
                    match v.as_slice() {
                        [x, y, z] => slots.push(Vec3::new(*x, *y, *z)),
                        _ => return Err(format!("slots[{k}]: expected [x, y, z]")),
                    }
                }
                if slots.is_empty() {
                    return Err("slots: empty".into());
                }
                Shape::Custom(slots)
            }
            (None, None) => Shape::Vee,
        };
        let spacing = g.qty("spacing", Unit::Distance).unwrap_or(SPACING);
        if spacing <= 0.0 {
            return Err("spacing: must be more than 0".into());
        }
        Ok(FormationDef { id: id.to_string(), shape, spacing })
    }

    /// The built-in vee, used when a campaign defines no formations at all.
    pub fn fallback() -> FormationDef {
        FormationDef { id: "vee".into(), shape: Shape::Vee, spacing: SPACING }
    }

    /// Follower `k`'s slot (0 = the first follower), metres in the leader's frame.
    pub fn slot(&self, k: usize) -> Vec3 {
        self.unit_slot(k) * self.spacing
    }

    /// Follower `k`'s slot in spacing units.
    fn unit_slot(&self, k: usize) -> Vec3 {
        // Rank (1, 1, 2, 2, 3, …) and side (left, right, left, …) for alternating shapes.
        let vee = |k: usize| {
            let rank = (k / 2 + 1) as f32;
            let side = if k % 2 == 0 { -1.0 } else { 1.0 };
            (rank, side)
        };
        match &self.shape {
            Shape::Vee => {
                let (r, s) = vee(k);
                Vec3::new(s * r, 0.0, r)
            }
            Shape::LineAbreast => {
                let (r, s) = vee(k);
                Vec3::new(s * r, 0.0, 0.0)
            }
            Shape::Echelon => {
                let r = (k + 1) as f32;
                Vec3::new(r, 0.0, r)
            }
            Shape::Box => {
                // Positions 1, 2, 3, … in rows of two; the leader is position 0.
                let i = k + 1;
                Vec3::new((i % 2) as f32, 0.0, (i / 2) as f32)
            }
            Shape::Diamond => match k {
                0 => Vec3::new(-1.0, 0.0, 1.0),
                1 => Vec3::new(1.0, 0.0, 1.0),
                k => Vec3::new(0.0, 0.0, k as f32),
            },
            Shape::Custom(slots) => match slots.get(k) {
                Some(s) => *s,
                None => {
                    // After the list: a vee behind the deepest listed slot.
                    let back = slots.iter().map(|s| s.z).fold(0.0, f32::max);
                    let (r, s) = vee(k - slots.len());
                    Vec3::new(s * r, 0.0, back + r)
                }
            },
        }
    }
}

impl Shape {
    /// The shape for a `shape:` name.
    pub fn parse(s: &str) -> Option<Shape> {
        Some(match s {
            "vee" => Shape::Vee,
            "line_abreast" | "line" => Shape::LineAbreast,
            "echelon" => Shape::Echelon,
            "box" => Shape::Box,
            "diamond" => Shape::Diamond,
            _ => return None,
        })
    }
}

/// The schema of one `formations.<id>` entry (for the checker and the editor schema).
pub fn formation_schema() -> Ty {
    Ty::Def(vec![
        f("shape", Ty::Enum(&["vee", "line_abreast", "echelon", "box", "diamond"]), "a built-in shape"),
        f("slots", Ty::List(Box::new(Ty::Numbers(3))), "custom slots [x, y, z] in spacing units; +x right, +z behind"),
        f("spacing", Ty::Qty(Unit::Distance), "metres per slot unit (default 60 m)"),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formats::yaml;

    fn parse(text: &str) -> Result<FormationDef, String> {
        let n = yaml::parse(text).unwrap();
        FormationDef::parse("f", Get(&n))
    }

    #[test]
    fn shapes_place_followers_behind_and_beside_the_leader() {
        let vee = parse("shape: vee\nspacing: 50m").unwrap();
        assert_eq!(vee.slot(0), Vec3::new(-50.0, 0.0, 50.0), "left and back");
        assert_eq!(vee.slot(1), Vec3::new(50.0, 0.0, 50.0), "right and back");
        assert_eq!(vee.slot(2), Vec3::new(-100.0, 0.0, 100.0), "next rank");
        let line = parse("shape: line_abreast").unwrap();
        assert!((0..6).all(|k| line.slot(k).z == 0.0), "level with the leader");
        let ech = parse("shape: echelon").unwrap();
        assert!((0..4).all(|k| ech.slot(k).x > 0.0 && ech.slot(k).z > 0.0));
        let bx = parse("shape: box").unwrap();
        assert_eq!(bx.slot(0), Vec3::new(SPACING, 0.0, 0.0));
        assert_eq!(bx.slot(1), Vec3::new(0.0, 0.0, SPACING));
        let dia = parse("shape: diamond").unwrap();
        assert_eq!(dia.slot(2), Vec3::new(0.0, 0.0, 2.0 * SPACING), "the tail");
        // No two followers share a slot, for every shape.
        for f in [&vee, &line, &ech, &bx, &dia] {
            for a in 0..8 {
                for b in a + 1..8 {
                    assert_ne!(f.slot(a), f.slot(b), "{:?} {a} {b}", f.shape);
                }
            }
        }
    }

    #[test]
    fn custom_slots_then_a_vee_and_errors() {
        let c = parse("slots: [[0, 0, 1], [0, 1, 2]]\nspacing: 10m").unwrap();
        assert_eq!(c.slot(0), Vec3::new(0.0, 0.0, 10.0));
        assert_eq!(c.slot(1), Vec3::new(0.0, 10.0, 20.0));
        assert_eq!(c.slot(2), Vec3::new(-10.0, 0.0, 30.0), "a vee behind the deepest slot");
        assert_eq!(parse("{}").unwrap().shape, Shape::Vee, "default shape");
        assert!(parse("shape: blob").unwrap_err().contains("not vee"));
        assert!(parse("shape: vee\nslots: [[1, 0, 1]]").unwrap_err().contains("not both"));
        assert!(parse("slots: [[1, 0]]").unwrap_err().contains("slots[0]"));
        assert!(parse("spacing: 0m").is_err());
    }
}
