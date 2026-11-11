---
spec_version: 1.9.1
status: current
updated: 2026-11-11
---

# Tutorial: a 3D X-COM cockpit using your own cockpit art

This tutorial builds a **3D cockpit for an X-COM: Interceptor craft**. The model is your own; the cockpit art painted on it comes from **your copy of the game**, referenced rather than copied. The finished mod contains no original content, so you can share it with anyone who owns the game.

> **Status: try it and report back.** Original art referenced from a material is new in app 0.4.13 ([R-MOD-055](../specification/requirements/modding.md)). We are evaluating in practice how well it works: UV mapping onto 640×480 art, transparency of the cyan areas, and texture sharpness. Note what you find on [issue #72](https://github.com/bbogovich/OpenInterceptor/issues/72).

You need:
- the game's data, as an installed copy or a CD copy (see [Getting started](../user-guide/getting-started.md));
- a 3D modeller that exports **Wavefront OBJ/MTL**. Blender is recommended and free, but it is not required; any modeller that writes OBJ/MTL works.

## 1. Make the mod folder

Your mods live in the user data folder (`~/Library/Application Support/OpenInterceptor/mods/` on macOS). Create one folder for the mod:

```text
mods/xcom-3d-cockpit/
  pack.yaml
  rules/cockpits.yaml
  models/lightning_pit.obj
  models/lightning_pit.mtl
```

`pack.yaml` makes it an extension of the `interceptor` campaign: it adds to and overrides that campaign's content.

```yaml
id: xcom-3d-cockpit
name: "X-COM 3D cockpit"
kind: addon                 # an extension: it extends another pack
targets: [interceptor]      # the campaign it extends
requires_original: true     # it uses the player's cockpit art
```

## 2. Get a reference copy of the cockpit art (for modelling only)

To UV-map the model you need to see the art. Extract it into a working folder **outside the mod**:

```sh
openinterceptor --campaign interceptor --extract ~/xcom-reference
```

The cockpit art is then in `~/xcom-reference/images/`:

| Cockpit | Used by | Reference image | Reference for the material |
|---|---|---|---|
| A | (see `cockpit:` on each ship in `packs/interceptor/rules/`) | `images/cockpita/12000.png` | `original:CockpitA.dll#12000` |
| B | | `images/cockpitb/12200.png` | `original:CockpitB.dll#12200` |
| C | | `images/cockpitc/12400.png` | `original:CockpitC.dll#12400` |

Use these PNGs only while modelling. **Do not put them in the mod.** The mod points at the original instead (step 4).

## 3. Model the cockpit

The engine's conventions for a cockpit model ([Cockpits](cockpits.md#a-3d-cockpit-instead-of-the-overlay)):
- units are **metres**;
- the **pilot's eye is at the origin**, **+Y is up**, and the pilot looks along **−Z**;
- faces are one-sided: their front faces the pilot.

Then:
1. Build the dashboard, canopy frame and side panels around the eye.
2. Assign a material that uses the reference image, and UV-map faces onto the parts of the art you want there: the dashboard, the bezels, the decals. The art is 640×480 (4:3), and UVs run 0–1 across it.
3. The art's **cyan areas are see-through** in the game, as in the 2D cockpit. Faces mapped onto the canopy's cyan parts become clear glass. Give structural faces areas of solid art.
4. Leave flat faces where the instruments go (the radar scopes, the MFD, the gauges, the message and status strips). Instruments are drawn onto those faces (step 6).

**In Blender** (recommended): model, assign the reference PNG as the base colour texture, UV unwrap, then **File › Export › Wavefront (.obj)** with:
- Forward axis −Z, Up axis Y;
- Materials: Export on;
- Path mode: Strip;
- Selection only, if the scene has helpers.

You can keep the `.blend` file next to your work as the source. Only the OBJ/MTL goes in the mod.

## 4. Point the material at the original art

Open the exported `models/lightning_pit.mtl` in a text editor. Replace the texture file name in each material that uses the cockpit art with the original reference:

```text
newmtl cockpit_art
Kd 1 1 1
map_Kd original:CockpitA.dll#12000
```

Rules for the texture line:
- `map_Kd original:<File>.dll#<resource>` loads that original image, with the cyan colour key made transparent;
- `map_Kd original:<NAME>` loads an original model texture by name (for example `original:INTTOP`);
- an ordinary file name (`map_Kd panel.png`) loads your own image, relative to the `.mtl` file and inside the mod;
- lines beginning with `#` are comments. A `#` later in the line is part of the value, as in `#12000`.

Your own extra textures (a new panel, a decal) are ordinary PNG files in the mod.

## 5. Use the model for the cockpit

`rules/cockpits.yaml` in the mod overrides cockpit `a` of the interceptor campaign key by key:

```yaml
# yaml-language-server: $schema=../../../schemas/rules.schema.json
cockpits:
  a:
    image: none                       # no 2D overlay: the 3D model instead
    model: models/lightning_pit.obj
    model_offset: [0, 0, 0]           # metres from the eye, if the model is not centred on it
    design: [1280, 720]
    # Starting positions for the instruments, so each one has an anchor that the F12
    # editor can move (step 6). The values are rough guesses in front of the eye.
    anchors:
      radar_front: {at: [-0.30, -0.30, -0.70], normal: [0, 0.34, 0.94], size: [0.14, 0.14]}
      radar_rear:  {at: [0.30, -0.30, -0.70],  normal: [0, 0.34, 0.94], size: [0.14, 0.14]}
      mfd:         {at: [0, -0.32, -0.75],     normal: [0, 0.34, 0.94], size: [0.22, 0.18]}
      gauge1:      {at: [-0.15, -0.20, -0.80], normal: [0, 0.34, 0.94], size: [0.06, 0.06]}
      gauge2:      {at: [-0.05, -0.20, -0.80], normal: [0, 0.34, 0.94], size: [0.06, 0.06]}
      gauge3:      {at: [0.05, -0.20, -0.80],  normal: [0, 0.34, 0.94], size: [0.06, 0.06]}
      gauge4:      {at: [0.15, -0.20, -0.80],  normal: [0, 0.34, 0.94], size: [0.06, 0.06]}
      message:     {at: [0, 0.32, -0.90],      normal: [0, -0.2, 0.98], size: [0.60, 0.04]}
      status:      {at: [0, 0.28, -0.90],      normal: [0, -0.2, 0.98], size: [0.60, 0.04]}
      power:       {at: [-0.30, -0.45, -0.65], normal: [0, 0.5, 0.87],  size: [0.18, 0.06]}
      systems:     {at: [0.30, -0.45, -0.65],  normal: [0, 0.5, 0.87],  size: [0.18, 0.04]}
```

Do the same for `b` and `c` with their own models if you build them.

## 6. Place the instruments with F12

1. Start the game with your data and fly any interceptor scenario in the cockpit view (F1).
2. Press **F12** to open the layout editor. The battle pauses.
3. **Tab** selects an instrument.
4. **Click the face** where it belongs: a ray from your eye through the cursor finds the face, and the instrument moves onto it, facing out from it.
5. The **mouse wheel** or **+**/**−** resizes it. Repeat for each instrument.
6. **S** saves a small layout pack to your mods folder. Copy its `rules/cockpit-layout.yaml` into your mod's `rules/` to ship the layout with it ([Cockpits › F12](cockpits.md#the-cockpit-layout-editor-f12)).

The Mod Designer (planned, [MD8](https://github.com/bbogovich/OpenInterceptor/issues/71)) will let you drag the instruments with a live preview on the faces.

## 7. Check it

```sh
openinterceptor --campaign interceptor --check
```

This loads every model and texture, your `original:` references included, and reports a missing file or a wrong resource number with the file and line. Then fly a few scenarios and look for:
- see-through areas where you expect glass;
- blurry art on large faces. The art is 640×480, so very large faces show its resolution;
- instruments that sit flat on their faces in every view direction.

## 8. Share it

The mod holds your model, your MTL, your own textures and rules, and **no original art**: the materials only name it. Anyone who owns X-COM: Interceptor can drop the folder into their mods folder. Don't include the reference PNGs from step 2.

## Revision history

| Version | Date | Change |
|---|---|---|
| 1.9.1 | 2026-11-11 | Page created: the 3D cockpit workflow with original art referenced from materials (app 0.4.13) |
