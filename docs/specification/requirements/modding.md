---
spec_version: 1.9.0
status: current
updated: 2026-11-11
---

# Modding toolkit requirements

Requirements for packs, rule files, schemas, overrides, extraction, text, the condition language, validation and recipes, and (planned) the Mod Designer, 3D cockpit instrument placement, single-file models, cutscenes, video cutscenes and shared formations ([design](../design/designer.md)). The normative structure is [Mod structure](../design/mod-structure.md); every key is in the [Mod file reference](../../modding/reference.md).

Features: [Modding toolkit](../../definition/modding-toolkit.md). How to read a requirement: [Specification](../index.md#reading-a-requirement).

| Requirement | Status |
|---|---|
| [R-MOD-001 Pack manifests](#r-mod-001-pack-manifests) | Implemented |
| [R-MOD-002 Load order](#r-mod-002-load-order) | Implemented |
| [R-MOD-003 Rules layering](#r-mod-003-rules-layering) | Implemented |
| [R-MOD-004 Gameplay values are data](#r-mod-004-gameplay-values-are-data) | Implemented |
| [R-MOD-005 YAML subset](#r-mod-005-yaml-subset) | Implemented |
| [R-MOD-006 Schemas and messages](#r-mod-006-schemas-and-messages) | Implemented |
| [R-MOD-007 Settings and bindings migration](#r-mod-007-settings-and-bindings-migration) | Implemented |
| [R-MOD-008 Per-asset overrides](#r-mod-008-per-asset-overrides) | Implemented |
| [R-MOD-009 Extraction](#r-mod-009-extraction) | Implemented |
| [R-MOD-010 Open model and image formats](#r-mod-010-open-model-and-image-formats) | Implemented |
| [R-MOD-011 Paths stay inside the pack](#r-mod-011-paths-stay-inside-the-pack) | Implemented |
| [R-MOD-012 Text keys](#r-mod-012-text-keys) | Implemented |
| [R-MOD-013 Condition language](#r-mod-013-condition-language) | Implemented |
| [R-MOD-014 Campaign context](#r-mod-014-campaign-context) | Planned |
| [R-MOD-015 Tier 2 scripting decision](#r-mod-015-tier-2-scripting-decision) | Planned |
| [R-MOD-016 Headless validation](#r-mod-016-headless-validation) | Implemented |
| [R-MOD-017 Full validator](#r-mod-017-full-validator) | Planned |
| [R-MOD-018 Recipes](#r-mod-018-recipes) | Planned |
| [R-MOD-019 Sound cue table](#r-mod-019-sound-cue-table) | Implemented |
| [R-MOD-020 Every key documented](#r-mod-020-every-key-documented) | Implemented |
| [R-MOD-021 Designer application](#r-mod-021-designer-application) | Planned |
| [R-MOD-022 Files stay the source of truth](#r-mod-022-files-stay-the-source-of-truth) | Planned |
| [R-MOD-023 New pack from a base](#r-mod-023-new-pack-from-a-base) | Planned |
| [R-MOD-024 Pack tree, forms and YAML view](#r-mod-024-pack-tree-forms-and-yaml-view) | Planned |
| [R-MOD-025 Import an asset with its dependencies](#r-mod-025-import-an-asset-with-its-dependencies) | Planned |
| [R-MOD-026 Where an asset is used](#r-mod-026-where-an-asset-is-used) | Planned |
| [R-MOD-027 External editor and live reload](#r-mod-027-external-editor-and-live-reload) | Planned |
| [R-MOD-028 Validate on save, play in the testbed](#r-mod-028-validate-on-save-play-in-the-testbed) | Planned |
| [R-MOD-029 Provenance of imported assets](#r-mod-029-provenance-of-imported-assets) | Planned |
| [R-MOD-030 Package a pack](#r-mod-030-package-a-pack) | Planned |
| [R-MOD-031 Cutscene Designer workspace](#r-mod-031-cutscene-designer-workspace) | Planned |
| [R-MOD-032 Seeking with snapshots](#r-mod-032-seeking-with-snapshots) | Planned |
| [R-MOD-033 Cutscene files](#r-mod-033-cutscene-files) | Planned |
| [R-MOD-034 Deterministic playback with Simple physics](#r-mod-034-deterministic-playback-with-simple-physics) | Planned |
| [R-MOD-035 Scripted entities and keyframes](#r-mod-035-scripted-entities-and-keyframes) | Planned |
| [R-MOD-036 Paths and rotations](#r-mod-036-paths-and-rotations) | Planned |
| [R-MOD-037 Timed actions](#r-mod-037-timed-actions) | Planned |
| [R-MOD-038 Autopilot entities](#r-mod-038-autopilot-entities) | Planned |
| [R-MOD-039 Formations in cutscenes](#r-mod-039-formations-in-cutscenes) | Planned |
| [R-MOD-040 Timed sounds](#r-mod-040-timed-sounds) | Planned |
| [R-MOD-041 Captions](#r-mod-041-captions) | Planned |
| [R-MOD-042 Cameras, fixed or attached](#r-mod-042-cameras-fixed-or-attached) | Planned |
| [R-MOD-043 Camera cuts](#r-mod-043-camera-cuts) | Planned |
| [R-MOD-044 Where cutscenes play](#r-mod-044-where-cutscenes-play) | Planned |
| [R-MOD-045 Skipping a cutscene](#r-mod-045-skipping-a-cutscene) | Planned |
| [R-MOD-046 Video cutscenes](#r-mod-046-video-cutscenes) | Planned |
| [R-MOD-047 Shared formations](#r-mod-047-shared-formations) | Planned |
| [R-MOD-048 Drag instruments onto a 3D cockpit](#r-mod-048-drag-instruments-onto-a-3d-cockpit) | Planned |
| [R-MOD-049 Instrument size and preview on the surface](#r-mod-049-instrument-size-and-preview-on-the-surface) | Planned |
| [R-MOD-050 Create a 3D cockpit](#r-mod-050-create-a-3d-cockpit) | Planned |
| [R-MOD-051 Generic base cockpit](#r-mod-051-generic-base-cockpit) | Planned |
| [R-MOD-052 Example mod: X-COM 3D cockpit](#r-mod-052-example-mod-x-com-3d-cockpit) | Planned |
| [R-MOD-053 Single-file models](#r-mod-053-single-file-models) | Planned |
| [R-MOD-054 Blender sources](#r-mod-054-blender-sources) | Planned |
| [R-MOD-055 Materials keep their textures](#r-mod-055-materials-keep-their-textures) | Planned |

## Requirements

### R-MOD-001 Pack manifests

A pack manifest MUST declare its kind (`campaign`, `addon`, `library`), and MAY declare `parent`, `targets`, `format` and dependencies; bad manifests MUST be rejected with a message.

*Rationale:* Mod structure.

- Implements: [F-MOD-001](../../definition/modding-toolkit.md#f-mod-001-packs-and-mod-kinds)
- Status: implemented
- Verified by: `content::pack::parses_a_manifest`, `manifests_declare_kind_parent_targets_and_format`, `rejects_bad_manifests`, `discovers_pack_folders`
- Reverse-engineered: yes
- See: [Mod structure](../design/mod-structure.md), [Packs](../../modding/packs.md)

### R-MOD-002 Load order

Packs MUST load dependencies first and otherwise keep their order; missing, duplicate and cyclic dependencies MUST be reported.

*Rationale:* Predictable layering.

- Implements: [F-MOD-001](../../definition/modding-toolkit.md#f-mod-001-packs-and-mod-kinds)
- Status: implemented
- Verified by: `content::pack::orders_dependencies_first_and_keeps_order_otherwise`, `detects_missing_duplicate_and_cyclic_dependencies`
- Reverse-engineered: yes
- See: [Packs](../../modding/packs.md)

### R-MOD-003 Rules layering

Definitions MUST be able to inherit original INI sections (`inherit: original:…`); later packs MUST merge into, delete and edit (`key+`, `key-`) earlier definitions.

*Rationale:* Addons change campaigns without copying them.

- Implements: [F-MOD-001](../../definition/modding-toolkit.md#f-mod-001-packs-and-mod-kinds)
- Status: implemented
- Verified by: `content::rules::builds_typed_definitions_with_inheritance`, `later_packs_merge_delete_and_edit_lists`, `spelling_of_original_keys`
- Reverse-engineered: yes
- See: [Rulesets and layering](../../modding/rulesets.md)

### R-MOD-004 Gameplay values are data

Gameplay values, ship, weapon and object definitions, factions, scenarios and missions MUST live in pack data, not in engine code.

*Rationale:* Modding first (project rule).

- Implements: [F-MOD-001](../../definition/modding-toolkit.md#f-mod-001-packs-and-mod-kinds)
- Status: implemented
- Verified by: Code review against the project rules.
- Reverse-engineered: yes
- See: [Architectural design](../../learning/best-practices/architecture.md)

### R-MOD-005 YAML subset

Mod files MUST be a strict YAML subset; tabs, anchors, aliases, tags, multiple documents, complex keys and multi-line plain scalars MUST be rejected with their line number.

*Rationale:* One unambiguous reading, read in-crate (DEV-030).

- Implements: [F-MOD-002](../../definition/modding-toolkit.md#f-mod-002-readable-rule-files-that-explain-mistakes)
- Status: implemented
- Verified by: `formats::yaml::mappings_sequences_and_scalars`, `block_scalars_and_wrapped_flows`, `unsupported_and_broken_input_is_an_error_with_a_line`, `keys_with_colons_and_urls_stay_whole`, `writing_round_trips`, `empty_and_scalar_documents`
- Reverse-engineered: yes
- See: [Mod file reference](../../modding/reference.md)

### R-MOD-006 Schemas and messages

Every mod file type MUST be described by a schema; mistakes MUST be reported with file, line and key path, unknown keys with a did-you-mean suggestion; quantities MUST accept units (`2.4km`, `1.5s`, `12deg`, `40%`); `--write-schemas` MUST export JSON Schemas; loading MUST report every error and keep going.

*Rationale:* A modder never has to guess (plan item 2.19).

- Implements: [F-MOD-002](../../definition/modding-toolkit.md#f-mod-002-readable-rule-files-that-explain-mistakes)
- Status: implemented
- Verified by: `content::schema::good_files_pass`, `mistakes_are_reported_with_line_and_path`, `quantities_and_suggestions`, `json_schema_is_wellformed_enough`; `content::files::every_schema_builds_and_exports`; `content::rules::reports_every_error_and_keeps_going`; `content::scenario::errors_name_the_line_and_path`; MT-2-18
- Reverse-engineered: yes
- See: [Mod file reference](../../modding/reference.md)

### R-MOD-007 Settings and bindings migration

`settings.ini` and `bindings.ini` MUST be migrated once to `settings.yaml` and `bindings.yaml`.

*Rationale:* No lost settings across format changes.

- Implements: [F-MOD-002](../../definition/modding-toolkit.md#f-mod-002-readable-rule-files-that-explain-mistakes)
- Status: implemented
- Verified by: `settings::parses_and_round_trips`; `input::bindings_file_overrides_and_round_trips`
- Reverse-engineered: yes
- See: [Settings and data discovery](../../implementation/architecture/settings.md)

### R-MOD-008 Per-asset overrides

Any original model, texture, image or sound MUST be replaceable by a pack file at its override path; disabling the pack MUST restore the original.

*Rationale:* HD and 3D replacements (DEV-022).

- Implements: [F-MOD-003](../../definition/modding-toolkit.md#f-mod-003-replace-any-asset-in-open-formats)
- Status: implemented
- Verified by: `content::pack_files_override_original_assets`, `content::vfs::override_paths_follow_the_convention`; MT-1-25, MT-1-28, MT-1-29
- Reverse-engineered: yes
- See: [Overrides and extraction](../../modding/overrides.md)

### R-MOD-009 Extraction

`--extract` MUST write every original asset as OBJ, PNG or WAV at its override path, with keyed cockpit pixels as alpha, and OBJ exports MUST load back through the mod loader.

*Rationale:* Editable starting points.

- Implements: [F-MOD-003](../../definition/modding-toolkit.md#f-mod-003-replace-any-asset-in-open-formats)
- Status: implemented
- Verified by: `extract::extraction_writes_override_paths`, `obj_export_round_trips_through_the_mod_loader`, `png_encoding_decodes_back`; MT-1-27
- Reverse-engineered: yes
- See: [Extractor](../../implementation/architecture/extract.md)

### R-MOD-010 Open model and image formats

Mods MUST be able to supply Wavefront OBJ models with MTL materials (`slot_*` groups as attachment points) and PNG or TGA textures.

*Rationale:* Universal export targets.

- Implements: [F-MOD-003](../../definition/modding-toolkit.md#f-mod-003-replace-any-asset-in-open-formats)
- Status: implemented
- Verified by: `formats::obj::parses_positions_uvs_normals_and_faces`, `parses_materials`, `rejects_bad_indices_and_short_faces`; `model::obj_models_use_materials_scale_and_flip_v`; `content::png_and_tga_pack_images_decode`
- Reverse-engineered: yes
- See: [Models and textures](../../modding/models.md)

### R-MOD-011 Paths stay inside the pack

Asset references MUST be normalised and MUST NOT escape their pack.

*Rationale:* A mod cannot read arbitrary files.

- Implements: [F-MOD-001](../../definition/modding-toolkit.md#f-mod-001-packs-and-mod-kinds)
- Status: implemented
- Verified by: `content::vfs::parses_asset_references`, `refuses_paths_that_escape_the_pack`
- Reverse-engineered: yes
- See: [Packs, VFS and rules](../../implementation/architecture/packs.md)

### R-MOD-012 Text keys

Text MUST be referable by `@key` from `text/<language>/` files; original language files MUST be importable by reference; the language MUST be chosen by `--language` or `game.language`.

*Rationale:* Translation by addon.

- Implements: [F-MOD-004](../../definition/modding-toolkit.md#f-mod-004-text-and-translation)
- Status: implemented
- Verified by: `content::text::keys_resolve_plain_values_pass_through_and_imports_are_listed`; `formats::langfile::keys_text_and_line_breaks`; MT-2-34
- Reverse-engineered: yes
- See: [Text and translation](../../modding/text.md)

### R-MOD-013 Condition language

Conditions MUST be parsed and type-checked at load (unknown names, argument counts and types reported) and evaluated with precedence and short-circuit, with units, in the object and battle contexts.

*Rationale:* Data that depends on the situation.

- Implements: [F-MOD-005](../../definition/modding-toolkit.md#f-mod-005-conditions-actions-and-scripting)
- Status: implemented
- Verified by: `script::units_and_tokens`, `evaluates_with_precedence_and_short_circuits`, `errors_are_found_when_loading`; `script::object::ids_match_the_table_and_system_names_match_the_kinds`; `script::battle::ids_match_and_roles_are_checked_names`
- Reverse-engineered: yes
- See: [Format: behaviours and scripting](../design/format-behaviour.md), [Conditions (script)](../../implementation/architecture/script.md)

### R-MOD-014 Campaign context

Conditions MUST gain a campaign context with system and facility queries (`deposit`, `feature`, `research_site`, `survey_known`, `owned`, `lane_distance`, `facility_progress`, `construction_attacks`).

*Rationale:* Strategy-layer rules and logistics.

- Implements: [F-MOD-005](../../definition/modding-toolkit.md#f-mod-005-conditions-actions-and-scripting), [F-STR-005](../../definition/strategy-logistics.md#f-str-005-supply-chain-logistics)
- Status: planned
- Verified by: Planned.
- See: [Logistics](../design/logistics.md)

### R-MOD-015 Tier 2 scripting decision

Before a second-tier script language is added, its dependency evaluation MUST be recorded on the dependency page; hooks and a sandboxed API follow.

*Rationale:* Dependency policy.

- Implements: [F-MOD-005](../../definition/modding-toolkit.md#f-mod-005-conditions-actions-and-scripting)
- Status: planned
- Verified by: Planned.
- See: [Scripting (design)](../../modding/scripting.md)

### R-MOD-016 Headless validation

`--check` MUST validate the active campaign headlessly and report every error; the default campaign MUST pass with no errors and the full reference CD with no errors.

*Rationale:* Mistakes found before playing.

- Implements: [F-MOD-006](../../definition/modding-toolkit.md#f-mod-006-mod-validation)
- Status: implemented
- Verified by: `check::the_default_campaign_passes`, `broken_definitions_are_errors`
- Reverse-engineered: yes
- See: [Validating a mod](../../modding/validating.md)

### R-MOD-017 Full validator

The validator MUST check every definition, reference, condition and text key, per addon, and write a JSON report; it MUST catch every seeded error in a test mod.

*Rationale:* Stage 7 exit.

- Implements: [F-MOD-006](../../definition/modding-toolkit.md#f-mod-006-mod-validation)
- Status: planned
- Verified by: Planned.
- See: [Stage 7](../../implementation/plan/stage-7-modding.md)

### R-MOD-018 Recipes

Generated assets MUST be built by recipes (`upscale`, `mesh-patch`, `recolour`, `composite`) into a cache keyed by recipe, version and input hash, with outputs bit-identical across runs and machines.

*Rationale:* Remastered content without redistributing originals.

- Implements: [F-MOD-007](../../definition/modding-toolkit.md#f-mod-007-generated-assets-recipes)
- Status: planned
- Verified by: Planned.
- See: [Mod structure](../design/mod-structure.md)

### R-MOD-019 Sound cue table

Sounds MUST be referable by name and through the original cue table (`audio.cue_table`, `cue:<Name>`).

*Rationale:* Original sounds by their cue names.

- Implements: [F-MOD-001](../../definition/modding-toolkit.md#f-mod-001-packs-and-mod-kinds), [F-CMP-002](../../definition/compatibility.md#f-cmp-002-original-formats-read-faithfully)
- Status: implemented
- Verified by: `content::rules::sound_cues_resolve_through_the_original_table`; `audio::events_choose_sounds_and_volumes`; MT-1-18, MT-2-26
- Reverse-engineered: yes
- See: [Rulesets and layering](../../modding/rulesets.md)

### R-MOD-020 Every key documented

Every new rule key, scenario key or pack feature MUST be documented on the Modding pages in the same change.

*Rationale:* Modding first (project rule).

- Implements: [F-MOD-008](../../definition/modding-toolkit.md#f-mod-008-modding-guide)
- Status: implemented
- Verified by: Review against the project rules.
- Reverse-engineered: yes
- See: [Mod file reference](../../modding/reference.md)

### R-MOD-021 Designer application

The Mod Designer MUST be a separate program in the same repository, built as its own artifact beside the game, that uses the engine's own library (content, simulation, rendering and audio), so that every preview it shows is drawn by the game's renderer and simulated by the game's sim. The game and the designer MUST be able to run at the same time. Its interface MUST use macroquad's built-in UI module, with the engine renderer drawing the 3D map and preview viewports, and MUST NOT add a dependency.

*Rationale:* One engine: what the designer shows is what the game plays, and the designer cannot drift from it. A separate program keeps its interface focused (decisions D1 and D5, 2026-11-11).

- Implements: [F-MOD-009](../../definition/modding-toolkit.md#f-mod-009-mod-designer), [F-MOD-010](../../definition/modding-toolkit.md#f-mod-010-cutscene-designer)
- Status: planned
- Verified by: Planned (MD4).
- See: [Mod designer: how the designer is built](../design/designer.md#4-how-the-designer-is-built)

!!! note "Decided (2026-11-11)"
    D1: macroquad's built-in UI module plus the engine renderer; no new dependency. D5: a separate program in the same repository; the build produces two artifacts, the game and the designer.

### R-MOD-022 Files stay the source of truth

Everything the designer saves MUST be the documented open pack data (YAML, OBJ/MTL, PNG, WAV) that a modder could write by hand. Hand-written packs MUST open in the designer. Saving MUST rewrite a hand-written file only in the keys that changed, and SHOULD keep its comments. Edits MUST be undoable.

*Rationale:* Nothing the designer makes is locked to the tool; files can be read, diffed and hand-edited.

- Implements: [F-MOD-009](../../definition/modding-toolkit.md#f-mod-009-mod-designer)
- Status: planned
- Verified by: Planned (MD4: YAML round-trip unit tests).
- See: [Mod designer: goals and principles](../design/designer.md#1-goals-and-principles)

### R-MOD-023 New pack from a base

Creating a pack or mod MUST ask for an id, a name, a kind (campaign, addon or **extension**) and a base: the default campaign (`resources/default`), `interceptor` or any installed pack. An extension MUST declare `parent:` set to the base, and adds to or overrides the base's assets. The wizard MUST write `pack.yaml` with `parent:` set to the base, and the folder skeleton `rules/`, `models/`, `textures/`, `sounds/`, `text/en/`, `missions/`, `scenarios/` and `cutscenes/`.

*Rationale:* The user's request: "on creation of a new pack or mod, user may choose a base to use for assets that they are going to override". The name, from the user, 2026-11-11: "Let's call it an extension as opposed to an override, since another pack is the base and we are either adding to or overriding assets."

- Implements: [F-MOD-009](../../definition/modding-toolkit.md#f-mod-009-mod-designer)
- Status: planned
- Verified by: Planned (MD4).
- See: [Mod designer: packs and mods](../design/designer.md#21-packs-and-mods), [Mod structure](../design/mod-structure.md), [Packs](../../modding/packs.md#packs-and-mods)

!!! note "Resolved (2026-11-11): extension, not override"
    The wizard's kind is called **extension**: another pack is the base, declared with `parent:`, and the extension adds to it or overrides its assets.

!!! note "Open"
    `extension` is not a manifest kind today (R-MOD-001: `campaign`, `addon`, `library`; `parent:` is a campaign's key and an addon uses `targets:`). Whether `extension` becomes a manifest kind of its own or maps onto `addon` is decided with MD4.

### R-MOD-024 Pack tree, forms and YAML view

The designer MUST show an open pack as a tree with its definitions grouped by kind (ships, weapons, missions, cutscenes, text and so on). It MUST offer a form editor generated from the engine's schemas — the ones that produce the JSON Schemas — so that a new key appears without a designer change, and a raw YAML view.

*Rationale:* Editing without hand-writing files, and without a second description of the format to keep in step.

- Implements: [F-MOD-009](../../definition/modding-toolkit.md#f-mod-009-mod-designer)
- Status: planned
- Verified by: Planned (MD4).
- See: [Mod designer: packs and mods](../design/designer.md#21-packs-and-mods), [R-MOD-006](#r-mod-006-schemas-and-messages)

### R-MOD-025 Import an asset with its dependencies

The designer MUST import any asset the parent provides (model, texture, sound, image, cockpit, ship or object, mission or scenario) by copying it **and its dependencies** into the mod at the path that overrides the original, as listed on the design page: a model with its MTL, every texture the MTL names and its LODs; a ship or object with its definition, model set, debris and destroyed effect, materials and textures, sounds and text keys; a cockpit with its image or 3D model, instrument layout and textures; a mission or scenario with its text keys (its ships are offered as a separate import). Original formats MUST be converted to OBJ/MTL, PNG and WAV by the existing extractor.

*Rationale:* The user's request: importing "duplicates the original asset and its dependencies (model plus textures in most cases)".

- Implements: [F-MOD-009](../../definition/modding-toolkit.md#f-mod-009-mod-designer)
- Status: planned
- Verified by: Planned (MD5: dependency-graph import unit tests).
- See: [Mod designer: an asset and its dependencies](../design/designer.md#22-what-an-asset-and-its-dependencies-means), [R-MOD-008](#r-mod-008-per-asset-overrides), [R-MOD-009](#r-mod-009-extraction)

### R-MOD-026 Where an asset is used

The designer MUST build a dependency graph from the loaded content, MUST answer "where is this used?" for any definition or asset, and MUST warn before deleting anything still referenced.

*Rationale:* The same graph that drives import keeps a mod consistent.

- Implements: [F-MOD-009](../../definition/modding-toolkit.md#f-mod-009-mod-designer)
- Status: planned
- Verified by: Planned (MD5).
- See: [Mod designer: an asset and its dependencies](../design/designer.md#22-what-an-asset-and-its-dependencies-means)

### R-MOD-027 External editor and live reload

The designer MUST open an imported file in an external program chosen per file type (Blender, for example), and MUST reload the asset in its preview when the file is saved.

*Rationale:* The user's request: the copy is edited "in an external program, usually blender".

- Implements: [F-MOD-009](../../definition/modding-toolkit.md#f-mod-009-mod-designer)
- Status: planned
- Verified by: Planned (MD5; manual test).
- See: [Mod designer: packs and mods](../design/designer.md#21-packs-and-mods)

### R-MOD-028 Validate on save, play in the testbed

The designer MUST run the engine's validator on every save and report each problem with its file, line and key, and MUST start the testbed with the mod enabled on a chosen scenario or mission.

*Rationale:* Mistakes found at once, and the mod tried in the real game.

- Implements: [F-MOD-009](../../definition/modding-toolkit.md#f-mod-009-mod-designer), [F-MOD-006](../../definition/modding-toolkit.md#f-mod-006-mod-validation)
- Status: planned
- Verified by: Planned (MD4).
- See: [Mod designer: packs and mods](../design/designer.md#21-packs-and-mods), [R-MOD-016](#r-mod-016-headless-validation)

### R-MOD-029 Provenance of imported assets

Every file the designer converts from original data MUST be recorded as such in the pack's `pack.yaml`, and stays in the user's own mod folder for personal use only.

*Rationale:* Original content stays private: the bundler must be able to refuse it (R-PLT-009).

- Implements: [F-MOD-009](../../definition/modding-toolkit.md#f-mod-009-mod-designer), [F-PLT-005](../../definition/platforms.md#f-plt-005-game-bundling)
- Status: planned
- Verified by: Planned (MD5).
- See: [Mod designer: bundling](../design/designer.md#5-bundling-a-complete-game)

### R-MOD-030 Package a pack

The designer MUST export a pack as a folder or a zip for sharing, and MUST be able to add it to a product bundle (R-PLT-007).

*Rationale:* Sharing a mod, or releasing it as part of a game.

- Implements: [F-MOD-009](../../definition/modding-toolkit.md#f-mod-009-mod-designer)
- Status: planned
- Verified by: Planned (MD4, MD7).
- See: [Mod designer: packs and mods](../design/designer.md#21-packs-and-mods)

### R-MOD-031 Cutscene Designer workspace

The Cutscene Designer MUST show the scene's outline (entities, cameras, sounds, captions), a 3D map of the whole scene with paths, keyframes and camera frusta, a preview through the active camera at the current frame, a timeline with tracks, keyframes and a playhead, and an inspector for the selection. In it the user MUST be able to add any asset from the loaded packs to the scene, set the times it enters and leaves, move it in 3D and set keyframes.

*Rationale:* The user's request: "User needs to be able to see the overall 3d map and preview of the current animation frame."

- Implements: [F-MOD-010](../../definition/modding-toolkit.md#f-mod-010-cutscene-designer)
- Status: planned
- Verified by: Planned (MD6; manual tests).
- See: [Mod designer: layout](../design/designer.md#31-layout)

### R-MOD-032 Seeking with snapshots

While a cutscene plays in the designer, a snapshot of the whole world MUST be taken every 2 s; seeking MUST replay the simulation from the nearest snapshot, and playing on from a seek MUST give exactly the same frames as playing straight through.

*Rationale:* Scrubbing that is quick and exact.

- Implements: [F-MOD-010](../../definition/modding-toolkit.md#f-mod-010-cutscene-designer)
- Status: planned
- Verified by: Planned (MD6: "seek, then play, equals straight play" unit test).
- See: [Mod designer: how it plays](../design/designer.md#33-how-it-plays-deterministic-with-simple-physics)

### R-MOD-033 Cutscene files

A cutscene MUST be a pack file `cutscenes/<id>.yaml` with an id, a title (a text key), a length, an environment (starfield seed, sun, backdrop), entities, cameras, camera cuts, sounds and captions, or a video (R-MOD-046). Cutscene files MUST be validated with every other pack file, each mistake reported with its file, line and key.

*Rationale:* Cutscenes are open pack data like everything else a modder writes.

- Implements: [F-MOD-010](../../definition/modding-toolkit.md#f-mod-010-cutscene-designer), [F-MOD-011](../../definition/modding-toolkit.md#f-mod-011-cutscenes-in-the-game)
- Status: planned
- Verified by: Planned (MD2).
- See: [Mod designer: the cutscene file](../design/designer.md#34-the-cutscene-file)

### R-MOD-034 Deterministic playback with Simple physics

A cutscene MUST run on the normal 60 Hz simulation with a fixed random seed, and every ship in it MUST fly Simple physics (R-FLT-013), so that it plays the same way every time, in the designer and in the game.

*Rationale:* The user's request: "Cutscenes always use basic physics so that the animation behavior is predictable."

- Implements: [F-MOD-010](../../definition/modding-toolkit.md#f-mod-010-cutscene-designer), [F-MOD-011](../../definition/modding-toolkit.md#f-mod-011-cutscenes-in-the-game)
- Status: planned
- Verified by: Planned (MD2).
- See: [Mod designer: how it plays](../design/designer.md#33-how-it-plays-deterministic-with-simple-physics), [R-FLT-013](flight.md#r-flt-013-simple-physics-mode)

### R-MOD-035 Scripted entities and keyframes

A scripted entity MUST be any ship, object or debris from the loaded packs, with enter and leave times and keyframes for position, rotation and speed. When an entity has position keyframes, its speed MUST be derived from its path, overriding its speed keyframes, so that it is exactly at each keyframe's position at that keyframe's time.

*Rationale:* The user's request: "speed overridden when keyframe positions are set to guarantee that the positioning is correct".

- Implements: [F-MOD-010](../../definition/modding-toolkit.md#f-mod-010-cutscene-designer), [F-MOD-011](../../definition/modding-toolkit.md#f-mod-011-cutscenes-in-the-game)
- Status: planned
- Verified by: Planned (MD2: keyframe evaluation and speed-override unit tests).
- See: [Mod designer: what a scene contains](../design/designer.md#32-what-a-scene-contains)

### R-MOD-036 Paths and rotations

Position keyframes MUST be joined by a centripetal Catmull-Rom spline that passes through every key; rotation keyframes MUST be interpolated by quaternion slerp. An entity with position keys and no rotation keys MUST face along its path and bank into turns.

*Rationale:* Smooth paths without overshoot at sharp corners.

- Implements: [F-MOD-010](../../definition/modding-toolkit.md#f-mod-010-cutscene-designer), [F-MOD-011](../../definition/modding-toolkit.md#f-mod-011-cutscenes-in-the-game)
- Status: planned
- Verified by: Planned (MD2: spline and slerp sampling unit tests).
- See: [Mod designer: how it plays](../design/designer.md#33-how-it-plays-deterministic-with-simple-physics)

### R-MOD-037 Timed actions

A scripted entity MUST be able to act at given times: fire beams or a missile at a target, start chasing a target, capture a target with the tractor, be destroyed (with its looks and debris) and jump to hyperspace. Each action MUST call the same simulation functions as a battle, so the same effects and sounds play.

A `harmless` flag, on an entity (scripted or autopilot) or on a single action, MUST make that entity's or action's weapons fire and show their effects but deal no damage.

*Rationale:* The user's request: "assign targets for them to chase, capture, or shoot at" and "time triggered destruction"; with decision D4: "Also a flag for the animations for their weapons to cause no damage."

- Implements: [F-MOD-010](../../definition/modding-toolkit.md#f-mod-010-cutscene-designer), [F-MOD-011](../../definition/modding-toolkit.md#f-mod-011-cutscenes-in-the-game)
- Status: planned
- Verified by: Planned (MD2: action timing unit tests).
- See: [Mod designer: what a scene contains](../design/designer.md#32-what-a-scene-contains)

### R-MOD-038 Autopilot entities

An autopilot entity MUST be flown by the engine's AI with the campaign's beginner difficulty rules, towards designated targets (attack, escort, flee to a point). An optional per-entity `aggressive` flag MUST be available for background dogfights. An entity MUST be able to switch from autopilot to scripted at a given time, and back.

*Rationale:* The user's request: "autopilot will run based on beginner AI rules given designated targets and usually serve as background action".

- Implements: [F-MOD-010](../../definition/modding-toolkit.md#f-mod-010-cutscene-designer), [F-MOD-011](../../definition/modding-toolkit.md#f-mod-011-cutscenes-in-the-game)
- Status: planned
- Verified by: Planned (MD2).
- See: [Mod designer: what a scene contains](../design/designer.md#32-what-a-scene-contains)

!!! note "Decided (2026-11-11)"
    D4: beginner AI rules, with an optional per-entity `aggressive` flag; the `harmless` flag is R-MOD-037. What `aggressive` changes in the AI is fixed when MD2 is built.

### R-MOD-039 Formations in cutscenes

A group of fighters or ships in a cutscene MUST be placeable by a named formation from the pack's `formations` (R-MOD-047), following a leader that is scripted or autopilot.

*Rationale:* The user's request: "For fighters and ships, they can choose which formation to apply."

- Implements: [F-MOD-010](../../definition/modding-toolkit.md#f-mod-010-cutscene-designer), [F-MOD-013](../../definition/modding-toolkit.md#f-mod-013-shared-formations)
- Status: planned
- Verified by: Planned (MD2).
- See: [Mod designer: what a scene contains](../design/designer.md#32-what-a-scene-contains)

### R-MOD-040 Timed sounds

A cutscene MUST play sound clips (narration, music, effects) at given times, each with a volume and a fade in and out, either positioned on an entity or flat; music MUST be able to duck under narration.

*Rationale:* The user's request: "Sounds must be included at timed positions so that narration, music, and other audio can be played."

- Implements: [F-MOD-010](../../definition/modding-toolkit.md#f-mod-010-cutscene-designer), [F-MOD-011](../../definition/modding-toolkit.md#f-mod-011-cutscenes-in-the-game)
- Status: planned
- Verified by: Planned (MD2).
- See: [Mod designer: what a scene contains](../design/designer.md#32-what-a-scene-contains)

### R-MOD-041 Captions

A cutscene MUST be able to show caption text, given as text keys so that it can be translated, over given time ranges.

*Rationale:* Accessible narration (decision D3, 2026-11-11: yes).

- Implements: [F-MOD-010](../../definition/modding-toolkit.md#f-mod-010-cutscene-designer), [F-MOD-011](../../definition/modding-toolkit.md#f-mod-011-cutscenes-in-the-game)
- Status: planned
- Verified by: Planned (MD2).
- See: [Mod designer: what a scene contains](../design/designer.md#32-what-a-scene-contains), [R-MOD-012](#r-mod-012-text-keys)

### R-MOD-042 Cameras, fixed or attached

A cutscene MUST support **fixed** cameras, with keyframed world position, a keyframed look target or rotation and a field of view, and **attached** cameras, parented to an entity, whose coordinates are relative to the parent and whose distance, orbit (yaw and pitch around the parent), height and look offset are keyframed.

*Rationale:* The user's request: "Cameras can be defined at explicit positions or attached to an object … positioning is keyframed for distance and orbit."

- Implements: [F-MOD-010](../../definition/modding-toolkit.md#f-mod-010-cutscene-designer), [F-MOD-011](../../definition/modding-toolkit.md#f-mod-011-cutscenes-in-the-game)
- Status: planned
- Verified by: Planned (MD2).
- See: [Mod designer: what a scene contains](../design/designer.md#32-what-a-scene-contains)

### R-MOD-043 Camera cuts

The active camera MUST be chosen by a track of keyframes, each cut instant or a short blend.

*Rationale:* The user's request: "Switching between cameras in the cutscene is set through keyframes."

- Implements: [F-MOD-010](../../definition/modding-toolkit.md#f-mod-010-cutscene-designer), [F-MOD-011](../../definition/modding-toolkit.md#f-mod-011-cutscenes-in-the-game)
- Status: planned
- Verified by: Planned (MD2).
- See: [Mod designer: what a scene contains](../design/designer.md#32-what-a-scene-contains)

### R-MOD-044 Where cutscenes play

The game MUST play a cutscene as a campaign's introduction, from a campaign event's action, and at a mission's start (intro) and end (outro, chosen by the result, such as win or lose).

*Rationale:* The user's request: animations "that play for key campaign events or at the start of specific mission definitions".

- Implements: [F-MOD-011](../../definition/modding-toolkit.md#f-mod-011-cutscenes-in-the-game), [F-MOD-012](../../definition/modding-toolkit.md#f-mod-012-video-cutscenes)
- Status: planned
- Verified by: Planned (MD2).
- See: [Mod designer: the cutscene file](../design/designer.md#34-the-cutscene-file), [Format: campaign and game flow](../design/format-campaign.md), [Format: missions, environments, events](../design/format-missions.md)

### R-MOD-045 Skipping a cutscene

The player MUST always be able to skip a cutscene, engine-played or video, with Esc, the tablet console's skip button or the gamepad's Start button.

*Rationale:* A film never holds the player.

- Implements: [F-MOD-011](../../definition/modding-toolkit.md#f-mod-011-cutscenes-in-the-game), [F-MOD-012](../../definition/modding-toolkit.md#f-mod-012-video-cutscenes)
- Status: planned
- Verified by: Planned (MD2).
- See: [Mod designer: the cutscene file](../design/designer.md#34-the-cutscene-file)

### R-MOD-046 Video cutscenes

A pack MUST be able to provide a video file in place of any engine-played cutscene (`video:`), with optional captions. Video files MUST play through the operating system's media framework behind the platform shim: AVFoundation on macOS, Media Foundation on Windows, and on Linux GStreamer, when installed, loaded at run time with `dlopen` (no crate dependency and no link-time dependency; GStreamer MUST NOT be bundled). Image sequences with a WAV track MUST play on every platform, and MUST be the fallback where no media framework is available. An in-crate MP4/H.264 decoder is out of scope.

*Rationale:* The user's request: "a user may also provide a video to play instead of scripted engine animation"; decision D2 (2026-11-11). GStreamer is LGPL: loading the system's own copy dynamically, without bundling it, keeps the project clear of it.

- Implements: [F-MOD-012](../../definition/modding-toolkit.md#f-mod-012-video-cutscenes)
- Status: planned
- Verified by: Planned (MD3).
- See: [Mod designer: video instead of animation](../design/designer.md#35-video-instead-of-animation), [Format: interface, text and media](../design/format-interface.md#video), [R-CPN-012](campaigns.md#r-cpn-012-video-and-music)

!!! note "Open"
    The Linux path (GStreamer through `dlopen`) MUST be evaluated on the [dependency page](../../learning/best-practices/dependencies.md) before it is built (MD3).

### R-MOD-047 Shared formations

Formations MUST be pack data (`formations:`): named shapes such as vee, line abreast, echelon, box and diamond, and custom ones. Wingman flying and cutscenes MUST place craft from the same formations, and a formation MUST replace today's per-scenario slot offsets.

*Rationale:* A cutscene and a battle use the same shapes, and a modder defines them once.

- Implements: [F-MOD-013](../../definition/modding-toolkit.md#f-mod-013-shared-formations), [F-CMB-007](../../definition/combat.md#f-cmb-007-wingmen-and-orders)
- Status: planned
- Verified by: Planned (MD1).
- See: [Mod designer: what a scene contains](../design/designer.md#32-what-a-scene-contains), [R-CMB-015](combat.md#r-cmb-015-wingman-orders), [R-CMB-022](combat.md#r-cmb-022-wingmen-keep-their-distance)

### R-MOD-048 Drag instruments onto a 3D cockpit

For a cockpit with a 3D model, the designer MUST let the modder drag each instrument (radar, MFD, gauges, status, message, power, systems and any other the cockpit has) around the screen over the model. While an instrument is dragged, a ray from the eye through the cursor MUST be intersected with the cockpit model's triangles, and the instrument MUST be previewed applied to the face it is over: an anchor `{at, normal, size}` whose `at` is the hit point and whose `normal` is the hit face's normal. Dropping it MUST keep that anchor. An instrument that has only a 2D rectangle MUST be able to gain an anchor this way. Over no face, the instrument MUST keep its last anchor.

*Rationale:* The user, 2026-11-11: "for xcom cockpits, we will be designing a 3d cockpit in blender to include as an example mod that substitutes the 3d to replace the 2d image. This raises a point for the mod design tools, they need to accommodate placement of the UI elements so that they are projected onto the correct faces in a 3d view. Simple handling will be to allow each element to be dragged around the screen to detemine their positions and then previewed by applying to the geometry that it is over."

- Implements: [F-MOD-014](../../definition/modding-toolkit.md#f-mod-014-3d-cockpit-instrument-placement)
- Status: planned
- Verified by: Planned (MD8: unit tests of cursor ray to anchor on synthetic geometry; manual test of the drag and preview).
- See: [Mod designer: instruments on a 3D cockpit](../design/designer.md#23-instruments-on-a-3d-cockpit), [Cockpits (HD and 3D)](../../modding/cockpits.md#a-3d-cockpit-instead-of-the-overlay), [R-HUD-004](cockpit-hud.md#r-hud-004-anchored-3d-cockpit-instruments), [R-HUD-008](cockpit-hud.md#r-hud-008-cockpit-layout-editor)

!!! note "Reuse"
    The engine already has the pieces: `hud::layout::cursor_ray`, `hud::layout::ray_triangle` and `hud::layout::anchor_corners`; the `cockpits.<id>.anchors` key; and the game's F12 layout editor (`app/flight.rs`, `edit_layout`), which drags 2D rectangles and places an existing anchor by clicking the dashboard. The designer reuses them. As a first step the F12 editor could gain the drag with live preview, and anchors for instruments that have none yet.

### R-MOD-049 Instrument size and preview on the surface

The width and height of an anchored instrument on the surface MUST be adjustable in the designer. The preview MUST draw each anchored instrument through the game's own anchored-instrument path (drawn into a texture and mapped onto the corners from `hud::layout::anchor_corners`), so that it looks as it will in the game. The placements MUST be saved as `cockpits.<id>.anchors` in the mod's rules, in the documented format (R-MOD-022).

*Rationale:* The user, 2026-11-11: "previewed by applying to the geometry that it is over".

- Implements: [F-MOD-014](../../definition/modding-toolkit.md#f-mod-014-3d-cockpit-instrument-placement)
- Status: planned
- Verified by: Planned (MD8: saved anchors load back as valid rules; manual test).
- See: [Mod designer: instruments on a 3D cockpit](../design/designer.md#23-instruments-on-a-3d-cockpit), [R-HUD-004](cockpit-hud.md#r-hud-004-anchored-3d-cockpit-instruments)

### R-MOD-050 Create a 3D cockpit

In a mod that extends another pack, the designer MUST offer, for any cockpit of the parent:

1. **Extract cockpit images**: the modder selects which of the parent's cockpit images to extract, and they are copied into the mod, converted from original formats by the extractor with their provenance recorded (R-MOD-029; never bundled, R-PLT-009);
2. **Create 3D cockpit**: the parent's cockpit model, or, if the parent has none, the generic base cockpit of `resources/default` (R-MOD-051), is copied into the mod with its materials and textures, and the mod's cockpit entry is set to use it;
3. then editing the model in Blender (R-MOD-027) and placing the instruments on it (R-MOD-048).

*Rationale:* The user, 2026-11-11: "this will be the first test case for the mod design tools when available - a mod extending interceptor, where user selects the cockpit images to extract, they are copied into the mod, select to create a 3d cockpit (which will provide a generic base cockpit geometry defined in default if none is present in the parent). The user can then edit it in Blender."

- Implements: [F-MOD-014](../../definition/modding-toolkit.md#f-mod-014-3d-cockpit-instrument-placement), [F-MOD-009](../../definition/modding-toolkit.md#f-mod-009-mod-designer)
- Status: planned
- Verified by: Planned (MD9: the first test case for the mod tools; manual test).
- See: [Mod designer: first test case](../design/designer.md#24-first-test-case-an-x-com-3d-cockpit), [R-MOD-025](#r-mod-025-import-an-asset-with-its-dependencies)

### R-MOD-051 Generic base cockpit

`resources/default` MUST provide a generic base cockpit: an original-free OBJ/MTL model with its textures, generated by `tools/default_assets/`, modelled in metres around the eye at the origin (+Y up, looking along −Z) as the 3D cockpit path requires, with dashboard surfaces for the standard instruments and a default set of anchors.

*Rationale:* The user: "a generic base cockpit geometry defined in default if none is present in the parent".

- Implements: [F-MOD-014](../../definition/modding-toolkit.md#f-mod-014-3d-cockpit-instrument-placement)
- Status: planned
- Verified by: Planned (MD9: the model loads and every default anchor lies on a face).
- See: [Mod designer: first test case](../design/designer.md#24-first-test-case-an-x-com-3d-cockpit), [Cockpits (HD and 3D)](../../modding/cockpits.md#a-3d-cockpit-instead-of-the-overlay)

### R-MOD-052 Example mod: X-COM 3D cockpit

The bundled `packs/` MUST include an example mod that extends `interceptor` and replaces the 2D X-COM cockpit image with a 3D cockpit designed in Blender. Its geometry MUST be original-free; the X-COM cockpit textures MUST be referenced from the player's original data and never copied into the repository, and the mod MUST declare `requires_original: true`.

*Rationale:* The user, 2026-11-11: "for xcom cockpits, we will be designing a 3d cockpit in blender to include as an example mod that substitutes the 3d to replace the 2d image."

- Implements: [F-MOD-014](../../definition/modding-toolkit.md#f-mod-014-3d-cockpit-instrument-placement)
- Status: planned
- Verified by: Planned (MD9: manual test in the `interceptor` campaign).
- See: [Mod designer: first test case](../design/designer.md#24-first-test-case-an-x-com-3d-cockpit), [Overrides and extraction](../../modding/overrides.md)

!!! note "Open"
    An MTL names its textures by file path (`map_Kd`); how a model in the repository refers to a texture in the player's original data (for example an `original:` reference resolved by the rules or by the loader) is settled with MD9.

### R-MOD-053 Single-file models

Wavefront OBJ/MTL with separate texture files MUST remain a fully supported model format: it is the format the existing packs use (R-MOD-010). In addition, mods SHOULD be able to supply a model as one binary glTF file (`.glb`) holding its mesh, materials and textures, behind the same model interface as OBJ. A `.blend` file is a source only (R-MOD-054), never a format the engine loads. The `.glb` reader MUST be in-crate (the glTF JSON read with the crate's own parser, `remote::json`, plus its binary buffers) and MUST NOT add a dependency.

*Rationale:* The user, 2026-11-11: "If possible, it would be ideal to allow the .blend files to be included in addtition to standaole mtl and obj objects so that a user can package the object, material, and textures into a single file, but that is not strictly necessary as long as the correct relationshps between materials and their texture images is consistently maintained."

- Implements: [F-MOD-015](../../definition/modding-toolkit.md#f-mod-015-single-file-models-and-blender-sources)
- Status: planned
- Verified by: Planned (MD10: unit tests on synthetic `.glb` files built in the test).
- See: [Mod designer: single-file models](../design/designer.md#25-single-file-models-and-blender-sources), [R-MOD-010](#r-mod-010-open-model-and-image-formats)

!!! note "Confirmed by the user (2026-11-11)"
    The plan (OBJ/MTL kept, `.glb` as an additional option, `.blend` as a source exported through Blender) was confirmed: "ok that works. obj/mtl as in the existing packs of course would continue to be supported." `.glb` stays a SHOULD: in the user's first words it is "not strictly necessary as long as the correct relationshps between materials and their texture images is consistently maintained" (R-MOD-055).

### R-MOD-054 Blender sources

A mod MAY include `.blend` files as the source of its models. The engine MUST NOT read `.blend` files (their binary format changes between Blender versions). When Blender is installed, the designer SHOULD export a `.blend` to OBJ/MTL or `.glb` on save by running Blender headless (`blender -b file.blend --python-expr …`); without Blender, the exported files already in the pack are used as they are. Blender is an optional external tool, not a dependency.

*Rationale:* The user: "it would be ideal to allow the .blend files to be included in addtition to standaole mtl and obj objects so that a user can package the object, material, and textures into a single file".

- Implements: [F-MOD-015](../../definition/modding-toolkit.md#f-mod-015-single-file-models-and-blender-sources)
- Status: planned
- Verified by: Planned (MD10: manual test with Blender installed).
- See: [Mod designer: single-file models](../design/designer.md#25-single-file-models-and-blender-sources), [Dependency policy](../../learning/best-practices/dependencies.md#external-tools), [R-MOD-027](#r-mod-027-external-editor-and-live-reload)

### R-MOD-055 Materials keep their textures

Every import and export — the extractor, the designer's import, the Blender export and the `.glb` reader — MUST keep the relationship between each material and its texture images: an MTL names its textures with `map_Kd` paths relative to the `.mtl` file and inside the pack (R-MOD-011), and copying a model MUST copy and re-point its textures together. The designer MUST report a material whose texture is missing.

*Rationale:* The user: "as long as the correct relationshps between materials and their texture images is consistently maintained".

- Implements: [F-MOD-015](../../definition/modding-toolkit.md#f-mod-015-single-file-models-and-blender-sources), [F-MOD-003](../../definition/modding-toolkit.md#f-mod-003-replace-any-asset-in-open-formats)
- Status: planned
- Verified by: Planned (MD10, MD5: import and export round-trip unit tests on synthetic models).
- See: [Models and textures](../../modding/models.md), [R-MOD-025](#r-mod-025-import-an-asset-with-its-dependencies)

## Revision history

| Version | Date | Change |
|---|---|---|
| 1.0.0 | 2026-11-11 | Bootstrap: R-MOD-001 – R-MOD-020 (20 requirements; 16 reverse-engineered from the code) |
| 1.7.0 | 2026-11-11 | R-MOD-021 – R-MOD-047 (planned): the Mod Designer (R-MOD-021 – R-MOD-030), the Cutscene Designer and cutscene files, playback, entities, actions, sounds, captions, cameras and cuts, the `harmless` and `aggressive` flags (R-MOD-031 – R-MOD-043), where cutscenes play and skipping (R-MOD-044, R-MOD-045), video cutscenes (R-MOD-046), shared formations (R-MOD-047), from the user's request of 2026-11-11 and the decisions D1–D5 |
| 1.9.0 | 2026-11-11 | From the user's message of 2026-11-11: new R-MOD-048 – R-MOD-055 (planned): instruments dragged onto a 3D cockpit and previewed on the face below the cursor, their size and preview (MD8); Create 3D cockpit, the generic base cockpit in `resources/default` and the X-COM 3D cockpit example mod, the first test case for the mod tools (MD9); single-file `.glb` models as an additional option beside OBJ/MTL, which stays fully supported (confirmed by the user), `.blend` sources exported through Blender, materials keeping their textures (MD10). R-MOD-023: the wizard's kind is **extension** (declares `parent:`, adds to or overrides the base), Open note resolved; whether it is a manifest kind or maps onto `addon` stays open (MD4) |
