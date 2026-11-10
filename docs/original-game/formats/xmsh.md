# XMSH meshes

Every 3D object (ships, stations, debris, asteroids, nebula billboards, planets) is an `XMSH`
resource. Coordinates are Direct3D-style (left-handed, Y up); the engine converts to its own
right-handed frame at load. Units appear to be metres (a fighter has a bounding radius of about
10). **All 297 meshes parse with this layout.** Rendering three hulls and a saucer with their
original textures confirmed positions, UVs and texture binding.

## File header

| Offset | Type | Field | Confidence |
|---|---|---|---|
| 0x00 | char[12] | name, lowercase, NUL then `0xCD` fill | verified |
| 0x0C | char[4] | tag `MSH\0` | verified |
| 0x10 | u32 | 0 | verified (always 0) |
| 0x20 | f32 | bounding-sphere radius | verified |
| 0x28 | f32[6] | bounding box min xyz, max xyz | verified |
| 0x40 | u32 | flags; 1 on hulls with a manipulator, 0 on many others | unknown |
| 0x44 | i32[4] | −1 | unknown |
| 0x54 | i32[4] | offsets of up to 4 **LOD** blocks, −1 = none | verified |

## LOD block

All offsets inside a LOD block are relative to the block's start.

| Offset | Type | Field | Confidence |
|---|---|---|---|
| +0x00 | f32 | switch distance: use this LOD while closer than this (last LOD: 1e8) | derived |
| +0x04 | u32 | 0 | — |
| +0x08 | u32 | vertex count | verified |
| +0x0C | u32 | vertex offset | verified |
| +0x10 | u32 | edge count (includes unused edge 0) | verified |
| +0x14 | u32 | edge offset | verified |
| +0x18 | u32 | texture-name count | verified |
| +0x1C | u32 | texture-name offset | verified |
| +0x20 | u32 | manipulator count | verified |
| +0x24 | u32 | manipulator offset | verified |
| +0x28 | u32 | BSP root node offset | verified |

### Vertex — 32 bytes

`f32 x, y, z, nx, ny, nz, u, v`. Vertices are duplicated where UVs differ. **verified**

### Edge — 16 bytes

`u32 v0, u32 v1, u32 (0xCDCDCDCD), u32 0`. Edges are **1-based**: entry 0 is unused filler.
**verified**

### Texture name — 32 bytes

NUL-terminated lowercase name of an `XTXR` resource, `0xCD` fill. **verified**

### Manipulator — 64 bytes

`f32[12]` (a 3×4 transform) then two `i32`. The original program's strings call these
`XCGEN_MANIP_OBJECT` and "dummy" manipulators: attachment points such as gun mounts or engine
positions. **derived** — the meaning of each one per model is not yet known.

### BSP node — 64 bytes

| Offset | Type | Field | Confidence |
|---|---|---|---|
| +0x00 | f32[4] | splitting plane (normal, distance); zero in a leaf | derived |
| +0x10 | f32[6] | node bounding box | verified |
| +0x28 | i32 | front child offset, −1 = none | verified |
| +0x2C | i32 | back child offset, −1 = none | verified |
| +0x30 | i32 | 0 | unknown |
| +0x34 | i32 | polygon count | verified |
| +0x38 | i32 | −1 | unknown |
| +0x3C | — | `0xCD` fill | — |

The node's polygons follow immediately after its 64 bytes.

### Polygon — 40 + 4·n bytes

| Offset | Type | Field | Confidence |
|---|---|---|---|
| +0x00 | f32[4] | face plane (normal, distance) | verified |
| +0x10 | u32 | 0 | unknown |
| +0x14 | u32 | colour, `0xAARRGGBB`-like; used when untextured | derived |
| +0x18 | u32 | 0 | unknown |
| +0x1C | f32 | small value (≈0.03–0.05) | unknown |
| +0x20 | u32 | texture index into the LOD's name list | verified |
| +0x24 | i32 | edge count *n* | verified |
| +0x28 | i32[n] | signed edge references | verified |

A positive reference *e* walks edge *e* from `v0` to `v1`; a negative one walks edge |*e*|
backwards, so the polygon's corner is `v0` for positive and `v1` for negative references.
Polygons are convex fans with 3 to 36 corners (most are triangles and quads).

## Typical sizes

A fighter: four LODs from ~310 to ~140 vertices and ~200 to ~35 polygons. The X-COM base:
~2,450 vertices and ~660 polygons at LOD 0.

## Open questions

- The flag at `0x40` and the per-polygon unknowns (possibly shading or transparency flags).
- What each manipulator means per model (weapon mounts? engine glow? docking?).
- Whether the BSP order is needed for correct transparency.
