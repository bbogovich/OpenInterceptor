# Platforms with `cfg`

`#[cfg(...)]` includes code only when a condition holds at **compile time**. Code for another OS
is not compiled at all — not just skipped at run time.

## `src/platform/mod.rs`

```rust
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;

#[derive(Default)]
pub struct Gamepads {
    #[cfg(target_os = "linux")]
    linux: linux::JoystickReader,   // this field only exists in Linux builds
}

impl Gamepads {
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
```

Every caller writes `gamepads.poll()` and never sees `cfg`. Notice that even a *struct field*
can be conditional: on macOS, `Gamepads` is an empty struct.

## `cfg!` in ordinary code — `src/cli.rs`

`cfg!(…)` is a macro that evaluates to `true`/`false`; both branches are still compiled:

```rust
if cfg!(target_os = "macos") {
    Some(home.join("Library/Application Support/OpenInterceptor"))
} else {
    Some(base.join("openinterceptor"))
}
```

## In `build.rs`

A build script runs on the build machine, so it checks the *target* with environment variables:

```rust
if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
    macos_shim(); // xcrun clang … oi_platform.m; link GameController and Foundation
}
```

## Checking another platform's code

`cargo check --target x86_64-unknown-linux-gnu` type-checks the Linux paths from a Mac (after
`rustup target add x86_64-unknown-linux-gnu`) without building or running them.

## `cfg(test)`

`#[cfg(test)]` code exists only in test builds — see [Testing](testing.md).
