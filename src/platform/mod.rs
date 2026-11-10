//! # Platform layer
//!
//! The only place with operating-system-specific code. Everything else in the engine
//! is portable. `#[cfg(target_os = "…")]` attributes choose, *at compile time*, which
//! implementation is built: a macOS build contains no Linux code and vice versa.
//!
//! | Platform | Gamepads |
//! |---|---|
//! | macOS | GameController.framework, through the Objective-C shim (`macos.rs`) |
//! | Linux | the kernel joystick API, `/dev/input/js*` (`linux.rs`), std only |
//! | other | none yet (Windows XInput is planned) |

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;

use crate::input::gamepad::PadState;

/// Reads the first connected game controller. Holds whatever per-device state the
/// platform needs between frames (on Linux, the open device and its last values).
#[derive(Default)]
pub struct Gamepads {
    #[cfg(target_os = "linux")]
    linux: linux::JoystickReader,
}

impl Gamepads {
    pub fn new() -> Gamepads {
        Gamepads::default()
    }

    /// The first controller's state this frame, or `None` if none is connected.
    pub fn poll(&mut self) -> Option<PadState> {
        #[cfg(target_os = "macos")]
        {
            macos::pad_state(0)
        }
        #[cfg(target_os = "linux")]
        {
            self.linux.poll()
        }
        #[cfg(not(any(target_os = "macos", target_os = "linux")))]
        {
            None
        }
    }
}
