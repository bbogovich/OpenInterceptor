---
spec_version: 1.7.0
status: current
updated: 2026-11-11
---

# Mod designer, cutscene designer and game bundling

!!! warning "Status: planned, not built; decisions D1–D5 made by the user on 2026-11-11"
    This design is normative for the requirements that cite it. The five choices of the proposal
    were answered by the user on 2026-11-11 ([Decisions](#7-decisions)): D1 and D2 accepted (with
    the Linux video path recorded), D3 yes, D4 accepted with a further `harmless` flag, and D5 a
    separate designer program in the same repository. The Linux video path still needs its
    dependency evaluation before it is built.

**Features:** [F-MOD-009 – F-MOD-013](../../definition/modding-toolkit.md#f-mod-009-mod-designer),
[F-PLT-005](../../definition/platforms.md#f-plt-005-game-bundling).
**Requirements:** [R-MOD-021 – R-MOD-047](../requirements/modding.md#r-mod-021-designer-application),
[R-PLT-007 – R-PLT-010](../requirements/platforms.md#r-plt-007-product-file).
**Plan:** MD1 in [Stage 4](../../implementation/plan/stage-4-strategy.md#mod-and-cutscene-designer-md1),
MD2 – MD3 in [Stage 6](../../implementation/plan/stage-6-presentation.md#mod-and-cutscene-designer-md2-md3),
MD4 – MD6 in [Stage 7](../../implementation/plan/stage-7-modding.md#mod-and-cutscene-designer-md4-md6),
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
| New pack / mod | Asks for an id, a name and a kind: **campaign** (a new game), **addon** (adds to a campaign) or **override** (replaces assets). Asks for a **base** to override: the default campaign (`resources/default`), `interceptor`, or any installed pack. Writes `pack.yaml` with `parent:` and creates the folder skeleton (`rules/`, `models/`, `textures/`, `sounds/`, `text/en/`, `missions/`, `scenarios/`, `cutscenes/`). |
| Open / edit | A pack tree, with definitions grouped as ships, weapons, missions, cutscenes, text and so on. A form editor generated from the engine's schemas — the same ones that produce the JSON Schemas — so new keys appear without designer changes. A raw YAML view. |
| Import from parent | Pick any asset the parent provides: a model, texture, sound, image or cockpit. The designer copies it and its dependencies into the mod at the path that overrides the original ([2.2](#22-what-an-asset-and-its-dependencies-means)). Original formats (XMSH, XTXR, TGA, ADPCM WAV) are converted to OBJ/MTL, PNG and WAV by the extractor that already exists. |
| Edit externally | "Open in Blender" (or any program set per file type) launches the external editor on the copied file. A file watcher reloads the asset in the preview on save, using the engine's model cache. |
| Validate and play | The engine validator runs on every save; each problem is reported with its file, line and key. "Play" starts the testbed with the mod enabled, on a chosen scenario or mission. |
| Package | Exports the pack as a folder or zip for sharing, or adds it to a product bundle ([section 5](#5-bundling-a-complete-game)). |

!!! note "Open"
    The pack manifest's kinds today are `campaign`, `addon` and `library`
    ([R-MOD-001](../requirements/modding.md#r-mod-001-pack-manifests)). The wizard's **override**
    kind is an addon that only replaces assets; whether it is written as `kind: addon` or becomes a
    manifest kind of its own is decided when MD4 is built.

### 2.2 What "an asset and its dependencies" means

| Imported | Copied with it |
|---|---|
| Ship or object | Its definition (the rules entry, with inheritance resolved or kept, at the user's choice); its model set (intact, damage-level and construction-state meshes, LODs and overlays); debris pieces and the destroyed effect; each model's materials and textures; its engine and weapon sounds; its text keys. |
| Model | The OBJ, its MTL, every texture the MTL names, and its LOD files. |
| Cockpit | The cockpit image or 3D model, its instrument layout, and the textures the 3D model uses. |
| Mission or scenario | The definition and its text keys. The ships it uses stay references; importing them is offered as a separate choice. |

A **dependency graph** built from the loaded content drives this. The same graph powers "where
is this used?" and warns before deleting anything still referenced.

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
