# Collections

| Type | Use it for | In this codebase |
|---|---|---|
| `Vec<T>` | An ordered, growable array. The default choice. | Vertices, polygons, ships, shots, stars |
| `HashMap<K, V>` | Lookup by key | Rules by id, models by name, bindings by action |
| `BTreeMap<K, V>` | Lookup by key, iterated in **sorted** order | Scenario list in the launcher (stable order) |
| `HashSet<T>` | "Have I seen this?" | Cycle detection in `inherit` chains |
| `VecDeque<T>` | A queue | HUD message line |
| `[T; N]` | Fixed-size array | A polygon's plane `[f32; 4]`, a triangle `[u16; 3]` |
| `&[T]` | A borrowed view into any of the above | Function parameters |

## Rules of thumb

- **Default to `Vec`.** It is contiguous in memory, which is what CPUs are fastest at.
  A linear scan of a few hundred ships beats a `HashMap` lookup chain.
- Use a `HashMap` when you look things up **by name** many times (rules, assets).
- Store **indices** into a `Vec` (newtype ids) rather than references to its elements, so the
  `Vec` can grow and the borrow checker stays out of the way.
- Take `&[T]` (not `&Vec<T>`) as a parameter: it accepts arrays and slices too.
- Reuse a `Vec` across frames with `clear()` instead of creating a new one — `clear` keeps the
  allocated memory. See [Data model](../best-practices/data-model.md).

## Order matters for determinism

`HashMap` iteration order is random per run. Anything that affects the simulation or output
order (loading packs, spawning ships) iterates a `Vec` or `BTreeMap` instead.
