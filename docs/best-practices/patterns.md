# Coding patterns

## Newtype ids

```rust
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ModelId(u32);
```

Ids are small, `Copy`, and typed: a `ModelId` cannot be confused with a `TextureId`. For ships,
which are created and destroyed, the id carries a **generation** so a stale id is detected:

```rust
pub struct ShipId { index: u32, generation: u32 }
```

## Plain-data structs

Prefer structs with public fields and no hidden behaviour for data that is just data (`Body`,
`Controls`, `FlightSpec`). Construct with struct literals and `..Default::default()`:

```rust
let spec = FlightSpec { max_speed: 600.0, lock_speed: 1000.0, ..Default::default() };
```

A builder is used only when construction has rules to enforce (e.g. a `World` from a
scenario, which validates every placement).

## Explicit ownership of GPU resources

GPU textures and meshes are owned by `Assets` and referenced by id. Nothing else holds a
`Texture2D` long-term. Unloading a pack means dropping its entries in one place.

## No global mutable state

No `static mut`, no lazily-initialised global singletons for engine state. State is passed in:
`fn draw(&mut self, world: &World, assets: &Assets)`. This keeps functions testable and makes
data flow visible in signatures. (macroquad itself keeps a global context for the window; we
treat it as the platform and touch it only in presentation code.)

## Error types per layer

`FormatError` in `formats`, `PackError` in `packs`, `AssetError` in `assets`; each wraps the
layer below with `From`. Messages read as a path from the user's action down to the byte.

## Small functions with names that say what they do

`turn_rates(spec, speed)`, `fan(corners)`, `closest_enemy(world, ship)`. Pure helpers are free
functions so they can be unit tested without constructing a world.

## Enums over flags

`View::Chase { manual: bool }` rather than `is_chase: bool, is_manual: bool` — impossible
states cannot be represented.
