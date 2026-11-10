# Unit tests

What each module's tests cover. Filled in as modules land; a module is not done until its row
is filled.

| Module | Covers | Status |
|---|---|---|
| `formats::pe` | Section mapping, three-level resource tree, named and numeric entries, quoted names, out-of-range offsets | Stage 1 |
| `formats::xmsh` | Header, LOD table, vertices, 1-based signed edges, BSP walk, polygon corners, bad tag, truncation | Stage 1 |
| `formats::xtxr` | Size formula, palette 0 RGB555 → RGBA, bottom-up flip, mip offsets, truncation | Stage 1 |
| `formats::tga` | Type 2 16/24/32-bit, bottom-up, colour key | Stage 1 |
| `formats::wav` | PCM passthrough; MS-ADPCM block decode against known samples | Stage 1 |
| `formats::ini` | Sections, comments after values, tabs, empty values, case-insensitive lookup, duplicates | Stage 1 |
| `formats::obj` | Vertices, UVs, normals, negative indices, polygons, groups, materials | Stage 1 |
| `packs` | Load order and dependencies, override layering, `inherit` (incl. `original:` refs and cycles), `delete`, typed conversion, unknown references | Stage 1 |
| `packs::scenario` | Placement parsing, `choose`, unknown class diagnostics | Stage 1 |
| `assets` | D3D → right-handed conversion, winding, fan triangulation, per-texture split, LOD choice | Stage 1 |
| `sim::flight` | Turn-rate falloff with LockSpeed, damping, throttle steps, full/zero/reverse, afterburner energy | Stage 1 |
| `sim::weapons` | Rate of fire, energy cost, link modes, missile lock time and cone, homing | Stage 1 |
| `sim` targeting / collision / damage | Next/prev/closest/in-sights, ray-sphere hits, destruction | Stage 1 |
| `sim::ai` | Formation slot positions, pursuit steering | Stage 1 |
| `input` | Default bindings, `bindings.ini` parsing, axis combination and clamping | Stage 1 |
| `platform::linux` | `js_event` decoding into `PadState` | Stage 1 |
