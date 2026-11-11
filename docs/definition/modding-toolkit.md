---
spec_version: 1.12.0
status: current
updated: 2026-11-11
---

# Modding toolkit

What a modder gets: packs, readable rule files, asset replacement, translation, conditions, the tools that check and generate content and, planned, the Mod Designer and Cutscene Designer ([design](../specification/design/designer.md)).

Requirements for these features: [Modding toolkit requirements](../specification/requirements/modding.md).

| Feature | Status |
|---|---|
| [F-MOD-001 Packs and mod kinds](#f-mod-001-packs-and-mod-kinds) | Implemented |
| [F-MOD-002 Readable rule files that explain mistakes](#f-mod-002-readable-rule-files-that-explain-mistakes) | Implemented |
| [F-MOD-003 Replace any asset, in open formats](#f-mod-003-replace-any-asset-in-open-formats) | Implemented |
| [F-MOD-004 Text and translation](#f-mod-004-text-and-translation) | Implemented |
| [F-MOD-005 Conditions, actions and scripting](#f-mod-005-conditions-actions-and-scripting) | In progress |
| [F-MOD-006 Mod validation](#f-mod-006-mod-validation) | In progress |
| [F-MOD-007 Generated assets (recipes)](#f-mod-007-generated-assets-recipes) | Planned |
| [F-MOD-008 Modding guide](#f-mod-008-modding-guide) | In progress |
| [F-MOD-009 Mod Designer](#f-mod-009-mod-designer) | Planned |
| [F-MOD-010 Cutscene Designer](#f-mod-010-cutscene-designer) | Planned |
| [F-MOD-011 Cutscenes in the game](#f-mod-011-cutscenes-in-the-game) | Planned |
| [F-MOD-012 Video cutscenes](#f-mod-012-video-cutscenes) | Planned |
| [F-MOD-013 Shared formations](#f-mod-013-shared-formations) | Planned |
| [F-MOD-014 3D cockpit instrument placement](#f-mod-014-3d-cockpit-instrument-placement) | Planned |
| [F-MOD-015 Single-file models and Blender sources](#f-mod-015-single-file-models-and-blender-sources) | Planned |

## Features

### F-MOD-001 Packs and mod kinds

**What.** Content comes in packs: campaigns, addons that change a campaign, and libraries shared between them; a campaign can extend another.

**Packs and mods.** Both use the same format; the difference is where they come from and whether a build includes them:

- **packs** are "baked into" the game on build: `resources/` and the bundled `packs/`, included in a product bundle ([R-PLT-007](../specification/requirements/platforms.md#r-plt-007-product-file), [R-PLT-008](../specification/requirements/platforms.md#r-plt-008-bundled-game-builds));
- **mods** are added externally, in the user's local mods folder.

A mod that builds on another pack is an **extension**: the other pack is its base, and it adds to or overrides that pack's assets ([R-MOD-023](../specification/requirements/modding.md#r-mod-023-new-pack-from-a-base)).

!!! note "Open"
    Where exactly the line between a pack and a mod falls is to be settled: for example, whether a mod promoted into a product bundle becomes a pack. The user, 2026-11-11: "Will have to consider where the distinction is between pack and mod in that case, but conceptually packs are \"baked into\" the game on build where mods are added externally in user's local files."

**Why.** Modding first.

- **Status:** implemented
- **Source:** Plan item 2.1. Packs versus mods and the name *extension*: the user, 2026-11-11 ("Let's call it an extension as opposed to an override, since another pack is the base and we are either adding to or overriding assets.").
- **Specified by:** [R-CPN-001](../specification/requirements/campaigns.md#r-cpn-001-one-campaign-at-a-time), [R-MOD-001](../specification/requirements/modding.md#r-mod-001-pack-manifests), [R-MOD-002](../specification/requirements/modding.md#r-mod-002-load-order), [R-MOD-003](../specification/requirements/modding.md#r-mod-003-rules-layering), [R-MOD-004](../specification/requirements/modding.md#r-mod-004-gameplay-values-are-data), [R-MOD-011](../specification/requirements/modding.md#r-mod-011-paths-stay-inside-the-pack), [R-MOD-019](../specification/requirements/modding.md#r-mod-019-sound-cue-table), [R-CON-020](../specification/requirements/control-console.md#r-con-020-console-theme-from-the-active-pack)

### F-MOD-002 Readable rule files that explain mistakes

**What.** Every mod file is plain YAML with units; mistakes are reported with file, line and key path and a suggestion; editors get JSON Schemas.

**Why.** A modder should never have to guess what went wrong.

- **Status:** implemented
- **Source:** Plan item 2.19 (user feedback); DEV-030.
- **Specified by:** [R-MSN-004](../specification/requirements/missions.md#r-msn-004-mission-mistakes-found-on-load), [R-MOD-005](../specification/requirements/modding.md#r-mod-005-yaml-subset), [R-MOD-006](../specification/requirements/modding.md#r-mod-006-schemas-and-messages), [R-MOD-007](../specification/requirements/modding.md#r-mod-007-settings-and-bindings-migration), [R-CMP-011](../specification/requirements/compatibility.md#r-cmp-011-bad-data-never-panics)

### F-MOD-003 Replace any asset, in open formats

**What.** Any original model, texture, image or sound can be replaced by a file in an open format (OBJ, PNG, TGA, WAV), and the originals can be extracted as editable copies.

**Why.** Modding first: HD and 3D replacements without redistributing original files.

- **Status:** implemented
- **Source:** Plan items 1.15 and 1.16; DEV-022.
- **Specified by:** [R-MOD-008](../specification/requirements/modding.md#r-mod-008-per-asset-overrides), [R-MOD-009](../specification/requirements/modding.md#r-mod-009-extraction), [R-MOD-010](../specification/requirements/modding.md#r-mod-010-open-model-and-image-formats)

### F-MOD-004 Text and translation

**What.** All text is referenced by key and can be translated by an addon.

**Why.** Translations and new campaigns without editing code.

- **Status:** implemented
- **Source:** Plan item 2.1.
- **Specified by:** [R-MOD-012](../specification/requirements/modding.md#r-mod-012-text-keys)

### F-MOD-005 Conditions, actions and scripting

**What.** Behaviours, missions and campaign rules use a small checked language of conditions and actions; a second-tier script language is to be decided (evaluation recorded in 1.12.0, awaiting the user's decision).

**Why.** Rules that depend on the situation, without code changes.

- **Status:** in-progress
- **Source:** Plan items 2.9, 3.1 and 3.10.
- **Specified by:** [R-MOD-013](../specification/requirements/modding.md#r-mod-013-condition-language), [R-MOD-014](../specification/requirements/modding.md#r-mod-014-campaign-context), [R-MOD-015](../specification/requirements/modding.md#r-mod-015-tier-2-scripting-decision)

### F-MOD-006 Mod validation

**What.** A modder can check a campaign and its addons headlessly and get every error at once.

**Why.** Mistakes found before playing.

- **Status:** in-progress
- **Source:** Plan items 1.13 and 7.1.
- **Specified by:** [R-CPN-003](../specification/requirements/campaigns.md#r-cpn-003-default-campaign-plays-without-the-cd), [R-MOD-016](../specification/requirements/modding.md#r-mod-016-headless-validation), [R-MOD-017](../specification/requirements/modding.md#r-mod-017-full-validator)

### F-MOD-007 Generated assets (recipes)

**What.** Mods can generate assets from the player's data (upscaled textures, patched meshes, recolours) reproducibly on the player's machine.

**Why.** Remastered content without redistributing original files.

- **Status:** planned
- **Source:** Plan items 7.2–7.6.
- **Specified by:** [R-CPN-006](../specification/requirements/campaigns.md#r-cpn-006-interceptor-remastered), [R-MOD-018](../specification/requirements/modding.md#r-mod-018-recipes)

### F-MOD-008 Modding guide

**What.** Documentation of every key, and tutorials from changing one number to building a new campaign.

**Why.** Modding first.

- **Status:** in-progress
- **Source:** Plan item 7.9; project rule: every new key is documented in the same change.
- **Specified by:** [R-MOD-020](../specification/requirements/modding.md#r-mod-020-every-key-documented)

### F-MOD-009 Mod Designer

**What.** A companion application, separate from the game, for building packs and mods without editing files by hand:
- a new pack or mod — a campaign, an addon or an **extension** of another pack, which adds to or overrides its assets — starts from a chosen **base** (the default campaign, `interceptor` or any installed pack);
- any asset of the parent can be **imported**: it is copied with its dependencies (a model with its textures, a ship with its model set, debris, sounds and text) and converted from original formats where needed;
- the copy is edited in an **external program** such as Blender and reloads live;
- forms generated from the engine's schemas, a raw YAML view, "where is this used?", **validation** on every save, **play** in the testbed, and packaging as a folder or zip.

Everything it saves is the same open pack data a modder could write by hand.

**Why.** Modding first: building a mod should not need a text editor and the file reference open side by side.

- **Status:** planned
- **Source:** The user's request, 2026-11-11, quoted in full on the [design page](../specification/design/designer.md): "Add to the longer term plan a companion app for a mod designer, which allows a user to define custom packs and mods to add to the game. On creation of a new pack or mod, user may choose a base to use for assets that they are going to override. When editing a mod, they can choose to import an asset from the parent. This duplicates the original asset and its dependencies (model plus textures in most cases) which the user can then edit in an external program, usually blender." Decisions D1 (macroquad UI, no new dependency) and D5 (a separate program in the same repository; two build artifacts), 2026-11-11. The kind *extension* rather than *override*: the user, 2026-11-11. Plan items MD4, MD5.
- **Specified by:** [R-MOD-021](../specification/requirements/modding.md#r-mod-021-designer-application), [R-MOD-022](../specification/requirements/modding.md#r-mod-022-files-stay-the-source-of-truth), [R-MOD-023](../specification/requirements/modding.md#r-mod-023-new-pack-from-a-base), [R-MOD-024](../specification/requirements/modding.md#r-mod-024-pack-tree-forms-and-yaml-view), [R-MOD-025](../specification/requirements/modding.md#r-mod-025-import-an-asset-with-its-dependencies), [R-MOD-026](../specification/requirements/modding.md#r-mod-026-where-an-asset-is-used), [R-MOD-027](../specification/requirements/modding.md#r-mod-027-external-editor-and-live-reload), [R-MOD-028](../specification/requirements/modding.md#r-mod-028-validate-on-save-play-in-the-testbed), [R-MOD-029](../specification/requirements/modding.md#r-mod-029-provenance-of-imported-assets), [R-MOD-030](../specification/requirements/modding.md#r-mod-030-package-a-pack)

### F-MOD-010 Cutscene Designer

**What.** Inside the Mod Designer, a timeline editor for introduction films and story scenes played by the engine itself:
- **scripted** entities, added and removed at set times, keyframed for position, rotation and speed (the speed is overridden when positions are keyed, so every key is met exactly), with timed actions: fire beams or missiles, chase, capture with the tractor, be destroyed, jump out;
- **autopilot** entities flown by the beginner AI rules towards designated targets, as background action, with an optional `aggressive` flag;
- a `harmless` flag that makes an entity's or an action's weapons fire without dealing damage;
- groups placed by a chosen **formation**;
- **sounds** at timed positions (narration, music, effects) and translatable **captions**;
- **cameras**, fixed or attached to an object with keyframed distance and orbit, and **camera cuts** set by keyframes;
- a **3D map** of the whole scene and a **preview** of the current frame, with exact seeking from snapshots;
- deterministic playback with **Simple physics**.

**Why.** Campaigns need films for their story; a modder should be able to make them with the game's own ships and effects.

- **Status:** planned
- **Source:** The user's request, 2026-11-11, quoted in full on the [design page](../specification/design/designer.md): "A cutscene designer is also needed in the mod tool for users to add introduction film and custom animations … Cutscenes always use basic physics so that the animation behavior is predictable. There are both scripted entities and autopilot in the cut scenes … Switching between cameras in the cutscene is set through keyframes." With decision D4: "Also a flag for the animations for their weapons to cause no damage." Decision D3: captions as text keys. Plan item MD6 (the editor) and MD2 (the format and player).
- **Specified by:** [R-MOD-021](../specification/requirements/modding.md#r-mod-021-designer-application), [R-MOD-031](../specification/requirements/modding.md#r-mod-031-cutscene-designer-workspace), [R-MOD-032](../specification/requirements/modding.md#r-mod-032-seeking-with-snapshots), [R-MOD-033](../specification/requirements/modding.md#r-mod-033-cutscene-files), [R-MOD-034](../specification/requirements/modding.md#r-mod-034-deterministic-playback-with-simple-physics), [R-MOD-035](../specification/requirements/modding.md#r-mod-035-scripted-entities-and-keyframes), [R-MOD-036](../specification/requirements/modding.md#r-mod-036-paths-and-rotations), [R-MOD-037](../specification/requirements/modding.md#r-mod-037-timed-actions), [R-MOD-038](../specification/requirements/modding.md#r-mod-038-autopilot-entities), [R-MOD-039](../specification/requirements/modding.md#r-mod-039-formations-in-cutscenes), [R-MOD-040](../specification/requirements/modding.md#r-mod-040-timed-sounds), [R-MOD-041](../specification/requirements/modding.md#r-mod-041-captions), [R-MOD-042](../specification/requirements/modding.md#r-mod-042-cameras-fixed-or-attached), [R-MOD-043](../specification/requirements/modding.md#r-mod-043-camera-cuts)

### F-MOD-011 Cutscenes in the game

**What.** The game plays a pack's cutscenes as the campaign's introduction, at key campaign events, and at the start and end of specific missions (an outro chosen by the result). The player can always skip one with Esc, the tablet console's skip button or the gamepad's Start button.

**Why.** Story moments in a campaign, the same way every time.

- **Status:** planned
- **Source:** The user's request, 2026-11-11, quoted in full on the [design page](../specification/design/designer.md): "introduction film and custom animations that play for key campaign events or at the start of specific mission definitions". Plan item MD2.
- **Specified by:** [R-MOD-033](../specification/requirements/modding.md#r-mod-033-cutscene-files), [R-MOD-034](../specification/requirements/modding.md#r-mod-034-deterministic-playback-with-simple-physics), [R-MOD-035](../specification/requirements/modding.md#r-mod-035-scripted-entities-and-keyframes), [R-MOD-036](../specification/requirements/modding.md#r-mod-036-paths-and-rotations), [R-MOD-037](../specification/requirements/modding.md#r-mod-037-timed-actions), [R-MOD-038](../specification/requirements/modding.md#r-mod-038-autopilot-entities), [R-MOD-039](../specification/requirements/modding.md#r-mod-039-formations-in-cutscenes), [R-MOD-040](../specification/requirements/modding.md#r-mod-040-timed-sounds), [R-MOD-041](../specification/requirements/modding.md#r-mod-041-captions), [R-MOD-042](../specification/requirements/modding.md#r-mod-042-cameras-fixed-or-attached), [R-MOD-043](../specification/requirements/modding.md#r-mod-043-camera-cuts), [R-MOD-044](../specification/requirements/modding.md#r-mod-044-where-cutscenes-play), [R-MOD-045](../specification/requirements/modding.md#r-mod-045-skipping-a-cutscene)

### F-MOD-012 Video cutscenes

**What.** A pack can provide a video file to play instead of any engine-played cutscene, with optional captions. Videos play through the operating system's media framework (on Linux, GStreamer when installed); image sequences with a WAV track always work.

**Why.** A creator may make a film in another program.

- **Status:** planned
- **Source:** The user's request, 2026-11-11, quoted in full on the [design page](../specification/design/designer.md): "Note that a user may also provide a video to play instead of scripted engine animation." Decision D2, 2026-11-11 (the Linux path to be evaluated first). Plan item MD3.
- **Specified by:** [R-MOD-044](../specification/requirements/modding.md#r-mod-044-where-cutscenes-play), [R-MOD-045](../specification/requirements/modding.md#r-mod-045-skipping-a-cutscene), [R-MOD-046](../specification/requirements/modding.md#r-mod-046-video-cutscenes)

### F-MOD-013 Shared formations

**What.** Formations (vee, line abreast, echelon, box, diamond and custom ones) are pack data, used by wingmen in battle and by groups in cutscenes alike.

**Why.** A modder defines a shape once; a cutscene and a battle look the same.

- **Status:** planned
- **Source:** The user's request, 2026-11-11, quoted in full on the [design page](../specification/design/designer.md): "For fighters and ships, they can choose which formation to apply." Plan item MD1 (it also serves wingmen and replaces per-scenario slot offsets).
- **Specified by:** [R-MOD-039](../specification/requirements/modding.md#r-mod-039-formations-in-cutscenes), [R-MOD-047](../specification/requirements/modding.md#r-mod-047-shared-formations), [R-CMB-022](../specification/requirements/combat.md#r-cmb-022-wingmen-keep-their-distance)

### F-MOD-014 3D cockpit instrument placement

**What.** In the Mod Designer, the instruments of a 3D cockpit (radar, MFD, gauges, status, message, power, systems and so on) are placed by dragging each one around the screen over the cockpit model: a ray from the eye through the cursor finds the face below it, and the instrument is previewed applied to that face, at the hit point with the face's normal, at an adjustable size. The result is the cockpit's `anchors`. A mod extending a pack can **create a 3D cockpit** from the parent's: the modder picks the cockpit images to extract (copied into the mod, provenance recorded), starts from the parent's 3D model or, if there is none, a generic base cockpit from `resources/default`, edits it in Blender, then places the instruments.

The first test case for the mod tools is exactly this: a mod extending `interceptor` with a 3D X-COM cockpit. The user also intends to design an X-COM 3D cockpit in Blender and ship it as an example mod that replaces the 2D cockpit image.

**Why.** Modding first: a 3D cockpit is only usable when its instruments sit on the right faces, and a modder should not have to measure anchors by hand.

- **Status:** planned
- **Source:** The user, 2026-11-11: "for xcom cockpits, we will be designing a 3d cockpit in blender to include as an example mod that substitutes the 3d to replace the 2d image. This raises a point for the mod design tools, they need to accommodate placement of the UI elements so that they are projected onto the correct faces in a 3d view. Simple handling will be to allow each element to be dragged around the screen to detemine their positions and then previewed by applying to the geometry that it is over. this will be the first test case for the mod design tools when available - a mod extending interceptor, where user selects the cockpit images to extract, they are copied into the mod, select to create a 3d cockpit (which will provide a generic base cockpit geometry defined in default if none is present in the parent). The user can then edit it in Blender." Plan items MD8, MD9.
- **Specified by:** [R-MOD-048](../specification/requirements/modding.md#r-mod-048-drag-instruments-onto-a-3d-cockpit), [R-MOD-049](../specification/requirements/modding.md#r-mod-049-instrument-size-and-preview-on-the-surface), [R-MOD-050](../specification/requirements/modding.md#r-mod-050-create-a-3d-cockpit), [R-MOD-051](../specification/requirements/modding.md#r-mod-051-generic-base-cockpit), [R-MOD-052](../specification/requirements/modding.md#r-mod-052-example-mod-x-com-3d-cockpit), [R-HUD-004](../specification/requirements/cockpit-hud.md#r-hud-004-anchored-3d-cockpit-instruments)

### F-MOD-015 Single-file models and Blender sources

**What.** OBJ/MTL with separate texture files stays a fully supported model format, the one the existing packs use. In addition, a model can be one `.glb` file (binary glTF: mesh, materials and textures together), read in-crate with no dependency. A mod can also carry its `.blend` files as the source: the designer exports them to OBJ/MTL or `.glb` through Blender, run headless when installed; the engine never reads `.blend`. Every import and export keeps each material with its texture images.

**Why.** A modder can package a model, its materials and its textures as one file, and keep the Blender source beside it.

- **Status:** planned
- **Source:** The user, 2026-11-11: "If possible, it would be ideal to allow the .blend files to be included in addtition to standaole mtl and obj objects so that a user can package the object, material, and textures into a single file, but that is not strictly necessary as long as the correct relationshps between materials and their texture images is consistently maintained." Confirmed: "ok that works. obj/mtl as in the existing packs of course would continue to be supported." Plan item MD10.
- **Specified by:** [R-MOD-053](../specification/requirements/modding.md#r-mod-053-single-file-models), [R-MOD-054](../specification/requirements/modding.md#r-mod-054-blender-sources), [R-MOD-055](../specification/requirements/modding.md#r-mod-055-materials-keep-their-textures), [R-MOD-010](../specification/requirements/modding.md#r-mod-010-open-model-and-image-formats)

## Revision history

| Version | Date | Change |
|---|---|---|
| 1.0.0 | 2026-11-11 | Bootstrap: F-MOD-001 – F-MOD-008 (8 features) |
| 1.7.0 | 2026-11-11 | F-MOD-009 – F-MOD-013 (planned): Mod Designer, Cutscene Designer, cutscenes in the game, video cutscenes, shared formations (user request and decisions D1–D5) |
| 1.9.0 | 2026-11-11 | F-MOD-014 3D cockpit instrument placement and F-MOD-015 single-file models and Blender sources (planned), from the user's message of 2026-11-11. F-MOD-001: packs (built into the game) versus mods (added from the user's mods folder), the term *extension*, Open note on where the line falls. F-MOD-009: the wizard's kind is *extension*, not *override* |
| 1.12.0 | 2026-11-11 | F-MOD-005: the Tier 2 evaluation is recorded on the dependency page (R-MOD-015); awaiting the user's decision |
