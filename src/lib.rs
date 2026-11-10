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
pub mod formats;
pub mod model;
pub mod content;
pub mod sim;
pub mod check;
pub mod cli;
pub mod app;
