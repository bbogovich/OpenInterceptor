# Cargo

Cargo builds the project, fetches dependencies and runs tests.

## `Cargo.toml`

```toml
[package]
name = "openinterceptor"
version = "0.1.0"
edition = "2021"          # language edition
rust-version = "1.86"     # oldest compiler supported

[dependencies]
macroquad = "=0.4.14"     # "=" pins this exact version

[profile.dev]
opt-level = 1             # debug builds still fast enough to fly

[profile.release]
lto = "thin"              # link-time optimisation for release builds
```

## Pinned versions and `Cargo.lock`

`"=0.4.14"` accepts only that version; plain `"0.4.14"` would accept any compatible 0.4.x.
We pin exactly because an upgrade is a decision ([dependency policy](../best-practices/dependencies.md)).
`Cargo.lock` records the exact version of every transitive dependency; it is **committed**, so
every build of a given commit uses the same code.

## Profiles

| Command | Profile | Use |
|---|---|---|
| `cargo build` / `cargo run` | dev | Fast compile, debug checks (integer overflow panics) |
| `cargo build --release` | release | Optimised; what testers run |
| `cargo test` | test (dev settings) | Unit tests |

## Features

Optional parts of a crate, switched on in `Cargo.toml`. We use macroquad's defaults; if we
ever disable its audio, it would be `default-features = false`.

## `build.rs`

A program Cargo runs before compiling the crate. Ours compiles the macOS gamepad shim and tells
Cargo which frameworks to link (`cargo:rustc-link-lib=framework=GameController`).

## Everyday commands

`cargo check` (type-check only, fastest), `cargo clippy` (lints), `cargo fmt` (formatting),
`cargo doc --open` (browse the `///` docs of the whole crate).
