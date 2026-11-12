//! # Input: bindings from keys, mouse and gamepad to flight controls
//!
//! Input handling is split in two so most of it can be unit-tested without a window:
//!
//! 1. The **app** layer reads the real devices each frame and fills a [`RawInput`]
//!    (which keys are held or were just pressed, mouse movement, gamepad state). Keys
//!    are identified by *names* such as `"Up"`, `"Kp7"` or `"F1"`.
//! 2. This module turns `RawInput` into [`Controls`] for the simulation plus a list of
//!    one-shot [`Command`]s (target next, change view, pause…), using [`Bindings`] that
//!    come from an editable INI file — and from the in-game controls menu, whose model
//!    lives in [`editor`].
//!
//! ```text
//!  keyboard ─┐
//!  mouse ────┼─▶ RawInput ──[Bindings]──▶ Controls (sim) + Vec<Command> (app)
//!  gamepad ──┘
//! ```
//!
//! ## Chords
//!
//! A keyboard binding may need Shift (`Shift+F`), and a gamepad binding may need a
//! **modifier button** held (`LeftShoulder+A`). A controller has far fewer buttons than
//! a keyboard has keys, so modifier chords are what make *every* action reachable from
//! the pad. A button used as a modifier anywhere does nothing on its own while it is held:
//! holding it switches the pad to its second layer.
//!
//! The default keyboard layout follows the original game's documented keys; the docs
//! site's *Original game → Controls* page lists them with their sources.

pub mod editor;
pub mod gamepad;

use crate::formats::ini::Ini;
use crate::sim::{Controls, Requests};
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
    BeamCharge,
    ShieldCharge,
    PowerToShields,
    PowerToBeams,
    ShieldBalance,
    Tractor,
    Cloak,
    Ecm,
    Hyperspace,
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
    WingMenu,
    WingFireAtWill,
    WingAttackMyTarget,
    WingFormation,
    WingEvasive,
    WingRetreat,
    WingStatus,
    WingNextFormation,
    RepeatMessage,
    NextNav,
    Pause,
    Menu,
    MouseFlight,
    Help,
}

/// Groups for the controls menu and the docs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Group {
    Flight,
    Throttle,
    Weapons,
    Targeting,
    Power,
    Wingmen,
    Views,
    Camera,
    Interface,
}

impl Group {
    pub fn title(self) -> &'static str {
        match self {
            Group::Flight => "Flight",
            Group::Throttle => "Throttle and speed",
            Group::Weapons => "Weapons",
            Group::Targeting => "Targeting",
            Group::Power => "Power and systems",
            Group::Wingmen => "Wingmen and comms",
            Group::Views => "Views",
            Group::Camera => "Chase camera",
            Group::Interface => "Interface",
        }
    }
}

/// Everything known about one action: its `bindings.yaml` name, defaults, menu group,
/// label and description. One table drives parsing, defaults, the controls menu, the keys
/// help, the remote console and the documentation, so they cannot drift apart.
#[derive(Debug, Clone, Copy)]
pub struct ActionInfo {
    /// The action's name in `bindings.yaml` and on the console protocol (`fire_beams`).
    pub name: &'static str,
    /// The engine [`Action`] this entry describes.
    pub action: Action,
    /// Default keys, comma-separated (`Shift+` for chords).
    pub keys: &'static str,
    /// Default gamepad buttons, comma-separated (`Modifier+Button` for chords).
    pub pad: &'static str,
    /// The menu group the action is listed under.
    pub group: Group,
    /// A short name for buttons and menus ("Fire beams").
    pub label: &'static str,
    /// One sentence telling a player what the action does. The keys help shows it under
    /// the label, and the remote console shows it on the action's button.
    pub description: &'static str,
}

/// Builds one [`ActionInfo`]. A `const fn` can run at compile time, which is what lets
/// [`ACTIONS`] be a `const` table: it is built into the program, not at start-up.
const fn a(
    name: &'static str,
    action: Action,
    keys: &'static str,
    pad: &'static str,
    group: Group,
    label: &'static str,
    description: &'static str,
) -> ActionInfo {
    ActionInfo { name, action, keys, pad, group, label, description }
}

use Action as A;
use Group as G;

/// The action table (see [`ActionInfo`]). Gamepad defaults use the left shoulder as the
/// modifier for the second layer.
// One action per line reads as a table, so `cargo fmt` is told to leave it alone
// (`rustfmt::skip`); otherwise it would split each long entry over nine lines.
#[rustfmt::skip]
pub const ACTIONS: &[ActionInfo] = &[
    a("pitch_up", A::PitchUp, "Down", "", G::Flight, "Nose up", "Pulls the nose up to climb."),
    a("pitch_down", A::PitchDown, "Up", "", G::Flight, "Nose down", "Pushes the nose down to dive."),
    a("yaw_left", A::YawLeft, "Left", "", G::Flight, "Turn left", "Swings the nose to the left."),
    a("yaw_right", A::YawRight, "Right", "", G::Flight, "Turn right", "Swings the nose to the right."),
    a("roll_left", A::RollLeft, "Kp7, Q", "", G::Flight, "Roll left", "Rolls the craft anticlockwise about its nose."),
    a("roll_right", A::RollRight, "Kp9, E", "", G::Flight, "Roll right", "Rolls the craft clockwise about its nose."),
    a("slide_left", A::SlideLeft, "Kp4", "", G::Flight, "Slide left (VCT)", "Thrusts sideways to the left without turning."),
    a("slide_right", A::SlideRight, "Kp6", "", G::Flight, "Slide right (VCT)", "Thrusts sideways to the right without turning."),
    a("slide_up", A::SlideUp, "Kp8", "", G::Flight, "Slide up (VCT)", "Thrusts straight up without turning."),
    a("slide_down", A::SlideDown, "Kp2", "", G::Flight, "Slide down (VCT)", "Thrusts straight down without turning."),
    a("throttle_up", A::ThrottleUp, "Equal, KpAdd", "DpadUp", G::Throttle, "Throttle up 10%", "Raises the set speed by a tenth of maximum."),
    a("throttle_down", A::ThrottleDown, "Minus, KpSubtract", "DpadDown", G::Throttle, "Throttle down 10%", "Lowers the set speed by a tenth of maximum."),
    a("throttle_full", A::ThrottleFull, "Backspace", "", G::Throttle, "Full throttle", "Sets the throttle to maximum speed."),
    a("throttle_zero", A::ThrottleZero, "Backslash", "", G::Throttle, "Zero throttle", "Sets the throttle to zero so the craft coasts to a stop."),
    a("reverse", A::Reverse, "Z", "", G::Throttle, "Reverse thrust", "Reverses thrust to fly backwards."),
    a("match_speed", A::MatchSpeed, "M", "X", G::Throttle, "Match target speed", "Sets your speed to match the current target's."),
    a("afterburner", A::Afterburner, "Tab", "A", G::Throttle, "Afterburner (hold)", "Burns extra fuel for a burst of speed while held."),
    a("brake", A::Brake, "B", "B", G::Throttle, "Brake (hold)", "Slows the craft down quickly while held."),
    a("fire_beams", A::FireBeams, "Enter, LeftControl", "RightTrigger", G::Weapons, "Fire beams", "Fires the beam weapons while held."),
    a("fire_missile", A::FireMissile, "Space", "LeftTrigger", G::Weapons, "Fire missile", "Launches the selected missile at the target."),
    a("cycle_link", A::CycleLink, "X", "Back", G::Weapons, "Beam link mode", "Switches between firing beams together or one at a time."),
    a("cycle_missile", A::CycleMissile, "W", "LeftThumb", G::Weapons, "Next missile type", "Selects the next missile type carried."),
    a("next_target", A::NextTarget, "T", "RightShoulder", G::Targeting, "Next target", "Locks on to the next contact in range."),
    a("prev_target", A::PrevTarget, "Y", "LeftShoulder+RightShoulder", G::Targeting, "Previous target", "Locks on to the previous contact in range."),
    a("nearest_enemy", A::NearestEnemy, "R", "RightThumb", G::Targeting, "Nearest enemy", "Locks on to the closest hostile craft."),
    a("target_in_sights", A::TargetInSights, "G", "LeftShoulder+RightThumb", G::Targeting, "Target in sights", "Locks on to the contact nearest your crosshair."),
    a("beam_charge", A::BeamCharge, "F9", "LeftShoulder+DpadLeft", G::Power, "Beam charge rate (A-E)", "Steps the beam recharge rate through settings A to E."),
    a("shield_charge", A::ShieldCharge, "F10", "LeftShoulder+DpadRight", G::Power, "Shield charge rate (A-E)", "Steps the shield recharge rate through settings A to E."),
    a("power_to_shields", A::PowerToShields, "Semicolon", "LeftShoulder+DpadDown", G::Power, "Power to shields", "Moves energy from the beams into the shields."),
    a("power_to_beams", A::PowerToBeams, "Apostrophe", "LeftShoulder+DpadUp", G::Power, "Power to beams", "Moves energy from the shields into the beams."),
    a("shield_balance", A::ShieldBalance, "S", "LeftShoulder+B", G::Power, "Shield balance", "Spreads shield strength evenly front and rear."),
    a("tractor", A::Tractor, "Shift+T", "LeftShoulder+X", G::Power, "Tractor beam", "Switches the tractor beam on the target on or off."),
    a("cloak", A::Cloak, "Shift+C", "LeftShoulder+Y", G::Power, "Cloaking device", "Switches the cloaking device on or off."),
    a("ecm", A::Ecm, "Shift+J", "LeftShoulder+Back", G::Power, "ECM", "Switches the missile-jamming countermeasures on or off."),
    a("hyperspace", A::Hyperspace, "H", "LeftShoulder+A", G::Power, "Hyperspace out / abort", "Jumps out of the battle, or cancels a jump in progress."),
    a("mfd_next", A::MfdNext, "Period", "DpadRight", G::Interface, "Next MFD page", "Shows the next page on the cockpit display."),
    a("mfd_prev", A::MfdPrev, "Comma", "DpadLeft", G::Interface, "Previous MFD page", "Shows the previous page on the cockpit display."),
    a("view_cockpit", A::ViewCockpit, "F1", "", G::Views, "Cockpit view", "Looks forward from inside the cockpit."),
    a("view_forward", A::ViewForward, "F2", "", G::Views, "Full forward view", "Looks forward with the cockpit frame hidden."),
    a("view_left", A::ViewLeft, "F3", "", G::Views, "Left view", "Looks out of the left side of the craft."),
    a("view_right", A::ViewRight, "F4", "", G::Views, "Right view", "Looks out of the right side of the craft."),
    a("view_rear_left", A::ViewRearLeft, "F5", "", G::Views, "Rear-left view", "Looks back over the left shoulder."),
    a("view_rear_right", A::ViewRearRight, "F6", "", G::Views, "Rear-right view", "Looks back over the right shoulder."),
    a("view_chase", A::ViewChase, "F7", "Y", G::Views, "Chase view", "Watches your craft from a camera following behind."),
    a("view_chase_manual", A::ViewChaseManual, "F8", "", G::Views, "Manual chase view", "Watches your craft from a camera you steer yourself."),
    a("chase_left", A::ChaseLeft, "Insert", "", G::Camera, "Orbit left", "Moves the chase camera round to the left."),
    a("chase_right", A::ChaseRight, "Delete", "", G::Camera, "Orbit right", "Moves the chase camera round to the right."),
    a("chase_up", A::ChaseUp, "PageUp", "", G::Camera, "Orbit up", "Moves the chase camera up over the craft."),
    a("chase_down", A::ChaseDown, "PageDown", "", G::Camera, "Orbit down", "Moves the chase camera down under the craft."),
    a("chase_in", A::ChaseIn, "Home", "", G::Camera, "Zoom in", "Moves the chase camera closer to the craft."),
    a("chase_out", A::ChaseOut, "End", "", G::Camera, "Zoom out", "Moves the chase camera further from the craft."),
    a("chase_reset", A::ChaseReset, "Kp5", "", G::Camera, "Reset camera", "Puts the chase camera back behind the craft."),
    a("wing_menu", A::WingMenu, "RightBracket", "LeftShoulder+LeftThumb", G::Wingmen, "Wingman menu", "Opens the list of orders for your wingmen."),
    a("wing_fire_at_will", A::WingFireAtWill, "Shift+F", "", G::Wingmen, "Wingmen: fire at will", "Lets your wingmen pick and attack their own targets."),
    a("wing_attack_my_target", A::WingAttackMyTarget, "Shift+A", "", G::Wingmen, "Wingmen: attack my target", "Orders your wingmen to attack your current target."),
    a("wing_formation", A::WingFormation, "Shift+I", "", G::Wingmen, "Wingmen: ignore my target (form up)", "Orders your wingmen to leave your target and form up on you."),
    a("wing_evasive", A::WingEvasive, "Shift+E", "", G::Wingmen, "Wingmen: evasive", "Orders your wingmen to break off and dodge enemy fire."),
    a("wing_retreat", A::WingRetreat, "Shift+R", "", G::Wingmen, "Wingmen: retreat", "Orders your wingmen to leave the battle."),
    a("wing_status", A::WingStatus, "Shift+S", "", G::Wingmen, "Wingmen: status report", "Asks your wingmen to report their condition."),
    a("wing_next_formation", A::WingNextFormation, "Shift+O", "", G::Wingmen, "Wingmen: next formation", "Changes the shape your wingmen fly in."),
    a("repeat_message", A::RepeatMessage, "L", "", G::Wingmen, "Repeat last message", "Shows the last radio message again."),
    a("next_nav", A::NextNav, "N", "", G::Interface, "Next nav point", "Moves the navigation marker to the next nav point."),
    a("pause", A::Pause, "P", "Start", G::Interface, "Pause", "Pauses or resumes the battle."),
    a("menu", A::Menu, "Escape", "LeftShoulder+Start", G::Interface, "Menu", "Opens the in-flight menu."),
    a("mouse_flight", A::MouseFlight, "C", "", G::Interface, "Mouse flight on/off", "Switches steering with the mouse on or off."),
    a("help", A::Help, "Shift+Slash, Shift+F1", "", G::Interface, "Keys help (this list)", "Shows or hides this list of keys."),
];

/// One row of the keys help: an action's label and description, and the player's current
/// keys for it.
#[derive(Debug, Clone, PartialEq)]
pub struct HelpRow {
    /// The action's short label ("Fire beams").
    pub label: &'static str,
    /// What the action does, in one sentence.
    pub description: &'static str,
    /// The current keys and pad buttons, comma-separated, or "unbound".
    pub keys: String,
}

/// The keys help: every group in menu order, each with one [`HelpRow`] per action showing
/// the player's *current* bindings (keyboard first, then gamepad), or "unbound". Built
/// from [`ACTIONS`], so a new action appears here, in the controls menu and on the console
/// without further work.
pub fn help_rows(b: &Bindings) -> Vec<(Group, Vec<HelpRow>)> {
    let mut groups: Vec<(Group, Vec<HelpRow>)> = Vec::new();
    for info in ACTIONS {
        let mut keys: Vec<String> = b.keys.get(&info.action).into_iter().flatten().map(KeyChord::text).collect();
        keys.extend(b.pad.get(&info.action).into_iter().flatten().map(|p| format!("pad {}", p.text())));
        let keys = if keys.is_empty() { "unbound".to_string() } else { keys.join(", ") };
        let row = HelpRow { label: info.label, description: info.description, keys };
        // Groups appear in the order their first action appears in the table.
        match groups.iter_mut().find(|(g, _)| *g == info.group) {
            Some((_, rows)) => rows.push(row),
            None => groups.push((info.group, vec![row])),
        }
    }
    groups
}

/// The table entry for an action.
pub fn info(action: Action) -> &'static ActionInfo {
    ACTIONS.iter().find(|i| i.action == action).expect("every action is in ACTIONS")
}

/// A key with optional Shift: `Shift+F` or `F`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyChord {
    pub key: String,
    pub shift: bool,
}

impl KeyChord {
    /// Parses `"Shift+F"`, `"f"`, `" Kp7 "`. Key names are case-insensitive and stored
    /// lower-case, the canonical form used by [`RawInput`].
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

    /// The form written to `bindings.yaml`.
    pub fn text(&self) -> String {
        if self.shift {
            format!("Shift+{}", self.key)
        } else {
            self.key.clone()
        }
    }
}

/// A gamepad button with an optional modifier button held: `LeftShoulder+A` or `A`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PadChord {
    pub modifier: Option<Button>,
    pub button: Button,
}

impl PadChord {
    /// A plain button.
    pub fn plain(button: Button) -> PadChord {
        PadChord { modifier: None, button }
    }

    /// Parses `"A"` or `"LeftShoulder+A"` (case-insensitive).
    pub fn parse(s: &str) -> Option<PadChord> {
        let s = s.trim();
        match s.split_once('+') {
            Some((m, b)) => {
                let (modifier, button) = (Button::parse(m)?, Button::parse(b)?);
                (modifier != button).then_some(PadChord { modifier: Some(modifier), button })
            }
            None => Some(PadChord::plain(Button::parse(s)?)),
        }
    }

    /// The form written to `bindings.yaml`.
    pub fn text(&self) -> String {
        match self.modifier {
            Some(m) => format!("{}+{}", m.name(), self.button.name()),
            None => self.button.name().to_string(),
        }
    }
}

/// Canonical key name: case-insensitive input, stored lower-case.
pub fn canonical(key: &str) -> String {
    key.trim().to_ascii_lowercase()
}

/// How the two gamepad sticks map to the flight axes (the original's Alt+J swap).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StickLayout {
    /// Left stick: yaw and pitch; right stick: roll and vertical slide.
    Space,
    /// Left stick: roll and pitch (like an aircraft joystick); right stick: yaw and
    /// vertical slide.
    Aircraft,
}

/// Key and button bindings, plus analogue tuning.
#[derive(Debug, Clone, PartialEq)]
pub struct Bindings {
    pub keys: HashMap<Action, Vec<KeyChord>>,
    pub pad: HashMap<Action, Vec<PadChord>>,
    /// Gamepad stick dead zone, 0..1.
    pub dead_zone: f32,
    /// Mouse pixels per full stick deflection.
    pub mouse_range: f32,
    /// Push the stick forward to dive (aircraft style). True inverts.
    pub pad_invert_pitch: bool,
    pub stick_layout: StickLayout,
    /// Mouse flight: moving the mouse up the screen lowers the nose instead of raising it.
    pub mouse_invert: bool,
}

impl Default for Bindings {
    fn default() -> Self {
        Self::from_ini(&Ini::default())
    }
}

impl Bindings {
    /// Builds bindings from a `bindings.ini` of an earlier build (read once, then
    /// replaced by `bindings.yaml`); anything not mentioned keeps its default.
    ///
    /// ```ini
    /// [keyboard]
    /// fire_beams = Enter, Space
    /// [gamepad]
    /// fire_beams = RightTrigger
    /// hyperspace = LeftShoulder+A
    /// dead_zone = 0.15
    /// stick_layout = aircraft
    /// ```
    pub fn from_ini(ini: &Ini) -> Bindings {
        let kb = ini.section("keyboard");
        let gp = ini.section("gamepad");
        let mut keys = HashMap::new();
        let mut pad = HashMap::new();
        for i in ACTIONS {
            let spec = kb.and_then(|s| s.get(i.name)).unwrap_or(i.keys);
            keys.insert(i.action, spec.split(',').filter_map(KeyChord::parse).collect());
            let spec = gp.and_then(|s| s.get(i.name)).unwrap_or(i.pad);
            pad.insert(i.action, spec.split(',').filter_map(PadChord::parse).collect());
        }
        let layout = match gp.and_then(|s| s.get("stick_layout")).map(str::to_ascii_lowercase).as_deref() {
            Some("aircraft") => StickLayout::Aircraft,
            _ => StickLayout::Space,
        };
        Bindings {
            keys,
            pad,
            dead_zone: gp.and_then(|s| s.get_f32("dead_zone")).unwrap_or(0.15).clamp(0.0, 0.9),
            mouse_range: ini.get("mouse", "range").and_then(|v| v.parse().ok()).unwrap_or(200.0f32).max(10.0),
            pad_invert_pitch: gp.and_then(|s| s.get_bool("invert_pitch")).unwrap_or(false),
            stick_layout: layout,
            mouse_invert: ini.section("mouse").and_then(|s| s.get_bool("invert")).unwrap_or(false),
        }
    }

    /// Writes the full bindings as INI text (the earlier format; kept for tests of the
    /// migration).
    pub fn to_ini_text(&self) -> String {
        let yes = |b: bool| if b { "yes" } else { "no" };
        let mut out = String::from(
            "; OpenInterceptor controls. Edit here or in the game's Controls menu.\n\
             ; Key names: see the docs site (Original game > Controls).\n[keyboard]\n",
        );
        for i in ACTIONS {
            let keys: Vec<String> = self.keys[&i.action].iter().map(KeyChord::text).collect();
            out += &format!("{} = {}\n", i.name, keys.join(", "));
        }
        out += "\n[gamepad]\n";
        for i in ACTIONS {
            let b: Vec<String> = self.pad[&i.action].iter().map(PadChord::text).collect();
            out += &format!("{} = {}\n", i.name, b.join(", "));
        }
        out += &format!(
            "dead_zone = {}\ninvert_pitch = {}\nstick_layout = {}\n\n[mouse]\nrange = {}\ninvert = {}\n",
            self.dead_zone,
            yes(self.pad_invert_pitch),
            match self.stick_layout {
                StickLayout::Space => "space",
                StickLayout::Aircraft => "aircraft",
            },
            self.mouse_range,
            yes(self.mouse_invert)
        );
        out
    }

    /// Builds bindings from `bindings.yaml`; anything not mentioned keeps its default.
    ///
    /// ```yaml
    /// keyboard:
    ///   fire_beams: [Enter, Space]
    /// gamepad:
    ///   hyperspace: [LeftShoulder+A]
    /// options:
    ///   dead_zone: 0.15
    ///   invert_pitch: false
    ///   stick_layout: aircraft
    ///   mouse_range: 200
    ///   mouse_invert: false
    /// ```
    pub fn from_yaml(doc: &crate::formats::yaml::Node) -> Bindings {
        use crate::content::schema::Get;
        let d = Bindings::default();
        let section = |k: &str| doc.get(k).cloned().unwrap_or(crate::formats::yaml::Node::null(0));
        let (kb, gp, opt) = (section("keyboard"), section("gamepad"), section("options"));
        let mut b = d.clone();
        for i in ACTIONS {
            if kb.get(i.name).is_some() {
                b.keys.insert(i.action, Get(&kb).list(i.name).iter().filter_map(|k| KeyChord::parse(k)).collect());
            }
            if gp.get(i.name).is_some() {
                b.pad.insert(i.action, Get(&gp).list(i.name).iter().filter_map(|k| PadChord::parse(k)).collect());
            }
        }
        let o = Get(&opt);
        b.dead_zone = o.num("dead_zone").unwrap_or(d.dead_zone).clamp(0.0, 0.9);
        b.pad_invert_pitch = o.bool("invert_pitch").unwrap_or(d.pad_invert_pitch);
        b.stick_layout = match o.str("stick_layout").map(str::to_ascii_lowercase).as_deref() {
            Some("aircraft") => StickLayout::Aircraft,
            Some("space") => StickLayout::Space,
            _ => d.stick_layout,
        };
        b.mouse_range = o.num("mouse_range").unwrap_or(d.mouse_range).max(10.0);
        b.mouse_invert = o.bool("mouse_invert").unwrap_or(d.mouse_invert);
        b
    }

    /// Writes the full bindings as `bindings.yaml` text.
    pub fn to_yaml_text(&self) -> String {
        let list = |items: Vec<String>| {
            format!("[{}]", items.iter().map(|i| format!("\"{i}\"")).collect::<Vec<_>>().join(", "))
        };
        let mut out = String::from(
            "# OpenInterceptor controls. Edit here or in the game's Controls menu.\n\
             # Key names: see the docs site (Original game > Controls). Shift+ makes a chord;\n\
             # on the gamepad, Modifier+Button (LeftShoulder+A) needs the modifier held.\n\
             keyboard:\n",
        );
        for i in ACTIONS {
            out += &format!("  {}: {}\n", i.name, list(self.keys[&i.action].iter().map(KeyChord::text).collect()));
        }
        out += "gamepad:\n";
        for i in ACTIONS {
            out += &format!("  {}: {}\n", i.name, list(self.pad[&i.action].iter().map(PadChord::text).collect()));
        }
        out += &format!(
            "options:\n  dead_zone: {}\n  invert_pitch: {}\n  stick_layout: {}\n  mouse_range: {}\n  mouse_invert: {}\n",
            self.dead_zone,
            self.pad_invert_pitch,
            match self.stick_layout {
                StickLayout::Space => "space",
                StickLayout::Aircraft => "aircraft",
            },
            self.mouse_range,
            self.mouse_invert
        );
        out
    }

    /// Keys used in some `Shift+` chord. While Shift is held these keys belong to their
    /// chords, so a held action bound to the plain key (roll on `E`) ignores them
    /// (`Shift+E` orders wingmen to evade without also rolling the ship).
    fn shifted_keys(&self) -> HashSet<&str> {
        self.keys.values().flatten().filter(|k| k.shift).map(|k| k.key.as_str()).collect()
    }

    /// Buttons used as modifiers in some pad chord.
    fn modifiers(&self) -> HashSet<Button> {
        self.pad.values().flatten().filter_map(|c| c.modifier).collect()
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
    /// Actions held on a remote control surface (the tablet console, later a USB
    /// panel). They count exactly like a held key bound to the action.
    pub remote_held: HashSet<Action>,
    /// Actions tapped on a remote control surface this frame.
    pub remote_pressed: HashSet<Action>,
    /// A throttle position set on a remote slider (0..1), this frame.
    pub remote_throttle: Option<f32>,
    /// Power set on remote sliders this frame: beam level, shield level, shield balance.
    pub remote_power: (Option<u8>, Option<u8>, Option<crate::sim::ShieldBalance>),
}

impl RawInput {
    /// True if gamepad button `b` went down this frame.
    pub fn pad_pressed(&self, b: Button) -> bool {
        match self.pad {
            Some(p) => p.button(b) && !self.pad_previous.unwrap_or_default().button(b),
            None => false,
        }
    }
}

/// Hides inputs that were already held when a screen started, until each is released.
///
/// Enter starts a mission from the launcher, the restart prompt and the debrief, and
/// Enter is also a fire key: without this guard, the key still down on the battle's
/// first frame fires the guns. The same goes for the pad's A button (start, and the
/// afterburner) and the mouse. The guard records what is held on its first frame and
/// filters it out of every later frame until it is let go; pressing it again then
/// works normally.
#[derive(Debug, Default)]
pub struct StartGuard {
    started: bool,
    keys: HashSet<String>,
    buttons: u16,
    triggers: (bool, bool),
    mouse: bool,
}

impl StartGuard {
    /// Filters `raw` in place (call once per frame, right after reading the devices).
    pub fn filter(&mut self, raw: &mut RawInput) {
        if !self.started {
            self.started = true;
            self.keys = raw.held.clone();
            if let Some(p) = &raw.pad {
                self.buttons = p.buttons;
                self.triggers = (p.lt > 127, p.rt > 127);
            }
            self.mouse = raw.mouse_left;
        }
        // Released inputs leave the guard for good.
        self.keys.retain(|k| raw.held.contains(k));
        let pad = raw.pad.unwrap_or_default();
        self.buttons &= pad.buttons;
        self.triggers = (self.triggers.0 && pad.lt > 127, self.triggers.1 && pad.rt > 127);
        self.mouse &= raw.mouse_left;
        // Hide what is still guarded.
        for k in &self.keys {
            raw.held.remove(k);
            raw.pressed.remove(k);
        }
        for p in [&mut raw.pad, &mut raw.pad_previous].into_iter().flatten() {
            p.buttons &= !self.buttons;
            if self.triggers.0 {
                p.lt = 0;
            }
            if self.triggers.1 {
                p.rt = 0;
            }
        }
        if self.mouse {
            raw.mouse_left = false;
        }
    }
}

/// One-shot requests handled by the app rather than the simulation's [`Controls`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    NextTarget,
    PrevTarget,
    NearestEnemy,
    TargetInSights,
    MfdNext,
    MfdPrev,
    View(View),
    WingMenu,
    WingFireAtWill,
    WingAttackMyTarget,
    WingFormation,
    WingEvasive,
    WingRetreat,
    WingStatus,
    WingNextFormation,
    RepeatMessage,
    NextNav,
    Pause,
    Menu,
    MouseFlight,
    Help,
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

/// Actions that become one-shot [`Command`]s.
const COMMANDS: &[(Action, Command)] = &[
    (A::NextTarget, Command::NextTarget),
    (A::PrevTarget, Command::PrevTarget),
    (A::NearestEnemy, Command::NearestEnemy),
    (A::TargetInSights, Command::TargetInSights),
    (A::MfdNext, Command::MfdNext),
    (A::MfdPrev, Command::MfdPrev),
    (A::ViewCockpit, Command::View(View::Cockpit)),
    (A::ViewForward, Command::View(View::Forward)),
    (A::ViewLeft, Command::View(View::Left)),
    (A::ViewRight, Command::View(View::Right)),
    (A::ViewRearLeft, Command::View(View::RearLeft)),
    (A::ViewRearRight, Command::View(View::RearRight)),
    (A::ViewChase, Command::View(View::Chase)),
    (A::ViewChaseManual, Command::View(View::ChaseManual)),
    (A::WingMenu, Command::WingMenu),
    (A::WingFireAtWill, Command::WingFireAtWill),
    (A::WingAttackMyTarget, Command::WingAttackMyTarget),
    (A::WingFormation, Command::WingFormation),
    (A::WingEvasive, Command::WingEvasive),
    (A::WingRetreat, Command::WingRetreat),
    (A::WingStatus, Command::WingStatus),
    (A::WingNextFormation, Command::WingNextFormation),
    (A::RepeatMessage, Command::RepeatMessage),
    (A::NextNav, Command::NextNav),
    (A::Pause, Command::Pause),
    (A::Menu, Command::Menu),
    (A::MouseFlight, Command::MouseFlight),
    (A::Help, Command::Help),
];

/// Per-frame helper holding the chord sets, so they are computed once per frame
/// rather than once per action.
struct Frame<'a> {
    b: &'a Bindings,
    raw: &'a RawInput,
    shifted: HashSet<&'a str>,
    modifiers: HashSet<Button>,
}

impl Frame<'_> {
    /// True when a modifier other than `except` is held on the pad.
    fn other_modifier_held(&self, p: &PadState, except: Button) -> bool {
        self.modifiers.iter().any(|m| *m != except && p.button(*m))
    }

    fn pad_chord_held(&self, c: &PadChord) -> bool {
        let Some(p) = &self.raw.pad else { return false };
        match c.modifier {
            Some(m) => p.button(m) && p.button(c.button),
            None => p.button(c.button) && !self.modifiers.contains(&c.button) && !self.other_modifier_held(p, c.button),
        }
    }

    fn pad_chord_pressed(&self, c: &PadChord) -> bool {
        let Some(p) = &self.raw.pad else { return false };
        let went_down = p.button(c.button) && !self.raw.pad_previous.unwrap_or_default().button(c.button);
        match c.modifier {
            Some(m) => p.button(m) && went_down,
            // A modifier button pressed on its own does nothing.
            None => went_down && !self.modifiers.contains(&c.button) && !self.other_modifier_held(p, c.button),
        }
    }

    fn held(&self, a: Action) -> bool {
        let keys = self.b.keys.get(&a).map(Vec::as_slice).unwrap_or(&[]);
        let key_held = keys.iter().any(|k| {
            self.raw.held.contains(&k.key)
                && if k.shift { self.raw.shift } else { !self.raw.shift || !self.shifted.contains(k.key.as_str()) }
        });
        let pad = self.b.pad.get(&a).map(Vec::as_slice).unwrap_or(&[]);
        key_held || pad.iter().any(|c| self.pad_chord_held(c)) || self.raw.remote_held.contains(&a)
    }

    fn pressed(&self, a: Action) -> bool {
        let keys = self.b.keys.get(&a).map(Vec::as_slice).unwrap_or(&[]);
        // A Shift chord only fires with Shift held; a plain key only fires without it,
        // so `F` and `Shift+F` can do different things.
        let key = keys.iter().any(|k| self.raw.pressed.contains(&k.key) && k.shift == self.raw.shift);
        let pad = self.b.pad.get(&a).map(Vec::as_slice).unwrap_or(&[]);
        key || pad.iter().any(|c| self.pad_chord_pressed(c)) || self.raw.remote_pressed.contains(&a)
    }

    fn axis(&self, neg: Action, pos: Action) -> f32 {
        (self.held(pos) as i32 - self.held(neg) as i32) as f32
    }
}

impl Bindings {
    /// Converts one frame of raw input. `mouse_flight` enables mouse steering.
    pub fn translate(&self, raw: &RawInput, mouse_flight: bool) -> (Controls, Vec<Command>, ChaseInput) {
        let f = Frame { b: self, raw, shifted: self.shifted_keys(), modifiers: self.modifiers() };
        let mut c = Controls {
            pitch: f.axis(A::PitchDown, A::PitchUp),
            yaw: f.axis(A::YawLeft, A::YawRight),
            roll: f.axis(A::RollLeft, A::RollRight),
            slide: glam::Vec2::new(f.axis(A::SlideLeft, A::SlideRight), f.axis(A::SlideDown, A::SlideUp)),
            afterburner: f.held(A::Afterburner),
            brake: f.held(A::Brake),
            fire_beams: f.held(A::FireBeams),
            fire_missile: f.pressed(A::FireMissile),
            cycle_link: f.pressed(A::CycleLink),
            cycle_missile: f.pressed(A::CycleMissile),
            toggle_reverse: f.pressed(A::Reverse),
            match_speed: f.pressed(A::MatchSpeed),
            requests: Requests {
                beam_charge: f.pressed(A::BeamCharge),
                shield_charge: f.pressed(A::ShieldCharge),
                power_to_shields: f.pressed(A::PowerToShields),
                power_to_beams: f.pressed(A::PowerToBeams),
                shield_balance: f.pressed(A::ShieldBalance),
                tractor: f.pressed(A::Tractor),
                cloak: f.pressed(A::Cloak),
                ecm: f.pressed(A::Ecm),
                hyperspace: f.pressed(A::Hyperspace),
                detonate: false,
                set_beam: raw.remote_power.0,
                set_shield: raw.remote_power.1,
                set_balance: raw.remote_power.2,
            },
            ..Default::default()
        };
        if f.pressed(A::ThrottleUp) {
            c.throttle_delta += THROTTLE_STEP;
        }
        if f.pressed(A::ThrottleDown) {
            c.throttle_delta -= THROTTLE_STEP;
        }
        if f.pressed(A::ThrottleFull) {
            c.throttle_set = Some(1.0);
        }
        if f.pressed(A::ThrottleZero) {
            c.throttle_set = Some(0.0);
        }
        if let Some(t) = raw.remote_throttle {
            c.throttle_set = Some(t.clamp(0.0, 1.0));
        }
        // Gamepad sticks add to (and are clamped with) the keys.
        if let Some(p) = &raw.pad {
            let dz = self.dead_zone;
            let (lx, ly, rx, ry) =
                (dead(p.axis_lx(), dz), dead(p.axis_ly(), dz), dead(p.axis_rx(), dz), dead(p.axis_ry(), dz));
            // Pushing the stick forward (up) dives, like an aircraft joystick.
            c.pitch += if self.pad_invert_pitch { ly } else { -ly };
            match self.stick_layout {
                StickLayout::Space => {
                    c.yaw += lx;
                    c.roll += rx;
                }
                StickLayout::Aircraft => {
                    c.roll += lx;
                    c.yaw += rx;
                }
            }
            c.slide.y += ry;
        }
        // Mouse flight. The original readme maps "Nose Down = Mouse Back, Nose Up =
        // Mouse Up": moving the cursor down the screen lowers the nose (mouse-look
        // style, not an inverted aircraft stick). Screen y grows downwards.
        if mouse_flight {
            let (mx, my) = raw.mouse_offset;
            c.yaw += (mx / self.mouse_range).clamp(-1.0, 1.0);
            let pitch = (my / self.mouse_range).clamp(-1.0, 1.0);
            c.pitch += if self.mouse_invert { pitch } else { -pitch };
            c.fire_beams |= raw.mouse_left;
            c.fire_missile |= raw.mouse_right_pressed;
        }
        c.pitch = c.pitch.clamp(-1.0, 1.0);
        c.yaw = c.yaw.clamp(-1.0, 1.0);
        c.roll = c.roll.clamp(-1.0, 1.0);
        c.slide = c.slide.clamp_length_max(1.0);

        let cmds = COMMANDS.iter().filter(|(a, _)| f.pressed(*a)).map(|(_, cmd)| *cmd).collect();
        let chase = ChaseInput {
            yaw: f.axis(A::ChaseLeft, A::ChaseRight),
            pitch: f.axis(A::ChaseDown, A::ChaseUp),
            zoom: f.axis(A::ChaseOut, A::ChaseIn),
            reset: f.pressed(A::ChaseReset),
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
    fn keys_held_when_a_battle_starts_are_ignored_until_released() {
        let b = Bindings::default();
        let mut guard = StartGuard::default();
        // Frame 1: Enter is still down from the launcher (and a pad A press).
        let mut r = raw(&["Enter"], &["Enter"]);
        r.pad = Some(PadState { buttons: Button::A.bit().unwrap(), ..Default::default() });
        guard.filter(&mut r);
        let (c, _, _) = b.translate(&r, false);
        assert!(!c.fire_beams && !c.afterburner, "the start press does nothing");
        // Still held a moment later: still nothing.
        let mut r = raw(&["Enter"], &[]);
        guard.filter(&mut r);
        assert!(!b.translate(&r, false).0.fire_beams);
        // Released, then pressed again: fires.
        let mut r = raw(&[], &[]);
        guard.filter(&mut r);
        let mut r = raw(&["Enter"], &["Enter"]);
        guard.filter(&mut r);
        assert!(b.translate(&r, false).0.fire_beams, "a fresh press works");
        // Other keys are never affected.
        let mut g = StartGuard::default();
        let mut r = raw(&["Space"], &["Space"]);
        g.filter(&mut r);
        let mut r = raw(&["LeftControl"], &[]);
        g.filter(&mut r);
        assert!(b.translate(&r, false).0.fire_beams, "a key pressed after the start counts");
    }

    #[test]
    fn every_action_has_a_one_sentence_description() {
        for a in ACTIONS {
            let d = a.description.trim();
            assert!(!d.is_empty(), "{} has no description", a.name);
            // A sentence for a player: starts with a capital, ends with a full stop, and
            // says more than the label does.
            assert!(d.starts_with(|c: char| c.is_ascii_uppercase()), "{}: {d:?} starts in lower case", a.name);
            assert!(d.ends_with('.'), "{}: {d:?} does not end with a full stop", a.name);
            assert_ne!(d, a.label, "{}: the description only repeats the label", a.name);
        }
    }

    #[test]
    fn the_keys_help_lists_every_action_with_its_current_keys() {
        let mut b = Bindings::default();
        let rows = help_rows(&b);
        assert_eq!(rows.iter().map(|(_, r)| r.len()).sum::<usize>(), ACTIONS.len(), "every action once");
        assert_eq!(rows[0].0, Group::Flight, "groups in table order");
        let find = |rows: &[(Group, Vec<HelpRow>)], label: &str| {
            rows.iter().flat_map(|(_, r)| r.iter()).find(|r| r.label == label).map(|r| r.keys.clone()).unwrap()
        };
        // Each row carries its action's description from the table.
        assert_eq!(rows[0].1[0].description, ACTIONS[0].description);
        assert_eq!(find(&rows, "Afterburner (hold)"), "tab, pad A");
        b.keys.insert(Action::Afterburner, vec![]);
        b.pad.insert(Action::Afterburner, vec![]);
        assert_eq!(find(&help_rows(&b), "Afterburner (hold)"), "unbound");
        // `?` (Shift+Slash) and Shift+F1 open it (F12 is the cockpit layout editor).
        let mut r = raw(&[], &["Slash"]);
        r.shift = true;
        assert_eq!(Bindings::default().translate(&r, false).1, vec![Command::Help]);
        let mut r = raw(&[], &["F1"]);
        r.shift = true;
        assert_eq!(Bindings::default().translate(&r, false).1, vec![Command::Help]);
        assert_ne!(
            Bindings::default().translate(&raw(&[], &["F1"]), false).1,
            vec![Command::Help],
            "plain F1 is the cockpit view"
        );
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
    fn power_and_equipment_keys_follow_the_original() {
        let b = Bindings::default();
        let (c, _, _) = b.translate(&raw(&[], &["F9", "F10", "Semicolon", "Apostrophe", "S", "H"]), false);
        let r = c.requests;
        assert!(r.beam_charge && r.shield_charge && r.power_to_shields && r.power_to_beams);
        assert!(r.shield_balance && r.hyperspace && !r.tractor);
        let mut shifted = raw(&[], &["T", "C", "J"]);
        shifted.shift = true;
        let (c, cmds, _) = b.translate(&shifted, false);
        assert!(c.requests.tractor && c.requests.cloak && c.requests.ecm);
        assert!(cmds.is_empty(), "Shift+T is not next target");
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
        let mut r = raw(&[], &["A", "R", "E", "S", "I"]);
        r.shift = true;
        assert_eq!(
            b.translate(&r, false).1,
            vec![
                Command::WingAttackMyTarget,
                Command::WingFormation,
                Command::WingEvasive,
                Command::WingRetreat,
                Command::WingStatus
            ]
        );
        let (_, cmds, _) = b.translate(&raw(&[], &["RightBracket", "L"]), false);
        assert_eq!(cmds, vec![Command::WingMenu, Command::RepeatMessage]);
    }

    #[test]
    fn a_shift_chord_does_not_also_trigger_the_plain_held_key() {
        let b = Bindings::default();
        let mut r = raw(&["E"], &["E"]);
        assert_eq!(b.translate(&r, false).0.roll, 1.0, "E rolls");
        r.shift = true;
        let (c, cmds, _) = b.translate(&r, false);
        assert_eq!(c.roll, 0.0, "Shift+E belongs to the wingman order");
        assert_eq!(cmds, vec![Command::WingEvasive]);
        // A key with no Shift chord still works with Shift held.
        let mut up = raw(&["Up"], &[]);
        up.shift = true;
        assert_eq!(b.translate(&up, false).0.pitch, -1.0);
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
    fn pad_modifier_layer_reaches_every_power_control() {
        let b = Bindings::default();
        let press = |mods: &[Button], button: Button| {
            let mut prev = PadState::default();
            for m in mods {
                prev.set(*m, true);
            }
            let mut now = prev;
            now.set(button, true);
            b.translate(&RawInput { pad: Some(now), pad_previous: Some(prev), ..Default::default() }, false)
        };
        let ls = Button::LeftShoulder;
        assert!(press(&[ls], Button::A).0.requests.hyperspace);
        assert!(!press(&[ls], Button::A).0.afterburner, "A is not the afterburner while the modifier is held");
        assert!(press(&[ls], Button::DpadUp).0.requests.power_to_beams);
        assert_eq!(press(&[ls], Button::DpadUp).0.throttle_delta, 0.0, "not throttle on the second layer");
        assert!(press(&[ls], Button::DpadDown).0.requests.power_to_shields);
        assert!(press(&[ls], Button::DpadLeft).0.requests.beam_charge);
        assert!(press(&[ls], Button::DpadRight).0.requests.shield_charge);
        assert!(press(&[ls], Button::B).0.requests.shield_balance);
        assert!(press(&[ls], Button::X).0.requests.tractor);
        assert!(press(&[ls], Button::Y).0.requests.cloak);
        assert!(press(&[ls], Button::Back).0.requests.ecm);
        assert_eq!(press(&[ls], Button::RightShoulder).1, vec![Command::PrevTarget]);
        assert_eq!(press(&[ls], Button::LeftThumb).1, vec![Command::WingMenu]);
        assert_eq!(press(&[ls], Button::Start).1, vec![Command::Menu]);
        assert_eq!(press(&[], ls).1, Vec::<Command>::new(), "the modifier alone does nothing");
        assert_eq!(press(&[], Button::Start).1, vec![Command::Pause]);
        assert!(press(&[], Button::A).0.afterburner);
    }

    #[test]
    fn every_action_is_reachable_from_keyboard_and_flight_actions_from_the_pad() {
        let b = Bindings::default();
        for i in ACTIONS {
            assert!(!b.keys[&i.action].is_empty(), "{} has no key", i.name);
        }
        for a in [
            A::BeamCharge,
            A::ShieldCharge,
            A::PowerToBeams,
            A::PowerToShields,
            A::ShieldBalance,
            A::Tractor,
            A::Cloak,
            A::Ecm,
            A::Hyperspace,
            A::WingMenu,
        ] {
            assert!(!b.pad[&a].is_empty(), "{:?} has no pad button", a);
        }
        // No two actions share a default chord.
        let mut seen = HashMap::new();
        for i in ACTIONS {
            for k in &b.keys[&i.action] {
                assert!(seen.insert(("k", k.text()), i.name).is_none(), "{} reused", k.text());
            }
            for p in &b.pad[&i.action] {
                assert!(seen.insert(("p", p.text()), i.name).is_none(), "{} reused", p.text());
            }
        }
    }

    #[test]
    fn stick_layouts_and_mouse_inversion() {
        let mut b = Bindings::default();
        let r =
            RawInput { pad: Some(PadState { lx: i16::MAX, rx: i16::MIN, ..Default::default() }), ..Default::default() };
        assert_eq!((b.translate(&r, false).0.yaw, b.translate(&r, false).0.roll), (1.0, -1.0));
        b.stick_layout = StickLayout::Aircraft;
        assert_eq!((b.translate(&r, false).0.yaw, b.translate(&r, false).0.roll), (-1.0, 1.0));
        let m = RawInput { mouse_offset: (0.0, 400.0), ..Default::default() };
        assert_eq!(b.translate(&m, true).0.pitch, -1.0);
        b.mouse_invert = true;
        assert_eq!(b.translate(&m, true).0.pitch, 1.0);
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
        let ini = Ini::parse(
            "[keyboard]\nfire_beams = Space, Shift+B\nfire_missile = Enter\n[gamepad]\nfire_beams = A\n\
             hyperspace = Back+Y\ndead_zone = 0.3\ninvert_pitch = yes\nstick_layout = aircraft\n\
             [mouse]\nrange = 50\ninvert = yes\n",
        );
        let b = Bindings::from_ini(&ini);
        assert_eq!(
            b.keys[&Action::FireBeams],
            vec![KeyChord::parse("space").unwrap(), KeyChord::parse("shift+b").unwrap()]
        );
        assert_eq!(b.pad[&Action::FireBeams], vec![PadChord::plain(Button::A)]);
        assert_eq!(b.pad[&Action::Hyperspace], vec![PadChord { modifier: Some(Button::Back), button: Button::Y }]);
        assert_eq!((b.dead_zone, b.mouse_range, b.pad_invert_pitch), (0.3, 50.0, true));
        assert_eq!((b.stick_layout, b.mouse_invert), (StickLayout::Aircraft, true));
        let again = Bindings::from_ini(&Ini::parse(&b.to_ini_text()));
        assert_eq!(again, b);
        let doc = crate::formats::yaml::parse(&b.to_yaml_text()).unwrap();
        assert_eq!(Bindings::from_yaml(&doc), b, "YAML round trip");
        let partial =
            crate::formats::yaml::parse("keyboard:\n  fire_beams: [Space]\noptions: {stick_layout: aircraft}\n")
                .unwrap();
        let p = Bindings::from_yaml(&partial);
        assert_eq!(p.keys[&Action::FireBeams], vec![KeyChord::parse("space").unwrap()]);
        assert_eq!(
            p.keys[&Action::FireMissile],
            Bindings::default().keys[&Action::FireMissile],
            "unmentioned: default"
        );
        assert_eq!(p.stick_layout, StickLayout::Aircraft);
        assert_eq!(Bindings::from_ini(&Ini::parse(&Bindings::default().to_ini_text())), Bindings::default());
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
        assert_eq!(PadChord::parse("A+A"), None, "a button cannot modify itself");
        assert_eq!(PadChord::parse("Turbo"), None);
        assert_eq!(PadChord::parse("leftshoulder+dpadup").unwrap().text(), "LeftShoulder+DpadUp");
        assert_eq!(dead(0.1, 0.2), 0.0);
        assert!((dead(1.0, 0.2) - 1.0).abs() < 1e-6);
        assert!((dead(-0.6, 0.2) + 0.5).abs() < 1e-6);
    }
}
