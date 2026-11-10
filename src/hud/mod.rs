//! # Cockpit and HUD
//!
//! The 2D layer drawn over the 3D view: cockpit art, instruments and target markers.
//!
//! ## Design space
//!
//! The original cockpits are 640×480 images, and cockpit definitions place instruments
//! in that *design space* (`Instrument.radar_front = x, y, w, h`). [`Layout`] maps design
//! space to the real window: it scales by height and centres horizontally, so a 4:3
//! cockpit sits in the middle of a 16:9 screen with the space view visible either side.
//!
//! ## Instruments
//!
//! | Name | Shows |
//! |---|---|
//! | `radar_front`, `radar_rear` | contacts in the front / rear hemisphere, by angle off the nose |
//! | `mfd` | the multi-function display: target, weapons, ship status or navigation page |
//! | `gauge1`–`gauge4` | speed, energy, shields, hull |
//! | `message` | the latest pilot messages |
//!
//! The logic (where a contact goes on the radar, what each MFD page says, which messages
//! are visible) is pure and unit-tested; the `draw_*` functions only paint it.

use crate::content::rules::{FactionId, Ruleset};
use crate::sim::{LinkMode, Ship, ShipId, World};
use glam::{Vec2, Vec3};
use std::collections::HashMap;

/// Width and height of the design space.
pub const DESIGN: Vec2 = Vec2::new(640.0, 480.0);

/// Maps design-space rectangles to screen pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Layout {
    pub scale: f32,
    pub offset: Vec2,
}

impl Layout {
    /// Fit the design space to the screen height, centred horizontally.
    pub fn new(screen: Vec2) -> Layout {
        let scale = screen.y / DESIGN.y;
        Layout { scale, offset: Vec2::new((screen.x - DESIGN.x * scale) * 0.5, 0.0) }
    }

    /// A design rectangle `[x, y, w, h]` in screen pixels.
    pub fn rect(&self, r: [f32; 4]) -> [f32; 4] {
        [self.offset.x + r[0] * self.scale, self.offset.y + r[1] * self.scale, r[2] * self.scale, r[3] * self.scale]
    }
}

/// Instrument rectangles used when a ship has no cockpit art (or in the F2 full view):
/// a compact glass HUD along the bottom of the design space.
pub fn default_instruments() -> HashMap<String, [f32; 4]> {
    [
        ("message", [140.0, 8.0, 360.0, 20.0]),
        ("radar_front", [8.0, 392.0, 80.0, 80.0]),
        ("radar_rear", [552.0, 392.0, 80.0, 80.0]),
        ("mfd", [96.0, 400.0, 150.0, 72.0]),
        ("gauge1", [400.0, 418.0, 34.0, 34.0]),
        ("gauge2", [438.0, 418.0, 34.0, 34.0]),
        ("gauge3", [476.0, 418.0, 34.0, 34.0]),
        ("gauge4", [514.0, 418.0, 34.0, 34.0]),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_string(), v))
    .collect()
}

/// Where a contact appears on a radar scope, as a point in the unit disc (x right, y
/// down, like the screen). `local` is the contact's direction in the viewer's frame
/// (x right, y up, −z ahead). The front scope shows the front hemisphere, the rear scope
/// the rear one (seen looking backwards, so left and right swap). Distance from the
/// centre is the angle off the scope's axis: 0 at the centre, 90° at the rim.
pub fn radar_point(local: Vec3, front: bool) -> Option<Vec2> {
    let d = local.normalize_or_zero();
    if d == Vec3::ZERO {
        return Some(Vec2::ZERO);
    }
    let axis = if front { -d.z } else { d.z };
    if axis < 0.0 {
        return None; // in the other hemisphere
    }
    let angle = axis.clamp(-1.0, 1.0).acos(); // 0..π/2
    let lateral = Vec2::new(if front { d.x } else { -d.x }, -d.y);
    let dir = lateral.normalize_or_zero();
    Some(dir * (angle / std::f32::consts::FRAC_PI_2))
}

/// Needle angle (radians, screen convention: 0 = right, clockwise positive) for a
/// gauge reading `fraction` (0..1): sweeps 270° from bottom-left to bottom-right.
pub fn gauge_angle(fraction: f32) -> f32 {
    let start = 135f32.to_radians();
    start + fraction.clamp(0.0, 1.0) * 270f32.to_radians()
}

/// The four gauge readings for a ship: speed, energy, shields, hull (0..1 each).
pub fn gauge_values(rules: &Ruleset, ship: &Ship) -> [f32; 4] {
    let c = rules.ship(ship.class);
    let frac = |v: f32, max: f32| if max > 0.0 { (v / max).clamp(0.0, 1.0) } else { 0.0 };
    [
        frac(ship.forward_speed().abs(), c.max_speed + rules.flight.afterburner_boost),
        frac(ship.energy, c.energy),
        frac(ship.shield_fore + ship.shield_aft, c.shields * 2.0),
        frac(ship.hull, c.hull),
    ]
}

/// Multi-function display pages, cycled with `<` and `>`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MfdPage {
    Target,
    Weapons,
    Status,
    Nav,
}

impl MfdPage {
    const ALL: [MfdPage; 4] = [MfdPage::Target, MfdPage::Weapons, MfdPage::Status, MfdPage::Nav];

    /// The next (or previous) page, wrapping.
    pub fn cycle(self, forwards: bool) -> MfdPage {
        let i = Self::ALL.iter().position(|p| *p == self).unwrap_or(0) as i32;
        let n = Self::ALL.len() as i32;
        Self::ALL[((i + if forwards { 1 } else { -1 }).rem_euclid(n)) as usize]
    }
}

/// Text lines for an MFD page. `nav` is the selected nav point, if any.
pub fn mfd_lines(world: &World, player: ShipId, page: MfdPage, nav: Option<(&str, Vec3)>) -> Vec<String> {
    let Some(me) = world.get(player) else { return vec!["-- NO SIGNAL --".into()] };
    let rules = &world.rules;
    match page {
        MfdPage::Target => match me.target.and_then(|t| world.get(t)) {
            None => vec!["TARGET".into(), "none".into(), "T next  R nearest".into()],
            Some(t) => {
                let c = rules.ship(t.class);
                let dist = t.position.distance(me.position);
                let lock = if me.lock.locked {
                    "LOCKED"
                } else if me.lock.progress > 0.0 {
                    "locking"
                } else {
                    ""
                };
                vec![
                    format!("{} [{}]", c.name, rules.faction(t.faction).name),
                    format!("RANGE {}  SPD {:.0}", format_distance(dist), t.forward_speed()),
                    format!("SHD {:.0}/{:.0}  ARM {:.0}", t.shield_fore, t.shield_aft, t.armor),
                    format!("HULL {:.0}%  {lock}", 100.0 * t.hull / c.hull.max(1.0)),
                ]
            }
        },
        MfdPage::Weapons => {
            let c = rules.ship(me.class);
            let mut lines = vec![format!("BEAMS x{}  {}", c.beams.len(), link_name(me.link_mode))];
            if let Some(b) = c.beams.first() {
                lines.push(rules.beam(*b).name.clone());
            }
            for (i, (m, n)) in me.missiles.iter().enumerate() {
                let mark = if i == me.selected_rack { ">" } else { " " };
                lines.push(format!("{mark}{} x{n}", rules.missile(*m).name));
            }
            lines
        }
        MfdPage::Status => {
            let c = rules.ship(me.class);
            vec![
                c.name.clone(),
                format!("SHIELD F {:.0} A {:.0}", me.shield_fore, me.shield_aft),
                format!("ARMOR {:.0}  HULL {:.0}", me.armor, me.hull),
                format!("ENERGY {:.0}/{:.0}", me.energy, c.energy),
            ]
        }
        MfdPage::Nav => match nav {
            None => vec!["NAV".into(), "no nav points".into()],
            Some((name, pos)) => {
                vec!["NAV".into(), name.to_uppercase(), format!("RANGE {}", format_distance(pos.distance(me.position)))]
            }
        },
    }
}

fn link_name(m: LinkMode) -> &'static str {
    match m {
        LinkMode::Linked => "LINKED",
        LinkMode::Full => "FULL",
        LinkMode::Unlinked => "UNLINKED",
    }
}

/// `850 m` or `12.4 km`.
pub fn format_distance(m: f32) -> String {
    if m < 1000.0 {
        format!("{m:.0} m")
    } else {
        format!("{:.1} km", m / 1000.0)
    }
}

/// Recent pilot messages with timestamps.
#[derive(Debug, Default)]
pub struct MessageLog {
    entries: Vec<(f64, String)>,
}

/// Seconds a message stays on the message line.
pub const MESSAGE_TIME: f64 = 4.0;

impl MessageLog {
    pub fn push(&mut self, now: f64, text: impl Into<String>) {
        self.entries.push((now, text.into()));
        if self.entries.len() > 32 {
            self.entries.remove(0);
        }
    }

    /// Up to `n` messages younger than [`MESSAGE_TIME`], newest last.
    pub fn visible(&self, now: f64, n: usize) -> Vec<&str> {
        let recent: Vec<&str> =
            self.entries.iter().filter(|(t, _)| now - t < MESSAGE_TIME).map(|(_, s)| s.as_str()).collect();
        recent[recent.len().saturating_sub(n)..].to_vec()
    }
}

/// Faction colour as RGBA floats.
pub fn faction_color(rules: &Ruleset, f: FactionId) -> [f32; 4] {
    let c = rules.faction(f).color;
    [c[0], c[1], c[2], 1.0]
}

pub mod draw;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sim::test_support::world;
    use crate::sim::Pilot;
    use glam::Quat;

    #[test]
    fn layout_scales_by_height_and_centres() {
        let l = Layout::new(Vec2::new(1920.0, 960.0));
        assert_eq!(l.scale, 2.0);
        assert_eq!(l.offset.x, (1920.0 - 1280.0) / 2.0);
        assert_eq!(l.rect([10.0, 20.0, 30.0, 40.0]), [320.0 + 20.0, 40.0, 60.0, 80.0]);
    }

    #[test]
    fn radar_places_contacts_by_angle() {
        assert_eq!(radar_point(Vec3::NEG_Z, true), Some(Vec2::ZERO), "dead ahead: centre");
        let right = radar_point(Vec3::X, true).unwrap();
        assert!((right - Vec2::X).length() < 1e-5, "90° right: right rim");
        let up45 = radar_point(Vec3::new(0.0, 1.0, -1.0), true).unwrap();
        assert!((up45 - Vec2::new(0.0, -0.5)).length() < 1e-5, "45° up: half-way, screen-up");
        assert_eq!(radar_point(Vec3::Z, true), None, "behind is not on the front scope");
        let behind_right = radar_point(Vec3::new(1.0, 0.0, 1.0), false).unwrap();
        assert!(behind_right.x < 0.0, "rear scope mirrors left/right");
        assert_eq!(radar_point(Vec3::ZERO, true), Some(Vec2::ZERO));
    }

    #[test]
    fn gauges_sweep_and_read_the_ship() {
        assert_eq!(gauge_angle(0.0), 135f32.to_radians());
        assert!((gauge_angle(1.0) - 405f32.to_radians()).abs() < 1e-6);
        assert_eq!(gauge_angle(5.0), gauge_angle(1.0));
        let mut w = world();
        let f = w.rules.ship_id("fighter").unwrap();
        let id = w.spawn(f, FactionId(0), "p", Vec3::ZERO, Quat::IDENTITY, Pilot::External);
        w.get_mut(id).unwrap().hull = 50.0;
        let g = gauge_values(&w.rules, w.get(id).unwrap());
        assert_eq!(g, [0.0, 1.0, 1.0, 0.5]);
    }

    #[test]
    fn mfd_pages_cycle_and_describe() {
        assert_eq!(MfdPage::Target.cycle(true), MfdPage::Weapons);
        assert_eq!(MfdPage::Target.cycle(false), MfdPage::Nav);
        let mut w = world();
        let f = w.rules.ship_id("fighter").unwrap();
        let r = w.rules.ship_id("raider").unwrap();
        let me = w.spawn(f, FactionId(0), "p", Vec3::ZERO, Quat::IDENTITY, Pilot::External);
        let foe = w.spawn(r, FactionId(1), "f", Vec3::new(0.0, 0.0, -1500.0), Quat::IDENTITY, Pilot::External);
        assert_eq!(mfd_lines(&w, me, MfdPage::Target, None)[1], "none");
        w.get_mut(me).unwrap().target = Some(foe);
        let t = mfd_lines(&w, me, MfdPage::Target, None);
        assert!(t[0].starts_with("raider") && t[1].contains("1.5 km"), "{t:?}");
        let wp = mfd_lines(&w, me, MfdPage::Weapons, None);
        assert!(wp[0].contains("LINKED") && wp[2].starts_with(">dart x2"), "{wp:?}");
        assert!(mfd_lines(&w, me, MfdPage::Status, None)[3].starts_with("ENERGY"));
        assert_eq!(mfd_lines(&w, me, MfdPage::Nav, Some(("alpha", Vec3::new(0.0, 0.0, -500.0))))[2], "RANGE 500 m");
        assert_eq!(mfd_lines(&w, me, MfdPage::Nav, None)[1], "no nav points");
        w.get_mut(me).unwrap().alive = false;
        assert_eq!(mfd_lines(&w, me, MfdPage::Status, None), vec!["-- NO SIGNAL --".to_string()]);
    }

    #[test]
    fn messages_expire_and_keep_the_newest() {
        let mut log = MessageLog::default();
        log.push(0.0, "old");
        log.push(3.0, "a");
        log.push(3.5, "b");
        assert_eq!(log.visible(4.5, 5), vec!["a", "b"]);
        assert_eq!(log.visible(4.5, 1), vec!["b"]);
        for i in 0..40 {
            log.push(10.0, format!("{i}"));
        }
        assert_eq!(log.visible(10.0, 100).len(), 32, "bounded history");
        assert_eq!(format_distance(12_400.0), "12.4 km");
        assert_eq!(default_instruments().len(), 8);
    }
}
