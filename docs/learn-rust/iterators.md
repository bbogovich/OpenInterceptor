# Iterators and closures

An **iterator** produces values one at a time; adapters like `map` and `filter` build pipelines
that compile down to plain loops.

## Turning a polygon into triangles

An original polygon is a convex fan of `n` corners; the GPU wants triangles:

```rust
/// Corner indices of a convex polygon → triangle index triples (a fan from corner 0).
fn fan(corners: &[u16]) -> impl Iterator<Item = [u16; 3]> + '_ {
    (1..corners.len().saturating_sub(1)).map(move |i| [corners[0], corners[i], corners[i + 1]])
}

let indices: Vec<u16> = fan(&poly).flatten().collect();
```

`move |i| …` is a **closure**: an anonymous function that can capture variables from where it is
written (here `corners`).

## Picking the closest enemy

```rust
let closest = world
    .ships()
    .filter(|s| s.side.is_hostile_to(player.side) && s.alive())
    .min_by(|a, b| {
        let da = a.body.position.distance_squared(player.body.position);
        let db = b.body.position.distance_squared(player.body.position);
        da.total_cmp(&db)
    })
    .map(|s| s.id);
```

Compare with a hand-written loop: same speed, but the intent ("hostile, alive, minimum distance")
is in the code's shape.

## When we use a plain `for` loop

When the loop body mutates several things, has early exits, or is clearer as steps — e.g.
`World::step`. Iterators are a tool, not a rule.

## `collect`

`collect()` builds a collection from an iterator; the target type decides which:
`Vec<_>`, `HashMap<_, _>`, `String`, or even `Result<Vec<_>, E>` (stops at the first error).
