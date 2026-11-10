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
macroquad = { version = "=0.4.14", features = ["audio"] }   # "=" pins this exact version
glam = "=0.27.0"                                            # maths (already in macroquad's tree)
image = { version = "=0.24.9", default-features = false, features = ["png"] }

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

Optional parts of a crate, switched on in `Cargo.toml`. macroquad 0.4.14 does **not** enable
sound by default, so we turn on its `audio` feature (it logs "audio feature disabled" if you
forget). For `image` we do the opposite: `default-features = false` and only `png`, so no other
decoders are compiled.

## `build.rs`

A program Cargo runs before compiling the crate. Ours compiles the macOS gamepad shim and tells
Cargo which frameworks to link (`cargo:rustc-link-lib=framework=GameController`).

## Library and binary

`src/lib.rs` makes the package a library crate named `openinterceptor`; `src/main.rs` is the
binary of the same name, which uses the library like any other crate would. `cargo test` runs
the library's tests.

## Everyday commands

`cargo check` (type-check only, fastest), `cargo clippy` (lints), `cargo fmt` (formatting),
`cargo doc --open` (browse the `///` docs of the whole crate).
