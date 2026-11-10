//! # The controls menu's model
//!
//! Everything the in-game *Controls* screen does, without drawing anything: the list of
//! rows (actions grouped by [`Group`], then device options), which cell is selected,
//! capturing a new key or gamepad chord, clearing and restoring defaults, and keeping
//! every chord bound to at most one action. The screen in `app::controls` only paints
//! this model and feeds it input, so all of the behaviour is unit-tested here.
//!
//! ## Capturing a gamepad chord
//!
//! A chord is "hold a modifier, press a button". The capture cannot decide on the first
//! button down whether it is a plain press or the start of a chord, so it waits:
//!
//! ```text
//!  first button down ──▶ second button down while the first is held ──▶ chord first+second
//!                   └──▶ first button released alone ─────────────────▶ plain first
//! ```

use super::gamepad::{Button, PadState};
use super::{info, Action, Bindings, Group, KeyChord, PadChord, StickLayout, ACTIONS};

/// A device option shown below the actions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Setting {
    DeadZone,
    InvertPitch,
    StickLayout,
    MouseRange,
    MouseInvert,
}

impl Setting {
    pub const ALL: [Setting; 5] =
        [Setting::DeadZone, Setting::InvertPitch, Setting::StickLayout, Setting::MouseRange, Setting::MouseInvert];

    pub fn label(self) -> &'static str {
        match self {
            Setting::DeadZone => "Gamepad stick dead zone",
            Setting::InvertPitch => "Gamepad: pull back to dive",
            Setting::StickLayout => "Gamepad stick layout",
            Setting::MouseRange => "Mouse flight range (pixels)",
            Setting::MouseInvert => "Mouse flight: up = nose down",
        }
    }
}

/// One line of the menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Row {
    Header(&'static str),
    Action(Action),
    Setting(Setting),
}

/// Columns: two keyboard slots and two gamepad slots per action.
pub const COLUMNS: [&str; 4] = ["Key", "Alt key", "Gamepad", "Alt gamepad"];

/// What the menu is waiting for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Capture {
    Key,
    /// Waiting for a pad button; `first` is the button already down, if any.
    Pad {
        first: Option<Button>,
    },
}

/// The controls menu state.
#[derive(Debug, Clone, PartialEq)]
pub struct Editor {
    /// The bindings being edited (copied back to the game by the screen).
    pub bindings: Bindings,
    pub rows: Vec<Row>,
    /// Index into `rows`; never a header.
    pub selected: usize,
    /// Index into [`COLUMNS`] (ignored on setting rows).
    pub column: usize,
    pub capture: Option<Capture>,
    /// Feedback for the status line.
    pub message: String,
    /// True when something changed since the menu opened.
    pub dirty: bool,
}

impl Editor {
    /// Opens the menu on a copy of `bindings`.
    pub fn new(bindings: &Bindings) -> Editor {
        let mut rows = Vec::new();
        let mut group: Option<Group> = None;
        // Groups appear in the order of their first action in the table.
        let mut groups: Vec<Group> = Vec::new();
        for i in ACTIONS {
            if !groups.contains(&i.group) {
                groups.push(i.group);
            }
        }
        for g in groups {
            for i in ACTIONS.iter().filter(|i| i.group == g) {
                if group != Some(g) {
                    rows.push(Row::Header(g.title()));
                    group = Some(g);
                }
                rows.push(Row::Action(i.action));
            }
        }
        rows.push(Row::Header("Devices"));
        rows.extend(Setting::ALL.map(Row::Setting));
        Editor {
            bindings: bindings.clone(),
            rows,
            selected: 1,
            column: 0,
            capture: None,
            message: "Enter: change   Delete: clear   D: default   Esc: save and leave".into(),
            dirty: false,
        }
    }

    /// The selected row.
    pub fn row(&self) -> Row {
        self.rows[self.selected]
    }

    /// Moves the selection by `delta` rows, skipping headers and stopping at the ends.
    pub fn move_row(&mut self, delta: i32) {
        let mut i = self.selected as i32;
        let n = self.rows.len() as i32;
        let step = delta.signum();
        for _ in 0..delta.abs() {
            let mut j = i + step;
            while (0..n).contains(&j) && matches!(self.rows[j as usize], Row::Header(_)) {
                j += step;
            }
            if (0..n).contains(&j) {
                i = j;
            }
        }
        self.selected = i as usize;
    }

    /// Moves the selected column (wrapping).
    pub fn move_column(&mut self, delta: i32) {
        self.column = (self.column as i32 + delta).rem_euclid(COLUMNS.len() as i32) as usize;
    }

    /// Enter on the selection: start capturing for an action cell, or step a setting.
    pub fn activate(&mut self) {
        match self.row() {
            Row::Action(a) => {
                self.capture = Some(if self.column < 2 { Capture::Key } else { Capture::Pad { first: None } });
                self.message = format!(
                    "{}: press a {} (Esc cancels)",
                    info(a).label,
                    if self.column < 2 {
                        "key, Shift allowed"
                    } else {
                        "button, or hold a modifier and press a button"
                    }
                );
            }
            Row::Setting(_) => self.adjust(1),
            Row::Header(_) => {}
        }
    }

    /// Cancels a capture.
    pub fn cancel(&mut self) {
        if self.capture.take().is_some() {
            self.message = "Cancelled".into();
        }
    }

    /// Feeds a key press to a keyboard capture. `key` is a canonical key name.
    pub fn capture_key(&mut self, key: &str, shift: bool) {
        if self.capture != Some(Capture::Key) {
            return;
        }
        if key == "escape" {
            self.cancel();
            return;
        }
        if matches!(key, "leftshift" | "rightshift") {
            return; // wait for the key that goes with Shift
        }
        let Row::Action(a) = self.row() else { return };
        self.capture = None;
        self.assign_key(a, self.column, KeyChord { key: key.to_string(), shift });
    }

    /// Feeds one frame of pad state to a gamepad capture (see the module docs).
    pub fn capture_pad(&mut self, now: &PadState, previous: &PadState) {
        let Some(Capture::Pad { first }) = self.capture else { return };
        let Row::Action(a) = self.row() else { return };
        let went_down = Button::all().find(|b| now.button(*b) && !previous.button(*b));
        match first {
            None => {
                if let Some(b) = went_down {
                    self.capture = Some(Capture::Pad { first: Some(b) });
                }
            }
            Some(f) => {
                if let Some(b) = went_down.filter(|b| *b != f && now.button(f)) {
                    self.capture = None;
                    self.assign_pad(a, self.column - 2, PadChord { modifier: Some(f), button: b });
                } else if !now.button(f) {
                    self.capture = None;
                    self.assign_pad(a, self.column - 2, PadChord::plain(f));
                }
            }
        }
    }

    /// Binds `chord` to `action` in keyboard slot `slot`, taking it away from any other
    /// action that had it.
    pub fn assign_key(&mut self, action: Action, slot: usize, chord: KeyChord) {
        let mut taken = Vec::new();
        for (a, keys) in self.bindings.keys.iter_mut() {
            let before = keys.len();
            keys.retain(|k| k != &chord);
            if keys.len() != before && *a != action {
                taken.push(info(*a).label);
            }
        }
        set_slot(self.bindings.keys.get_mut(&action).expect("every action has a key list"), slot, chord.clone());
        self.dirty = true;
        self.message = moved_message(&pretty_key(&chord), info(action).label, &taken);
    }

    /// Binds a pad chord to `action` in gamepad slot `slot` (0 or 1), taking it away from
    /// any other action.
    pub fn assign_pad(&mut self, action: Action, slot: usize, chord: PadChord) {
        let mut taken = Vec::new();
        for (a, pads) in self.bindings.pad.iter_mut() {
            let before = pads.len();
            pads.retain(|p| p != &chord);
            if pads.len() != before && *a != action {
                taken.push(info(*a).label);
            }
        }
        set_slot(self.bindings.pad.get_mut(&action).expect("every action has a pad list"), slot, chord);
        self.dirty = true;
        self.message = moved_message(&pretty_pad(&chord), info(action).label, &taken);
    }

    /// Delete on the selection: removes that cell's binding.
    pub fn clear(&mut self) {
        let Row::Action(a) = self.row() else { return };
        let removed = if self.column < 2 {
            let keys = self.bindings.keys.get_mut(&a).expect("every action has a key list");
            (self.column < keys.len()).then(|| keys.remove(self.column)).is_some()
        } else {
            let pads = self.bindings.pad.get_mut(&a).expect("every action has a pad list");
            (self.column - 2 < pads.len()).then(|| pads.remove(self.column - 2)).is_some()
        };
        if removed {
            self.dirty = true;
            self.message = format!("{}: cleared", info(a).label);
        }
    }

    /// Restores the selected action's (or setting's) default.
    pub fn reset_selected(&mut self) {
        let d = Bindings::default();
        match self.row() {
            Row::Action(a) => {
                self.bindings.keys.insert(a, d.keys[&a].clone());
                self.bindings.pad.insert(a, d.pad[&a].clone());
                self.message = format!("{}: default restored", info(a).label);
            }
            Row::Setting(s) => {
                match s {
                    Setting::DeadZone => self.bindings.dead_zone = d.dead_zone,
                    Setting::InvertPitch => self.bindings.pad_invert_pitch = d.pad_invert_pitch,
                    Setting::StickLayout => self.bindings.stick_layout = d.stick_layout,
                    Setting::MouseRange => self.bindings.mouse_range = d.mouse_range,
                    Setting::MouseInvert => self.bindings.mouse_invert = d.mouse_invert,
                }
                self.message = format!("{}: default restored", s.label());
            }
            Row::Header(_) => return,
        }
        self.dirty = true;
    }

    /// Restores every default.
    pub fn reset_all(&mut self) {
        self.bindings = Bindings::default();
        self.dirty = true;
        self.message = "All controls restored to their defaults".into();
    }

    /// Steps a setting up (`delta` > 0) or down.
    pub fn adjust(&mut self, delta: i32) {
        let Row::Setting(s) = self.row() else { return };
        let b = &mut self.bindings;
        match s {
            Setting::DeadZone => b.dead_zone = ((b.dead_zone + 0.05 * delta as f32) * 100.0).round() / 100.0,
            Setting::MouseRange => b.mouse_range += 25.0 * delta as f32,
            Setting::InvertPitch => b.pad_invert_pitch = !b.pad_invert_pitch,
            Setting::MouseInvert => b.mouse_invert = !b.mouse_invert,
            Setting::StickLayout => {
                b.stick_layout = match b.stick_layout {
                    StickLayout::Space => StickLayout::Aircraft,
                    StickLayout::Aircraft => StickLayout::Space,
                }
            }
        }
        b.dead_zone = b.dead_zone.clamp(0.0, 0.5);
        b.mouse_range = b.mouse_range.clamp(50.0, 800.0);
        self.dirty = true;
        self.message = format!("{}: {}", s.label(), self.setting_text(s));
    }

    /// The text in an action cell.
    pub fn cell(&self, action: Action, column: usize) -> String {
        if column < 2 {
            self.bindings.keys[&action].get(column).map(pretty_key).unwrap_or_default()
        } else {
            self.bindings.pad[&action].get(column - 2).map(pretty_pad).unwrap_or_default()
        }
    }

    /// The value shown for a setting.
    pub fn setting_text(&self, s: Setting) -> String {
        let b = &self.bindings;
        let yes = |v: bool| if v { "yes" } else { "no" }.to_string();
        match s {
            Setting::DeadZone => format!("{:.2}", b.dead_zone),
            Setting::InvertPitch => yes(b.pad_invert_pitch),
            Setting::StickLayout => match b.stick_layout {
                StickLayout::Space => "space (left: yaw+pitch, right: roll)".into(),
                StickLayout::Aircraft => "aircraft (left: roll+pitch, right: yaw)".into(),
            },
            Setting::MouseRange => format!("{:.0}", b.mouse_range),
            Setting::MouseInvert => yes(b.mouse_invert),
        }
    }
}

/// Puts `item` at index `slot`, or appends it if the list is shorter.
fn set_slot<T>(list: &mut Vec<T>, slot: usize, item: T) {
    if slot < list.len() {
        list[slot] = item;
    } else {
        list.push(item);
    }
}

fn moved_message(chord: &str, label: &str, taken: &[&str]) -> String {
    if taken.is_empty() {
        format!("{label}: {chord}")
    } else {
        format!("{label}: {chord} (removed from {})", taken.join(", "))
    }
}

/// A key name as shown to players: `kp7` → `Num 7`, `leftcontrol` → `Left Ctrl`.
pub fn pretty_key(k: &KeyChord) -> String {
    let name = match k.key.as_str() {
        n if n.starts_with("kp") && n.len() == 3 => format!("Num {}", &n[2..]),
        "kpadd" => "Num +".into(),
        "kpsubtract" => "Num -".into(),
        "kpmultiply" => "Num *".into(),
        "kpdivide" => "Num /".into(),
        "kpenter" => "Num Enter".into(),
        "kpdecimal" => "Num .".into(),
        "leftcontrol" => "Left Ctrl".into(),
        "rightcontrol" => "Right Ctrl".into(),
        "leftalt" => "Left Alt".into(),
        "rightalt" => "Right Alt".into(),
        "pageup" => "Page Up".into(),
        "pagedown" => "Page Down".into(),
        "equal" => "=".into(),
        "minus" => "-".into(),
        "backslash" => "\\".into(),
        "semicolon" => ";".into(),
        "apostrophe" => "'".into(),
        "comma" => ",".into(),
        "period" => ".".into(),
        "slash" => "/".into(),
        "leftbracket" => "[".into(),
        "rightbracket" => "]".into(),
        "graveaccent" => "`".into(),
        n if n.starts_with("key") && n.len() == 4 => n[3..].to_string(),
        n => {
            let mut c = n.chars();
            match c.next() {
                Some(f) if n.len() <= 3 => f.to_ascii_uppercase().to_string() + &c.as_str().to_ascii_uppercase(),
                Some(f) => f.to_ascii_uppercase().to_string() + c.as_str(),
                None => String::new(),
            }
        }
    };
    if k.shift {
        format!("Shift+{name}")
    } else {
        name
    }
}

/// A pad chord as shown to players: `LeftShoulder+A` → `LB + A`.
pub fn pretty_pad(c: &PadChord) -> String {
    let short = |b: Button| match b {
        Button::DpadUp => "D-pad Up",
        Button::DpadDown => "D-pad Down",
        Button::DpadLeft => "D-pad Left",
        Button::DpadRight => "D-pad Right",
        Button::LeftThumb => "L-stick click",
        Button::RightThumb => "R-stick click",
        Button::LeftShoulder => "LB",
        Button::RightShoulder => "RB",
        Button::LeftTrigger => "LT",
        Button::RightTrigger => "RT",
        other => other.name(),
    };
    match c.modifier {
        Some(m) => format!("{} + {}", short(m), short(c.button)),
        None => short(c.button).to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn editor_at(action: Action, column: usize) -> Editor {
        let mut e = Editor::new(&Bindings::default());
        e.selected = e.rows.iter().position(|r| *r == Row::Action(action)).unwrap();
        e.column = column;
        e
    }

    #[test]
    fn rows_are_grouped_and_headers_are_skipped() {
        let mut e = Editor::new(&Bindings::default());
        assert!(matches!(e.rows[0], Row::Header("Flight")));
        assert_eq!(e.row(), Row::Action(Action::PitchUp));
        e.move_row(-5);
        assert_eq!(e.selected, 1, "cannot move onto the first header");
        let n_actions = e.rows.iter().filter(|r| matches!(r, Row::Action(_))).count();
        assert_eq!(n_actions, ACTIONS.len(), "every action is listed once");
        e.move_row(1000);
        assert_eq!(e.row(), Row::Setting(Setting::MouseInvert));
        // Moving down from the last action of a group lands on the next group's first.
        let mut e = editor_at(Action::SlideDown, 0);
        e.move_row(1);
        assert_eq!(e.row(), Row::Action(Action::ThrottleUp));
        e.move_column(-1);
        assert_eq!(e.column, 3);
    }

    #[test]
    fn capturing_a_key_rebinds_and_steals_it_from_others() {
        let mut e = editor_at(Action::Cloak, 0);
        e.activate();
        assert_eq!(e.capture, Some(Capture::Key));
        e.capture_key("leftshift", true);
        assert_eq!(e.capture, Some(Capture::Key), "Shift alone waits");
        e.capture_key("t", false); // T was next target
        assert_eq!(e.capture, None);
        assert_eq!(e.bindings.keys[&Action::Cloak][0], KeyChord::parse("T").unwrap());
        assert!(e.bindings.keys[&Action::NextTarget].is_empty());
        assert!(e.message.contains("removed from Next target"), "{}", e.message);
        assert!(e.dirty);
        // Escape cancels.
        e.activate();
        e.capture_key("escape", false);
        assert_eq!((e.capture, e.message.as_str()), (None, "Cancelled"));
    }

    #[test]
    fn capturing_pad_plain_presses_and_chords() {
        let mut e = editor_at(Action::Ecm, 2);
        let mut prev = PadState::default();
        let mut now = prev;
        e.activate();
        now.set(Button::Y, true);
        e.capture_pad(&now, &prev);
        assert_eq!(e.capture, Some(Capture::Pad { first: Some(Button::Y) }));
        prev = now;
        now.set(Button::Y, false);
        e.capture_pad(&now, &prev);
        assert_eq!(e.bindings.pad[&Action::Ecm][0], PadChord::plain(Button::Y));
        assert!(e.bindings.pad[&Action::ViewChase].is_empty(), "taken from the chase view");
        // Hold RB, press B: a chord, in the alternate slot.
        e.column = 3;
        e.activate();
        let mut prev = PadState::default();
        let mut now = prev;
        now.set(Button::RightShoulder, true);
        e.capture_pad(&now, &prev);
        prev = now;
        now.set(Button::B, true);
        e.capture_pad(&now, &prev);
        assert_eq!(e.bindings.pad[&Action::Ecm].len(), 2);
        assert_eq!(e.cell(Action::Ecm, 3), "RB + B");
    }

    #[test]
    fn clear_reset_and_settings() {
        let mut e = editor_at(Action::RollLeft, 1);
        assert_eq!(e.cell(Action::RollLeft, 0), "Num 7");
        assert_eq!(e.cell(Action::RollLeft, 1), "Q");
        e.clear();
        assert_eq!(e.cell(Action::RollLeft, 1), "");
        e.reset_selected();
        assert_eq!(e.cell(Action::RollLeft, 1), "Q");
        e.reset_all();
        assert_eq!(e.bindings, Bindings::default());
        e.selected = e.rows.iter().position(|r| *r == Row::Setting(Setting::DeadZone)).unwrap();
        e.adjust(1);
        assert!((e.bindings.dead_zone - 0.2).abs() < 1e-6);
        for _ in 0..20 {
            e.adjust(1);
        }
        assert_eq!(e.bindings.dead_zone, 0.5, "clamped");
        e.selected = e.rows.iter().position(|r| *r == Row::Setting(Setting::StickLayout)).unwrap();
        e.activate();
        assert_eq!(e.bindings.stick_layout, StickLayout::Aircraft);
        e.reset_selected();
        assert_eq!(e.bindings.stick_layout, StickLayout::Space);
    }

    #[test]
    fn pretty_names() {
        let k = |s: &str| pretty_key(&KeyChord::parse(s).unwrap());
        assert_eq!(k("kp7"), "Num 7");
        assert_eq!(k("Shift+t"), "Shift+T");
        assert_eq!(k("f10"), "F10");
        assert_eq!(k("backspace"), "Backspace");
        assert_eq!(k("semicolon"), ";");
        assert_eq!(k("key1"), "1");
        assert_eq!(pretty_pad(&PadChord::parse("LeftShoulder+DpadUp").unwrap()), "LB + D-pad Up");
    }
}
