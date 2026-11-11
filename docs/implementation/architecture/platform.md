# Platform layer

`src/platform/`: the only operating-system-specific code. `#[cfg(target_os = "…")]` chooses at
compile time which implementation is built. Plan: Stage 1 (gamepads); Stage 7 (packaging).

| Platform | Gamepads | Window / GL / audio |
|---|---|---|
| macOS | GameController.framework via the Objective-C shim `macos/shim/oi_platform.m` | macroquad (Cocoa, OpenGL, CoreAudio) |
| Linux | kernel joystick API `/dev/input/js0`, `std` only (`linux.rs`) | macroquad (X11, OpenGL, ALSA) |
| other | none yet (Windows XInput planned) | macroquad |

## Architecture Overview

```mermaid
flowchart LR
    A["app::devices::Devices::read"] --> G["platform::Gamepads::poll"]
    G -->|macOS| M["macos::pad_state(0)<br/>unsafe extern \"C\" oi_pad_state(index, *mut OiPad)"]
    G -->|Linux| J["linux::JoystickReader::poll<br/>non-blocking read of 8-byte js events"]
    M --> P["input::gamepad::PadState"]
    J --> P
```

- **macOS**: `build.rs` compiles `oi_platform.m` with `xcrun clang` and `ar` (no extra crates)
  and links `GameController` and `Foundation`. The shim reports the first controller with an
  extended-gamepad profile (Xbox, PlayStation, Switch Pro, MFi) in XInput layout. Adapted from
  the Looking Glass macOS port's shim (MIT).
- **Linux**: the device is opened with `O_NONBLOCK`; each read applies all pending events
  (`apply_event`): buttons 0–10 and axes 0–7 in the common xpad layout, stick Y inverted to
  XInput's "up is positive", triggers rescaled to 0–255. If no device exists it retries about
  every two seconds; unplugging drops the device and retries.

## Key Symbols

| Symbol | What it is |
|---|---|
| `Gamepads` | Owns per-platform reader state; `poll() -> Option<PadState>`. |
| `macos::OiPad` | `#[repr(C)]` mirror of the shim's struct. |
| `oi_pad_state` | The shim's C function (`macos/shim/oi_platform.m`). |
| `linux::JoystickReader`, `linux::apply_event` | Linux reader and its tested event mapping. |

## API Reference

- `Gamepads::new()`, `gamepads.poll()`.

## Debugging

- macOS: a controller must be paired and show in System Settings → Game Controllers.
- Linux: `ls -l /dev/input/js*`; your user needs read access. Non-Xbox pads may number axes
  differently — remap buttons in the controls menu or `bindings.yaml`.
- The only `unsafe` block in the project is the FFI call in `src/platform/macos.rs`, with a
  `// SAFETY:` comment stating why it is sound.
