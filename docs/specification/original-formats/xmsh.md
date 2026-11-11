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

Some edges have the same vertex at both ends (zero length): about 3% of polygons (1,014 of
33,012 across the CD) contain one, giving a repeated corner such as `11, 24, 24, 9`. They are
in the original data, not a decoding error, and are harmless — the fan triangulation turns the
repeated corner into a zero-area triangle. **verified**

### Texture name — 32 bytes

NUL-terminated lowercase name of an `XTXR` resource, `0xCD` fill. **verified**

### Manipulator — 64 bytes

`f32[12]` (a 3×4 transform) then two `i32`. The original program's strings call these
`XCGEN_MANIP_OBJECT` and "dummy" manipulators: attachment points such as gun mounts or engine
positions. **derived** — the meaning of each one per model is not yet known.

On the X-COM base (`XBASE7`) the 34 LOD-0 manipulators are **module mount points**: the first
`i32` (kind) runs 0–50 and the positions sit on the ring and spine. The modules themselves are
separate meshes (`XMMOD02`, `XMMODHUB`, `XLANDPC1–4`, `GUNTURRET2`, `SCICENTER02`, …), so a
station under construction is the base mesh plus whichever modules are attached. **derived**

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
| +0x10 | u32 | flags (see below); 0 on most polygons | derived |
| +0x14 | u32 | colour, `0xAARRGGBB`-like; used when untextured | derived |
| +0x18 | u32 / f32 | 0, or an opacity-like float when `0x80000000` is set | derived |
| +0x1C | f32 | small value (≈0.03–0.05) | unknown |
| +0x20 | u32 | texture index into the LOD's name list | verified |
| +0x24 | i32 | edge count *n* | verified |
| +0x28 | i32[n] | signed edge references | verified |

A positive reference *e* walks edge *e* from `v0` to `v1`; a negative one walks edge |*e*|
backwards, so the polygon's corner is `v0` for positive and `v1` for negative references.
Polygons are convex fans with 3 to 36 corners (most are triangles and quads).

The corner order is counter-clockwise around the face plane's normal (by the right-hand rule,
in file coordinates) for 32,893 of 33,012 polygons; the rest are degenerate. The plane normals
point **outward** (every asteroid face does), so faces are one-sided and the original culled
back faces. Thin surfaces seen from both sides — shield rings (`1ASHA`…`3SH3C`), the Avenger's
fin, `ALORBOUT` — are modelled as two polygons back to back. **verified**

Texture coordinates may lie outside 0..1 (the Patton `CPAT` has 564 such corners, `ROID100`
172): textures **tile** (Direct3D's default *wrap* addressing). **verified** by rendering.

### Polygon flags (+0x10)

| Bits | Meaning | Count (LOD 0, whole CD) | Confidence |
|---|---|---|---|
| `0x20000000` | **overlay**: a decal lying exactly on a larger base polygon in the same plane (station panels, alien hull markings) | 32 alone, 53 with `0x80000000` | derived |
| `0x80000000` | **translucent?** — always with a float 0..1 at +0x18 (0.33, 0.60, 0.65) | 569 alone, 101 with other bits | derived |
| `0x00200000`, `0x00020000`, `0x00008000`, low 9 bits (`0x115`–`0x11E` on station overlays) | not decoded | — | unknown |

Evidence for the overlay bit: of 124 same-facing overlapping coplanar pairs, 72 have the flag on
the later polygon and 8 on the earlier one. The original drew polygons in BSP order, so the
later polygon painted over the base; a depth-buffered renderer must instead be told which one
wins — the engine's *overlay layers* (see [Rendering](../../implementation/architecture/rendering.md)). The
opacity float is read but **not used** yet.

## Typical sizes

A fighter: four LODs from ~310 to ~140 vertices and ~200 to ~35 polygons. The X-COM base:
~2,450 vertices and ~660 polygons at LOD 0.

## Open questions

- The flag at `0x40`, and the undecoded polygon flag bits.
- Whether `0x80000000` really is translucency, and how the float is applied.
- Whether the low bits on station overlays (`0x115`–`0x11E`) select overlays by construction
  state; they do not match the base's manipulator kinds.
- What each manipulator means per model (weapon mounts? engine glow? docking?).
- The float at +0x1C (≈0.01–0.05).
- Whether the BSP order is needed for correct transparency.

The model viewer's wireframe and face selection (see [Testbed](../../implementation/architecture/testbed.md))
show each polygon's index, flags, layer and UVs, which helps this research.
