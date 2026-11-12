//! # Cutscenes: scripted scenes played by the engine (MD2)
//!
//! A cutscene is a file in a pack's `cutscenes/` folder: craft that fly keyframed paths,
//! timed actions (fire, destroy), cameras with cuts between them, sounds and captions.
//! It plays on the ordinary battle simulation at 60 Hz with a fixed seed, so it looks and
//! sounds exactly like a battle and plays the same every time. The Cutscene Designer
//! (MD6) edits these files; until then they are written by hand.
//!
//! ```yaml
//! id: intro
//! title: Perils of the Void
//! length: 24s
//! environment: {starfield_seed: 31, sun: [-0.4, 0.45, -0.8], environment: orbit}
//! entities:
//!   lead: {class: comet, keys: {position: [[0s, [0, 0, 0]], [6s, [400, 50, -1800]]]},
//!          actions: [[4.5s, fire_beams, 1s]]}
//!   saucer: {class: raider_saucer, enter: 2s, keys: {position: [[2s, [600, 80, -2600]], [6s, [420, 60, -2000]]]},
//!            destroy_at: 5.2s}
//! cameras:
//!   wide: {position: [[0s, [0, 300, 900]]], look_at: lead, fov: 55}
//!   chase: {attach: lead, distance: [[0s, 60], [5s, 25]], orbit: [[0s, [0, 10]], [5s, [40, 5]]]}
//! cuts: [[0s, wide], [3.5s, chase]]
//! sounds: [[0s, explosion, 0.8]]       # a named event sound of the campaign (sounds:)
//! captions: [[1s, 6s, "The frontier, 2084."]]
//! ```
//!
//! **Paths.** Position keys are joined by a **centripetal Catmull-Rom spline**
//! ([`catmull_rom`]): it passes through every key, and unlike the uniform kind it never
//! overshoots or loops at a sharp corner. A craft faces along its path and banks into
//! its turns ([`path_rotation`]).

use super::schema::{f, Get, Ty, Unit};
use glam::{Quat, Vec3};

/// A timed action on an entity.
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    /// Fire beams for `seconds`.
    FireBeams(f32),
    /// Fire a missile at the entity named.
    Missile(String),
    /// Explode.
    Destroy,
}

/// One craft or object in the scene.
#[derive(Debug, Clone, PartialEq)]
pub struct Entity {
    pub id: String,
    pub class: String,
    pub faction: Option<String>,
    /// Seconds it appears and (optionally) leaves.
    pub enter: f32,
    pub leave: Option<f32>,
    /// `(time, position)`, sorted.
    pub path: Vec<(f32, Vec3)>,
    pub actions: Vec<(f32, Action)>,
    /// Its weapons do no damage (a show of fire).
    pub harmless: bool,
}

/// How a camera is placed.
#[derive(Debug, Clone, PartialEq)]
pub enum Camera {
    /// Keyframed positions, looking at an entity (or a fixed point).
    Fixed { position: Vec<(f32, Vec3)>, look_at: Target, fov: f32 },
    /// Behind an entity: distance and orbit (yaw, pitch degrees) keys.
    Attach { entity: String, distance: Vec<(f32, f32)>, orbit: Vec<(f32, [f32; 2])>, fov: f32 },
}

/// What a fixed camera looks at.
#[derive(Debug, Clone, PartialEq)]
pub enum Target {
    Entity(String),
    Point(Vec3),
}

/// A cutscene.
#[derive(Debug, Clone, PartialEq)]
pub struct Cutscene {
    pub id: String,
    pub title: String,
    pub length: f32,
    pub starfield_seed: u64,
    pub sun: Vec3,
    pub environment: Option<String>,
    pub entities: Vec<Entity>,
    pub cameras: Vec<(String, Camera)>,
    /// `(time, camera index)`, sorted.
    pub cuts: Vec<(f32, usize)>,
    /// `(time, asset, volume)`.
    pub sounds: Vec<(f32, String, f32)>,
    /// `(from, to, text)`.
    pub captions: Vec<(f32, f32, String)>,
    pub pack: usize,
}

/// Seconds from a time value (`4.5s`, `1min`, or a plain number).
fn secs(n: &crate::formats::yaml::Node) -> Option<f32> {
    super::schema::quantity(n.as_str()?, Unit::Time).ok()
}

fn vec3(n: &crate::formats::yaml::Node) -> Option<Vec3> {
    let v: Vec<f32> = n.as_seq()?.iter().filter_map(|x| x.as_str()?.trim().parse().ok()).collect();
    (v.len() == 3).then(|| Vec3::new(v[0], v[1], v[2]))
}

/// `[[time, value], …]` pairs.
fn keys<T>(
    n: Option<&crate::formats::yaml::Node>,
    value: impl Fn(&crate::formats::yaml::Node) -> Option<T>,
) -> Result<Vec<(f32, T)>, String> {
    let mut out = Vec::new();
    for (i, k) in n.and_then(|n| n.as_seq()).unwrap_or(&[]).iter().enumerate() {
        let pair = k.as_seq().filter(|p| p.len() >= 2).ok_or(format!("key {i}: expected [time, value]"))?;
        let t = secs(&pair[0]).ok_or(format!("key {i}: bad time"))?;
        out.push((t, value(&pair[1]).ok_or(format!("key {i}: bad value"))?));
    }
    out.sort_by(|a, b| a.0.total_cmp(&b.0));
    Ok(out)
}

impl Cutscene {
    /// Parses a cutscene file.
    pub fn parse(g: Get, pack: usize) -> Result<Cutscene, String> {
        let id = g.str("id").ok_or("needs an id")?.to_string();
        let env = g.node("environment").map(Get);
        let mut entities = Vec::new();
        for (eid, n) in g.map("entities") {
            let e = Get(n);
            let at = |m: String| format!("entities.{eid}: {m}");
            let path = keys(e.node("keys").and_then(|k| Get(k).node("position")), vec3).map_err(at)?;
            let mut actions = Vec::new();
            for (i, a) in e.node("actions").and_then(|n| n.as_seq()).unwrap_or(&[]).iter().enumerate() {
                let p = a
                    .as_seq()
                    .filter(|p| p.len() >= 2)
                    .ok_or(at(format!("actions[{i}]: expected [time, action, …]")))?;
                let t = secs(&p[0]).ok_or(at(format!("actions[{i}]: bad time")))?;
                let action = match p[1].as_str() {
                    Some("fire_beams") => Action::FireBeams(p.get(2).and_then(secs).unwrap_or(1.0)),
                    Some("missile") => Action::Missile(p.get(2).and_then(|x| x.as_str()).unwrap_or("").to_string()),
                    Some("destroy") => Action::Destroy,
                    other => {
                        return Err(at(format!(
                            "actions[{i}]: '{}' is not fire_beams, missile or destroy",
                            other.unwrap_or("")
                        )))
                    }
                };
                actions.push((t, action));
            }
            if let Some(t) = e.qty("destroy_at", Unit::Time) {
                actions.push((t, Action::Destroy));
            }
            actions.sort_by(|a, b| a.0.total_cmp(&b.0));
            if path.is_empty() {
                return Err(at("needs keys: {position: [[time, [x, y, z]], …]}".into()));
            }
            entities.push(Entity {
                id: eid.clone(),
                class: e.str("class").ok_or(at("needs a class".into()))?.to_string(),
                faction: e.str("faction").map(str::to_string),
                enter: e.qty("enter", Unit::Time).unwrap_or(path[0].0),
                leave: e.qty("leave", Unit::Time),
                path,
                actions,
                harmless: e.bool("harmless").unwrap_or(false),
            });
        }
        let mut cameras = Vec::new();
        for (cid, n) in g.map("cameras") {
            let c = Get(n);
            let at = |m: String| format!("cameras.{cid}: {m}");
            let fov = c.num("fov").unwrap_or(60.0).clamp(10.0, 120.0);
            let cam = match c.str("attach") {
                Some(e) => Camera::Attach {
                    entity: e.to_string(),
                    distance: keys(c.node("distance"), |n| n.as_str()?.trim().parse().ok()).map_err(at)?,
                    orbit: keys(c.node("orbit"), |n| {
                        let v: Vec<f32> = n.as_seq()?.iter().filter_map(|x| x.as_str()?.trim().parse().ok()).collect();
                        (v.len() == 2).then(|| [v[0], v[1]])
                    })
                    .map_err(at)?,
                    fov,
                },
                None => Camera::Fixed {
                    position: keys(c.node("position"), vec3).map_err(at)?,
                    look_at: match (c.str("look_at"), c.node("look_at").and_then(vec3)) {
                        (_, Some(p)) => Target::Point(p),
                        (Some(e), None) => Target::Entity(e.to_string()),
                        _ => Target::Point(Vec3::ZERO),
                    },
                    fov,
                },
            };
            cameras.push((cid.clone(), cam));
        }
        for (cid, c) in &cameras {
            let named = match c {
                Camera::Attach { entity, .. } => Some(entity),
                Camera::Fixed { look_at: Target::Entity(e), .. } => Some(e),
                _ => None,
            };
            if let Some(e) = named.filter(|e| !entities.iter().any(|x: &Entity| &x.id == *e)) {
                return Err(format!("cameras.{cid}: no entity '{e}'"));
            }
            if let Camera::Fixed { position, .. } = c {
                if position.is_empty() {
                    return Err(format!("cameras.{cid}: needs position keys (or attach:)"));
                }
            }
        }
        let cuts = keys(g.node("cuts"), |n| n.as_str().map(str::to_string))?
            .into_iter()
            .map(|(t, c)| {
                cameras.iter().position(|(id, _)| *id == c).map(|k| (t, k)).ok_or(format!("cuts: no camera '{c}'"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        if cuts.is_empty() && !cameras.is_empty() {
            return Err("cuts: needs at least one [time, camera]".into());
        }
        let mut sounds = Vec::new();
        for s in g.node("sounds").and_then(|n| n.as_seq()).unwrap_or(&[]) {
            let p = s.as_seq().filter(|p| p.len() >= 2).ok_or("sounds: expected [time, sound, volume]")?;
            let vol = p.get(2).and_then(|v| v.as_str()?.trim().parse().ok()).unwrap_or(1.0);
            sounds.push((secs(&p[0]).ok_or("sounds: bad time")?, p[1].as_str().unwrap_or("").to_string(), vol));
        }
        let mut captions = Vec::new();
        for c in g.node("captions").and_then(|n| n.as_seq()).unwrap_or(&[]) {
            let p = c.as_seq().filter(|p| p.len() >= 3).ok_or("captions: expected [from, to, text]")?;
            captions.push((
                secs(&p[0]).ok_or("captions: bad time")?,
                secs(&p[1]).ok_or("captions: bad time")?,
                p[2].as_str().unwrap_or("").to_string(),
            ));
        }
        Ok(Cutscene {
            title: g.str("title").unwrap_or(&id).to_string(),
            length: g.qty("length", Unit::Time).unwrap_or(10.0).max(0.1),
            starfield_seed: env.and_then(|e| e.num("starfield_seed")).unwrap_or(1.0) as u64,
            sun: env.and_then(|e| e.node("sun")).and_then(vec3).unwrap_or(Vec3::new(-0.4, 0.45, -0.8)),
            environment: env.and_then(|e| e.str("environment")).map(str::to_string),
            id,
            entities,
            cameras,
            cuts,
            sounds,
            captions,
            pack,
        })
    }

    /// The camera cut in force at time `t`.
    pub fn camera_at(&self, t: f32) -> Option<usize> {
        self.cuts.iter().rev().find(|(at, _)| *at <= t).or(self.cuts.first()).map(|(_, c)| *c)
    }
}

/// A point on a path at time `t`: a centripetal Catmull-Rom spline through the keys
/// (clamped to the first and last).
///
/// Between keys `p1` and `p2`, the spline uses their neighbours `p0` and `p3` (the ends
/// repeat). "Centripetal" means the knots are spaced by the square root of the distance
/// between points (α = ½), which keeps the curve from overshooting or forming loops at
/// sharp corners; the curve is evaluated with the Barry–Goldman pyramid of lerps.
pub fn catmull_rom(keys: &[(f32, Vec3)], t: f32) -> Vec3 {
    match keys {
        [] => Vec3::ZERO,
        [(_, p)] => *p,
        _ => {
            let n = keys.len();
            if t <= keys[0].0 {
                return keys[0].1;
            }
            if t >= keys[n - 1].0 {
                return keys[n - 1].1;
            }
            let i = keys.windows(2).position(|w| t >= w[0].0 && t <= w[1].0).unwrap_or(0);
            let p0 = keys[i.saturating_sub(1)].1;
            let (p1, p2) = (keys[i].1, keys[i + 1].1);
            let p3 = keys[(i + 2).min(n - 1)].1;
            let u = (t - keys[i].0) / (keys[i + 1].0 - keys[i].0).max(1e-6);
            // Knot spacing: the square root of each segment's length (never zero).
            let gap = |a: Vec3, b: Vec3| a.distance(b).sqrt().max(1e-4);
            let t0 = 0.0;
            let t1 = t0 + gap(p0, p1);
            let t2 = t1 + gap(p1, p2);
            let t3 = t2 + gap(p2, p3);
            let tt = t1 + (t2 - t1) * u;
            let lerp = |a: Vec3, b: Vec3, ta: f32, tb: f32| a + (b - a) * ((tt - ta) / (tb - ta));
            let a1 = lerp(p0, p1, t0, t1);
            let a2 = lerp(p1, p2, t1, t2);
            let a3 = lerp(p2, p3, t2, t3);
            let b1 = lerp(a1, a2, t0, t2);
            let b2 = lerp(a2, a3, t1, t3);
            lerp(b1, b2, t1, t2)
        }
    }
}

/// A craft's rotation on its path at `t`: nose (−Z) along the path's direction, banked
/// into turns by up to 60° (from how fast the direction is changing sideways).
pub fn path_rotation(keys: &[(f32, Vec3)], t: f32) -> Quat {
    let dt = 0.05;
    let (a, b, c) = (catmull_rom(keys, t - dt), catmull_rom(keys, t), catmull_rom(keys, t + dt));
    let Some(fwd) = (c - a).try_normalize() else {
        return Quat::IDENTITY;
    };
    let look = Quat::from_rotation_arc(Vec3::NEG_Z, fwd);
    // Sideways acceleration in the craft's frame: positive to the right → bank right.
    let accel = (c - 2.0 * b + a) / (dt * dt);
    let right = look * Vec3::X;
    let bank = (accel.dot(right) * 0.02).clamp(-1.05, 1.05);
    look * Quat::from_rotation_z(-bank)
}

/// A scalar key track at `t` (linear between keys, held at the ends).
pub fn scalar(keys: &[(f32, f32)], t: f32) -> f32 {
    match keys {
        [] => 0.0,
        [(_, v)] => *v,
        _ => {
            if t <= keys[0].0 {
                return keys[0].1;
            }
            for w in keys.windows(2) {
                if t <= w[1].0 {
                    let u = (t - w[0].0) / (w[1].0 - w[0].0).max(1e-6);
                    return w[0].1 + (w[1].1 - w[0].1) * u;
                }
            }
            keys[keys.len() - 1].1
        }
    }
}

/// The schema of a cutscene file.
pub fn schema() -> Ty {
    let keys = || Ty::List(Box::new(Ty::List(Box::new(Ty::Text))));
    Ty::Struct(vec![
        f("id", Ty::Name, "the cutscene's id"),
        f("title", Ty::Text, "its title"),
        f("length", Ty::Qty(Unit::Time), "how long"),
        f(
            "environment",
            Ty::Struct(vec![
                f("starfield_seed", Ty::Num, "the stars"),
                f("sun", Ty::Numbers(3), "the sun's direction"),
                f("environment", Ty::Name, "a battle environment (sky, nebula, planet)"),
            ]),
            "the backdrop",
        ),
        f(
            "entities",
            Ty::Dict(Box::new(Ty::Struct(vec![
                f("class", Ty::Name, "ship or object class"),
                f("faction", Ty::Name, "overrides the class's"),
                f("enter", Ty::Qty(Unit::Time), "appears (default: its first key)"),
                f("leave", Ty::Qty(Unit::Time), "disappears"),
                f("keys", Ty::Struct(vec![f("position", keys(), "[[time, [x, y, z]], …]")]), "its path"),
                f("actions", keys(), "[[time, fire_beams, seconds] | [time, missile, target] | [time, destroy]]"),
                f("destroy_at", Ty::Qty(Unit::Time), "explodes then"),
                f("harmless", Ty::Bool, "its weapons do no damage"),
            ]))),
            "craft and objects",
        ),
        f(
            "cameras",
            Ty::Dict(Box::new(Ty::Struct(vec![
                f("position", keys(), "fixed: [[time, [x, y, z]], …]"),
                f("look_at", Ty::Text, "fixed: an entity, or [x, y, z]"),
                f("attach", Ty::Name, "behind this entity"),
                f("distance", keys(), "attached: [[time, metres], …]"),
                f("orbit", keys(), "attached: [[time, [yaw, pitch]], …] degrees"),
                f("fov", Ty::Num, "vertical field of view, degrees"),
            ]))),
            "cameras",
        ),
        f("cuts", keys(), "[[time, camera], …]"),
        f("sounds", keys(), "[[time, sound, volume], …]"),
        f("captions", keys(), "[[from, to, text], …]"),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formats::yaml;

    #[test]
    fn catmull_rom_passes_through_keys_without_overshoot() {
        let k = [
            (0.0, Vec3::ZERO),
            (1.0, Vec3::new(10.0, 0.0, 0.0)),
            (2.0, Vec3::new(10.0, 0.0, 10.0)),
            (3.0, Vec3::new(20.0, 0.0, 10.0)),
        ];
        for (t, p) in k {
            assert!((catmull_rom(&k, t) - p).length() < 1e-3, "through key at {t}");
        }
        // A sharp corner: the curve stays near the hull of its points (no wild loop).
        for i in 0..=30 {
            let p = catmull_rom(&k, i as f32 / 10.0);
            assert!(p.x > -1.5 && p.x < 21.5 && p.z > -1.5 && p.z < 11.5, "{p:?}");
        }
        assert_eq!(catmull_rom(&k, -5.0), Vec3::ZERO);
        assert_eq!(catmull_rom(&k, 9.0), Vec3::new(20.0, 0.0, 10.0));
        // Facing along the path: straight along +X at the start.
        let r = path_rotation(&k, 0.3);
        assert!((r * Vec3::NEG_Z).x > 0.9, "{:?}", r * Vec3::NEG_Z);
        assert_eq!(scalar(&[(0.0, 10.0), (2.0, 30.0)], 1.0), 20.0);
    }

    #[test]
    fn cutscenes_parse_and_check_their_references() {
        let text = "id: s\nlength: 10s\nentities:\n  a: {class: x, keys: {position: [[0s, [0, 0, 0]], [5s, [0, 0, -100]]]}, actions: [[2s, fire_beams, 1s]], destroy_at: 4s}\n\
                    cameras:\n  w: {position: [[0s, [0, 10, 50]]], look_at: a}\n  c: {attach: a, distance: [[0s, 40]], orbit: [[0s, [10, 5]]]}\n\
                    cuts: [[0s, w], [3s, c]]\ncaptions: [[1s, 3s, Hello]]\n";
        let c = Cutscene::parse(Get(&yaml::parse(text).unwrap()), 0).unwrap();
        assert_eq!(c.length, 10.0);
        assert_eq!(c.entities[0].actions, vec![(2.0, Action::FireBeams(1.0)), (4.0, Action::Destroy)]);
        assert_eq!(c.camera_at(1.0), Some(0));
        assert_eq!(c.camera_at(5.0), Some(1));
        assert_eq!(c.captions[0].2, "Hello");
        let bad = text.replace("look_at: a", "look_at: b");
        assert!(Cutscene::parse(Get(&yaml::parse(&bad).unwrap()), 0).unwrap_err().contains("no entity 'b'"));
        let bad = text.replace("[3s, c]", "[3s, nope]");
        assert!(Cutscene::parse(Get(&yaml::parse(&bad).unwrap()), 0).unwrap_err().contains("nope"));
    }
}
