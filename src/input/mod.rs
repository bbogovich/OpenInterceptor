//! # Input: bindings from keys, mouse and gamepad to flight controls
//!
//! Input handling is split in two so most of it can be unit-tested without a window:
//!
//! 1. The **app** layer reads the real devices each frame and fills a [`RawInput`]
//!    (which keys are held or were just pressed, mouse movement, gamepad state). Keys
//!    are identified by *names* such as `"Up"`, `"Kp7"` or `"F1"`.
//! 2. This module turns `RawInput` into [`Controls`] for the simulation plus a list of
//!    one-shot [`Command`]s (target next, change view, pause…), using [`Bindings`] that
//!    come from an editable INI file.
//!
//! ```text
//!  keyboard ─┐
//!  mouse ────┼─▶ RawInput ──[Bindings]──▶ Controls (sim) + Vec<Command> (app)
//!  gamepad ──┘
//! ```
//!
//! The default keyboard layout follows the original game's documented keys; the docs
//! site's *Original game → Controls* page lists them with their sources.

pub mod gamepad;

use crate::formats::ini::Ini;
use crate::sim::Controls;
use gamepad::{Button, PadState};
use std::collections::{HashMap, HashSet};

/// Every bindable action. Axis actions (`PitchUp`…) are held; the rest fire once.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Action {
    PitchUp,
    PitchDown,
    YawLeft,
    YawRight,
    RollLeft,
    RollRight,
    SlideLeft,
    SlideRight,
    SlideUp,
    SlideDown,
    ThrottleUp,
    ThrottleDown,
    ThrottleFull,
    ThrottleZero,
    Reverse,
    MatchSpeed,
    Afterburner,
    Brake,
    FireBeams,
    FireMissile,
    CycleLink,
    CycleMissile,
    NextTarget,
    PrevTarget,
    NearestEnemy,
    TargetInSights,
    MfdNext,
    MfdPrev,
    ViewCockpit,
    ViewForward,
    ViewLeft,
    ViewRight,
    ViewRearLeft,
    ViewRearRight,
    ViewChase,
    ViewChaseManual,
    ChaseLeft,
    ChaseRight,
    ChaseUp,
    ChaseDown,
    ChaseIn,
    ChaseOut,
    ChaseReset,
    WingFireAtWill,
    WingAttackMyTarget,
    WingFormation,
    NextNav,
    Pause,
    Menu,
    MouseFlight,
}

/// `(name in bindings.ini, action, default keys)`. One table drives parsing, defaults
/// and the documentation, so they cannot drift apart.
pub const ACTIONS: &[(&str, Action, &str)] = &[
    ("pitch_up", Action::PitchUp, "Down"),
    ("pitch_down", Action::PitchDown, "Up"),
    ("yaw_left", Action::YawLeft, "Left"),
    ("yaw_right", Action::YawRight, "Right"),
    ("roll_left", Action::RollLeft, "Kp7, Q"),
    ("roll_right", Action::RollRight, "Kp9, E"),
    ("slide_left", Action::SlideLeft, "Kp4"),
    ("slide_right", Action::SlideRight, "Kp6"),
    ("slide_up", Action::SlideUp, "Kp8"),
    ("slide_down", Action::SlideDown, "Kp2"),
    ("throttle_up", Action::ThrottleUp, "Equal, KpAdd"),
    ("throttle_down", Action::ThrottleDown, "Minus, KpSubtract"),
    ("throttle_full", Action::ThrottleFull, "Backspace"),
    ("throttle_zero", Action::ThrottleZero, "Backslash"),
    ("reverse", Action::Reverse, "Z"),
    ("match_speed", Action::MatchSpeed, "M"),
    ("afterburner", Action::Afterburner, "Tab"),
    ("brake", Action::Brake, "B"),
    ("fire_beams", Action::FireBeams, "Enter, LeftControl"),
    ("fire_missile", Action::FireMissile, "Space"),
    ("cycle_link", Action::CycleLink, "X"),
    ("cycle_missile", Action::CycleMissile, "W"),
    ("next_target", Action::NextTarget, "T"),
    ("prev_target", Action::PrevTarget, "Y"),
    ("nearest_enemy", Action::NearestEnemy, "R"),
    ("target_in_sights", Action::TargetInSights, "G"),
    ("mfd_next", Action::MfdNext, "Period"),
    ("mfd_prev", Action::MfdPrev, "Comma"),
    ("view_cockpit", Action::ViewCockpit, "F1"),
    ("view_forward", Action::ViewForward, "F2"),
    ("view_left", Action::ViewLeft, "F3"),
    ("view_right", Action::ViewRight, "F4"),
    ("view_rear_left", Action::ViewRearLeft, "F5"),
    ("view_rear_right", Action::ViewRearRight, "F6"),
    ("view_chase", Action::ViewChase, "F7"),
    ("view_chase_manual", Action::ViewChaseManual, "F8"),
    ("chase_left", Action::ChaseLeft, "Insert"),
    ("chase_right", Action::ChaseRight, "Delete"),
    ("chase_up", Action::ChaseUp, "PageUp"),
    ("chase_down", Action::ChaseDown, "PageDown"),
    ("chase_in", Action::ChaseIn, "Home"),
    ("chase_out", Action::ChaseOut, "End"),
    ("chase_reset", Action::ChaseReset, "Kp5"),
    ("wing_fire_at_will", Action::WingFireAtWill, "Shift+F"),
    ("wing_attack_my_target", Action::WingAttackMyTarget, "Shift+A"),
    ("wing_formation", Action::WingFormation, "Shift+I, Shift+R"),
    ("next_nav", Action::NextNav, "N"),
    ("pause", Action::Pause, "P"),
    ("menu", Action::Menu, "Escape"),
    ("mouse_flight", Action::MouseFlight, "C"),
];

/// `(name in bindings.ini, action, default buttons)` for the gamepad.
pub const PAD_ACTIONS: &[(&str, Action, &str)] = &[
    ("fire_beams", Action::FireBeams, "RightTrigger"),
    ("fire_missile", Action::FireMissile, "LeftTrigger"),
    ("next_target", Action::NextTarget, "RightShoulder"),
    ("prev_target", Action::PrevTarget, "LeftShoulder"),
    ("afterburner", Action::Afterburner, "A"),
    ("brake", Action::Brake, "B"),
    ("nearest_enemy", Action::NearestEnemy, "RightThumb"),
    ("match_speed", Action::MatchSpeed, "X"),
    ("view_chase", Action::ViewChase, "Y"),
    ("throttle_up", Action::ThrottleUp, "DpadUp"),
    ("throttle_down", Action::ThrottleDown, "DpadDown"),
    ("mfd_prev", Action::MfdPrev, "DpadLeft"),
    ("mfd_next", Action::MfdNext, "DpadRight"),
    ("cycle_link", Action::CycleLink, "Back"),
    ("pause", Action::Pause, "Start"),
    ("cycle_missile", Action::CycleMissile, "LeftThumb"),
    ("target_in_sights", Action::TargetInSights, ""),
];

/// A key with optional Shift: `Shift+F` or `F`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyChord {
    pub key: String,
    pub shift: bool,
}

impl KeyChord {
    /// Parses `"Shift+F"`, `"f"`, `" Kp7 "`. Key names are case-insensitive and stored
    /// in the canonical capitalisation used by [`RawInput`] (first letter upper-case).
    pub fn parse(s: &str) -> Option<KeyChord> {
        let s = s.trim();
        if s.is_empty() {
            return None;
        }
        let (shift, key) = match s.split_once('+') {
            Some((m, k)) if m.trim().eq_ignore_ascii_case("shift") => (true, k.trim()),
            Some(_) => return None,
            None => (false, s),
        };
        Some(KeyChord { key: canonical(key), shift })
    }
}

/// Canonical key name: case-insensitive input, stored lower-case.
pub fn canonical(key: &str) -> String {
    key.trim().to_ascii_lowercase()
}

/// Key and button bindings, plus analogue tuning.
#[derive(Debug, Clone, PartialEq)]
pub struct Bindings {
    pub keys: HashMap<Action, Vec<KeyChord>>,
    pub pad: HashMap<Action, Vec<Button>>,
    /// Gamepad stick dead zone, 0..1.
    pub dead_zone: f32,
    /// Mouse pixels per full stick deflection.
    pub mouse_range: f32,
    /// Push the stick forward to dive (aircraft style). False inverts.
    pub pad_invert_pitch: bool,
}

impl Default for Bindings {
    fn default() -> Self {
        Self::from_ini(&Ini::default())
    }
}

impl Bindings {
    /// Builds bindings from `bindings.ini` text; anything not mentioned keeps its default.
    ///
    /// ```ini
    /// [keyboard]
    /// fire_beams = Enter, Space
    /// [gamepad]
    /// fire_beams = RightTrigger
    /// dead_zone = 0.15
    /// ```
    pub fn from_ini(ini: &Ini) -> Bindings {
        let kb = ini.section("keyboard");
        let gp = ini.section("gamepad");
        let mut keys = HashMap::new();
        for (name, action, default) in ACTIONS {
            let spec = kb.and_then(|s| s.get(name)).unwrap_or(default);
            keys.insert(*action, spec.split(',').filter_map(KeyChord::parse).collect());
        }
        let mut pad = HashMap::new();
        for (name, action, default) in PAD_ACTIONS {
            let spec = gp.and_then(|s| s.get(name)).unwrap_or(default);
            pad.insert(*action, spec.split(',').filter_map(Button::parse).collect());
        }
        Bindings {
            keys,
            pad,
            dead_zone: gp.and_then(|s| s.get_f32("dead_zone")).unwrap_or(0.15).clamp(0.0, 0.9),
            mouse_range: ini.get("mouse", "range").and_then(|v| v.parse().ok()).unwrap_or(200.0f32).max(10.0),
            pad_invert_pitch: gp.and_then(|s| s.get_bool("invert_pitch")).unwrap_or(false),
        }
    }

    /// Writes the full bindings as INI text (used to create a user's bindings file).
    pub fn to_ini_text(&self) -> String {
        let mut out =
            String::from("; OpenInterceptor controls. Edit and restart. Key names: see the docs site.\n[keyboard]\n");
        for (name, action, _) in ACTIONS {
            let keys: Vec<String> = self.keys[action]
                .iter()
                .map(|k| if k.shift { format!("Shift+{}", k.key) } else { k.key.clone() })
                .collect();
            out += &format!("{name} = {}\n", keys.join(", "));
        }
        out += "\n[gamepad]\n";
        for (name, action, _) in PAD_ACTIONS {
            let b: Vec<&str> = self.pad[action].iter().map(|b| b.name()).collect();
            out += &format!("{name} = {}\n", b.join(", "));
        }
        out += &format!(
            "dead_zone = {}\ninvert_pitch = {}\n\n[mouse]\nrange = {}\n",
            self.dead_zone,
            if self.pad_invert_pitch { "yes" } else { "no" },
            self.mouse_range
        );
        out
    }
}

/// One frame of raw device state, filled in by the app layer.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct RawInput {
    /// Canonical (lower-case) names of keys held down.
    pub held: HashSet<String>,
    /// Keys that went down this frame.
    pub pressed: HashSet<String>,
    pub shift: bool,
    /// Mouse offset from the screen centre, pixels (used while mouse flight is on).
    pub mouse_offset: (f32, f32),
    pub mouse_left: bool,
    pub mouse_right_pressed: bool,
    pub pad: Option<PadState>,
    /// The pad as it was last frame, for detecting button presses.
    pub pad_previous: Option<PadState>,
}

/// One-shot requests that are not part of [`Controls`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    NextTarget,
    PrevTarget,
    NearestEnemy,
    TargetInSights,
    MfdNext,
    MfdPrev,
    View(View),
    WingFireAtWill,
    WingAttackMyTarget,
    WingFormation,
    NextNav,
    Pause,
    Menu,
    MouseFlight,
}

/// Camera views (F1–F8).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Cockpit,
    Forward,
    Left,
    Right,
    RearLeft,
    RearRight,
    Chase,
    ChaseManual,
}

impl View {
    /// Parses a view name as used on the command line (`--view chase`).
    pub fn parse(s: &str) -> Option<View> {
        Some(match s.to_ascii_lowercase().replace('_', "-").as_str() {
            "cockpit" => View::Cockpit,
            "forward" => View::Forward,
            "left" => View::Left,
            "right" => View::Right,
            "rear-left" => View::RearLeft,
            "rear-right" => View::RearRight,
            "chase" => View::Chase,
            "chase-manual" | "orbit" => View::ChaseManual,
            _ => return None,
        })
    }
}

/// Chase-camera adjustment for this frame: (yaw, pitch, zoom) in −1..1 and a reset flag.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct ChaseInput {
    pub yaw: f32,
    pub pitch: f32,
    pub zoom: f32,
    pub reset: bool,
}

/// Throttle change per press (10 %, as in the original).
pub const THROTTLE_STEP: f32 = 0.1;

impl Bindings {
    fn held(&self, raw: &RawInput, a: Action) -> bool {
        let keys = self.keys.get(&a).map(Vec::as_slice).unwrap_or(&[]);
        let key_held = keys.iter().any(|k| raw.held.contains(&k.key) && (!k.shift || raw.shift));
        let pad_held = match (&raw.pad, self.pad.get(&a)) {
            (Some(p), Some(buttons)) => buttons.iter().any(|b| p.button(*b)),
            _ => false,
        };
        key_held || pad_held
    }

    fn pressed(&self, raw: &RawInput, a: Action) -> bool {
        let keys = self.keys.get(&a).map(Vec::as_slice).unwrap_or(&[]);
        // A Shift chord only fires with Shift held; a plain key only fires without it,
        // so `F` and `Shift+F` can do different things.
        let key = keys.iter().any(|k| raw.pressed.contains(&k.key) && k.shift == raw.shift);
        let pad = match (&raw.pad, self.pad.get(&a)) {
            (Some(p), Some(buttons)) => {
                let prev = raw.pad_previous.unwrap_or_default();
                buttons.iter().any(|b| p.button(*b) && !prev.button(*b))
            }
            _ => false,
        };
        key || pad
    }

    /// Converts one frame of raw input. `mouse_flight` enables mouse steering.
    pub fn translate(&self, raw: &RawInput, mouse_flight: bool) -> (Controls, Vec<Command>, ChaseInput) {
        let axis = |neg: Action, pos: Action| (self.held(raw, pos) as i32 - self.held(raw, neg) as i32) as f32;
        let mut c = Controls {
            pitch: axis(Action::PitchDown, Action::PitchUp),
            yaw: axis(Action::YawLeft, Action::YawRight),
            roll: axis(Action::RollLeft, Action::RollRight),
            slide: glam::Vec2::new(
                axis(Action::SlideLeft, Action::SlideRight),
                axis(Action::SlideDown, Action::SlideUp),
            ),
            afterburner: self.held(raw, Action::Afterburner),
            brake: self.held(raw, Action::Brake),
            fire_beams: self.held(raw, Action::FireBeams),
            fire_missile: self.pressed(raw, Action::FireMissile),
            cycle_link: self.pressed(raw, Action::CycleLink),
            cycle_missile: self.pressed(raw, Action::CycleMissile),
            toggle_reverse: self.pressed(raw, Action::Reverse),
            match_speed: self.pressed(raw, Action::MatchSpeed),
            ..Default::default()
        };
        if self.pressed(raw, Action::ThrottleUp) {
            c.throttle_delta += THROTTLE_STEP;
        }
        if self.pressed(raw, Action::ThrottleDown) {
            c.throttle_delta -= THROTTLE_STEP;
        }
        if self.pressed(raw, Action::ThrottleFull) {
            c.throttle_set = Some(1.0);
        }
        if self.pressed(raw, Action::ThrottleZero) {
            c.throttle_set = Some(0.0);
        }
        // Gamepad sticks add to (and are clamped with) the keys.
        if let Some(p) = &raw.pad {
            let dz = self.dead_zone;
            let (lx, ly, rx, ry) =
                (dead(p.axis_lx(), dz), dead(p.axis_ly(), dz), dead(p.axis_rx(), dz), dead(p.axis_ry(), dz));
            // Pushing the stick forward (up) dives, like an aircraft joystick.
            c.pitch += if self.pad_invert_pitch { ly } else { -ly };
            c.yaw += lx;
            c.roll += rx;
            c.slide.y += ry;
        }
        // Mouse flight. The original readme maps "Nose Down = Mouse Back, Nose Up =
        // Mouse Up": moving the cursor down the screen lowers the nose (mouse-look
        // style, not an inverted aircraft stick). Screen y grows downwards.
        if mouse_flight {
            let (mx, my) = raw.mouse_offset;
            c.yaw += (mx / self.mouse_range).clamp(-1.0, 1.0);
            c.pitch -= (my / self.mouse_range).clamp(-1.0, 1.0);
            c.fire_beams |= raw.mouse_left;
            c.fire_missile |= raw.mouse_right_pressed;
        }
        c.pitch = c.pitch.clamp(-1.0, 1.0);
        c.yaw = c.yaw.clamp(-1.0, 1.0);
        c.roll = c.roll.clamp(-1.0, 1.0);
        c.slide = c.slide.clamp_length_max(1.0);

        let mut cmds = Vec::new();
        let one_shots = [
            (Action::NextTarget, Command::NextTarget),
            (Action::PrevTarget, Command::PrevTarget),
            (Action::NearestEnemy, Command::NearestEnemy),
            (Action::TargetInSights, Command::TargetInSights),
            (Action::MfdNext, Command::MfdNext),
            (Action::MfdPrev, Command::MfdPrev),
            (Action::ViewCockpit, Command::View(View::Cockpit)),
            (Action::ViewForward, Command::View(View::Forward)),
            (Action::ViewLeft, Command::View(View::Left)),
            (Action::ViewRight, Command::View(View::Right)),
            (Action::ViewRearLeft, Command::View(View::RearLeft)),
            (Action::ViewRearRight, Command::View(View::RearRight)),
            (Action::ViewChase, Command::View(View::Chase)),
            (Action::ViewChaseManual, Command::View(View::ChaseManual)),
            (Action::WingFireAtWill, Command::WingFireAtWill),
            (Action::WingAttackMyTarget, Command::WingAttackMyTarget),
            (Action::WingFormation, Command::WingFormation),
            (Action::NextNav, Command::NextNav),
            (Action::Pause, Command::Pause),
            (Action::Menu, Command::Menu),
            (Action::MouseFlight, Command::MouseFlight),
        ];
        for (a, cmd) in one_shots {
            if self.pressed(raw, a) {
                cmds.push(cmd);
            }
        }
        let chase = ChaseInput {
            yaw: axis(Action::ChaseLeft, Action::ChaseRight),
            pitch: axis(Action::ChaseDown, Action::ChaseUp),
            zoom: axis(Action::ChaseOut, Action::ChaseIn),
            reset: self.pressed(raw, Action::ChaseReset),
        };
        (c, cmds, chase)
    }
}

/// Applies a dead zone and rescales so the output still reaches ±1.
pub fn dead(v: f32, dz: f32) -> f32 {
    if v.abs() <= dz {
        0.0
    } else {
        v.signum() * (v.abs() - dz) / (1.0 - dz)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn raw(held: &[&str], pressed: &[&str]) -> RawInput {
        RawInput {
            held: held.iter().map(|k| canonical(k)).collect(),
            pressed: pressed.iter().map(|k| canonical(k)).collect(),
            ..Default::default()
        }
    }

    #[test]
    fn default_keys_follow_the_original_layout() {
        let b = Bindings::default();
        let (c, _, _) = b.translate(&raw(&["Up", "Right", "Kp7", "Tab", "Enter", "B"], &["Space", "X"]), false);
        assert!(c.brake, "B brakes");
        assert_eq!((c.pitch, c.yaw, c.roll), (-1.0, 1.0, -1.0), "Up = nose down");
        assert!(c.afterburner && c.fire_beams && c.fire_missile && c.cycle_link);
        let (c, _, _) = b.translate(&raw(&[], &["Equal"]), false);
        assert!((c.throttle_delta - 0.1).abs() < 1e-6);
        let (c, _, _) = b.translate(&raw(&[], &["Backspace"]), false);
        assert_eq!(c.throttle_set, Some(1.0));
        let (c, _, _) = b.translate(&raw(&[], &["Backslash", "Minus", "Z", "M"]), false);
        assert_eq!(c.throttle_set, Some(0.0));
        assert!(c.toggle_reverse && c.match_speed && c.throttle_delta < 0.0);
        let (c, _, _) = b.translate(&raw(&["Kp6", "Kp8"], &[]), false);
        assert!(c.slide.x > 0.0 && c.slide.y > 0.0 && c.slide.length() <= 1.0 + 1e-6);
    }

    #[test]
    fn one_shot_commands_and_shift_chords() {
        let b = Bindings::default();
        let (_, cmds, _) = b.translate(&raw(&[], &["T", "F7", "P"]), false);
        assert_eq!(cmds, vec![Command::NextTarget, Command::View(View::Chase), Command::Pause]);
        let mut r = raw(&[], &["F"]);
        assert!(b.translate(&r, false).1.is_empty(), "plain F is unbound");
        r.shift = true;
        assert_eq!(b.translate(&r, false).1, vec![Command::WingFireAtWill]);
        let mut r = raw(&[], &["A", "R"]);
        r.shift = true;
        assert_eq!(b.translate(&r, false).1, vec![Command::WingAttackMyTarget, Command::WingFormation]);
    }

    #[test]
    fn gamepad_sticks_buttons_and_presses() {
        let b = Bindings::default();
        let mut p = PadState { lx: i16::MAX, ly: i16::MAX, rx: i16::MIN, rt: 255, ..Default::default() };
        p.set(Button::RightShoulder, true);
        let r = RawInput { pad: Some(p), ..Default::default() };
        let (c, cmds, _) = b.translate(&r, false);
        assert_eq!((c.yaw, c.pitch, c.roll), (1.0, -1.0, -1.0), "stick forward dives");
        assert!(c.fire_beams);
        assert_eq!(cmds, vec![Command::NextTarget]);
        let held = RawInput { pad: Some(p), pad_previous: Some(p), ..Default::default() };
        assert!(b.translate(&held, false).1.is_empty(), "a held button fires once");
        let small = RawInput { pad: Some(PadState { lx: 1000, ..Default::default() }), ..Default::default() };
        assert_eq!(b.translate(&small, false).0.yaw, 0.0, "inside the dead zone");
    }

    #[test]
    fn mouse_flight_only_when_enabled() {
        let b = Bindings::default();
        let r = RawInput {
            mouse_offset: (100.0, 400.0),
            mouse_left: true,
            mouse_right_pressed: true,
            ..Default::default()
        };
        assert_eq!(b.translate(&r, false).0, Controls::default());
        let (c, _, _) = b.translate(&r, true);
        assert_eq!((c.yaw, c.pitch), (0.5, -1.0), "mouse back (down the screen) = nose down, per the readme");
        assert!(c.fire_beams && c.fire_missile);
    }

    #[test]
    fn chase_camera_keys() {
        let b = Bindings::default();
        let (_, _, chase) = b.translate(&raw(&["Insert", "PageUp", "Home"], &["Kp5"]), false);
        assert_eq!(chase, ChaseInput { yaw: -1.0, pitch: 1.0, zoom: 1.0, reset: true });
    }

    #[test]
    fn bindings_file_overrides_and_round_trips() {
        let ini = Ini::parse("[keyboard]\nfire_beams = Space, Shift+B\nfire_missile = Enter\n[gamepad]\nfire_beams = A\ndead_zone = 0.3\ninvert_pitch = yes\n[mouse]\nrange = 50\n");
        let b = Bindings::from_ini(&ini);
        assert_eq!(
            b.keys[&Action::FireBeams],
            vec![KeyChord::parse("space").unwrap(), KeyChord::parse("shift+b").unwrap()]
        );
        assert_eq!(b.pad[&Action::FireBeams], vec![Button::A]);
        assert_eq!((b.dead_zone, b.mouse_range, b.pad_invert_pitch), (0.3, 50.0, true));
        let again = Bindings::from_ini(&Ini::parse(&b.to_ini_text()));
        assert_eq!(again, b);
    }

    #[test]
    fn view_names_parse() {
        assert_eq!(View::parse("Chase"), Some(View::Chase));
        assert_eq!(View::parse("rear_left"), Some(View::RearLeft));
        assert_eq!(View::parse("orbit"), Some(View::ChaseManual));
        for v in ["cockpit", "forward", "left", "right", "rear-right", "chase-manual"] {
            assert!(View::parse(v).is_some(), "{v}");
        }
        assert_eq!(View::parse("sideways"), None);
    }

    #[test]
    fn chord_parsing_and_dead_zone() {
        assert_eq!(KeyChord::parse(""), None);
        assert_eq!(KeyChord::parse("Ctrl+X"), None);
        assert_eq!(KeyChord::parse(" Kp7 "), Some(KeyChord { key: "kp7".into(), shift: false }));
        assert_eq!(dead(0.1, 0.2), 0.0);
        assert!((dead(1.0, 0.2) - 1.0).abs() < 1e-6);
        assert!((dead(-0.6, 0.2) + 0.5).abs() < 1e-6);
    }
}
