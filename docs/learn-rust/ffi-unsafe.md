# FFI, `repr(C)` and `unsafe`

macOS reports gamepads through Apple's **GameController** framework, which is Objective-C. Rust
calls it through a tiny Objective-C file (`macos/shim/oi_platform.m`) compiled by `build.rs`,
using the **foreign function interface** (FFI).

## The shared struct: `#[repr(C)]`

Rust may reorder a struct's fields for efficiency. When C code reads the same memory, the layout
must match C's, which `#[repr(C)]` guarantees:

```rust
/// Must match `OiPad` in the shim, field for field.
#[repr(C)]
#[derive(Default)]
struct OiPad {
    buttons: u16, // bitmask in XInput order
    lt: u8,
    rt: u8,
    lx: i16,
    ly: i16,
    rx: i16,
    ry: i16,
}
```

## Declaring the C function

```rust
extern "C" {
    fn oi_pad_state(index: u32, out: *mut OiPad) -> i32;
}
```

`*mut OiPad` is a **raw pointer**: Rust cannot check what the other side does with it.

## `unsafe`: a promise the compiler cannot check

Calling a foreign function is `unsafe`: the programmer, not the compiler, vouches that the call
is correct. We confine it to one small function and explain why it is sound:

```rust
pub fn pad_state(index: u32) -> Option<PadState> {
    let mut raw = OiPad::default();
    // SAFETY: `raw` is a valid, writable `OiPad` with the shim's exact C layout
    // (`#[repr(C)]`, same field order and types); the shim writes it only on success and
    // keeps no pointer to it after returning.
    let connected = unsafe { oi_pad_state(index, &mut raw) } == 1;
    connected.then_some(PadState { buttons: raw.buttons, lt: raw.lt, rt: raw.rt, … })
}
```

## Our rules for `unsafe`

1. Only where there is no safe alternative. There are exactly **two** `unsafe` blocks:
   - `src/platform/macos.rs` — the FFI call above;
   - `src/render/mod.rs`, `unsafe_gl()` — macroquad's `get_internal_gl()` is marked `unsafe`
     because the returned reference must not be held across `next_frame().await`; we use it
     immediately to push and pop the model matrix and never keep it.
2. Each block is as small as possible and has a `// SAFETY:` comment saying why it is sound.
3. A safe function wraps it, so callers never write `unsafe`.

The Linux joystick reader needs **no** `unsafe`: it opens `/dev/input/js0` non-blocking
(`OpenOptionsExt::custom_flags`) and decodes 8-byte events with `from_le_bytes`.

## Compiling the C side — `build.rs`

The Objective-C file is compiled by the build script with the system toolchain
(`xcrun clang -c -fobjc-arc …`, then `xcrun ar crs liboi_platform.a`), and the script prints
`cargo:rustc-link-lib=static=oi_platform` and `cargo:rustc-link-lib=framework=GameController`
so Cargo links them. No extra crate is needed.
