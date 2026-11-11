---
spec_version: 1.0.0
status: current
updated: 2026-11-11
---

# Modding toolkit

What a modder gets: packs, readable rule files, asset replacement, translation, conditions and the tools that check and generate content.

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

## Features

### F-MOD-001 Packs and mod kinds

**What.** Content comes in packs: campaigns, addons that change a campaign, and libraries shared between them; a campaign can extend another.

**Why.** Modding first.

- **Status:** implemented
- **Source:** Plan item 2.1.
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

**What.** Behaviours, missions and campaign rules use a small checked language of conditions and actions; a second-tier script language is to be decided.

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

## Revision history

| Version | Date | Change |
|---|---|---|
| 1.0.0 | 2026-11-11 | Bootstrap: F-MOD-001 – F-MOD-008 (8 features) |
