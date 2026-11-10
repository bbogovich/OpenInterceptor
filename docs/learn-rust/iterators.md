# Iterators and closures

An **iterator** produces values one at a time; adapters like `map` and `filter` build pipelines
that compile down to plain loops.

## Triangles of a model — `src/model.rs`

```rust
pub fn triangles(&self) -> impl Iterator<Item = [Vec3; 3]> + '_ {
    self.groups.iter().flat_map(move |g| {
        g.indices.chunks_exact(3).map(move |t| {
            [
                self.vertices[t[0] as usize].position,
                self.vertices[t[1] as usize].position,
                self.vertices[t[2] as usize].position,
            ]
        })
    })
}
```

`move |t| …` is a **closure**: an anonymous function that can capture variables from where it is
written (here `self`). The collision mesh for a station is just
`model.collision_lod().triangles().collect()`.

## Picking the nearest hostile — `src/sim/targeting.rs`

```rust
pub fn nearest_hostile(w: &World, me: ShipId) -> Option<ShipId> {
    let my_faction = w.get(me)?.faction;
    others(w, me)                       // Vec<(ShipId, distance)>
        .into_iter()
        .filter(|(id, _)| w.rules.hostile(my_faction, w.ships[id.index as usize].faction))
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(id, _)| id)
}
```

Compare with a hand-written loop: same speed, but the intent ("hostile, minimum distance") is
in the code's shape. `total_cmp` gives floats a total order (floats have no `Ord` because of
NaN).

## When we use a plain `for` loop

When the loop body mutates several things, has early exits, or is clearer as steps — e.g.
`World::step`, `combat::move_bolts`, `wav::decode_ms_adpcm`. Iterators are a tool, not a rule.

## `collect`

`collect()` builds a collection from an iterator; the target type decides which: `Vec<_>`,
`HashMap<_, _>`, `String`, or even `Result<Vec<_>, E>` (stops at the first error) — see
[Error handling](errors.md).
