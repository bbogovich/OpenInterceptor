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

### Enums as decisions — `src/sim/systems.rs`

A tractor beam changes *another* ship's velocity. Changing ship B while iterating over ship A
would need two mutable borrows of `world.ships` at once, which Rust refuses. So the update runs
in two phases: first read the world and record each decision as a value, then apply them. An
enum whose variants carry data is the natural record:

```rust
enum Tow {
    /// Drag ship .0 towards velocity .1, changing it by at most .2 m/s².
    Pull(ShipId, glam::Vec3, f32),
    /// The beam of ship .0 lost its hold and switches off.
    Release(ShipId),
}

// Phase 1 (shared borrows only): tows.push(Tow::Pull(target, wanted, pull)) …
// Phase 2 (one mutable borrow at a time):
for tow in tows {
    match tow {
        Tow::Pull(t, wanted, pull) => { /* change ship t's velocity */ }
        Tow::Release(captor) => { /* switch captor's beam off */ }
    }
}
```

The same pattern drives `ai::think`. As a bonus, the result no longer depends on the order the
ships are visited.

### Tables built at compile time: `const fn` — `src/input/mod.rs`

Every bindable action is one row of a `const` table: its `bindings.yaml` name, default keys and
pad buttons, menu group and label. The rows are built by a small constructor marked `const fn`,
which the compiler may run while compiling, so the whole table is a constant baked into the
program — no start-up code, no allocation:

```rust
const fn a(name: &'static str, action: Action, keys: &'static str, pad: &'static str,
           group: Group, label: &'static str) -> ActionInfo {
    ActionInfo { name, action, keys, pad, group, label }
}

pub const ACTIONS: &[ActionInfo] = &[
    a("hyperspace", A::Hyperspace, "H", "LeftShoulder+A", G::Power, "Hyperspace out / abort"),
    …
];
```

`&'static str` fields fit because string literals live for the whole program. One table feeds
parsing, defaults, the controls menu and the docs, so they cannot disagree.

### A table computed at compile time: a `const` block — `src/formats/smacker.rs`

Sometimes a table is easier to *compute* than to write out. Smacker's block run lengths are 1 to
59 and then 128, 256, 512, 1024 and 2048. Instead of 64 literals, the decoder fills the array in
a **`const` block**: a `{ … }` expression after `const NAME: Type =`, which the compiler runs
while compiling.

```rust
/// Block run lengths, indexed by bits 2..7 of a `type` code.
const SIZES: [u32; 64] = {
    let mut t = [0u32; 64];
    let mut i = 0;
    while i < 59 {
        t[i] = i as u32 + 1;
        i += 1;
    }
    t[59] = 128;
    t[60] = 256;
    t[61] = 512;
    t[62] = 1024;
    t[63] = 2048;
    t
};
```

Code in a `const` context may only do what the compiler can evaluate: local `let mut`
variables, arithmetic, array indexing, `if` and `while` loops are fine; **`for` loops are not**
(they go through the `Iterator` trait, whose methods are not `const`), which is why this one is
a `while`. Allocation, I/O and ordinary function calls are not allowed either — only `const fn`s.
The block's last expression, `t`, is the value. The result is the same as typing the 64 numbers:
a constant baked into the program, with no start-up code and nothing to initialise at run time,
and the rule that made it stays readable in the source.

### Recursive enums and recursive descent — `src/script/mod.rs`

A parsed condition such as `hull < 25% and not disabled` is a **tree**. An enum can describe a
tree if a variant holds more of the same enum — but a value cannot contain itself directly (its
size would be infinite), so the children are put on the heap with `Box`:

```rust
pub enum Expr {
    Num(f32),
    Bool(bool),
    Query(u16, Vec<Expr>),          // a Vec is already on the heap
    Not(Box<Expr>),
    Binary(Op, Box<Expr>, Box<Expr>),
    …
}
```

The parser that builds it is **recursive descent**: one method per precedence level, each
calling the next-tighter level for its operands. Lower levels bind tighter, so `a or b and c`
parses as `a or (b and c)` with no precedence table:

```rust
fn or(&mut self) -> Result<(Expr, Type), String> {
    let (mut e, mut t) = self.and()?;               // the tighter level first
    while self.eat_word("or") {
        let (r, rt) = self.and()?;
        Self::want(Type::Bool, t, "'or'")?;          // type check while parsing
        Self::want(Type::Bool, rt, "'or'")?;
        e = Expr::Binary(Op::Or, Box::new(e), Box::new(r));
        t = Type::Bool;
    }
    Ok((e, t))
}
```

`not` calls itself for `not not x`, and the innermost level (`primary`) calls back up to `or`
for a bracketed sub-expression — that is the "recursive". Evaluation is a `match` over the
enum that calls `eval` on the boxed children.

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
