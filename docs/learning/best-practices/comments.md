# Commenting standard

The code is meant to be read by people learning Rust and game engines, so it explains itself.

## Every module starts with `//!`

A module-level doc comment says what the module is for, in terms a beginner can follow, and
where it sits in the layers:

```rust
//! XMSH mesh reader.
//!
//! Every 3D model in X-COM: Interceptor (ships, stations, asteroids) is an `XMSH` resource
//! inside `xobjects1.dll` / `xobjects2.dll`. This module turns those bytes into a [`Mesh`]:
//! up to four levels of detail, each with vertices, polygons and texture names.
//!
//! Layer: formats (pure functions on bytes; no files, no GPU).
//! Format reference: docs site → Original game → Formats → XMSH meshes.
```

## Every public item has `///`

Structs, enums, functions, fields that are not self-evident: what it is, units, and what the
caller must know.

```rust
/// Maximum forward speed in metres per second (`MaxSpeed` in the original rules).
pub max_speed: f32,
```

## Comment non-obvious lines

Say **why**, not what. Units, coordinate conversions, format quirks, invariants:

```rust
// Original meshes are Direct3D (left-handed): flip Z and reverse the winding so faces stay
// front-facing in our right-handed frame.
let p = Vec3::new(v.x, v.y, -v.z);
```

## `SAFETY:` on every `unsafe`

See [FFI and unsafe](../rust/ffi-unsafe.md).

## Keep comments true

A wrong comment is worse than none. Change the comment in the same commit as the code.
`cargo doc` renders all `//!` and `///` comments into browsable documentation.
