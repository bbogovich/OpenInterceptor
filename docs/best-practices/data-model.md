# Data model and performance

A 1998 space sim is light work for a modern machine; these practices keep it that way as mods
add content, and they are the habits a larger engine needs.

## Pre-baked GPU meshes per texture

At load, each model LOD is converted once (`Model::from_xmsh` / `from_obj`, then
`render::gpu::split_lod`) into one chunk per texture: triangle lists with `u16` indices. Drawing
is then one `draw_mesh` per chunk, with no parsing or triangulation per frame.

## Flattened arrays instead of per-item allocations

An XMSH polygon does not own a `Vec` of its vertex indices. All polygons' indices go into one
shared `Lod::polygon_vertices: Vec<u32>`, and each `Polygon` stores `first` and `count` into
it. Thousands of tiny heap allocations become one, and iterating polygons reads contiguous
memory. `ObjMesh` does the same with face corners.

## LOD selection

Each original model has four LODs with switch distances from the file (e.g. 75, 150, 450 m and
"always" for the Lightning II; 4.2, 8.4, 25 km for the X-COM base). `GpuModel::lod(distance)`
picks the first LOD whose switch distance exceeds the camera distance, cutting triangle counts
for distant objects several-fold (the base goes from 659 to 126 polygons).

## Camera-relative precision

Positions are `f32` metres. Before drawing, the camera position is subtracted on the CPU
(`SpaceCamera::model_matrix`), and the view matrix is rotation-only, so the camera sits at the
origin of render space: nearby geometry is precise and does not jitter even tens of kilometres
out.

## No per-frame allocation

The glow and sky geometry is rebuilt every frame into `Vec`s owned by the `Renderer`.
They are `clear()`ed (which keeps capacity), moved into a `Mesh` with `std::mem::take`, drawn,
and moved back — so after the first frame they allocate nothing. In `World::step` the ruleset
is reached through a cloned `Arc` rather than by cloning a ship class per ship per tick.
Remaining small per-frame allocations (HUD text, engine-glow attachment lists) are known and
cheap; they are candidates if profiling ever shows them.

## Index handles instead of references

Ships refer to their target by `ShipId`, not by `&Ship`. References into a `Vec` would block
mutation of the `Vec` (borrow rules) and dangle if it reallocated; indices do neither, and
they serialise naturally for saves.

## Structure of arrays where it matters

Everything in Stage 1 is an array of structs (`Vec<Ship>`, `Vec<Bolt>`, `Vec<Star>`): counts are
tens to a few thousand, and AoS is the simplest. Where counts grow large and a hot loop touches
only one field, a structure of arrays (`positions: Vec<Vec3>`, `ages: Vec<f32>`) reads
contiguous memory and is the next step. Choose by measuring, not by habit.

## Measure first

Optimise what a profiler shows, and keep the simple version when it is fast enough. A developer
overlay with frame time, draw calls and triangle counts is planned (Stage 2); until then, the
model viewer reports triangles per LOD and macOS Instruments or `perf` profile the release
build.
