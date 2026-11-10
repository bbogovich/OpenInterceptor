# Models and textures

## Formats a mod can use

| Kind | Format | Notes |
|---|---|---|
| Model | Wavefront **OBJ** (+ MTL) | Triangles or polygons; `v`, `vt`, `vn`, `f`, `usemtl`, `mtllib`. One file per LOD: `name.obj`, `name.lod1.obj`, … |
| Texture | **PNG** or **TGA** | Any size; powers of two recommended |
| Cockpit frame | PNG with alpha, or TGA with cyan key | Drawn over the 3D view, scaled to the window height |

The engine reads original `XMSH` and `XTXR` from the CD itself; mods never need those formats.

## Conventions

- **Units**: metres. A fighter is about 15–20 m across.
- **Axes**: right-handed, **Y up**, the nose points along **−Z**. (Blender: export OBJ with
  "Forward: −Z, Up: Y".)
- **Origin**: the centre of mass; the engine computes the bounding radius.
- **Attachment points** (gun muzzles, engine glow): OBJ objects or groups named `mount_beam_1`,
  `mount_missile_1`, `engine_1`. Their centroid becomes the point.
- **Materials**: `map_Kd` names the texture; `Kd` is used when there is none.

## Replacing an original model

Ship `models/inttypea.obj` in your pack: every rule that says `model = inttypea` now uses it.

## Planned

A reference exporter (`--export-obj <mesh>`) for studying original meshes in Blender, for
personal use only (Stage 6), and glTF import if demand appears.
