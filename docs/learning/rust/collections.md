# Collections

| Type | Use it for | In this codebase |
|---|---|---|
| `Vec<T>` | An ordered, growable array. The default choice. | Vertices, polygons, ships, bolts, stars, rules definitions, INI entries |
| `HashMap<K, V>` | Lookup by key | Resources by `(type, id)` (`pe::ResourceFile`), original files by name (`OriginalData`), bindings by `Action`, the model and texture caches (`ModelCache`), cockpit instruments by name |
| `VecDeque<T>` | A queue: push at the back, pop at the front, both O(1) | The breadth-first search of `Campaign::lane_hops` (app 0.16.3) |
| `HashSet<T>` | "Is this in the set?" | Keys held / pressed this frame (`RawInput`), textures already checked (`check::run`) |
| `[T; N]` | Fixed-size array | A colour `[f32; 3]`, a rectangle `[f32; 4]`, a palette `[u16; 256]`, the four gauge values |
| `&[T]` | A borrowed view into any of the above | Function parameters such as `&[u8]` |

## Rules of thumb

- **Default to `Vec`.** It is contiguous in memory, which is what CPUs are fastest at. A linear
  scan of a few dozen ships beats a `HashMap` lookup. `ini::Section` keeps its entries in a
  `Vec<(String, String)>` for that reason — sections are small and file order matters.
- Use a `HashMap` when you look things up **by key** many times (resources, caches).
- Store **indices** into a `Vec` (`ShipClassId`, `ShipId`) rather than references to its
  elements, so the `Vec` can grow and the borrow checker stays out of the way.
- Take `&[T]` (not `&Vec<T>`) as a parameter: it accepts arrays and slices too.
- Reuse a `Vec` across frames with `clear()` instead of creating a new one — `clear` keeps the
  allocated memory. `Renderer` does this for its glow and sky buffers.
- A bounded log: `hud::MessageLog` keeps at most 32 entries in a `Vec`, removing the oldest.
  (`VecDeque` would make that removal O(1); at 32 entries a `Vec` is simpler and fast enough.)

## A queue for a breadth-first search — `src/campaign/logistics.rs`

`lane_distance(a, b)` counts the **jumps** between two systems along the map's lanes. Counting
jumps (not distance) is a breadth-first search: visit every system one jump away, then every
system two jumps away, and so on. The systems waiting to be visited form a queue — first in,
first out — which is what `std::collections::VecDeque` is for: `push_back` and `pop_front` are
both O(1), where `Vec::remove(0)` would shift every element.

```rust
let mut hops = vec![u32::MAX; self.systems.len()];   // u32::MAX = "not reached yet"
hops[a] = 0;
let mut queue = std::collections::VecDeque::from([a]);
while let Some(s) = queue.pop_front() {              // `while let`: loop until None
    for &(x, y) in &lanes {
        let next = if x == s { y } else if y == s { x } else { continue };
        if hops[next] == u32::MAX {                  // first visit = fewest jumps
            hops[next] = hops[s] + 1;
            if next == b { return Some(hops[next]); }
            queue.push_back(next);
        }
    }
}
None                                                 // no route
```

Because the queue hands systems out in the order they were reached, the first time `b` is
reached is by the fewest jumps. Compare `Campaign::lane_route`, which wants the shortest
**distance** and so uses Dijkstra's algorithm instead.

## Order matters for determinism

`HashMap` iteration order is random per run. Anything that affects the simulation or what is
shown (pack order, rules, scenarios, the launcher list) iterates a `Vec` that is sorted
explicitly — `pack::discover` sorts folders, `files_with_ext` sorts file names, scenarios are
sorted by `order` then title, `OriginalData::resource_names` sorts names.

When the output of a map is *written out*, a `BTreeMap` (kept sorted by key) is the simpler
choice. The extractor (`src/extract.rs`, `model_to_obj`) collects the materials of a model in
a `BTreeMap<String, String>` and writes them with `into_values()`: the `.mtl` file is
byte-for-byte the same on every run, so extracting twice and comparing — or keeping the files
in version control of your own art project — shows only real changes.
