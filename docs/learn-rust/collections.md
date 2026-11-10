# Collections

| Type | Use it for | In this codebase |
|---|---|---|
| `Vec<T>` | An ordered, growable array. The default choice. | Vertices, polygons, ships, bolts, stars, rules definitions, INI entries |
| `HashMap<K, V>` | Lookup by key | Resources by `(type, id)` (`pe::ResourceFile`), original files by name (`OriginalData`), bindings by `Action`, the model and texture caches (`ModelCache`), cockpit instruments by name |
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

## Order matters for determinism

`HashMap` iteration order is random per run. Anything that affects the simulation or what is
shown (pack order, rules, scenarios, the launcher list) iterates a `Vec` that is sorted
explicitly — `pack::discover` sorts folders, `files_with_ext` sorts file names, scenarios are
sorted by `order` then title, `OriginalData::resource_names` sorts names.
