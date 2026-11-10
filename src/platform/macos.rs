//! # macOS: the Rust side of the Objective-C gamepad shim
//!
//! This is the project's only `unsafe` code. Calling a C function is `unsafe` in Rust
//! because the compiler cannot check what the foreign code does with the pointers it
//! receives. We keep that promise small and documented: one function, one plain struct.
//!
//! `#[repr(C)]` makes Rust lay the struct out exactly as C would, field by field, so
//! both sides agree on where each field lives in memory.

use crate::input::gamepad::PadState;

/// Must match `OiPad` in `macos/shim/oi_platform.m`.
#[repr(C)]
#[derive(Default)]
struct OiPad {
    buttons: u16,
    lt: u8,
    rt: u8,
    lx: i16,
    ly: i16,
    rx: i16,
    ry: i16,
}

extern "C" {
    /// Fills `out` and returns 1 when controller `index` is connected; otherwise 0.
    fn oi_pad_state(index: u32, out: *mut OiPad) -> i32;
}

/// The `index`-th controller, or `None`.
pub fn pad_state(index: u32) -> Option<PadState> {
    let mut raw = OiPad::default();
    // SAFETY: `raw` is a valid, writable `OiPad` with the shim's exact C layout
    // (`#[repr(C)]`, same field order and types); the shim writes it only on success and
    // keeps no pointer to it after returning.
    let connected = unsafe { oi_pad_state(index, &mut raw) } == 1;
    connected.then_some(PadState {
        buttons: raw.buttons,
        lt: raw.lt,
        rt: raw.rt,
        lx: raw.lx,
        ly: raw.ly,
        rx: raw.rx,
        ry: raw.ry,
    })
}
