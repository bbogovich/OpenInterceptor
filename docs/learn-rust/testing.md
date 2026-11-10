# Testing in Rust

Rust has testing built in. `cargo test` compiles a test build and runs every function marked
`#[test]` — 146 of them in this project (list: [Unit tests](../testing/unit-tests.md)).

## Unit tests: next to the code — `src/sim/flight.rs`

```rust
/// How much of the turn rate remains at `speed` (1 at rest, 0 at `lock_speed`).
pub fn turn_factor(speed: f32, lock_speed: f32) -> f32 {
    if lock_speed <= 0.0 {
        return 1.0;
    }
    (1.0 - speed.abs() / lock_speed).clamp(0.0, 1.0)
}

#[cfg(test)]           // compiled only for `cargo test`
mod tests {
    use super::*;      // the parent module's items, including private ones

    #[test]
    fn helpers() {
        assert_eq!(turn_factor(0.0, 1000.0), 1.0);
        assert_eq!(turn_factor(500.0, 1000.0), 0.5);
        assert_eq!(turn_factor(2000.0, 1000.0), 0.0);
        …
    }
}
```

Because the test module is a child of the code's module, it can test private helpers too.

## Testing readers without the game data

Original data cannot be in the repository, so format tests **build tiny valid inputs by hand**.
`src/formats/xmsh.rs` has a crate-visible builder used by its own tests and by `model`'s:

```rust
#[test]
fn signed_edges_give_the_vertex_loop() {
    let m = parse(&quad()).unwrap();        // quad() calls build_test_xmsh(…)
    let lod = &m.lods[0];
    // +1 (0→1) starts at 0, +2 (1→2) at 1, -3 (2←3) at 2, -4 (3←0) at 3.
    assert_eq!(lod.polygon_indices(&lod.polygons[0]), &[0, 1, 2, 3]);
}
```

Checks against the **real** data run through `openinterceptor --check` when the player's data is
present (the gate does this).

## Testing the simulation

`sim::test_support` (a `#[cfg(test)] pub(crate) mod`) builds a tiny in-memory ruleset, so any
sim test can set up a world in two lines and step it:

```rust
let mut w = world();                          // test_support::world()
let f = w.rules.ship_id("fighter").unwrap();
let me = w.spawn(f, FactionId(0), "me", Vec3::ZERO, Quat::IDENTITY, Pilot::External);
w.get_mut(me).unwrap().controls.fire_beams = true;
w.step();
assert_eq!(w.bolts.len(), 2, "linked: both mounts");
```

Determinism (fixed `TICK`, seeded random numbers) is what makes these tests reliable.

## Testing with files

When a feature really reads folders (pack discovery, content loading), tests write a small
pack into `std::env::temp_dir()` with a unique name (the process id), run, and delete it.

## Integration tests

Files in a `tests/` folder are compiled as separate crates and see only the public API. This
project does not need them yet: unit tests plus `--check` against the real packs cover the
pipelines (`check::tests::the_sample_pack_passes` loads the repository's own `packs/`).

## Useful forms

- `assert!(cond, "message {x}")`, `assert_eq!(a, b)`, `assert!((a - b).abs() < 1e-5)` for floats.
- `assert!(matches!(result, Err(FormatError::BadMagic { .. })))` to check an enum variant.
- `cargo test proximity` runs only tests whose name contains `proximity`.
