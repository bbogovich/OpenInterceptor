# Structs, enums and pattern matching

## Structs: plain data — `src/sim/mod.rs`

```rust
/// What a pilot (human or AI) wants the ship to do this tick.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Controls {
    pub pitch: f32,                 // +1 = nose up
    pub yaw: f32,                   // +1 = nose right
    pub roll: f32,                  // +1 = roll right
    pub slide: Vec2,                // VCT: x right, y up
    pub throttle_set: Option<f32>,  // set the throttle this tick
    pub throttle_delta: f32,
    pub afterburner: bool,
    pub fire_beams: bool,
    pub fire_missile: bool,         // one-shot
    …
}
```

`#[derive(...)]` asks the compiler to write common trait implementations: copying (`Clone`,
`Copy`), printing with `{:?}` (`Debug`), comparing (`PartialEq`), a default value (`Default`).
Plain data with public fields, built with struct literals and `..Default::default()`:

```rust
Controls { yaw: 0.12, throttle_set: Some(0.35), ..Default::default() }   // sim/ai.rs, a practice target
```

## Enums: one of several things

Rust enums can carry data in each variant — like a tagged union, but safe.

```rust
// src/sim/mod.rs
pub enum SimEvent {
    BeamFired { ship: ShipId, position: Vec3 },
    Hit { ship: ShipId, position: Vec3, shield: bool },
    Destroyed { ship: ShipId, position: Vec3, radius: f32 },
    LockAcquired { ship: ShipId },
    Message { ship: ShipId, text: String },
    …
}

// src/content/vfs.rs
pub enum AssetRef {
    PackFile(String),
    Original { file: Option<String>, resource: Option<String> },
}
```

## `match`: handle every case — `src/app/flight.rs`

```rust
match view {
    View::Cockpit | View::Forward => (eye, rot),
    View::Left => (eye, look(90.0)),
    View::Right => (eye, look(-90.0)),
    View::RearLeft => (eye, look(135.0)),
    View::RearRight => (eye, look(-135.0)),
    View::Chase => (pos + rot * Vec3::new(0.0, radius * 0.7, radius * 3.2), …),
    View::ChaseManual => { … }
}
```

`match` must be **exhaustive**: add a `View::Target` variant and every `match` that forgot it
stops compiling. That turns "forgot to handle the new view" from a runtime bug into a build
error. Patterns can also test values: `SimEvent::LockAcquired { ship } if *ship == player =>`
in `event_messages`.

## `Option`: maybe a value

There is no null. A value that may be absent is `Option<T>`: `Some(x)` or `None`.

```rust
pub target: Option<ShipId>,                       // Ship, src/sim/mod.rs

// hud::mfd_lines: `and_then` chains two "maybe" steps
match me.target.and_then(|t| world.get(t)) {
    None => vec!["TARGET".into(), "none".into(), "T next  R nearest".into()],
    Some(t) => { … }
}
```

Longer chains read like a sentence. The flight session's cockpit setup
(`FlightSession::start`, `src/app/flight.rs`) loads a 3D cockpit only if the ship *has* a
cockpit, that cockpit *has* a `Model`, and the model *loads*:

```rust
let cockpit_model = cockpit_def.and_then(|c| {
    let model = c.model.as_ref()?;          // `?` on an Option: return None early
    let gpu = ctx.renderer.models.get(content, model, c.image_pack, c.model_scale)?;
    Some((gpu, Vec3::from(c.model_offset)))
});
// `filter` keeps a Some only if the condition holds:
let (instruments, design) = match cockpit_def.filter(|_| has_cockpit) {
    Some(c) => (c.instruments.clone(), Vec2::from(c.design)),
    None => (hud::default_instruments(), hud::DESIGN),
};
```

`?` works on `Option` inside a closure or function that returns `Option`, just as it does on
`Result` — the first `None` ends the chain.

## Newtypes

A struct with one field gives a plain number a distinct type:

```rust
// src/content/rules.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ShipClassId(pub u16);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BeamId(pub u16);
```

Now a `BeamId` cannot be passed where a `ShipClassId` is expected, even though both are `u16`.

## A numeric gotcha: `f32::signum(0.0)` is `1.0`

Floats are not integers. `0.0_f32.signum()` returns **1.0** (zero is "positive zero"), unlike
`0_i32.signum()`, which returns 0. The model viewer once used `mouse_wheel().1.signum()` as the
zoom step, so a wheel that had not moved still zoomed in every frame and the camera ran away.
The fix is an explicit helper — `src/app/viewer.rs`:

```rust
pub fn wheel_steps(wheel: f32) -> f32 {
    if wheel == 0.0 { 0.0 } else { wheel.signum() }
}
```

Other float facts this code relies on: `NaN != NaN` (the gamepad shim checks
`value != value` to detect NaN); `min`/`max`/`clamp` exist on floats but `Ord` does not, so
sorting floats uses `total_cmp` (`collision`, `targeting`).
