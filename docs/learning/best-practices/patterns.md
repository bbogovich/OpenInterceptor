# Coding patterns

## Newtype ids

```rust
// src/content/rules.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ShipClassId(pub u16);
```

Ids are small, `Copy`, and typed: a `BeamId` cannot be confused with a `MissileId`. Ships are
created and destroyed, so their id carries a **generation** and a stale id is detected:

```rust
// src/sim/mod.rs
pub struct ShipId { pub index: u32, pub generation: u32 }

pub fn get(&self, id: ShipId) -> Option<&Ship> {
    let i = id.index as usize;
    (self.generations.get(i) == Some(&id.generation)).then(|| &self.ships[i]).filter(|s| s.alive)
}
```

## Plain-data structs

Prefer structs with public fields and no hidden behaviour for data that is just data
(`Controls`, `ShipClass`, `Bolt`, `Placement`). Construct with struct literals and
`..Default::default()`:

```rust
let c = Controls { throttle_set: Some(1.0), fire_beams: true, ..Default::default() };
```

A constructor function is used only when construction has rules to enforce (e.g.
`World::from_scenario`, which validates every placement and resolves leaders).

## Explicit ownership of GPU resources

GPU meshes and textures are owned by the `Renderer`'s `ModelCache`, keyed by asset reference
and pack. Drawing code borrows an `Rc<GpuModel>` for the frame; nothing else holds a `Texture2D`
long-term except the flight session's cockpit image.

## No global mutable state

No `static mut`, no lazily-initialised global singletons for engine state. State is passed in:
`renderer.draw_world(&content, &world, &effects, &cam, sun, hide)`. This keeps functions
testable and makes data flow visible in signatures. (macroquad itself keeps a global context for the window; we
treat it as the platform and touch it only in presentation code.)

## Error types per layer

`FormatError` in `formats` (an enum, because callers can branch on it). Above it, errors are
only ever shown to people, so they are `String`s built with context
(`map_err(|e| format!("mesh {name}: {e}"))`): messages read as a path from the user's action
down to the byte. A typed error enum is introduced the day a caller needs to branch on one.

## Small functions with names that say what they do

`flight::turn_factor(speed, lock_speed)`, `targeting::nearest_hostile(world, me)`,
`hud::radar_point(local, front)`, `app::flight::view_pose(view, …)`. Pure helpers are free
functions so they can be unit tested without constructing a world or a window.

## Separate logic from presentation inside a module

Where a module must draw, the decisions are pure functions and the drawing is a thin layer on
top: `hud::mfd_lines` (tested) vs `hud::draw::draw_instruments` (painted);
`app::launcher::Menu` (tested) vs `Launcher::draw`.

## Enums over flags

`enum LinkMode { Linked, Full, Unlinked }` rather than two booleans; `enum Pilot { External,
Ai(AiState) }` so AI memory exists only for AI ships — impossible states cannot be represented.
