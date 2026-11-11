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
//! | `power` | beam and shield charge settings (A–E), power budget, shield balance and the fore/aft shield levels |
//! | `systems` | one lamp per fitted system: working, damaged, knocked out, destroyed, switched on, being repaired |
//!
//! A cockpit that does not place `power` or `systems` gets them in the top-right corner
//! of the screen.
//!
//! The logic (where a contact goes on the radar, what each MFD page says, which messages
//! are visible) is pure and unit-tested; the `draw_*` functions only paint it.

use crate::content::equipment::{PowerRules, SystemKind};
use crate::content::rules::{FactionId, Ruleset};
use crate::sim::{systems, Condition, LinkMode, ShieldBalance, Ship, ShipId, World};
use glam::{Vec2, Vec3};
use std::collections::HashMap;

/// Width and height of the design space.
pub const DESIGN: Vec2 = Vec2::new(640.0, 480.0);

/// Maps design-space rectangles to screen pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Layout {
    pub scale: f32,
    pub offset: Vec2,
    /// The design space's size (640×480 for the original cockpits).
    pub design: Vec2,
}

impl Layout {
    /// Fits a design space of size `design` inside the screen, as large as possible,
    /// centred. A 4:3 cockpit on a 16:9 screen fills the height with space visible at the
    /// sides; a 16:9 cockpit design fills a 16:9 screen exactly.
    pub fn new(screen: Vec2, design: Vec2) -> Layout {
        let scale = (screen.x / design.x).min(screen.y / design.y);
        Layout { scale, offset: (screen - design * scale) * 0.5, design }
    }

    /// The screen rectangles `[x, y, w, h]` left and right of the design space, from
    /// design-space height `from` down to the bottom of the screen. Empty (zero-width)
    /// when the screen is not wider than the design (R-HUD-013).
    pub fn side_bands(&self, screen: Vec2, from: f32) -> [[f32; 4]; 2] {
        let top = (self.offset.y + from.max(0.0) * self.scale).min(screen.y);
        let h = screen.y - top;
        let left = self.offset.x.max(0.0);
        let right_x = self.offset.x + self.design.x * self.scale;
        [[0.0, top, left, h], [right_x, top, (screen.x - right_x).max(0.0), h]]
    }

    /// The whole design space in screen pixels (where cockpit art is drawn).
    pub fn full(&self) -> [f32; 4] {
        self.rect([0.0, 0.0, self.design.x, self.design.y])
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
        ("power", [252.0, 400.0, 66.0, 72.0]),
        ("systems", [324.0, 400.0, 70.0, 72.0]),
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

/// What the `power` instrument shows.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PowerReadout {
    /// Level indices (0 = A) and the number of levels.
    pub beam: u8,
    pub shield: u8,
    pub levels: u8,
    /// Steps in use and available.
    pub used: u32,
    pub budget: u32,
    pub balance: ShieldBalance,
    /// Fore and aft shields as fractions of their (balanced) capacity.
    pub fore: f32,
    pub aft: f32,
}

/// Reads the ship's power state for the `power` instrument.
pub fn power_readout(rules: &Ruleset, ship: &Ship) -> PowerReadout {
    let c = rules.ship(ship.class);
    let (cap_f, cap_a) = systems::shield_caps(c.shields, ship.power.balance, rules.power.balance_shift);
    let frac = |v: f32, max: f32| if max > 0.0 { (v / max).clamp(0.0, 1.0) } else { 0.0 };
    PowerReadout {
        beam: ship.power.beam,
        shield: ship.power.shield,
        levels: rules.power.levels(),
        used: systems::power_used(ship, rules),
        budget: rules.power.budget as u32,
        balance: ship.power.balance,
        fore: frac(ship.shield_fore, cap_f),
        aft: frac(ship.shield_aft, cap_a),
    }
}

/// How a system lamp is lit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lamp {
    /// Working (green).
    Ok,
    /// Switched on (bright cyan): an active tractor, cloak or ECM.
    On,
    /// Damaged: half effect (amber).
    Damaged,
    /// Knocked out (red); blinks while being repaired.
    Out,
    /// Destroyed for the battle (dark).
    Destroyed,
}

/// Three-letter lamp label for a system kind.
pub fn system_abbrev(kind: SystemKind) -> &'static str {
    match kind {
        SystemKind::Engines => "ENG",
        SystemKind::Manoeuvring => "MAN",
        SystemKind::Sensors => "SEN",
        SystemKind::Shields => "SHD",
        SystemKind::Hyperdrive => "HYP",
        SystemKind::Vct => "VCT",
        SystemKind::Tractor => "TRC",
        SystemKind::Cloak => "CLK",
        SystemKind::Ecm => "ECM",
        SystemKind::DamageControl => "DCC",
    }
}

/// One lamp per fitted system: `(label, lamp, being repaired)`.
pub fn system_lamps(ship: &Ship) -> Vec<(&'static str, Lamp, bool)> {
    ship.systems
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let lamp = match s.condition() {
                Condition::Ok if s.on => Lamp::On,
                Condition::Ok => Lamp::Ok,
                Condition::Damaged if s.on => Lamp::On,
                Condition::Damaged => Lamp::Damaged,
                Condition::Inoperative => Lamp::Out,
                Condition::Destroyed => Lamp::Destroyed,
            };
            (system_abbrev(s.kind), lamp, ship.repairing == Some(i))
        })
        .collect()
}

/// Short status flags for the strip under the message line.
pub fn status_flags(ship: &Ship) -> Vec<String> {
    let mut v = Vec::new();
    if ship.disabled {
        v.push("DISABLED".to_string());
    }
    if ship.cloaked() {
        v.push("CLOAKED".into());
    }
    if ship.is_on(SystemKind::Tractor) {
        v.push("TRACTOR".into());
    }
    if ship.is_on(SystemKind::Ecm) {
        v.push("ECM".into());
    }
    if let Some(t) = ship.hyperspace {
        v.push(format!("JUMP {:.0}", t.ceil()));
    }
    v
}

/// Multi-function display pages, cycled with `<` and `>`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MfdPage {
    Target,
    Weapons,
    Status,
    Systems,
    Nav,
}

impl MfdPage {
    const ALL: [MfdPage; 5] = [MfdPage::Target, MfdPage::Weapons, MfdPage::Status, MfdPage::Systems, MfdPage::Nav];

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
                let mut lines = vec![format!("{} [{}]", c.name, rules.faction(t.faction).name)];
                if t.protected_by == Some(me.faction) {
                    lines.push("OBJECTIVE: PROTECT".into());
                }
                lines.extend([
                    format!("RANGE {}  SPD {:.0}", format_distance(dist), t.forward_speed()),
                    format!("SHD {:.0}/{:.0}  ARM {:.0}", t.shield_fore, t.shield_aft, t.armor),
                    format!("HULL {:.0}%  {lock}", 100.0 * t.hull / c.hull.max(1.0)),
                ]);
                lines
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
            let p = power_readout(rules, me);
            let balance = match p.balance {
                ShieldBalance::Equal => "EVEN",
                ShieldBalance::Forward => "FWD",
                ShieldBalance::Aft => "AFT",
            };
            vec![
                c.name.clone(),
                format!("SHIELD F {:.0} A {:.0} {balance}", me.shield_fore, me.shield_aft),
                format!("ARMOR {:.0}  HULL {:.0}", me.armor, me.hull),
                format!("ENERGY {:.0}/{:.0}", me.energy, c.energy),
                format!(
                    "BEAM {} SHLD {} PWR {}/{}",
                    PowerRules::letter(p.beam),
                    PowerRules::letter(p.shield),
                    p.used,
                    p.budget
                ),
            ]
        }
        MfdPage::Systems => {
            let mut lines = vec!["SYSTEMS".to_string()];
            for (i, s) in me.systems.iter().enumerate() {
                let state = match s.condition() {
                    Condition::Ok if s.on => "ON".to_string(),
                    Condition::Ok => "OK".to_string(),
                    Condition::Damaged => format!("DMG {:.0}%", 100.0 * s.hp / s.max),
                    Condition::Inoperative => "OUT".to_string(),
                    Condition::Destroyed => "DESTROYED".to_string(),
                };
                let fix = if me.repairing == Some(i) { " *" } else { "" };
                lines.push(format!("{} {state}{fix}", system_abbrev(s.kind)));
            }
            lines
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

    /// The newest message, however old (for "repeat last message").
    pub fn last(&self) -> Option<&str> {
        self.entries.last().map(|(_, s)| s.as_str())
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

/// The radar colour of a contact of faction `other` seen by faction `me`: friend, foe
/// or neutral, in the campaign's `flight.radar_colors`.
pub fn relation_color(rules: &Ruleset, me: FactionId, other: FactionId) -> [f32; 4] {
    let c = rules.flight.radar_colors.of(rules.relation(me, other));
    [c[0], c[1], c[2], 1.0]
}

/// The HUD colour of ship `s` for a player of faction `me`: the objective colour for
/// something the player protects, otherwise friend, foe or neutral.
pub fn contact_color(rules: &Ruleset, me: FactionId, s: &crate::sim::Ship) -> [f32; 4] {
    if s.protected_by == Some(me) {
        let c = rules.flight.radar_colors.objective;
        return [c[0], c[1], c[2], 1.0];
    }
    relation_color(rules, me, s.faction)
}

pub mod draw;
pub mod glyphs;
pub mod layout;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sim::test_support::world;
    use crate::sim::Pilot;
    use glam::Quat;

    #[test]
    fn side_bands_cover_the_screen_beside_the_cockpit() {
        // A 640×480 cockpit on a 1920×1080 screen: scale 2.25, art 1440 wide, 240 each side.
        let l = Layout::new(Vec2::new(1920.0, 1080.0), Vec2::new(640.0, 480.0));
        let [left, right] = l.side_bands(Vec2::new(1920.0, 1080.0), 0.0);
        assert_eq!(left, [0.0, 0.0, 240.0, 1080.0]);
        assert_eq!(right, [1680.0, 0.0, 240.0, 1080.0]);
        // From the dashboard's top (design y 305): only the lower part.
        let [left, _] = l.side_bands(Vec2::new(1920.0, 1080.0), 305.0);
        assert!((left[1] - 686.25).abs() < 1e-3 && (left[3] - 393.75).abs() < 1e-3);
        // A 4:3 screen: no bands.
        let l = Layout::new(Vec2::new(1024.0, 768.0), Vec2::new(640.0, 480.0));
        let [left, right] = l.side_bands(Vec2::new(1024.0, 768.0), 0.0);
        assert!(left[2] < 1e-3 && right[2] < 1e-3);
    }

    #[test]
    fn layout_scales_by_height_and_centres() {
        let l = Layout::new(Vec2::new(1920.0, 960.0), DESIGN);
        assert_eq!(l.scale, 2.0);
        assert_eq!(l.offset.x, (1920.0 - 1280.0) / 2.0);
        assert_eq!(l.rect([10.0, 20.0, 30.0, 40.0]), [320.0 + 20.0, 40.0, 60.0, 80.0]);
        assert_eq!(l.full(), [320.0, 0.0, 1280.0, 960.0]);
        // A widescreen design on a 4:3 screen fits the width and centres vertically.
        let w = Layout::new(Vec2::new(1024.0, 768.0), Vec2::new(1920.0, 1080.0));
        assert!((w.scale - 1024.0 / 1920.0).abs() < 1e-6);
        assert!(w.offset.x.abs() < 1e-3 && w.offset.y > 0.0);
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
        assert_eq!(MfdPage::Status.cycle(true), MfdPage::Systems);
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
    fn power_and_system_readouts() {
        let mut w = world();
        let f = w.rules.ship_id("fighter").unwrap();
        let id = w.spawn(f, FactionId(0), "p", Vec3::ZERO, Quat::IDENTITY, Pilot::External);
        let s = w.get_mut(id).unwrap();
        s.power.balance = ShieldBalance::Forward;
        s.shield_fore = 30.0;
        s.systems[0].set_hp(0.0);
        s.repairing = Some(0);
        s.systems[2].set_hp(10.0);
        s.hyperspace = Some(3.2);
        let s = w.get(id).unwrap();
        let p = power_readout(&w.rules, s);
        assert_eq!((p.beam, p.shield, p.levels, p.used, p.budget), (3, 3, 5, 6, 6));
        assert!((p.fore - 0.5).abs() < 1e-6, "30 of a 60-point forward-balanced shield");
        let lamps = system_lamps(s);
        assert_eq!(lamps[0], ("ENG", Lamp::Out, true));
        assert_eq!(lamps[1], ("MAN", Lamp::Ok, false));
        assert_eq!(lamps[2].1, Lamp::Damaged);
        assert_eq!(status_flags(s), vec!["JUMP 4"]);
        let sys = mfd_lines(&w, id, MfdPage::Systems, None);
        assert_eq!(sys[1], "ENG OUT *");
        assert!(sys[3].starts_with("SEN DMG"), "{sys:?}");
        assert!(mfd_lines(&w, id, MfdPage::Status, None)[4].starts_with("BEAM D SHLD D PWR 6/6"));
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
        assert_eq!(default_instruments().len(), 10);
    }
}
