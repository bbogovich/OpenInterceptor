# Ownership and borrowing

Rust has no garbage collector and no manual `free`. Instead, **every value has exactly one
owner**, and the value is dropped (freed) when its owner goes out of scope.

```rust
fn load() {
    let bytes: Vec<u8> = std::fs::read("xobjects1.dll").unwrap(); // `bytes` owns 28 MB
    let table = ResourceTable::parse(&bytes);                       // lends it, no copy
} // `bytes` dropped here: the memory is freed, exactly once
```

## Moving

Assigning or passing a value **moves** it: the old name can no longer be used.

```rust
let world = World::from_scenario(&scenario, &rules, &assets)?;
let session = FlightSession { world, view: View::Cockpit }; // `world` moved into the session
// world.step(...)  // error: value moved
```

This is how OpenInterceptor makes ownership obvious: the `FlightSession` owns its `World`; the
`Assets` own the GPU textures. Nobody else can free them by accident.

Small plain values (`f32`, `u32`, `Vec3`, our `ShipId`) implement `Copy`, so they are copied
instead of moved.

## Borrowing: `&` and `&mut`

Instead of moving, you can **borrow**:

- `&T` — a shared, read-only reference. Any number may exist at once.
- `&mut T` — an exclusive, writable reference. Only one may exist, and no `&T` at the same time.

```rust
impl Scene {
    // Drawing only reads the world and the assets, and changes the scene's scratch buffers.
    pub fn draw(&mut self, world: &World, assets: &Assets, camera: &CameraState) { … }
}
impl World {
    // Stepping changes the world.
    pub fn step(&mut self, dt: f32, player: &Controls) { … }
}
```

The signatures tell you, without reading the body, that drawing cannot change the battle. The
compiler enforces it: this rule ("one writer **or** many readers") is what removes data races
and many use-after-free bugs at compile time.

## A common beginner problem

```rust
let ship = &mut world.ships[0];
let target = &world.ships[1];   // error: world.ships is already borrowed mutably
```

Fixes used in this codebase: work with **indices** (`ShipId`) instead of long-lived references,
copy out the small values you need first (`let target_pos = world.ships[1].body.position;`), or
split the slice (`split_at_mut`). See [Data model](../best-practices/data-model.md).
