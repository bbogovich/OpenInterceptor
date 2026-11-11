# Interceptor Remastered

**Status: design; scheduled in [Stage 7](../implementation-plan/stage-7-modding.md).** The
example of a campaign that **extends** another: `parent: interceptor`, plus sharper textures
for modern screens and an upgraded X-1A Lightning II whose panel lines, vents and intakes —
today only painted on — are real geometry.

## The constraint: nothing derived from the CD may be shipped

An upscaled original texture is a conversion of MicroProse's art, and so is a model built by
editing the original Lightning mesh. Project rule: no original content, conversions included,
enters the repository or a published mod. So the remastered mod ships **recipes**, and the
engine produces the remastered assets on the player's machine from the player's own CD
([generated assets](mod-structure.md#generated-assets-recipes)). What is published is the mod's
own work only: recipe files, patch instructions, and any art made from scratch.

```text
mods/interceptor-remastered/
  pack.yaml                    kind: campaign, parent: interceptor, requires_original: true
  recipes/textures.yaml        upscale every original texture x4
  recipes/images.yaml          upscale the HUD and menu images x2 (cockpits: see below)
  recipes/lightning.yaml       build the enhanced Lightning from the original mesh
  patches/lightning.meshpatch  the face edits (this project's own work: numbers, no geometry)
  rules/models.yaml            point the Lightning's model set at the generated model
```

## Textures: the `upscale` generator

```yaml
recipes:
  textures-x4:
    kind: upscale
    inputs: original:*.xtxr            # all 799 textures
    scale: 4
    method: edge-directed
    sharpen: 0.3
    output: "textures/{name}.png"      # = the texture override path, so it replaces the original
```

**Method.** The originals are 8-bit palettised and mostly 64–256 px: blocky edges and
dithering are the problem, not a lack of high-frequency detail. The built-in, deterministic
pipeline (all in-crate, no ML model, no dependency):

1. **Decode** at full colour (palette 0; Stage 1 reader).
2. **De-dither**: a small edge-preserving smoothing pass (a bilateral filter in a 3×3 window)
   removes the palette's ordered-dither noise without blurring panel edges.
3. **Edge-directed upscale** ×2 per step (two steps for ×4): each new pixel is interpolated
   *along* the local edge direction (estimated from gradients) rather than across it — the idea
   behind NEDI/xBR-family scalers — so diagonal panel lines stay sharp and stair-free.
4. **Sharpen** lightly (unsharp mask) and **re-apply** the colour key / alpha mask at the new
   resolution, with a 1-pixel feathered edge.
5. **Wrap-aware borders**: the textures tile (Stage 1 found UVs outside 0..1), so filtering
   wraps around the edges instead of clamping, avoiding seams.

Each step is a pure function on an image buffer with unit tests on synthetic images (a hard
diagonal edge stays a single sharp edge at ×4; a checkerboard dither becomes flat colour; a
tiling texture stays seamless). Output quality is judged by a manual test with side-by-side
screenshots in the viewer.

**Cockpits** are a special case: they are large hand-painted images with text. The recipe
upscales them ×2 the same way, and the mod documents the HD-repaint path
([overrides](../modding/overrides.md)) for anyone who wants hand-made art.

## The enhanced Lightning: the `mesh-patch` generator

The X-1A Lightning II (`INTTYPEA`, 198 faces at LOD 0) is the smallest playable hull and the
one seen most, so it is the showcase. Its textures paint raised panels, recessed intakes, vents
and a canopy frame on flat faces. A **mesh patch** is a list of geometric edits addressed by the
original face numbers that the viewer shows (`W`, `[ ]`, `I`):

```text
# patches/lightning.meshpatch — edits to INTTYPEA LOD 0
# Face numbers come from the model viewer. Units are metres. Texture coordinates of new faces
# are derived from the face they came from, so the original paint lines up with the new shape.

inset    face 41  by 0.12               # intake rim
extrude  face 41  by -0.35              # recess the intake
inset    face 42  by 0.12
extrude  face 42  by -0.35
bevel    edges of faces 10-17 by 0.04   # rounded nose panels
extrude  face 63  by 0.06  taper 0.9    # raised spine panel
cut      face 88  along uv-line (0.31,0.10)-(0.31,0.92) then extrude by -0.02   # panel seam
subdivide faces 20-27 smooth 0.5        # canopy curvature
relief   face 70  from texture height scale 0.03 grid 8   # vents: displacement from the paint
lod      keep 1-3                        # leave the far LODs as they are
```

| Operation | Effect |
|---|---|
| `inset face N by d` | Shrinks a face inwards, creating a ring of new faces (a rim). |
| `extrude face N by d [taper t]` | Moves a face along its normal (negative = into the hull), building side walls. |
| `bevel edges of faces … by d` | Chamfers sharp edges. |
| `cut face N along uv-line a-b` | Splits a face along a line given in texture space, so a painted seam can become a real one. |
| `subdivide faces … smooth s` | Splits faces and moves new vertices towards a smooth surface. |
| `relief face N from texture height scale h grid g` | Displaces a grid of new vertices by the texture's brightness (a height map from the paint). |
| `lod keep …` | Copies other LODs unchanged. |

All operations keep the **one-sided, counter-clockwise** convention and the overlay layers
(Stage 1), produce new texture coordinates by interpolation from the source face, and are
deterministic. Each is a pure function on the engine's `Model` type with unit tests on small
synthetic meshes (a cube face extruded by −1 gives the expected volume; a cut along a UV line
produces two faces whose UVs meet on the line).

The **patch file contains no original geometry** — only face numbers and distances chosen by
the author — so it can be published. Applied to someone else's mesh it simply fails the
recipe's check (face count and a checksum of the source mesh are recorded in the recipe):

```yaml
recipes:
  lightning:
    kind: mesh-patch
    input: original:INTTYPEA
    input_faces: 198                # refuse to patch an unexpected mesh
    patch: patches/lightning.meshpatch
    output: models/inttypea.obj     # = the model override path: replaces the Lightning everywhere
```

**Authoring workflow**: open the mesh in the viewer (`--model INTTYPEA`, `W`, `[ ]`, `I`),
note face numbers, write the patch, and the viewer reloads it on save (Stage 7 adds patch
hot-reload) to show the result next to the original.

## Rules

```yaml
# rules/models.yaml — nothing else changes: the parent campaign's rules all apply
modelsets:
  lightning:
    intact: models/inttypea.obj     # generated by recipes.lightning
```

Every other model set keeps pointing at the originals, and because the upscaled textures land
on the texture override paths, every model in the game picks them up without a rule change.

## Manual tests (Stage 7)

- Side-by-side viewer screenshots of five textures and the Lightning before/after.
- The enhanced Lightning at every LOD distance; no cracks, no flipped faces (wireframe and
  culling), paint aligned with the new geometry.
- A full mission flown with the remastered campaign; frame time unchanged within 10%.
