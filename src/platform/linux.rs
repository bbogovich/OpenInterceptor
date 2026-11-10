//! # Linux: gamepads through the kernel joystick API
//!
//! Linux exposes each controller as `/dev/input/js0`, `js1`, … Reading the file yields
//! 8-byte events: `u32 time, i16 value, u8 type, u8 number`. Type 1 is a button, type 2
//! an axis; bit `0x80` marks the initial-state events sent when the device is opened.
//! We open the device *non-blocking* so a read with no new events returns immediately
//! instead of freezing the game.
//!
//! Button and axis numbers follow the common `xpad` / SDL layout used by Xbox-style
//! controllers (the most widespread on Linux); other pads may map differently and can
//! be remapped in `bindings.ini`.
//!
//! Only `std` is used: no udev, no extra crates.

use crate::input::gamepad::{Button, PadState};
use std::fs::File;
use std::io::Read;
use std::os::unix::fs::OpenOptionsExt;

/// `O_NONBLOCK` on Linux (the same value on x86-64 and ARM).
const O_NONBLOCK: i32 = 0o4000;

/// Keeps the open device and the state accumulated from its events.
#[derive(Default)]
pub struct JoystickReader {
    device: Option<File>,
    state: PadState,
    /// Frames until we try to open a device again after none was found.
    retry: u32,
}

impl JoystickReader {
    /// Applies all pending events and returns the current state.
    pub fn poll(&mut self) -> Option<PadState> {
        if self.device.is_none() {
            if self.retry > 0 {
                self.retry -= 1;
                return None;
            }
            self.device = std::fs::OpenOptions::new().read(true).custom_flags(O_NONBLOCK).open("/dev/input/js0").ok();
            if self.device.is_none() {
                self.retry = 120; // look again in about two seconds
                return None;
            }
            self.state = PadState::default();
        }
        let mut buf = [0u8; 8 * 64];
        loop {
            let read = self.device.as_mut().map(|d| d.read(&mut buf));
            match read {
                Some(Ok(0)) => break,
                Some(Ok(n)) => {
                    for e in buf[..n].chunks_exact(8) {
                        apply_event(&mut self.state, i16::from_le_bytes([e[4], e[5]]), e[6] & 0x7f, e[7]);
                    }
                }
                Some(Err(e)) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                _ => {
                    // Unplugged or unreadable: forget the device and retry later.
                    self.device = None;
                    return None;
                }
            }
        }
        Some(self.state)
    }
}

/// Applies one joystick event to `state` (xpad layout).
pub fn apply_event(state: &mut PadState, value: i16, kind: u8, number: u8) {
    match kind {
        1 => {
            let b = match number {
                0 => Button::A,
                1 => Button::B,
                2 => Button::X,
                3 => Button::Y,
                4 => Button::LeftShoulder,
                5 => Button::RightShoulder,
                6 => Button::Back,
                7 => Button::Start,
                9 => Button::LeftThumb,
                10 => Button::RightThumb,
                _ => return,
            };
            state.set(b, value != 0);
        }
        2 => match number {
            0 => state.lx = value,
            // The kernel reports Y positive when pushed *down*; XInput is positive up.
            1 => state.ly = value.saturating_neg(),
            2 => state.lt = trigger(value),
            3 => state.rx = value,
            4 => state.ry = value.saturating_neg(),
            5 => state.rt = trigger(value),
            6 => {
                state.set(Button::DpadLeft, value < 0);
                state.set(Button::DpadRight, value > 0);
            }
            7 => {
                state.set(Button::DpadUp, value < 0);
                state.set(Button::DpadDown, value > 0);
            }
            _ => {}
        },
        _ => {}
    }
}

/// Kernel triggers run −32767 (released) … 32767 (pressed).
fn trigger(value: i16) -> u8 {
    ((value as i32 + 32767) * 255 / 65534) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn events_map_to_xinput_layout() {
        let mut s = PadState::default();
        apply_event(&mut s, 1, 1, 0);
        assert!(s.button(Button::A));
        apply_event(&mut s, 0, 1, 0);
        assert!(!s.button(Button::A));
        apply_event(&mut s, 32767, 2, 1);
        assert_eq!(s.ly, -32767, "down on the stick is negative Y");
        apply_event(&mut s, 32767, 2, 5);
        assert_eq!(s.rt, 255);
        apply_event(&mut s, -32767, 2, 7);
        assert!(s.button(Button::DpadUp) && !s.button(Button::DpadDown));
        apply_event(&mut s, 1, 1, 42);
        apply_event(&mut s, 1, 9, 0);
    }
}
