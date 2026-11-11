# Models and textures

| Asset | Mod format | Notes |
|---|---|---|
| Model | Wavefront **OBJ** + **MTL** | `v`, `vt`, `vn`, `f` (forms `1`, `1/2`, `1//3`, `1/2/3`, negative indices), `usemtl`, `mtllib`. Polygons are fan-triangulated, so keep them convex. Groups and objects are ignored. |
| Material | MTL | `newmtl`, `Kd r g b` (flat colour, 0–1), `map_Kd file` (texture, relative to the `.mtl`; options before the file name are allowed). |
| Texture | **PNG** or **TGA** | Any size; RGBA honoured. |
| Cockpit art | PNG or TGA | Any resolution; stretched over the cockpit's design space (640×480 unless `design` says otherwise). Use alpha for the window. See [Cockpits](cockpits.md). |
| 3D cockpit | OBJ + MTL | Metres around the pilot's eye at the origin, +Y up, looking along −Z. |
| Replacement of an original asset | OBJ / PNG / WAV at its override path | See [Overrides and extraction](overrides.md); `--extract` produces starting files. |

The original formats (XMSH meshes, XTXR textures, TGA cockpits) are read straight from the CD
behind the same interfaces, so original and mod assets can be mixed freely.

## Conventions

- **Units: metres.** Use `model_scale` in the ship's rules if your file uses other units.
- **Axes: right-handed, +Y up, the nose pointing along −Z** (the engine's convention). Blender's
  OBJ exporter with "Forward: −Z, Up: Y" produces this.
- **Texture coordinates**: standard OBJ (v up); the loader flips them for the engine.
- **Normals**: used if present; otherwise each face gets a flat normal.
- **Faces are one-sided.** A face is visible only from the side where its corners run
  **counter-clockwise** (the OBJ and Blender convention for an outward face); from behind it is
  culled, exactly like the original models. A surface that must be seen from both sides — a
  fin, a shield, a sign — needs two faces back to back, one wound each way. If a face vanishes
  when you orbit it in the model viewer, it is wound the wrong way (Blender: *Mesh → Normals →
  Flip* or *Recalculate Outside*).
- **Decals.** To lay a marking over a hull face, put a second face exactly in the same plane,
  facing the same way, **later in the file** than the face it covers. The engine detects such
  overlapping coplanar faces and draws the later one on top (a tiny depth bias), so there is no
  flicker and no need to lift the decal off the surface. Original models mark their decals
  with a polygon flag instead ([XMSH](../specification/original-formats/xmsh.md#polygon-flags-0x10)).
- **Texture coordinates repeat**: a UV of 0..4 tiles the texture four times across the face.
- **One LOD** per OBJ model in Stage 1 (original models keep their four LODs). Multiple LOD
  files are planned for Stage 6.
- **Attachment points.** A group (or object) named **`slot_<name>`** is not drawn: it marks
  where a station module mounts — at the centre of its faces, facing along its first face's
  normal. One small square, wound counter-clockwise seen from where the module should point,
  is enough. Modules are turned so their −Z follows that normal; model a module standing on
  its base at the origin and extending along −Z. See [Stations and modules](stations.md).
  Original models' attachment points are their manipulators, addressed by index. OBJ models get
  one engine glow at the back of their bounding box.
- **Looks.** Damage textures, named states, debris pieces and explosions belong to the model
  set, not the model: see [Looks](looks.md). Debris pieces are modelled in the object's own
  space so they reassemble it.
- **Collision**: craft collide as spheres (bounding radius); stations (`kind: station`)
  collide against their triangles, so model the station's real shape.

## Re-skinning original models

A model's textures are looked up by name in every pack's `textures/` folder (`<name>.png`, then
`<name>.tga`, newest pack first) before the original `XTXR` texture. To re-skin, export your
image with the original texture's name — the [model viewer](../implementation/architecture/testbed.md) lists
the models, and `--check` reports texture names that fail to load.

## Checking a model

Open it straight in the viewer: `openinterceptor --model <ship id or mesh name>`. In the viewer,
**W** cycles solid / solid + wireframe / wireframe, **D** cycles the class's looks (intact, damage
levels, states, debris pulled apart), **[ ]** select faces (Shift: ten at a time)
and show each face's number, corner count, texture, flags, overlay layer and UVs, **I** labels
every face towards the camera, and **O** hides overlay-flagged faces.

## Example

Every model of the `default` campaign — rockets, saucers, the robot sentinel, freighters,
stations, rubble, debris pieces and the Guard 3D cockpit — is written by the Python scripts in
`tools/default_assets/` (`models.py`, with `textures.py` for hull paint and its scorched and
burnt damage variants — and *wrecked* for station modules — and `sounds.py` for synthesised
effects). Run
`python3 tools/default_assets/make_all.py` to rebuild them, or `make_all.py --check` to build
twice and prove the output is byte-identical. The scripts use only the Python standard library,
seed every random choice, and validate every model before writing it (closed solids wound
counter-clockwise from outside, no degenerate faces, within a triangle budget). Look at them to
see how a model, its `.mtl` materials and its textures are written programmatically.

### The generator's conventions

| Convention | In `tools/default_assets/` |
|---|---|
| Debris | Each model is cut by planes into its `<name>_debris<n>.obj` pieces, in the model's own space. |
| Attachment points | `slot(name, at, normal)` adds one small square face in a group `slot_<name>`, centred on `at` and wound so its normal is `normal`; slots are open parts, so the validator checks them for degeneracy only, and debris cutting leaves them out. |
| Station modules | Stand on their base at the origin and extend along −Z (the slot's normal), e.g. the Guard ray turret and force-field generator, the Dominion death-ray pod and shield pod. |
| Damage textures | `<texture>_scorched.png`, `<texture>_burnt.png`; modules also `<texture>_wrecked.png` for their `destroyed` state. |
| Instrument placement | The Guard cockpit's rectangles and anchors, written to `models/guard_cockpit.instruments.yaml`. |
