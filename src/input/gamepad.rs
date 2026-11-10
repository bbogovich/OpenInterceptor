//! # Gamepad state
//!
//! Every platform reports controllers differently (GameController on macOS, the kernel
//! joystick API on Linux, XInput on Windows). The platform layer converts all of them
//! into this one [`PadState`], laid out like Microsoft's XInput: a bit mask of buttons,
//! two 8-bit triggers and four 16-bit stick axes. Everything above the platform layer
//! sees only `PadState`, so bindings work the same on every system.

/// Bit flags for [`PadState::buttons`], in XInput's order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Button {
    DpadUp,
    DpadDown,
    DpadLeft,
    DpadRight,
    Start,
    Back,
    LeftThumb,
    RightThumb,
    LeftShoulder,
    RightShoulder,
    A,
    B,
    X,
    Y,
    /// Triggers are analogue; as buttons they count as pressed past half-way.
    LeftTrigger,
    RightTrigger,
}

/// `(name, button)` for parsing and printing bindings.
const NAMES: [(&str, Button); 16] = [
    ("DpadUp", Button::DpadUp),
    ("DpadDown", Button::DpadDown),
    ("DpadLeft", Button::DpadLeft),
    ("DpadRight", Button::DpadRight),
    ("Start", Button::Start),
    ("Back", Button::Back),
    ("LeftThumb", Button::LeftThumb),
    ("RightThumb", Button::RightThumb),
    ("LeftShoulder", Button::LeftShoulder),
    ("RightShoulder", Button::RightShoulder),
    ("A", Button::A),
    ("B", Button::B),
    ("X", Button::X),
    ("Y", Button::Y),
    ("LeftTrigger", Button::LeftTrigger),
    ("RightTrigger", Button::RightTrigger),
];

impl Button {
    /// Every button, in XInput order (triggers last).
    pub fn all() -> impl Iterator<Item = Button> {
        NAMES.iter().map(|(_, b)| *b)
    }

    /// Parses a button name (case-insensitive).
    pub fn parse(s: &str) -> Option<Button> {
        NAMES.iter().find(|(n, _)| n.eq_ignore_ascii_case(s.trim())).map(|(_, b)| *b)
    }

    /// The name used in `bindings.ini`.
    pub fn name(self) -> &'static str {
        NAMES.iter().find(|(_, b)| *b == self).map(|(n, _)| *n).expect("every button is in NAMES")
    }

    /// The XInput bit for digital buttons (`None` for the triggers).
    fn bit(self) -> Option<u16> {
        Some(match self {
            Button::DpadUp => 0x0001,
            Button::DpadDown => 0x0002,
            Button::DpadLeft => 0x0004,
            Button::DpadRight => 0x0008,
            Button::Start => 0x0010,
            Button::Back => 0x0020,
            Button::LeftThumb => 0x0040,
            Button::RightThumb => 0x0080,
            Button::LeftShoulder => 0x0100,
            Button::RightShoulder => 0x0200,
            Button::A => 0x1000,
            Button::B => 0x2000,
            Button::X => 0x4000,
            Button::Y => 0x8000,
            Button::LeftTrigger | Button::RightTrigger => return None,
        })
    }
}

/// One controller's state, XInput layout. Stick Y is positive when pushed up.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PadState {
    pub buttons: u16,
    pub lt: u8,
    pub rt: u8,
    pub lx: i16,
    pub ly: i16,
    pub rx: i16,
    pub ry: i16,
}

impl PadState {
    /// True if `b` is pressed.
    pub fn button(&self, b: Button) -> bool {
        match b {
            Button::LeftTrigger => self.lt > 127,
            Button::RightTrigger => self.rt > 127,
            _ => self.buttons & b.bit().unwrap_or(0) != 0,
        }
    }

    /// Sets or clears a digital button (or fully presses/releases a trigger).
    pub fn set(&mut self, b: Button, down: bool) {
        match (b, b.bit()) {
            (Button::LeftTrigger, _) => self.lt = if down { 255 } else { 0 },
            (Button::RightTrigger, _) => self.rt = if down { 255 } else { 0 },
            (_, Some(bit)) if down => self.buttons |= bit,
            (_, Some(bit)) => self.buttons &= !bit,
            (_, None) => {}
        }
    }

    pub fn axis_lx(&self) -> f32 {
        norm(self.lx)
    }
    pub fn axis_ly(&self) -> f32 {
        norm(self.ly)
    }
    pub fn axis_rx(&self) -> f32 {
        norm(self.rx)
    }
    pub fn axis_ry(&self) -> f32 {
        norm(self.ry)
    }
}

/// Maps −32768..32767 to −1..1.
fn norm(v: i16) -> f32 {
    (v as f32 / 32767.0).clamp(-1.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buttons_round_trip_names_and_bits() {
        for (name, b) in NAMES {
            assert_eq!(Button::parse(&name.to_lowercase()), Some(b));
            assert_eq!(b.name(), name);
            let mut p = PadState::default();
            p.set(b, true);
            assert!(p.button(b), "{name}");
            p.set(b, false);
            assert!(!p.button(b), "{name}");
        }
        assert_eq!(Button::parse("Turbo"), None);
    }

    #[test]
    fn axes_are_normalised() {
        let p = PadState { lx: i16::MAX, ly: i16::MIN, rx: 0, ry: 16384, ..Default::default() };
        assert_eq!((p.axis_lx(), p.axis_ly(), p.axis_rx()), (1.0, -1.0, 0.0));
        assert!((p.axis_ry() - 0.5).abs() < 0.01);
    }
}
