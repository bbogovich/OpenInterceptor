//! # Sound effects
//!
//! Sounds are triggered by the simulation's [`SimEvent`]s, like the visual effects.
//! [`event_sound`] decides *which* sound an event makes and *how loud* (by distance from
//! the listener) — a pure, tested function. [`SoundBank`] owns the decoded sounds and
//! plays them through macroquad.
//!
//! Original sounds are ADPCM-compressed WAVs; [`crate::formats::wav`] decodes them to
//! PCM, which is re-wrapped as a PCM WAV in memory for macroquad's player.

use crate::content::Content;
use crate::sim::{ShipId, SimEvent, World};
use glam::Vec3;
use macroquad::audio::{load_sound_from_bytes, play_sound, PlaySoundParams, Sound};
use std::collections::HashMap;

/// Which sound to play.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SoundKey {
    /// A named event sound from the rules' `[sounds]` section.
    Named(&'static str),
    /// The firing sound of a beam weapon (index into `rules.beams`).
    Beam(u16),
    /// The launch sound of a missile type.
    Missile(u16),
    /// The sound of an effect (index into `rules.effects`).
    Effect(u16),
    /// A radio voice clip (index into `rules.voice_clips`).
    Voice(u32),
}

/// Volume falls off with distance: full within 50 m, half at ~350 m.
pub fn attenuate(distance: f32) -> f32 {
    (1.0 / (1.0 + (distance - 50.0).max(0.0) / 300.0)).clamp(0.0, 1.0)
}

/// The sound for one event heard by `listener` at `ear`, if any.
pub fn event_sound(world: &World, e: &SimEvent, listener: ShipId, ear: Vec3) -> Option<(SoundKey, f32)> {
    let at = |p: Vec3| attenuate(p.distance(ear));
    let mine = |s: &ShipId| *s == listener;
    Some(match e {
        SimEvent::BeamFired { ship, position } => {
            let s = &world.ships[ship.index as usize];
            let beam = world.rules.ship(s.class).beams.first()?;
            (SoundKey::Beam(beam.0), if mine(ship) { 0.6 } else { at(*position) * 0.5 })
        }
        SimEvent::MissileLaunched { ship, position } => {
            let s = &world.ships[ship.index as usize];
            let rack = s.missiles.get(s.selected_rack).or(s.missiles.first())?;
            (SoundKey::Missile(rack.0 .0), if mine(ship) { 0.8 } else { at(*position) })
        }
        SimEvent::Hit { ship, position, shield } => {
            let key = if *shield { "shield_hit" } else { "hull_hit" };
            (SoundKey::Named(key), if mine(ship) { 1.0 } else { at(*position) * 0.7 })
        }
        SimEvent::Destroyed { ship, position, radius } => {
            // The model set's destroyed effect may carry its own sound.
            let s = &world.ships[ship.index as usize];
            let fx = world.rules.model_set(world.rules.ship(s.class).model_set).destroyed_effect;
            let key = match fx.filter(|e| world.rules.effect(*e).sound.is_some()) {
                Some(e) => SoundKey::Effect(e.0),
                None if *radius > 100.0 => SoundKey::Named("big_explosion"),
                None => SoundKey::Named("explosion"),
            };
            (key, at(*position).max(0.15))
        }
        SimEvent::MissileExploded { position, effect } => {
            let key = match effect.filter(|e| world.rules.effect(*e).sound.is_some()) {
                Some(e) => SoundKey::Effect(e.0),
                None => SoundKey::Named("explosion"),
            };
            (key, at(*position) * 0.6)
        }
        SimEvent::PointDefence { from, .. } => (SoundKey::Named("point_defence"), at(*from) * 0.4),
        SimEvent::LockAcquired { ship } if mine(ship) => (SoundKey::Named("lock"), 0.8),
        SimEvent::Collision { position, .. } => (SoundKey::Named("collision"), at(*position)),
        SimEvent::NoEnergy { ship } if mine(ship) => (SoundKey::Named("denied"), 0.6),
        SimEvent::SystemChanged { ship, repaired: false, .. } if mine(ship) => (SoundKey::Named("system_damage"), 0.8),
        SimEvent::Switched { ship, kind, on } => {
            let key = match kind {
                crate::content::equipment::SystemKind::Tractor => "tractor",
                crate::content::equipment::SystemKind::Cloak => "cloak",
                _ => "ecm",
            };
            let pos = world.ships.get(ship.index as usize).map(|s| s.position).unwrap_or(ear);
            (
                SoundKey::Named(key),
                if mine(ship) {
                    if *on {
                        0.8
                    } else {
                        0.5
                    }
                } else {
                    at(pos) * 0.6
                },
            )
        }
        SimEvent::Departed { ship, position } => {
            (SoundKey::Named("hyperspace"), if mine(ship) { 1.0 } else { at(*position) })
        }
        _ => return None,
    })
}

/// Named event sounds the engine knows about.
pub const NAMED: [&str; 18] = [
    "mission_accomplished",
    "mission_failed",
    "explosion",
    "big_explosion",
    "shield_hit",
    "hull_hit",
    "lock",
    "missile_launch",
    "afterburner",
    "target",
    "denied",
    "collision",
    "system_damage",
    "tractor",
    "cloak",
    "ecm",
    "hyperspace",
    "point_defence",
];

/// Decoded, playable sounds.
#[derive(Default)]
pub struct SoundBank {
    sounds: HashMap<SoundKey, Sound>,
    pub errors: Vec<String>,
}

impl SoundBank {
    /// Loads every sound the rules mention. Missing sounds are recorded, not fatal.
    pub async fn load(content: &Content) -> SoundBank {
        let mut bank = SoundBank::default();
        let rules = &content.rules;
        let mut wanted: Vec<(SoundKey, String, usize)> = Vec::new();
        for name in NAMED {
            if let Some((r, p)) = rules.sounds.get(name) {
                wanted.push((SoundKey::Named(name), r.clone(), *p));
            }
        }
        // Effect sounds: an event name from `sounds:`, or an asset in the effect's pack.
        for (i, fx) in rules.effects.iter().enumerate() {
            if let Some(snd) = &fx.sound {
                let (r, p) = rules.sounds.get(snd.as_str()).cloned().unwrap_or((snd.clone(), fx.pack));
                wanted.push((SoundKey::Effect(i as u16), r, p));
            }
        }
        for (i, b) in rules.beams.iter().enumerate() {
            if let Some(s) = &b.sound {
                wanted.push((SoundKey::Beam(i as u16), s.clone(), b.pack));
            }
        }
        for (i, m) in rules.missiles.iter().enumerate() {
            if let Some(s) = &m.sound {
                wanted.push((SoundKey::Missile(i as u16), s.clone(), m.pack));
            }
        }
        for (i, (r, p)) in rules.voice_clips.iter().enumerate() {
            wanted.push((SoundKey::Voice(i as u32), r.clone(), *p));
        }
        for (key, reference, pack) in wanted {
            match content.load_sound(&reference, pack) {
                Ok(pcm) => match load_sound_from_bytes(&pcm.to_wav_bytes()).await {
                    Ok(s) => {
                        bank.sounds.insert(key, s);
                    }
                    Err(e) => bank.errors.push(format!("{reference}: {e:?}")),
                },
                Err(e) => bank.errors.push(e),
            }
        }
        bank
    }

    /// Plays one sound at `volume` (0..1).
    pub fn play(&self, key: &SoundKey, volume: f32) {
        if volume < 0.02 {
            return;
        }
        if let Some(s) = self.sounds.get(key) {
            play_sound(s, PlaySoundParams { looped: false, volume });
        }
    }

    /// Plays the sounds for this tick's events.
    pub fn play_events(&self, world: &World, listener: ShipId, ear: Vec3) {
        for e in &world.events {
            if let Some((key, volume)) = event_sound(world, e, listener, ear) {
                self.play(&key, volume);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::rules::FactionId;
    use crate::sim::test_support::world;
    use crate::sim::Pilot;
    use glam::Quat;

    #[test]
    fn attenuation_falls_with_distance() {
        assert_eq!(attenuate(10.0), 1.0);
        assert!((attenuate(350.0) - 0.5).abs() < 1e-6);
        assert!(attenuate(5000.0) < 0.1);
    }

    #[test]
    fn events_choose_sounds_and_volumes() {
        let mut w = world();
        let f = w.rules.ship_id("fighter").unwrap();
        let me = w.spawn(f, FactionId(0), "me", Vec3::ZERO, Quat::IDENTITY, Pilot::External);
        let other = w.spawn(f, FactionId(1), "o", Vec3::new(0.0, 0.0, -2000.0), Quat::IDENTITY, Pilot::External);
        let ear = Vec3::ZERO;
        let s = |e: SimEvent| event_sound(&w, &e, me, ear);
        assert_eq!(s(SimEvent::BeamFired { ship: me, position: Vec3::ZERO }), Some((SoundKey::Beam(0), 0.6)));
        let far = s(SimEvent::BeamFired { ship: other, position: Vec3::new(0.0, 0.0, -2000.0) }).unwrap();
        assert!(far.1 < 0.1);
        assert_eq!(s(SimEvent::MissileLaunched { ship: me, position: Vec3::ZERO }).unwrap().0, SoundKey::Missile(0));
        assert_eq!(
            s(SimEvent::Hit { ship: me, position: Vec3::ZERO, shield: true }),
            Some((SoundKey::Named("shield_hit"), 1.0))
        );
        assert_eq!(
            s(SimEvent::Hit { ship: me, position: Vec3::ZERO, shield: false }).unwrap().0,
            SoundKey::Named("hull_hit")
        );
        assert_eq!(
            s(SimEvent::Destroyed { ship: other, position: Vec3::ZERO, radius: 500.0 }).unwrap().0,
            SoundKey::Named("big_explosion")
        );
        assert_eq!(
            s(SimEvent::Destroyed { ship: other, position: Vec3::new(0.0, 0.0, -1e6), radius: 5.0 }).unwrap().1,
            0.15,
            "always faintly audible"
        );
        assert_eq!(s(SimEvent::LockAcquired { ship: me }).unwrap().0, SoundKey::Named("lock"));
        assert_eq!(s(SimEvent::LockAcquired { ship: other }), None, "only the listener's lock tone");
        assert_eq!(s(SimEvent::NoEnergy { ship: me }).unwrap().0, SoundKey::Named("denied"));
        assert_eq!(
            s(SimEvent::MissileExploded { position: Vec3::ZERO, effect: None }).unwrap().0,
            SoundKey::Named("explosion")
        );
        assert_eq!(s(SimEvent::Collision { ship: me, position: Vec3::ZERO }).unwrap().0, SoundKey::Named("collision"));
        assert_eq!(s(SimEvent::Message { ship: me, text: "x".into() }), None);
        let dmg =
            SimEvent::SystemChanged { ship: me, system: 0, condition: crate::sim::Condition::Damaged, repaired: false };
        assert_eq!(s(dmg).unwrap().0, SoundKey::Named("system_damage"));
        let on = SimEvent::Switched { ship: me, kind: crate::content::equipment::SystemKind::Cloak, on: true };
        assert_eq!(s(on), Some((SoundKey::Named("cloak"), 0.8)));
        assert_eq!(s(SimEvent::Departed { ship: me, position: Vec3::ZERO }).unwrap().0, SoundKey::Named("hyperspace"));
    }
}
