# Data model and performance

A 1998 space sim is light work for a modern machine; these practices keep it that way as mods
add content, and they are the habits a larger engine needs.

## Pre-baked GPU meshes per texture

At load, each model LOD is converted once into one `MeshPart` per texture: triangle lists with
`u16` indices. Drawing is then one `draw_mesh` per part, with no parsing or triangulation per
frame.

## LOD selection

Each original model has four LODs with switch distances. The renderer picks the LOD from the
camera distance (scaled by the field of view), cutting triangle counts for distant ships and
stations by 4–10×.

## Camera-relative precision

Positions are stored with enough range for a whole arena; before drawing, the camera position is
subtracted, and only then are values converted to `f32` vertices. The camera is at the origin
of render space, so nearby geometry is exact and does not jitter even tens of kilometres out.

## No per-frame allocation

Per-frame buffers (transformed vertices, visible-object lists, HUD text) live in the `Scene` or
`Hud` and are `clear()`ed, not recreated. `Vec::clear` keeps capacity, so after the first frames
the steady state allocates nothing.

## Index handles instead of references

Ships refer to their target by `ShipId`, not by `&Ship`. References into a `Vec` would block
mutation of the `Vec` (borrow rules) and dangle if it reallocated; indices do neither, and
they serialise naturally for saves.

## Structure of arrays where it matters

Most objects are few (tens of ships) and stored as an array of structs (`Vec<Ship>`) — simplest
and fast enough. Where counts are large and loops touch one field — the starfield, projectiles
and particles — data is laid out as a structure of arrays (`positions: Vec<Vec3>`,
`ages: Vec<f32>`) so the hot loop reads contiguous memory. Choose by measuring, not by habit.

## Measure first

The F12 overlay reports frame time, draw calls and triangles. Optimise what it shows, and keep
the simple version when it is fast enough.
