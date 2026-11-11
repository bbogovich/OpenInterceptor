---
spec_version: 1.0.0
status: current
updated: 2026-11-11
---

# Modding toolkit requirements

Requirements for packs, rule files, schemas, overrides, extraction, text, the condition language, validation and recipes. The normative structure is [Mod structure](../design/mod-structure.md); every key is in the [Mod file reference](../../modding/reference.md).

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

## Revision history

| Version | Date | Change |
|---|---|---|
| 1.0.0 | 2026-11-11 | Bootstrap: R-MOD-001 – R-MOD-020 (20 requirements; 16 reverse-engineered from the code) |
