//! # Music: interactive music sets (Stage 6.4)
//!
//! A music set has **states** — `cruise` (nothing about), `threat` (enemies on the
//! radar), `battle` (enemies close), `strategy` (the star map) — each a list of
//! **segments** played one after another, and **stingers** played once (death, victory,
//! a good or bad result). The state changes only when a segment ends, so the music never
//! cuts mid-phrase (the original's `segment_end` transition).
//!
//! ```yaml
//! music:
//!   set_1:
//!     states:
//!       cruise: [original:Loops/cruise1-1.wav, original:Loops/cruise1-2.wav]
//!       threat: [original:Loops/threat1-1.wav]
//!       battle: [original:Loops/battle1-1.wav]
//!     stingers: {death: original:Loops/death1.wav, victory: original:Loops/victory1.wav}
//!     threat_range: 12km      # an enemy on the radar this close: threat
//!     battle_range: 4km       # this close: battle
//!     volume: 60%
//! ```
//!
//! A battle picks one of the campaign's sets (by the battle's star field seed, so the
//! same battle always has the same music); the strategy screen plays the first set's
//! `strategy` state (or its `cruise`).

use super::schema::{f, Get, Ty, Unit};

/// One music set.
#[derive(Debug, Clone, PartialEq)]
pub struct MusicSet {
    pub id: String,
    /// `(state, segments)`: asset references, with the pack that named them.
    pub states: Vec<(String, Vec<String>)>,
    /// `(stinger, asset)`.
    pub stingers: Vec<(String, String)>,
    pub threat_range: f32,
    pub battle_range: f32,
    pub volume: f32,
    pub pack: usize,
}

impl MusicSet {
    /// Parses `music.<id>`.
    pub fn parse(id: &str, g: Get, pack: usize) -> Result<MusicSet, String> {
        let mut states = Vec::new();
        for (name, list) in g.map("states") {
            let segs: Vec<String> = match (list.as_seq(), list.as_str()) {
                (Some(s), _) => s.iter().filter_map(|x| x.as_str().map(str::to_string)).collect(),
                (None, Some(one)) => vec![one.to_string()],
                _ => Vec::new(),
            };
            if segs.is_empty() {
                return Err(format!("states.{name}: no segments"));
            }
            states.push((name.to_ascii_lowercase(), segs));
        }
        if states.is_empty() {
            return Err("a music set needs states (cruise, threat, battle, strategy)".into());
        }
        Ok(MusicSet {
            id: id.to_string(),
            states,
            stingers: g
                .map("stingers")
                .iter()
                .filter_map(|(k, v)| Some((k.to_ascii_lowercase(), v.as_str()?.to_string())))
                .collect(),
            threat_range: g.qty("threat_range", Unit::Distance).unwrap_or(12_000.0),
            battle_range: g.qty("battle_range", Unit::Distance).unwrap_or(4_000.0),
            volume: g.qty("volume", Unit::Fraction).unwrap_or(0.6).clamp(0.0, 1.0),
            pack,
        })
    }

    /// The segments of a state, falling back cruise → any.
    pub fn segments(&self, state: &str) -> &[String] {
        self.states
            .iter()
            .find(|(s, _)| s == state)
            .or_else(|| self.states.iter().find(|(s, _)| s == "cruise"))
            .or_else(|| self.states.first())
            .map(|(_, v)| v.as_slice())
            .unwrap_or(&[])
    }
}

/// The state a battle's music should be in: `battle` with a hostile within
/// `battle_range` of the player, `threat` with one within `threat_range`, else `cruise`.
/// `hostiles` are the distances to the player's live hostiles.
pub fn battle_state(set: &MusicSet, hostiles: impl Iterator<Item = f32>) -> &'static str {
    let nearest = hostiles.fold(f32::MAX, f32::min);
    if nearest <= set.battle_range {
        "battle"
    } else if nearest <= set.threat_range {
        "threat"
    } else {
        "cruise"
    }
}

/// The schema of `music.<id>`.
pub fn schema() -> Ty {
    Ty::Def(vec![
        f(
            "states",
            Ty::Dict(Box::new(Ty::Either(Box::new(Ty::List(Box::new(Ty::Asset))), Box::new(Ty::Asset)))),
            "state: segments",
        ),
        f("stingers", Ty::Dict(Box::new(Ty::Asset)), "death, victory, good, bad: a sound played once"),
        f("threat_range", Ty::Qty(Unit::Distance), "an enemy this close: threat"),
        f("battle_range", Ty::Qty(Unit::Distance), "an enemy this close: battle"),
        f("volume", Ty::Qty(Unit::Fraction), "music volume"),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formats::yaml;

    #[test]
    fn sets_parse_and_the_state_follows_the_nearest_enemy() {
        let n = yaml::parse("states: {cruise: [a.wav, b.wav], battle: c.wav}\nbattle_range: 3km\nthreat_range: 10km\nstingers: {death: d.wav}").unwrap();
        let s = MusicSet::parse("m", Get(&n), 0).unwrap();
        assert_eq!(s.segments("cruise"), &["a.wav".to_string(), "b.wav".to_string()]);
        assert_eq!(s.segments("threat"), s.segments("cruise"), "a missing state falls back to cruise");
        assert_eq!(battle_state(&s, [20_000.0].into_iter()), "cruise");
        assert_eq!(battle_state(&s, [20_000.0, 8_000.0].into_iter()), "threat");
        assert_eq!(battle_state(&s, [2_000.0, 8_000.0].into_iter()), "battle");
        assert_eq!(battle_state(&s, std::iter::empty()), "cruise");
        assert!(MusicSet::parse("m", Get(&yaml::parse("{}").unwrap()), 0).is_err());
    }
}
