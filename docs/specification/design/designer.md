---
spec_version: 1.16.0
status: current
updated: 2026-11-11
---

# Mod designer, cutscene designer and game bundling

!!! warning "Status: planned, not built; decisions D1–D5 made by the user on 2026-11-11"
    This design is normative for the requirements that cite it. The five choices of the proposal
    were answered by the user on 2026-11-11 ([Decisions](#7-decisions)): D1 and D2 accepted (with
    the Linux video path recorded), D3 yes, D4 accepted with a further `harmless` flag, and D5 a
    separate designer program in the same repository. The Linux video path still needs its
    dependency evaluation before it is built. Further decisions of 2026-11-11 added 3D cockpit
    instrument placement, the first test case (an X-COM 3D cockpit), single-file models and the
    name *extension* ([sections 2.1 – 2.5](#21-packs-and-mods)).

**Features:** [F-MOD-009 – F-MOD-015](../../definition/modding-toolkit.md#f-mod-009-mod-designer),
[F-PLT-005](../../definition/platforms.md#f-plt-005-game-bundling).
**Requirements:** [R-MOD-021 – R-MOD-055](../requirements/modding.md#r-mod-021-designer-application),
[R-PLT-007 – R-PLT-010](../requirements/platforms.md#r-plt-007-product-file).
**Plan:** MD1 in [Stage 4](../../implementation/plan/stage-4-strategy.md#mod-and-cutscene-designer-md1),
MD2 – MD3 in [Stage 6](../../implementation/plan/stage-6-presentation.md#mod-and-cutscene-designer-md2-md3),
MD4 – MD6 and MD8 – MD10 in [Stage 7](../../implementation/plan/stage-7-modding.md#mod-and-cutscene-designer-md4-md6),
MD7 in [Stage 8](../../implementation/plan/stage-8-release.md#game-bundling-md7)
([where it goes](#6-where-it-goes-in-the-plan)).

A companion **Mod Designer** application lets a player build packs and mods without editing
files by hand:

- A new pack starts from a chosen **base**.
- Any asset of the parent can be **imported** for editing. The model, its materials and
  textures are copied together and edited in an external program such as Blender; the designer
  reloads them live.
- A **Cutscene Designer** inside it builds introduction films and story scenes with the game
  engine itself. It provides:
    - a timeline of keyframed **scripted** entities and AI-driven **autopilot** entities;
    - formations, weapons fire and timed destruction;
    - timed sounds;
    - fixed or attached cameras, with camera cuts;
    - a 3D map of the whole scene and a live preview of the current frame.
- Any cutscene can instead be a **video** file.
- The build system gains **product bundling**: a creator can ship a complete game, engine plus
  chosen packs, branded as their own, without players installing anything by hand.

Everything the designer writes is the same open YAML, OBJ, PNG and WAV data the engine already
reads, so nothing it makes is locked to the tool.

??? quote "Source: the user's request, 2026-11-11"
    "Add to the longer term plan a companion app for a mod designer, which allows a user to
    define custom packs and mods to add to the game. On creation of a new pack or mod, user may
    choose a base to use for assets that they are going to override. When editing a mod, they can
    choose to import an asset from the parent. This duplicates the original asset and its
    dependencies (model plus textures in most cases) which the user can then edit in an external
    program, usually blender. A cutscene designer is also needed in the mod tool for users to add
    introduction film and custom animations that play for key campaign events or at the start of
    specific mission definitions. The designer can import any assets into the cut scene. For
    fighters and ships, they can choose which formation to apply. Cutscenes always use basic
    physics so that the animation behavior is predictable. There are both scripted entities and
    autopilot in the cut scenes - autopilot will run based on beginner AI rules given designated
    targets and usually serve as background action. Scripted entities can be keyframed, allowing
    motion to be defined along with weapons fire and time triggered destruction. So this requires
    the ability to add and remove objects from the scene at specific times, move them in 3d space,
    set keyframes for position, rotation, speed (speed overridden when keyframe positions are set
    to guarantee that the positioning is correct), and assign targets for them to chase, capture,
    or shoot at. Sounds must be included at timed positions so that narration, music, and other
    audio can be played. Cameras can be defined at explicit positions or attached to an object. If
    attached, then the camera's coordinates are set relative to the parent object and positioning
    is keyframed for distance and orbit. User needs to be able to see the overall 3d map and
    preview of the current animation frame. Switching between cameras in the cutscene is set
    through keyframes. Produce a design PDF for the modding and animation features. Note that a
    user may also provide a video to play instead of scripted engine animation. Build system needs
    to accommodate bundling specific mods/packs into the distributable or application bundle so
    that users can release a complete game without requiring manual download and installation of
    packs and mods to their local folders."

    With decision D4 the user added: "Also a flag for the animations for their weapons to cause
    no damage."

??? quote "Source: the user's message of 2026-11-11 (3D cockpits, extensions, packs and mods)"
    "for xcom cockpits, we will be designing a 3d cockpit in blender to include as an example mod
    that substitutes the 3d to replace the 2d image. This raises a point for the mod design tools,
    they need to accommodate placement of the UI elements so that they are projected onto the
    correct faces in a 3d view. Simple handling will be to allow each element to be dragged around
    the screen to detemine their positions and then previewed by applying to the geometry that it
    is over. this will be the first test case for the mod design tools when available - a mod
    extending interceptor, where user selects the cockpit images to extract, they are copied into
    the mod, select to create a 3d cockpit (which will provide a generic base cockpit geometry
    defined in default if none is present in the parent). The user can then edit it in Blender.
    If possible, it would be ideal to allow the .blend files to be included in addtition to
    standaole mtl and obj objects so that a user can package the object, material, and textures
    into a single file, but that is not strictly necessary as long as the correct relationshps
    between materials and their texture images is consistently maintained.

    Open points
    Hit static - correct, works well now
    Let's call it an extension as opposed to an override, since another pack is the base and we
    are either adding to or overriding assets. Will have to consider where the distinction is
    between pack and mod in that case, but conceptually packs are "baked into" the game on build
    where mods are added externally in user's local files."

    On the single-file plan (section 2.5): "ok that works. obj/mtl as in the existing packs of
    course would continue to be supported."

## 1. Goals and principles

- **The files stay the source of truth.** The designer is a convenience over the documented
  [pack format](mod-structure.md).
    - Everything it saves can be read, diffed and hand-edited.
    - Everything hand-written opens in it.
    - It validates with the engine's own `--check`.
- **One engine.** The previews are drawn by the game's renderer and simulated by the game's
  sim, so what the designer shows is what the game plays.
- **Original content stays private.**
    - Importing an asset from the X-COM data converts it into the user's own mod folder, for
      personal use only.
    - A pack holding converted original assets is marked as such and can never be bundled for
      release ([section 5](#5-bundling-a-complete-game)).
- **Deterministic cutscenes.** A cutscene plays the same way every time: fixed tick, seeded
  randomness, Simple physics.
- **Modest dependencies.** No new dependency without the evaluation the project requires;
  decisions D1 and D2 add none.

## 2. The Mod Designer

### 2.1 Packs and mods

| Action | What it does |
|---|---|
| New pack / mod | Asks for an id, a name and a kind: **campaign** (a new game), **addon** (adds to a campaign) or **extension** (another pack is the base; it adds to or overrides that pack's assets). Asks for a **base**: the default campaign (`resources/default`), `interceptor`, or any installed pack. Writes `pack.yaml` with `parent:` and creates the folder skeleton (`rules/`, `models/`, `textures/`, `sounds/`, `text/en/`, `missions/`, `scenarios/`, `cutscenes/`). |
| Open / edit | A pack tree, with definitions grouped as ships, weapons, missions, cutscenes, text and so on. A form editor generated from the engine's schemas — the same ones that produce the JSON Schemas — so new keys appear without designer changes. A raw YAML view. |
| Import from parent | Pick any asset the parent provides: a model, texture, sound, image or cockpit. The designer copies it and its dependencies into the mod at the path that overrides the original ([2.2](#22-what-an-asset-and-its-dependencies-means)). Original formats (XMSH, XTXR, TGA, ADPCM WAV) are converted to OBJ/MTL, PNG and WAV by the extractor that already exists. |
| Edit externally | "Open in Blender" (or any program set per file type) launches the external editor on the copied file. A file watcher reloads the asset in the preview on save, using the engine's model cache. |
| Validate and play | The engine validator runs on every save; each problem is reported with its file, line and key. "Play" starts the testbed with the mod enabled, on a chosen scenario or mission. |
| Package | Exports the pack as a folder or zip for sharing, or adds it to a product bundle ([section 5](#5-bundling-a-complete-game)). |

!!! note "Decided (2026-11-11): extension, not override"
    The user: "Let's call it an extension as opposed to an override, since another pack is the base
    and we are either adding to or overriding assets." An extension declares `parent:` and adds to
    or overrides the base's assets.

!!! note "Open"
    The pack manifest's kinds today are `campaign`, `addon` and `library`
    ([R-MOD-001](../requirements/modding.md#r-mod-001-pack-manifests)). Whether **extension**
    becomes a manifest kind of its own or maps onto `addon` is decided when MD4 is built.

**Packs and mods.** The same format; the difference is where they come from and whether a build
includes them. **Packs** are "baked into" the game on build: `resources/` and the bundled
`packs/`, included in a product bundle ([section 5](#5-bundling-a-complete-game)). **Mods** are
added externally, in the user's local mods folder. *Open:* where exactly the line falls — for
example, whether a mod promoted into a product bundle becomes a pack — is to be settled.

### 2.2 What "an asset and its dependencies" means

| Imported | Copied with it |
|---|---|
| Ship or object | Its definition (the rules entry, with inheritance resolved or kept, at the user's choice); its model set (intact, damage-level and construction-state meshes, LODs and overlays); debris pieces and the destroyed effect; each model's materials and textures; its engine and weapon sounds; its text keys. |
| Model | The OBJ, its MTL, every texture the MTL names, and its LOD files. |
| Cockpit | The cockpit image or 3D model, its instrument layout, and the textures the 3D model uses. |
| Mission or scenario | The definition and its text keys. The ships it uses stay references; importing them is offered as a separate choice. |

A **dependency graph** built from the loaded content drives this. The same graph powers "where
is this used?" and warns before deleting anything still referenced.

### 2.3 Instruments on a 3D cockpit

A 3D cockpit's instruments (radar, MFD, gauges, status, message, power, systems…) are drawn
on the model's surface at **anchors**, `{at, normal, size}`
([Cockpits](../../modding/cockpits.md#a-3d-cockpit-instead-of-the-overlay)). In the designer:

1. The modder **drags** an instrument around the screen over the cockpit model.
2. On every frame of the drag, a ray from the eye through the cursor is intersected with the
   model's triangles; the nearest hit gives the anchor's `at`, and the hit face's normal its
   `normal`.
3. The instrument is **previewed applied to that face**, drawn by the game's own
   anchored-instrument path, so it looks as it will in the game. Over no face, it keeps its last
   anchor.
4. Its **size** on the surface is adjustable. Dropping keeps the anchor; saving writes
   `cockpits.<id>.anchors` to the mod's rules.

An instrument with only a 2D rectangle gains an anchor the same way; its rectangle still serves
the other views.

**Reuse.** The engine already has the pieces: `hud::layout::cursor_ray` (the ray through the
cursor), `hud::layout::ray_triangle` (the hit) and `hud::layout::anchor_corners` (the
instrument's rectangle on the surface); the `cockpits.<id>.anchors` key; and the game's F12
layout editor (`app/flight.rs`, `edit_layout`), which drags 2D rectangles and already places an
existing anchor where the dashboard is clicked. The designer reuses them. As a first step, the
F12 editor could gain the drag with live preview, and anchors for instruments that have none.

Requirements: [R-MOD-048](../requirements/modding.md#r-mod-048-drag-instruments-onto-a-3d-cockpit),
[R-MOD-049](../requirements/modding.md#r-mod-049-instrument-size-and-preview-on-the-surface).

### 2.4 First test case: an X-COM 3D cockpit

The first test case for the mod tools, when they are available, is a mod that **extends
`interceptor`** with a 3D X-COM cockpit:

1. The modder selects the cockpit images to extract. They are copied into the mod — converted
   from the original data by the extractor, with their provenance recorded, for personal use
   only and never bundled.
2. **Create 3D cockpit** copies the parent's cockpit model into the mod or, when the parent has
   none, a **generic base cockpit** defined in `resources/default`: an original-free OBJ/MTL model
   with its textures, generated by `tools/default_assets/`, with dashboard surfaces and default
   anchors for the standard instruments.
3. The modder edits the model in Blender (live reload, [2.1](#21-packs-and-mods)).
4. The modder places the instruments on it ([2.3](#23-instruments-on-a-3d-cockpit)).

Separately, the user intends to design an X-COM 3D cockpit in Blender and ship it as an
**example mod** in `packs/` that replaces the 2D cockpit image with a 3D one. Its geometry is
original-free; the X-COM cockpit textures are referenced from the player's original data, not
copied, and the mod declares `requires_original: true`. *Open:* an MTL names its textures by file
path, so how the model refers to a texture in the original data is settled with MD9.

Requirements: [R-MOD-050](../requirements/modding.md#r-mod-050-create-a-3d-cockpit),
[R-MOD-051](../requirements/modding.md#r-mod-051-generic-base-cockpit),
[R-MOD-052](../requirements/modding.md#r-mod-052-example-mod-x-com-3d-cockpit).

### 2.5 Single-file models and Blender sources

- **OBJ/MTL stays.** Wavefront OBJ/MTL with separate texture files remains a fully supported
  format: it is what the existing packs use, and nothing below replaces it.
- **`.glb` as an additional option.** Binary glTF is one file holding mesh, materials and
  textures. Blender exports it natively, and the engine can read it in-crate: a JSON chunk (the
  crate already has a JSON parser, `src/remote/json.rs`) and binary buffers. No dependency.
- **`.blend` as the source only.** A mod may carry its `.blend` files. When Blender is
  installed, the designer runs it headless on save to export OBJ/MTL or `.glb`
  (`blender -b file.blend --python-expr …`). The engine never parses `.blend`, whose binary
  format changes between Blender versions. Blender is an optional external tool, not a
  dependency ([Dependency policy](../../learning/best-practices/dependencies.md#external-tools)).
- **Materials keep their textures.** On every import and export, each material stays with its
  texture images: MTL `map_Kd` paths are relative to the `.mtl` file and inside the pack.

The user called single files "not strictly necessary as long as the correct relationshps between
materials and their texture images is consistently maintained", and confirmed this plan: "ok that
works. obj/mtl as in the existing packs of course would continue to be supported."

Requirements: [R-MOD-053](../requirements/modding.md#r-mod-053-single-file-models),
[R-MOD-054](../requirements/modding.md#r-mod-054-blender-sources),
[R-MOD-055](../requirements/modding.md#r-mod-055-materials-keep-their-textures).

## 3. The Cutscene Designer

A cutscene is a short film played by the engine at a campaign event, at the start of a
specific mission, or as the campaign's introduction. The designer edits it on a timeline,
previews it frame by frame, and saves it as `cutscenes/<id>.yaml` in the pack.

### 3.1 Layout

```text
┌──────────────────┬───────────────────────────────┬───────────────────────────────┐
│ Scene            │ 3D map (whole scene)          │ Preview (active camera, t)    │
│ ▸ Entities       │                               │                               │
│   lightning_1    │   paths · keyframes ·         │   engine renderer,            │
│   lightning_2    │   camera frusta               │   current frame               │
│   wraith_* ×4    │                               │                               │
│   xcom_base      │                               │                               │
│ ▸ Cameras        │                               │                               │
│ ▸ Sounds         │                               │                               │
│ ▸ Captions       │                               │                               │
├──────────────────┴───────────────────────────────┴───────────────────────────────┤
│ Timeline  ▶ 00:12.40                                                             │
│ lightning_1 pos   ◆──────◆────────◆                                              │
│ lightning_1 fire          ●                                                      │
│ camera cut        ▮            ▮                                                 │
│ narration           ▭▭▭▭▭▭▭▭                                                     │
│ wraith_3 destroy                ✕                                                │
└──────────────────────────────────────────────────────────────────────────────────┘
```

The **outline** of the scene (left), the **3D map** of the whole scene with paths and camera
frusta (centre), the **preview** through the active camera at the current frame (right), and the
**timeline** with tracks, keyframes and the playhead (bottom). An **inspector** panel for the
selection opens on the right.

### 3.2 What a scene contains

| Element | Behaviour |
|---|---|
| Scripted entity | Any ship, object or debris from the loaded packs. It has **enter** and **leave** times, and keyframes for **position**, **rotation** and **speed**. When position keyframes exist, the speed is derived from the path (overriding speed keys), so the entity is exactly where the keyframes say at each time. It has **actions** at times: fire beams or a missile at a target, start chasing a target, capture a target with the tractor, be destroyed (with its looks and debris), jump to hyperspace. A **`harmless`** flag, on the entity or on one action, makes its weapons fire and show their effects but deal no damage (D4). |
| Autopilot entity | Flown by the engine's AI with the **beginner** difficulty rules, so it is predictable. It is given designated targets (attack, escort, flee to a point), usually as background action. An optional per-entity **`aggressive`** flag livens up background dogfights, and **`harmless`** works as for a scripted entity (D4). It may be switched to scripted at a time, and back. |
| Formation | A group of fighters or ships placed by a named formation (vee, line abreast, echelon, box, diamond, or a custom one). It has a leader, which is scripted or autopilot. Formations are pack data (`formations:`) shared with wingman flying, so a cutscene and a battle use the same shapes. |
| Sound | A clip at a time on a sound track: narration, music or effects. It has a volume and a fade in and out, and is either positioned (attached to an entity) or flat. Music can duck under narration. |
| Caption | Optional subtitle text (a text key, so it can be translated) shown over a time range. It makes narration accessible (D3: yes). |
| Camera | **Fixed**: keyframed world position and look target or rotation, plus field of view. **Attached**: parented to an entity, with keyframed distance, orbit (yaw and pitch around the parent), height and look offset; its coordinates are relative to the parent. |
| Camera cuts | A track of keyframes, each choosing the active camera. Each cut is instant or a short blend. |
| Environment | Starfield seed, sun, backdrop, and later terrain and atmosphere (Stage 3.3–3.5). |

### 3.3 How it plays: deterministic, with Simple physics

- A cutscene runs on the normal **60 Hz** sim with a **fixed seed**, and every ship flies
  **Simple physics** ([R-FLT-013](../requirements/flight.md#r-flt-013-simple-physics-mode)).
  Motion is what the keyframes say, and autopilot traffic is repeatable.
- **Paths:** position keyframes are joined by a **centripetal Catmull-Rom spline**, which passes
  through every key without overshooting at sharp corners. Rotations use **quaternion slerp**. A
  ship with position keys and no rotation keys faces along its path, banking into turns.
- **Scrubbing:** the preview seeks by replaying the simulation from the nearest **snapshot**.
  Snapshots of the whole world are taken every **2 s** while playing, so seeking stays quick and
  exact.
- **Destruction and fire:** these are actions on the timeline that call the same simulation
  functions as a battle — a beam shot, a missile launch, *Destroyed* with debris. The same
  effects and sounds play.

### 3.4 The cutscene file

```yaml
id: intro
title: "@cutscene.intro.title"
length: 48s
environment: {starfield_seed: 31, sun: [-0.4, 0.45, -0.8]}
entities:
  lead: {class: lightning, enter: 0s, keys: {position: [[0s, [0,0,0]], [6s, [400,50,-1800]]]},
         actions: [[4.5s, fire_beams, {target: drone_1}]]}
  wing: {formation: vee, leader: lead, class: [lightning, lightning], harmless: true}
  drone_1: {class: wraith, autopilot: {attack: lead}, aggressive: true, harmless: true,
            enter: 2s, leave: 9s, destroy_at: 5.2s}
cameras:
  wide:  {fixed: {keys: {position: [[0s, [0,300,900]]], look_at: [[0s, lead]]}, fov: 55}}
  chase: {attach: lead, keys: {distance: [[0s, 60], [5s, 25]], orbit: [[0s, [0,10]], [5s, [40,5]]]}}
cuts: [[0s, wide], [3.5s, chase, {blend: 0.5s}]]
sounds: [[0s, music/theme.wav, {volume: 0.6, fade_in: 2s, duck: true}],
         [1s, voice/narration_01.wav]]
captions: [[1s, 6s, "@cutscene.intro.line1"]]
```

Or, for a film made elsewhere: `id: intro`, `video: video/intro.mp4`, with optional
`captions:`.

**Where cutscenes play:**

- `campaign: {intro: intro}` — the campaign's introduction;
- an event action `play_cutscene(intro)`;
- a mission's `intro: <cutscene id>`, or `outro: {win: …, lose: …}`.

A cutscene can always be **skipped** with Esc, the console's skip button or the pad's Start
button.

*The key names above are the proposal's; they are fixed, with their schema and the Modding
pages, when MD2 is built.*

!!! note "As built (MD2, app 0.13.0)"
    The format as built is a subset, documented key by key on [Modding › Cutscenes](../../modding/cutscenes.md):
    `id`, `title`, `length`, `environment {starfield_seed, sun, environment}`; entities with
    `class`, `faction`, `enter`, `leave`, `keys.position`, `actions` (`[time, fire_beams, secs]`,
    `[time, missile, target]`, `[time, destroy]`), `destroy_at` and `harmless`; cameras **fixed**
    (`position` keys and `look_at` — an entity or a point — directly on the camera, not under
    `fixed: {keys: …}`) or **attached** (`attach`, `distance` and `orbit` keys directly on the
    camera, not under `keys:`), each with `fov`; `cuts` (instant); `sounds` naming the
    campaign's event sounds (`[time, name, volume]`); `captions` as plain text. It plays as the
    campaign intro (`campaign.intro`), from events (`play_cutscene`) and with `--cutscene`; Esc
    or Enter skips. Not built: formations, autopilot entities and `aggressive`, rotation and
    speed keys, chase/capture/jump actions, `harmless` on a single action (per entity it works since app 0.14.1), sound
    files with fades and ducking, text-key captions, blended cuts, mission intros and outros,
    `video:` in a cutscene, skipping from the console or pad, scrubbing and snapshots. The
    original's Smacker films play through `campaign.intro_video`, `video(…)` and an ending's
    `video` (app 0.12.0 – 0.13.0). [DEV-054](../../implementation/deviations/index.md).

### 3.5 Video instead of animation

A pack can provide a video file for any cutscene. The engine has no video decoder for modern
formats today, and the Smacker decoder planned for 6.3 covers only the original's films.
Decision D2: mod videos play through the **operating system's own media framework**, behind the
platform shim, with **image sequences plus a WAV track** as the fallback that always works:

- **macOS:** AVFoundation.
- **Windows:** Media Foundation.
- **Linux:** GStreamer, when installed, is **loaded at run time with `dlopen`** — no crate
  dependency and no link-time dependency. GStreamer is LGPL; loading the system's own copy
  dynamically, without bundling it, keeps the project clear of it. Without GStreamer, mod videos
  fall back to image sequences.

!!! note "Open"
    The Linux path (GStreamer through `dlopen`) must be evaluated on the
    [dependency page](../../learning/best-practices/dependencies.md) — complexity, stability,
    licence — before it is built (MD3).

## 4. How the designer is built

- **A separate program in the same repository, sharing the engine library** (D5).
  `openinterceptor-designer` is a second binary in the same crate, using `content`, `sim`,
  `render` and `audio` directly. The previews are therefore the real engine, and the designer
  cannot drift from it. Being its own program keeps its interface focused, and the game and the
  designer can run side by side. The build produces **two artifacts**: the game and the
  designer.
- **Interface** (D1). macroquad's built-in immediate-mode UI module for panels, forms and the
  timeline, with the engine renderer drawing the 3D map and preview viewports. This adds no new
  dependency. Forms are generated from the schema types
  (`content::schema::Ty`), which already describe every key with a help text.
- **Editing model.**
    - An in-memory document per open file, with an undo stack of edits.
    - Saving writes YAML through the existing writer and keeps comments where possible.
      Hand-written files are rewritten only in the keys that changed.
- **Engine additions.**
    - `cutscene`: the file format and validation, plus a player that drives `sim` and the camera
      from the timeline.
    - `formations`: shared pack data.
    - Video playback through the platform layer.
    - The game plays cutscenes at campaign events and before missions.
- **Testing.**
    - Unit tests for spline and slerp sampling, keyframe evaluation and the speed override,
      action timing, snapshot seeking (seek, then play, must equal straight play),
      dependency-graph import, and YAML round-trips.
    - Manual tests for the designer's interface.

## 5. Bundling a complete game

A creator should be able to release their campaign as a game in its own right, downloaded and
played with nothing to install.

- A **product file**, `product.yaml` in the creator's project, describes the bundle:
    - name, version, bundle id and icon;
    - splash image and the intro cutscene;
    - the packs to include, in load order;
    - the default campaign;
    - whether the testbed, model viewer and developer tools are shown (off for a release);
    - licence and credit files.
- The build produces two artifacts, the **game** and the **designer** (D5). A product bundle
  is built from the game artifact; the designer stays a separate program.
- The **build tool** — `tools/build_bundle.py --product product.yaml`, or the designer's
  "Package game":
    - copies the engine, `resources/default` (if used) and exactly the listed packs into the
      bundle;
    - names and brands the app;
    - starts it directly in the chosen campaign;
    - writes the macOS `.app` now, and the Linux and Windows packages when Stage 8 adds them.

    Players' own mods still load from their user folder on top.
- **Safety: nothing original is ever bundled.** The bundler refuses:
    - a pack with `requires_original`;
    - any file marked as converted from original data. The designer records the provenance of
      imported assets in `pack.yaml`, and the extractor already knows which files it converted.

    A pack built on `interceptor` can be shared as a mod for people who own the game, but never
    bundled into a product.
- **Validation first.** The full validator runs and must report no errors before a bundle is
  written.

## 6. Where it goes in the plan

| Item | Scope | Stage (design) | Placed in |
|---|---|---|---|
| MD1 | Shared formations: data, used by wingmen first (it also replaces today's per-scenario offsets) | Stage 3 / 4 | [Stage 4](../../implementation/plan/stage-4-strategy.md#mod-and-cutscene-designer-md1); it also serves wingmen |
| MD2 | Cutscene format, validation and the in-game player (timeline, spline paths, actions, cameras, cuts, sounds, captions); playing at campaign intro, events and mission intro and outro; skip | Stage 6 | [Stage 6](../../implementation/plan/stage-6-presentation.md#mod-and-cutscene-designer-md2-md3) |
| MD3 | Video playback for mod cutscenes via the platform layer (D2; the Linux `dlopen` path evaluated first) | Stage 6 | [Stage 6](../../implementation/plan/stage-6-presentation.md#mod-and-cutscene-designer-md2-md3) |
| MD4 | Designer application shell (a separate program and build artifact, D5; macroquad UI, D1): pack wizard with base selection, pack tree, schema-driven forms and YAML view, validation, play in testbed | Stage 7 | [Stage 7](../../implementation/plan/stage-7-modding.md#mod-and-cutscene-designer-md4-md6) |
| MD5 | Asset import with dependencies (including conversion from original formats), external editor launch, live reload, "where used", provenance recording | Stage 7 | [Stage 7](../../implementation/plan/stage-7-modding.md#mod-and-cutscene-designer-md4-md6) |
| MD6 | Cutscene Designer: outline, 3D map, preview, timeline with keyframes, cameras and cuts, sounds, captions, scrubbing with snapshots | Stage 7 | [Stage 7](../../implementation/plan/stage-7-modding.md#mod-and-cutscene-designer-md4-md6) |
| MD8 | 3D cockpit instrument placement: drag each instrument over the cockpit model, ray-cast to the face below the cursor, preview applied to it, adjustable size, saved as `anchors` | Stage 7 | [Stage 7](../../implementation/plan/stage-7-modding.md#mod-and-cutscene-designer-md4-md6) |
| MD9 | First test case for the mod tools: an `interceptor` extension with a 3D X-COM cockpit (extract the cockpit images, Create 3D cockpit, edit in Blender, place instruments); the generic base cockpit in `resources/default`; the X-COM 3D cockpit example mod in `packs/` | Stage 7 | [Stage 7](../../implementation/plan/stage-7-modding.md#mod-and-cutscene-designer-md4-md6) |
| MD10 | Single-file models: `.glb` reader (in-crate) beside OBJ/MTL; `.blend` sources exported through Blender when installed; materials keep their textures | Stage 7 | [Stage 7](../../implementation/plan/stage-7-modding.md#mod-and-cutscene-designer-md4-md6) |
| MD7 | Product bundling: the build's two artifacts (game and designer, D5); `product.yaml`, branded builds, release mode without developer tools, the provenance guard | Stage 8 | [Stage 8](../../implementation/plan/stage-8-release.md#game-bundling-md7) |

## 7. Decisions

Proposed with recommendations; **answered by the user on 2026-11-11**.

| # | Question | Decision (2026-11-11) |
|---|---|---|
| D1 | The designer's interface toolkit | **Accepted:** macroquad's built-in UI module plus the engine renderer for the viewports; no new dependency. (The alternative, a local web interface with streamed preview frames, is not taken.) |
| D2 | How to play mod videos | **Accepted:** the operating system's media framework through the platform shim — AVFoundation on macOS, Media Foundation on Windows — with image sequences plus WAV as the fallback that always works. **Linux:** GStreamer, when installed, is loaded at run time with `dlopen` (no crate dependency, no link-time dependency); GStreamer is LGPL, and loading the system's copy dynamically, without bundling it, keeps the project clear of it; without GStreamer, mod videos fall back to image sequences. *Open:* the Linux path must be evaluated on the dependency page before it is built. An in-crate MP4/H.264 decoder is out of scope. |
| D3 | Captions in cutscenes | **Yes:** captions are text keys, translatable. |
| D4 | Autopilot difficulty in cutscenes | **Accepted:** autopilot uses the beginner AI rules, with an optional per-entity `aggressive` flag. **Added by the user:** "Also a flag for the animations for their weapons to cause no damage." — a per-entity and per-action `harmless` flag: its weapons fire and show their effects but deal no damage. |
| D5 | Where the designer lives | **A separate program in the same repository**, so its interface stays focused and the game and the designer can run in parallel. The build produces two artifacts: the game and the designer. |

## Revision history

| Version | Date | Change |
|---|---|---|
| 1.7.0 | 2026-11-11 | New page, with the user's decisions D1–D5 (D2's Linux path via `dlopen` still to be evaluated; D4 adds the `harmless` flag; D5 two build artifacts): the Mod Designer, the Cutscene Designer, video cutscenes, shared formations and game bundling, from the user's request of 2026-11-11; plan items MD1–MD7 |
| 1.9.0 | 2026-11-11 | From the user's message of 2026-11-11 (quoted): new sections 2.3 instruments on a 3D cockpit (drag, ray-cast to the face, preview), 2.4 the first test case (an `interceptor` extension with a 3D X-COM cockpit; the generic base cockpit; the example mod) and 2.5 single-file models (`.glb` beside OBJ/MTL, which stays fully supported, `.blend` as source only; confirmed by the user); the wizard's kind renamed **extension**; packs versus mods; plan items MD8–MD10 |
| 1.16.0 | 2026-11-11 | §3.4 As-built note: the cutscene format as built in MD2 (app 0.13.0) and what is not built |
