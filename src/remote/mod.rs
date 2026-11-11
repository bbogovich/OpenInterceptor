//! # Remote control surfaces: the tablet console
//!
//! The game can serve a *control console*: a web page that a tablet on the same network
//! opens next to a joystick or gamepad. It shows the craft's condition live and offers
//! every control as a labelled button, so nothing depends on remembering shortcuts.
//!
//! ```text
//!  game loop (app::flight)                    tablet browser
//!   ├─ telemetry 20 Hz ─▶ ConsoleServer ──WebSocket──▶ web/console/*.js
//!   └─ RawInput.remote_* ◀── Event::Input ◀───────────── {type:"action", …}
//! ```
//!
//! - [`http`] and [`ws`]: just enough HTTP/1.1 and WebSocket (RFC 6455), with SHA-1 and
//!   base64, written in the crate (no dependency).
//! - [`json`]: a small JSON reader and writer.
//! - [`protocol`]: the messages (spec: *Console protocol*).
//! - [`server`]: threads, pairing and the local-network rule.
//!
//! Console input reaches the simulation through the same path as a key press: held and
//! tapped [`crate::input::Action`]s are added to [`crate::input::RawInput`], and
//! [`crate::input::Bindings::translate`] treats them like bound keys. `sim` knows nothing
//! about the console.
//!
//! This module sits in the app layer: it uses threads and sockets, and reads the world
//! only to describe it.

pub mod http;
pub mod json;
pub mod protocol;
pub mod server;
pub mod ws;

use crate::input::{Action, RawInput};
use std::collections::{HashMap, HashSet};

/// The console's input state, merged into each frame's [`RawInput`].
#[derive(Debug, Default)]
pub struct RemoteInput {
    /// Actions held, per console (so one console letting go does not release another's).
    held: HashMap<u32, HashSet<Action>>,
    pressed: HashSet<Action>,
    throttle: Option<f32>,
}

impl RemoteInput {
    /// Applies one console event.
    pub fn apply(&mut self, event: &server::Event) {
        match event {
            server::Event::Input(id, protocol::Inbound::Action { action, down }) => {
                if protocol::HELD.contains(action) {
                    let set = self.held.entry(*id).or_default();
                    if *down {
                        set.insert(*action);
                    } else {
                        set.remove(action);
                    }
                } else if *down {
                    self.pressed.insert(*action);
                }
            }
            server::Event::Input(_, protocol::Inbound::Throttle(t)) => self.throttle = Some(*t),
            server::Event::Left(id) => {
                // A console that drops off the network lets go of everything it held.
                self.held.remove(id);
            }
            _ => {}
        }
    }

    /// Adds this frame's console input to `raw`, and clears the one-frame taps.
    pub fn merge_into(&mut self, raw: &mut RawInput) {
        for set in self.held.values() {
            raw.remote_held.extend(set.iter().copied());
        }
        raw.remote_pressed.extend(self.pressed.drain());
        raw.remote_throttle = self.throttle.take();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use protocol::Inbound;
    use server::Event;

    #[test]
    fn held_actions_last_until_released_and_taps_fire_once() {
        let mut r = RemoteInput::default();
        r.apply(&Event::Input(1, Inbound::Action { action: Action::FireBeams, down: true }));
        r.apply(&Event::Input(1, Inbound::Action { action: Action::NextTarget, down: true }));
        r.apply(&Event::Input(2, Inbound::Throttle(0.4)));
        let mut raw = RawInput::default();
        r.merge_into(&mut raw);
        assert!(raw.remote_held.contains(&Action::FireBeams) && raw.remote_pressed.contains(&Action::NextTarget));
        assert_eq!(raw.remote_throttle, Some(0.4));
        let mut raw = RawInput::default();
        r.merge_into(&mut raw);
        assert!(raw.remote_held.contains(&Action::FireBeams), "still held");
        assert!(raw.remote_pressed.is_empty() && raw.remote_throttle.is_none(), "taps fire once");
        // The bindings see a console-held action like a held key.
        let (c, cmds, _) = crate::input::Bindings::default().translate(&raw, false);
        assert!(c.fire_beams && cmds.is_empty());
        r.apply(&Event::Left(1));
        let mut raw = RawInput::default();
        r.merge_into(&mut raw);
        assert!(raw.remote_held.is_empty(), "a console that leaves lets go");
    }
}
