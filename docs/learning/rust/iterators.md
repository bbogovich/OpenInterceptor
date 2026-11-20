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

## Ties: `max_by` keeps the last, `min_by` the first — `src/campaign/mod.rs`

A base type's `template` may name a module **group**; the base then gets the best module of that
group available now — the costliest (app 0.19.0):

```rust
let best = (0..def.modules.len())
    .filter(|&m| def.modules[m].group == *entry)
    .filter(|&m| self.available(def, &format!("modules.{}", def.modules[m].id)))
    // `max_by` keeps the last of equals: a later (newer) module wins a tie.
    .max_by(|&a, &b| def.modules[a].cost.total_cmp(&def.modules[b].cost));
out.extend(best);
```

When two elements compare equal, the standard library is precise about which one you get:
`max_by` (and `max`, `max_by_key`) returns the **last** of the equal maxima, `min_by` (and
`min`, `min_by_key`) the **first** of the equal minima. The code relies on it: two sensor
modules of the same price give the one written later in the data, which is usually the newer
one. A hand-written loop with `>` instead of `>=` would silently pick the other. When a tie
must be broken by something else, say so with `Ordering::then` instead of relying on the
order — `Campaign::nearest_base_system` compares lane jumps first and straight-line distance
second:

```rust
// simplified: `hops` and `dist` are small closures in the real code
.min_by(|&a, &b| hops(a).cmp(&hops(b)).then(dist(a).total_cmp(&dist(b))))
```

`out.extend(best)` works because an `Option` is itself iterable — zero or one item — so no
`if let` is needed to push "the best one, if there is one".

## Returning an iterator: `impl Iterator` — `src/content/pack.rs`

A pack must load after its `parent` and after every pack it `depends` on. Two places need "the
packs this one needs", so it is one small function — and it returns an *iterator*, not a
`Vec`:

```rust
fn needs(p: &PackInfo) -> impl Iterator<Item = &String> {
    p.parent.iter().chain(p.depends.iter())
}
```

`impl Iterator<Item = &String>` means "some iterator over `&String`; the caller need not know
which". The real type (a `Chain` of an `Option` iterator and a slice iterator) is long and an
implementation detail. `Option::iter()` yields zero or one item, which is how an optional
parent and a list of dependencies become one sequence. Nothing is allocated: the callers just
loop or ask a question —
`needs(p).all(|d| out.iter().any(|q| &q.id == d))` ("is every need already placed?").

The returned iterator borrows from `p`, so it cannot outlive it; with one reference parameter
the compiler infers that lifetime (compare `triangles` above, which spells it `+ '_`).

## When we use a plain `for` loop

When the loop body mutates several things, has early exits, or is clearer as steps — e.g.
`World::step`, `combat::move_bolts`, `wav::decode_ms_adpcm`. Iterators are a tool, not a rule.

## `collect`

`collect()` builds a collection from an iterator; the target type decides which: `Vec<_>`,
`HashMap<_, _>`, `String`, or even `Result<Vec<_>, E>` (stops at the first error) — see
[Error handling](errors.md).
