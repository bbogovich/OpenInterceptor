//! # Target selection
//!
//! The original game's targeting keys, as pure functions over the world:
//!
//! | Key | Function |
//! |---|---|
//! | `T` / `Y` | [`cycle`] forwards / backwards through every object |
//! | `R` | [`nearest_hostile`] |
//! | `G` | [`in_sights`] — the object closest to the centre of view |
//!
//! They take `&World` (a shared borrow) because choosing a target only *reads* the
//! world; the caller stores the result in `ship.target`.

use super::{ShipId, World};

/// Every live ship except `me`, as `(id, distance)`.
fn others(w: &World, me: ShipId) -> Vec<(ShipId, f32)> {
    let Some(ship) = w.get(me) else { return Vec::new() };
    w.live_ids()
        .into_iter()
        .filter(|&id| id != me)
        .map(|id| (id, w.ships[id.index as usize].position.distance(ship.position)))
        .collect()
}

/// The next (or previous) target after `current`, in slot order, wrapping around.
pub fn cycle(w: &World, me: ShipId, current: Option<ShipId>, forwards: bool) -> Option<ShipId> {
    let mut ids: Vec<ShipId> = others(w, me).into_iter().map(|(id, _)| id).collect();
    if ids.is_empty() {
        return None;
    }
    if !forwards {
        ids.reverse();
    }
    match current.and_then(|c| ids.iter().position(|&id| id == c)) {
        Some(i) => Some(ids[(i + 1) % ids.len()]),
        None => Some(ids[0]),
    }
}

/// The closest ship whose faction is hostile to `me`.
pub fn nearest_hostile(w: &World, me: ShipId) -> Option<ShipId> {
    let my_faction = w.get(me)?.faction;
    others(w, me)
        .into_iter()
        .filter(|(id, _)| w.rules.hostile(my_faction, w.ships[id.index as usize].faction))
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(id, _)| id)
}

/// The ship nearest the centre of `me`'s view, within `max_angle_deg` of the nose.
pub fn in_sights(w: &World, me: ShipId, max_angle_deg: f32) -> Option<ShipId> {
    let ship = w.get(me)?;
    let fwd = ship.forward();
    let min_cos = max_angle_deg.to_radians().cos();
    others(w, me)
        .into_iter()
        .filter_map(|(id, d)| {
            let to = (w.ships[id.index as usize].position - ship.position) / d.max(1e-3);
            let c = to.dot(fwd);
            (c >= min_cos).then_some((id, c))
        })
        .max_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(id, _)| id)
}

/// Angle in degrees between `me`'s nose and the direction to `other`.
pub fn off_nose_angle(w: &World, me: ShipId, other: ShipId) -> Option<f32> {
    let a = w.get(me)?;
    let b = w.get(other)?;
    let to = (b.position - a.position).normalize_or_zero();
    Some(to.dot(a.forward()).clamp(-1.0, 1.0).acos().to_degrees())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sim::test_support::world;
    use crate::sim::Pilot;
    use glam::{Quat, Vec3};

    fn setup() -> (World, ShipId, ShipId, ShipId, ShipId) {
        let mut w = world();
        let f = w.rules.ship_id("fighter").unwrap();
        let r = w.rules.ship_id("raider").unwrap();
        let blue = w.rules.faction_id("blue").unwrap();
        let red = w.rules.faction_id("red").unwrap();
        let me = w.spawn(f, blue, "me", Vec3::ZERO, Quat::IDENTITY, Pilot::External);
        let friend = w.spawn(f, blue, "friend", Vec3::new(0.0, 0.0, -50.0), Quat::IDENTITY, Pilot::External);
        let far_enemy = w.spawn(r, red, "far", Vec3::new(0.0, 0.0, -1000.0), Quat::IDENTITY, Pilot::External);
        let near_enemy = w.spawn(r, red, "near", Vec3::new(400.0, 0.0, 0.0), Quat::IDENTITY, Pilot::External);
        (w, me, friend, far_enemy, near_enemy)
    }

    #[test]
    fn cycles_forwards_and_backwards_with_wraparound() {
        let (w, me, friend, far, near) = setup();
        assert_eq!(cycle(&w, me, None, true), Some(friend));
        assert_eq!(cycle(&w, me, Some(friend), true), Some(far));
        assert_eq!(cycle(&w, me, Some(near), true), Some(friend), "wraps");
        assert_eq!(cycle(&w, me, Some(friend), false), Some(near));
        assert_eq!(cycle(&w, me, None, false), Some(near));
    }

    #[test]
    fn nearest_hostile_ignores_friends() {
        let (w, me, _, _, near) = setup();
        assert_eq!(nearest_hostile(&w, me), Some(near));
    }

    #[test]
    fn in_sights_picks_the_most_central() {
        let (mut w, me, friend, far, near) = setup();
        // Move the friend 5° off the nose; the far enemy stays dead ahead.
        w.get_mut(friend).unwrap().position = Vec3::new(50.0 * 5f32.to_radians().tan(), 0.0, -50.0);
        assert_eq!(in_sights(&w, me, 10.0), Some(far), "the most central wins, not the nearest");
        assert_eq!(off_nose_angle(&w, me, near).map(|a| a.round()), Some(90.0));
        assert_eq!(off_nose_angle(&w, me, friend).map(|a| a.round()), Some(5.0));
        // With a 2° cone only the far enemy qualifies; with nothing ahead, none.
        assert_eq!(in_sights(&w, me, 2.0), Some(far));
        w.get_mut(far).unwrap().alive = false;
        assert_eq!(in_sights(&w, me, 2.0), None);
    }

    #[test]
    fn nothing_to_target() {
        let mut w = world();
        let f = w.rules.ship_id("fighter").unwrap();
        let me = w.spawn(f, crate::content::rules::FactionId(0), "me", Vec3::ZERO, Quat::IDENTITY, Pilot::External);
        assert_eq!(cycle(&w, me, None, true), None);
        assert_eq!(nearest_hostile(&w, me), None);
        assert_eq!(in_sights(&w, me, 30.0), None);
    }
}
