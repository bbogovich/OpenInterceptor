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

### Changing shared data: `Arc::make_mut` — `src/content/mod.rs`

A compiled script (`Program`) sits in an `Arc` because action lists are cloned whenever they
run and every ship flying a behaviour uses the same program. But once, after loading, the
programs must be *changed*: their `@text` keys are replaced by the words of the chosen language.
`Arc::make_mut(&mut arc)` returns a `&mut` to the inside — **copying it first only if another
`Arc` also points at it** (*copy-on-write*). At load time no other `Arc` exists yet, so nothing is
copied; if one did, the change could not leak into the other owner's copy.

```rust
let program = std::sync::Arc::make_mut(p);   // &mut Program, cloned only if shared
program.texts_mut(&mut |s| { /* replace @key with its text */ });
```

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

## Interior mutability: `Cell` — `src/sim/ai.rs`

The condition language asks a context for values through a trait whose method takes `&self`:

```rust
pub trait Context {
    fn query(&self, id: u16, args: &[f32]) -> Value;
}
```

Shared access is right for almost every query — reading a ship's hull changes nothing. But
`random(0.3)` must advance the pilot's random-number generator, which is a change. Rather than
make every query take `&mut self`, `ObjectContext` keeps the generator state in a
`std::cell::Cell`:

```rust
pub struct ObjectContext<'a> {
    pub world: &'a World,
    pub me: ShipId,
    pub state_time: f32,
    pub rng: Cell<u64>,         // changeable through &self
}
// inside query(&self, …):
let mut r = self.rng.get();     // copy out
let roll = next_random(&mut r);
self.rng.set(r);                // copy back
```

`Cell<T>` allows changing a `Copy` value through a shared reference by only ever copying it
in and out — no reference to the inside is ever handed out, so the borrow rules still hold
without any run-time check. (`RefCell` does the same for non-`Copy` values, checking borrows at
run time.) After the transitions have been tested, the AI copies the advanced state back into
its own memory with `cx.rng.get()`.

### Lending one `&mut` as shared, many times — `src/campaign/story.rs`

A campaign script reads the campaign (queries take `&self`) and changes it (actions take
`&mut self`), alternately. `CampaignHost` holds the single `&mut Campaign`; for each query it
lends it out shared (a *reborrow*: through `&self`, the `&mut Campaign` can only be read as `&Campaign`) for just that call, and for each action uses it
mutably. Because the two never overlap, the borrow checker accepts both. The campaign's roll
generator must advance during queries (`random`), which take `&self`, so while the script runs
the host keeps it in a `Cell<u64>` — exactly the `ObjectContext` trick above — and writes it
back into the campaign around each action and at the end.

## Moving a value out of a place: `mem::replace` and `take`

Rust will not let you move a value out of a variable or field that is still in use — the place
would be left holding nothing. `std::mem::replace(&mut place, new)` swaps a new value in and
hands back the old one; `std::mem::take(&mut place)` does the same with the type's `Default`
(an empty `Vec`, `None` for an `Option`). `Option::take()` is the `Option` version.

The campaign's battle hand-off needs exactly this (`src/app/mod.rs`, app 0.7.1). The current
screen is an enum value, `screen: Screen`, and the strategy screen inside it owns the whole
campaign. To fly a battle, `app::run` must put a flight screen in `screen` **without dropping
the campaign**:

```rust
// Swap a launcher in, and get the strategy screen out by value.
let old = std::mem::replace(&mut screen, Screen::Launcher(launcher::Launcher::new(&ctx)));
if let Screen::Strategy(s) = old {
    ctx.campaign = Some(s); // AppContext keeps it while the battle is flown
}
```

and after the debrief, `ctx.campaign.take()` moves it back out of the `Option`, leaving `None`.
The strategy screen is a `Box<StrategyScreen>`, so these moves copy one pointer, not the
campaign. Without `replace`, the code would need `screen` to be an `Option<Screen>` checked
everywhere, or a clone of the whole campaign. The same idea empties buffers each frame
(`std::mem::take`, [Data model](../best-practices/data-model.md)) and a campaign's delivery
list each hour (`std::mem::take(&mut self.deliveries)`, then the ones still on their way are
put back).

## Two borrows from two owners

`StrategyScreen::frame(&mut self, ctx: &mut AppContext)` reads the strategy rules and changes
the campaign in the same lines:

```rust
let Some(def) = ctx.content.rules.strategy.as_ref() else { return Next::Launcher };
// … later, while `def` is still in use:
self.campaign.battle_result(def, id, "retreat", &[], 0);
```

`def` is a shared borrow of something inside `ctx`; `self.campaign` is borrowed mutably from
`self`. They are different owners, so the borrows cannot overlap and the compiler accepts
both. Had the campaign lived in `ctx` too (say `ctx.campaign`), the same two lines would
borrow `ctx` shared and mutably at once and fail to compile. Keeping the rules (`ctx.content`,
read-only during play) and the game state (`self.campaign`) in separate owners is what makes
the screen's code simple. While `def` is borrowed, `ctx` may still be used **shared**:
`self.save(ctx)` takes `&AppContext`, so it is allowed too.

## Ending a borrow early: copy the reference out — `src/script/stmt.rs`

The script interpreter `Vm` holds `prog: &Program` and calls its own `&mut self` methods while
walking a function's body, which lives *inside* the program:

```rust
fn call(&mut self, f: usize, base: usize) -> Result<Option<Value>, OutOfBudget> {
    // Copy the `&Program` out of `self` first: `func` then borrows the program, not
    // `self`, so `self` stays free for the `&mut self` calls below.
    let prog = self.prog;
    let func = &prog.funcs[f];
    let flow = self.block(&func.body, base);   // &mut self — allowed
    // …
}
```

Writing `let func = &self.prog.funcs[f];` would borrow *through* `self`, and `self.block(..)`
(which needs `&mut self`) would be refused while `func` is alive. A shared reference is `Copy`,
so `let prog = self.prog;` makes a second `&Program` with the same lifetime that is no longer
tied to `self`; borrowing from it leaves `self` free.

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
