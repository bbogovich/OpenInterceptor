# Platforms with `cfg`

`#[cfg(...)]` includes code only when a condition holds at compile time. Code for another OS is
not compiled at all — not just skipped at run time.

```rust
// src/platform/mod.rs
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "linux")]
mod linux;

/// State of controller `index`, or None. Works everywhere.
pub fn pad_state(index: u32) -> Option<PadState> {
    #[cfg(target_os = "macos")]
    return macos::pad_state(index);
    #[cfg(target_os = "linux")]
    return linux::pad_state(index);
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        let _ = index; // silence "unused" on other platforms
        None
    }
}
```

Every caller writes `platform::pad_state(0)` and never sees `cfg`.

## In `build.rs`

A build script runs on the build machine, so it checks the *target* with environment variables:

```rust
if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
    compile_shim(); // xcrun clang … oi_platform.m; link GameController, AppKit
}
```

## `cfg(test)`

`#[cfg(test)]` code exists only in test builds — see [Testing](testing.md).
