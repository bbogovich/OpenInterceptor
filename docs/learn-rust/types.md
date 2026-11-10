# Structs, enums and pattern matching

## Structs: plain data

```rust
/// One ship's physical state. Plain data: no behaviour hidden in setters.
#[derive(Clone, Copy, Debug, Default)]
pub struct Body {
    pub position: DVec3,     // metres, f64 for range; converted when drawing
    pub orientation: Quat,   // rotation from model space to world space
    pub velocity: Vec3,      // m/s
    pub angular_velocity: Vec3, // rad/s around the ship's own axes
}
```

`#[derive(...)]` asks the compiler to write common trait implementations: copying, printing with
`{:?}`, a default value.

## Enums: one of several things

Rust enums can carry data in each variant — like a tagged union, but safe.

```rust
pub enum View {
    Cockpit(Look),            // F1, F3–F6: which way the pilot looks
    FullForward,              // F2
    Chase { manual: bool },   // F7 / F8
}

pub enum FormatError {
    Truncated,
    BadTag { expected: [u8; 4], found: [u8; 4] },
    OffsetOutOfRange { what: &'static str, offset: usize },
}
```

## `match`: handle every case

```rust
match view {
    View::Cockpit(look) => camera_from_cockpit(ship, look),
    View::FullForward => camera_from_cockpit(ship, Look::Forward),
    View::Chase { manual: true } => chase_camera(ship, &self.manual_offset),
    View::Chase { manual: false } => chase_camera(ship, &auto_offset(ship)),
}
```

`match` must be **exhaustive**: add a `View::Target` variant and every `match` that forgot it
stops compiling. That turns "forgot to handle the new view" from a runtime bug into a build error.

## `Option`: maybe a value

There is no null. A value that may be absent is `Option<T>`: `Some(x)` or `None`.

```rust
pub target: Option<ShipId>,

if let Some(id) = ship.target {
    if let Some(target) = world.ship(id) { /* draw the target MFD */ }
}
```

## Newtypes

A struct with one field gives a plain number a distinct type:

```rust
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ModelId(u32);
```

Now a `ModelId` cannot be passed where a `TextureId` is expected, even though both are `u32`.
