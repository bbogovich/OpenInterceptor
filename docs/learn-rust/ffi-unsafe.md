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
    // SAFETY: `raw` is a valid, writable OiPad with the shim's exact C layout; the shim
    // checks `index` and writes only on success; it is called on the main thread.
    let ok = unsafe { oi_pad_state(index, &mut raw) } == 1;
    ok.then(|| PadState::from(raw))
}
```

## Our rules for `unsafe`

1. Only in `src/platform/`. No other module contains the word.
2. Each `unsafe` block is as small as possible and has a `// SAFETY:` comment.
3. A safe function wraps it, so callers never need `unsafe`.

The Linux joystick reader needs **no** `unsafe`: it reads 8-byte events from
`/dev/input/js0` with ordinary file I/O and decodes them with `from_le_bytes`.
