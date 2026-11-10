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
// src/app/flight.rs, FlightSession::start (simplified)
let (world, player) = World::from_scenario(&scenario, rules.clone(), Arc::new(shapes), choice)?;
let session = FlightSession { world, player, … }; // `world` moved into the session
// world.step();  // error: value moved
```

This is how OpenInterceptor makes ownership obvious: the `FlightSession` owns its `World`; the
`Renderer`'s `ModelCache` owns the GPU meshes and textures. Nobody else can free them by
accident.

## Sharing: `Arc` and `Rc`

Sometimes several owners really do need the same data. The `World` and the session share one
read-only `Ruleset` through `Arc<Ruleset>` (*atomically reference-counted*): cloning an `Arc`
copies a pointer and bumps a counter; the ruleset is freed when the last owner goes. The model
cache hands out `Rc<GpuModel>` — the cheaper single-threaded version — because rendering
happens on one thread.

Small plain values (`f32`, `u32`, `Vec3`, our `ShipId`) implement `Copy`, so they are copied
instead of moved.

## Borrowing: `&` and `&mut`

Instead of moving, you can **borrow**:

- `&T` — a shared, read-only reference. Any number may exist at once.
- `&mut T` — an exclusive, writable reference. Only one may exist, and no `&T` at the same time.

```rust
impl Renderer {
    // Drawing only reads the world; it changes the renderer's own caches and buffers.
    pub fn draw_world(&mut self, content: &Content, world: &World, effects: &Effects,
                      cam: &SpaceCamera, sun: Vec3, hide: Option<ShipId>) { … }
}
impl World {
    // Stepping changes the world.
    pub fn step(&mut self) { … }
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

Fixes used in this codebase:

- Work with **indices** (`ShipId`) instead of long-lived references.
- Copy out the small values you need first (`let target_pos = t.position;`).
- Borrow **different fields** at the same time — allowed, because the compiler can see they do
  not overlap. `combat::fire_beams` holds `&mut w.ships[i]` while pushing into `w.bolts`.
- Read first, write later: `ai::think` computes every decision into a list, then applies it.
- Hold your own handle: `World::step` does `let rules = self.rules.clone();` (an `Arc` clone) so
  it can borrow a ship class from `rules` while mutably iterating `self.ships`.

See [Data model](../best-practices/data-model.md).
