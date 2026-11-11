//! # OpenInterceptor engine library
//!
//! The engine is a *library crate* (`src/lib.rs`) with a thin *binary crate*
//! (`src/main.rs`) on top. Keeping almost all code in the library means every module can
//! be unit-tested with `cargo test`, and other tools (a validator, an exporter) can reuse
//! it.
//!
//! Layers, lowest first — a layer may use the ones above it in this list, never below:
//!
//! 1. [`formats`] — bytes → plain data (original and mod file formats).
//! 2. [`model`] — the engine's format-neutral 3D model.
//! 3. [`content`] — packs, the player's original data, rulesets, scenarios, asset loading.
//! 4. [`sim`] — the deterministic space simulation (flight, combat, AI). No graphics.
//! 5. [`input`] (bindings → controls) and [`platform`] (OS-specific devices).
//! 6. [`render`], [`hud`] and [`audio`] — presentation: they read the simulation.
//! 7. [`app`] — the testbed: launcher, model viewer and flight sessions.
//!
//! Plus [`check`] (headless content validation), [`extract`] (original assets → OBJ/PNG/WAV), [`cli`] (command-line options) and
//! [`settings`] (the user data folder and where the original data is found).
pub mod app;
pub mod audio;
pub mod check;
pub mod cli;
pub mod content;
pub mod extract;
pub mod formats;
pub mod hud;
pub mod input;
pub mod mission;
pub mod model;
pub mod platform;
pub mod render;
pub mod script;
pub mod settings;
pub mod sim;
pub mod soak;
