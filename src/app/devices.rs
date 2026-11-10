//! # Reading the real input devices
//!
//! Converts macroquad's keyboard and mouse state, and the platform gamepad, into the
//! device-independent [`RawInput`] the input module works with.

use crate::input::{canonical, RawInput};
use crate::platform::Gamepads;
use macroquad::prelude::*;

/// The canonical name of a key: macroquad's `KeyCode` variant name, lower-cased
/// (`KeyCode::Kp7` → `"kp7"`, `KeyCode::PageUp` → `"pageup"`). Deriving names from the
/// enum's `Debug` output means every key macroquad knows is bindable automatically.
pub fn key_name(k: KeyCode) -> String {
    canonical(&format!("{k:?}"))
}

/// Gathers this frame's input. Keeps last frame's pad state for press detection.
pub struct Devices {
    pads: Gamepads,
    previous_pad: Option<crate::input::gamepad::PadState>,
}

impl Devices {
    pub fn new() -> Devices {
        Devices { pads: Gamepads::new(), previous_pad: None }
    }

    pub fn read(&mut self) -> RawInput {
        let held = get_keys_down().into_iter().map(key_name).collect();
        let pressed = get_keys_pressed().into_iter().map(key_name).collect();
        let (mx, my) = mouse_position();
        let pad = self.pads.poll();
        let raw = RawInput {
            held,
            pressed,
            shift: is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift),
            mouse_offset: (mx - screen_width() * 0.5, my - screen_height() * 0.5),
            mouse_left: is_mouse_button_down(MouseButton::Left),
            mouse_right_pressed: is_mouse_button_pressed(MouseButton::Right),
            pad,
            pad_previous: self.previous_pad,
        };
        self.previous_pad = pad;
        // Debug aid (documented on the docs site): OI_DEBUG_KEYS=1 prints the name of
        // every key pressed, which is the name to use in bindings.ini.
        if !raw.pressed.is_empty() && std::env::var_os("OI_DEBUG_KEYS").is_some() {
            eprintln!("keys pressed: {:?}", raw.pressed);
        }
        raw
    }
}

impl Default for Devices {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_names_match_the_bindings_vocabulary() {
        assert_eq!(key_name(KeyCode::Kp7), "kp7");
        assert_eq!(key_name(KeyCode::PageUp), "pageup");
        assert_eq!(key_name(KeyCode::F1), "f1");
        assert_eq!(key_name(KeyCode::Backslash), "backslash");
        // Every default binding names a key in the vocabulary below.
        for i in crate::input::ACTIONS {
            for k in i.keys.split(',') {
                let chord = crate::input::KeyChord::parse(k).unwrap();
                assert!(KNOWN.contains(&chord.key.as_str()), "{}: unknown key {}", i.name, chord.key);
            }
        }
    }

    /// Key names used by the default bindings, checked against `key_name` above.
    const KNOWN: &[&str] = &[
        "up",
        "down",
        "left",
        "right",
        "kp7",
        "kp9",
        "q",
        "e",
        "kp4",
        "kp6",
        "kp8",
        "kp2",
        "kp5",
        "equal",
        "kpadd",
        "minus",
        "kpsubtract",
        "backspace",
        "backslash",
        "z",
        "m",
        "tab",
        "enter",
        "leftcontrol",
        "space",
        "x",
        "w",
        "t",
        "y",
        "r",
        "g",
        "period",
        "comma",
        "f1",
        "f2",
        "f3",
        "f4",
        "f5",
        "f6",
        "f7",
        "f8",
        "insert",
        "delete",
        "pageup",
        "pagedown",
        "home",
        "end",
        "f",
        "a",
        "i",
        "n",
        "p",
        "escape",
        "c",
        "b",
        "f9",
        "f10",
        "semicolon",
        "apostrophe",
        "s",
        "h",
        "j",
        "rightbracket",
        "l",
    ];

    #[test]
    fn known_names_are_real_keycodes() {
        let codes = [
            KeyCode::Up,
            KeyCode::Kp9,
            KeyCode::KpAdd,
            KeyCode::KpSubtract,
            KeyCode::Equal,
            KeyCode::Minus,
            KeyCode::LeftControl,
            KeyCode::Period,
            KeyCode::Comma,
            KeyCode::Insert,
            KeyCode::Delete,
            KeyCode::Home,
            KeyCode::End,
            KeyCode::Escape,
            KeyCode::Kp5,
            KeyCode::Enter,
            KeyCode::Tab,
            KeyCode::Space,
            KeyCode::Semicolon,
            KeyCode::Apostrophe,
            KeyCode::RightBracket,
            KeyCode::F10,
        ];
        for c in codes {
            assert!(KNOWN.contains(&key_name(c).as_str()), "{}", key_name(c));
        }
    }
}
