//! # Looks: model sets, debris sets and effects
//!
//! How an object *looks* in every state is kept apart from what it *is* (its class), so
//! several classes can share a look and a remaster can replace a look without touching a
//! class:
//!
//! ```yaml
//! modelsets:
//!   comet:
//!     intact: models/comet.obj
//!     damage:                                   # texture swaps as the hull falls
//!       - {below: 66%, textures: {guard_hull: textures/guard_hull_scorched.png}}
//!       - {below: 33%, textures: {guard_hull: textures/guard_hull_burnt.png}}
//!     states:                                   # named conditions (stations: building…)
//!       building: {textures: {xmmod: original:hang-const}}
//!     debris: comet                             # the debris set used when it explodes
//!     destroyed_effect: explosion_small
//! debris:
//!   comet:
//!     pieces: [models/comet_debris1.obj, models/comet_debris2.obj]
//!     pick: [2, 2]                              # how many pieces, chosen at random
//!     speed: [20, 60]                           # m/s, thrown outwards
//!     spin: [30, 120]                           # degrees per second
//!     life: 20s
//!     chunks: {models: [models/chunk1.obj], count: 6, speed: [40, 120]}
//! effects:
//!   explosion_small: {size: 2.5, duration: 1.2s, color: [1, 0.6, 0.2], shockwave: false}
//! ```
//!
//! Texture swaps match a model's texture by **name**: the file name without folder or
//! extension, compared case-insensitively (`guard_hull` matches
//! `../textures/guard_hull.png`; `inttop` matches the original texture `INTTOP`). A
//! ship class names its look with `model_set:`; the Stage 1 keys `model:` and `debris:`
//! still work and create a simple model set of their own.

use super::schema::{Get, Unit};

/// Index of a [`ModelSet`] in the ruleset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ModelSetId(pub u16);
/// Index of a [`DebrisSet`] in the ruleset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DebrisId(pub u16);
/// Index of an [`EffectDef`] in the ruleset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EffectId(pub u16);

/// Texture swaps: `(texture name to replace, replacement asset reference)`.
pub type Swaps = Vec<(String, String)>;

/// One damage level of a model set.
#[derive(Debug, Clone, PartialEq)]
pub struct DamageLevel {
    /// Applies while hull fraction is below this.
    pub below: f32,
    pub swaps: Swaps,
}

/// Everything needed to draw one object in every state.
#[derive(Debug, Clone, PartialEq)]
pub struct ModelSet {
    pub id: String,
    /// The intact model's asset reference, and the pack that named it.
    pub intact: String,
    pub pack: usize,
    pub scale: f32,
    /// Swaps that always apply (faction colours, variants).
    pub textures: Swaps,
    /// Damage levels, highest threshold first.
    pub damage: Vec<DamageLevel>,
    /// Named states (construction, destroyed-in-place) and their swaps.
    pub states: Vec<(String, Swaps)>,
    pub debris: Option<DebrisId>,
    pub destroyed_effect: Option<EffectId>,
}

impl ModelSet {
    /// The texture swaps for an object at `hull` fraction in named `state` (if any):
    /// the set's own swaps, then every damage level the hull is below (cumulative,
    /// highest threshold first, so the worst level wins), then the state's.
    pub fn swaps_for(&self, hull: f32, state: Option<&str>) -> Swaps {
        let mut out = self.textures.clone();
        for level in &self.damage {
            if hull < level.below {
                apply(&mut out, &level.swaps);
            }
        }
        if let Some(s) = state.and_then(|n| self.states.iter().find(|(k, _)| k == n)) {
            apply(&mut out, &s.1);
        }
        out
    }
}

/// Adds `more` to `swaps`, a later swap of the same name replacing an earlier one.
fn apply(swaps: &mut Swaps, more: &Swaps) {
    for (k, v) in more {
        match swaps.iter_mut().find(|(x, _)| x == k) {
            Some(slot) => slot.1 = v.clone(),
            None => swaps.push((k.clone(), v.clone())),
        }
    }
}

/// The name a texture is matched by: the file name without folders or extension.
pub fn texture_key(reference: &str) -> String {
    let base = reference.rsplit(['/', '\\', ':']).next().unwrap_or(reference);
    let stem = base.rsplit_once('.').map(|(s, _)| s).unwrap_or(base);
    stem.to_ascii_lowercase()
}

/// What happens to the pieces of a destroyed object.
#[derive(Debug, Clone, PartialEq)]
pub struct DebrisSet {
    pub id: String,
    /// Piece models (asset references), each the part of the object it came from, in
    /// the object's own model space.
    pub pieces: Vec<String>,
    pub pack: usize,
    /// How many pieces to throw (chosen at random): `(min, max)`.
    pub pick: (u32, u32),
    /// Outward speed range, m/s.
    pub speed: (f32, f32),
    /// Tumble rate range, degrees per second.
    pub spin: (f32, f32),
    /// Seconds the pieces last.
    pub life: f32,
    /// Generic rubble: models, how many, speed range.
    pub chunks: Vec<String>,
    pub chunk_count: u32,
    pub chunk_speed: (f32, f32),
}

/// An explosion or similar effect.
#[derive(Debug, Clone, PartialEq)]
pub struct EffectDef {
    pub id: String,
    /// Fireball size as a multiple of the object's radius (or metres for missiles).
    pub size: f32,
    pub duration: f32,
    pub color: [f32; 3],
    /// An expanding ring of light in the explosion's plane.
    pub shockwave: bool,
    /// Sound event name (from `sounds:`) or asset reference, played when it goes off.
    pub sound: Option<String>,
    /// The pack that defined it (an asset `sound` resolves there).
    pub pack: usize,
}

impl EffectDef {
    /// The engine's explosion when nothing else is named.
    pub fn builtin(id: &str) -> EffectDef {
        EffectDef {
            id: id.into(),
            size: 2.5,
            duration: 1.6,
            color: [1.0, 0.6, 0.25],
            shockwave: false,
            sound: None,
            pack: 0,
        }
    }

    pub fn parse(id: &str, g: Get, pack: usize) -> EffectDef {
        let d = EffectDef::builtin(id);
        EffectDef {
            id: id.into(),
            size: g.num("size").unwrap_or(d.size).max(0.01),
            duration: g.qty("duration", Unit::Time).unwrap_or(d.duration).max(0.05),
            color: match g.nums("color") {
                Some(v) if v.len() == 3 => [v[0], v[1], v[2]],
                _ => d.color,
            },
            shockwave: g.bool("shockwave").unwrap_or(false),
            sound: g.str("sound").map(str::to_string),
            pack,
        }
    }
}

fn range(g: Get, key: &str, default: (f32, f32)) -> (f32, f32) {
    match g.nums(key) {
        Some(v) if v.len() == 2 => (v[0].min(v[1]), v[0].max(v[1])),
        Some(v) if v.len() == 1 => (v[0], v[0]),
        _ => default,
    }
}

impl DebrisSet {
    pub fn parse(id: &str, g: Get, pack: usize) -> DebrisSet {
        let pieces = g.list("pieces");
        let n = pieces.len() as f32;
        let (lo, hi) = range(g, "pick", (n, n));
        let chunks = g.node("chunks").map(Get);
        DebrisSet {
            id: id.into(),
            pick: (lo as u32, (hi as u32).max(lo as u32)),
            pieces,
            pack,
            speed: range(g, "speed", (20.0, 60.0)),
            spin: range(g, "spin", (30.0, 120.0)),
            life: g.qty("life", Unit::Time).unwrap_or(15.0),
            chunks: chunks.map(|c| c.list("models")).unwrap_or_default(),
            chunk_count: chunks.and_then(|c| c.num("count")).unwrap_or(0.0) as u32,
            chunk_speed: chunks.map(|c| range(c, "speed", (40.0, 120.0))).unwrap_or((40.0, 120.0)),
        }
    }

    /// A debris set from a Stage 1 `debris:` list: every piece, average speeds.
    pub fn from_list(id: &str, pieces: Vec<String>, pack: usize) -> DebrisSet {
        let n = pieces.len() as u32;
        DebrisSet {
            id: id.into(),
            pieces,
            pack,
            pick: (n, n),
            speed: (10.0, 50.0),
            spin: (30.0, 120.0),
            life: 12.0,
            chunks: Vec::new(),
            chunk_count: 0,
            chunk_speed: (40.0, 120.0),
        }
    }
}

/// Parses texture swaps: `{name: replacement, …}`.
pub fn swaps(g: Get, key: &str) -> Swaps {
    g.map(key).iter().filter_map(|(k, v)| Some((texture_key(k), v.as_str()?.to_string()))).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formats::yaml;

    #[test]
    fn swaps_accumulate_with_damage_and_state() {
        let set = ModelSet {
            id: "s".into(),
            intact: "m".into(),
            pack: 0,
            scale: 1.0,
            textures: vec![("hull".into(), "blue.png".into())],
            damage: vec![
                DamageLevel { below: 0.66, swaps: vec![("hull".into(), "scorched.png".into())] },
                DamageLevel {
                    below: 0.33,
                    swaps: vec![("hull".into(), "burnt.png".into()), ("glass".into(), "cracked.png".into())],
                },
            ],
            states: vec![("building".into(), vec![("hull".into(), "frame.png".into())])],
            debris: None,
            destroyed_effect: None,
        };
        assert_eq!(set.swaps_for(1.0, None), vec![("hull".to_string(), "blue.png".to_string())]);
        assert_eq!(set.swaps_for(0.5, None)[0].1, "scorched.png");
        let worst = set.swaps_for(0.1, None);
        assert_eq!((worst[0].1.as_str(), worst.len()), ("burnt.png", 2));
        assert_eq!(set.swaps_for(1.0, Some("building"))[0].1, "frame.png");
        assert_eq!(set.swaps_for(1.0, Some("nope"))[0].1, "blue.png");
    }

    #[test]
    fn texture_keys_ignore_folders_case_and_extensions() {
        assert_eq!(texture_key("../textures/Guard_Hull.png"), "guard_hull");
        assert_eq!(texture_key("original:INTTOP"), "inttop");
        assert_eq!(texture_key("inttop"), "inttop");
    }

    #[test]
    fn debris_and_effects_parse_with_defaults() {
        let doc = yaml::parse(
            "d: {pieces: [a.obj, b.obj, c.obj], pick: [2, 3], speed: [5, 9], life: 8s, chunks: {models: [x.obj], count: 4}}\n\
             e: {size: 3, duration: 900ms, shockwave: true}\nf: {}\n",
        );
        // "900ms" is not a unit the schema knows; the effect keeps its default duration.
        let doc = doc.unwrap();
        let d = DebrisSet::parse("d", Get(doc.get("d").unwrap()), 1);
        assert_eq!((d.pick, d.speed, d.life, d.chunk_count, d.pack), ((2, 3), (5.0, 9.0), 8.0, 4, 1));
        let e = EffectDef::parse("e", Get(doc.get("e").unwrap()), 0);
        assert_eq!((e.size, e.duration, e.shockwave), (3.0, 1.6, true));
        assert_eq!(EffectDef::parse("f", Get(doc.get("f").unwrap()), 0), EffectDef::builtin("f"));
        let l = DebrisSet::from_list("x", vec!["p.obj".into()], 0);
        assert_eq!(l.pick, (1, 1));
    }
}
