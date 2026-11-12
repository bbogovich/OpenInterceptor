---
spec_version: 1.16.0
status: current
updated: 2026-11-11
---

# Compatibility (original data) requirements

Requirements for discovering and reading the player's original data. The normative format descriptions are the [Original data formats](../original-formats/index.md) pages.

Features: [Compatibility (original data)](../../definition/compatibility.md). How to read a requirement: [Specification](../index.md#reading-a-requirement).

| Requirement | Status |
|---|---|
| [R-CMP-001 Finding the player's data](#r-cmp-001-finding-the-players-data) | Implemented |
| [R-CMP-002 Original data by name in any layout](#r-cmp-002-original-data-by-name-in-any-layout) | Implemented |
| [R-CMP-003 PE resources](#r-cmp-003-pe-resources) | Implemented |
| [R-CMP-004 XMSH meshes](#r-cmp-004-xmsh-meshes) | Implemented |
| [R-CMP-005 XTXR textures](#r-cmp-005-xtxr-textures) | Implemented |
| [R-CMP-006 TGA images and colour key](#r-cmp-006-tga-images-and-colour-key) | Implemented |
| [R-CMP-007 Audio](#r-cmp-007-audio) | Implemented |
| [R-CMP-008 INI rules and language files](#r-cmp-008-ini-rules-and-language-files) | Implemented |
| [R-CMP-009 XSPT sprites](#r-cmp-009-xspt-sprites) | Planned |
| [R-CMP-010 Smacker video](#r-cmp-010-smacker-video) | Implemented |
| [R-CMP-011 Bad data never panics](#r-cmp-011-bad-data-never-panics) | Implemented |
| [R-CMP-012 No original content](#r-cmp-012-no-original-content) | Implemented |
| [R-CMP-013 Facts and deviations marked](#r-cmp-013-facts-and-deviations-marked) | Implemented |
| [R-CMP-014 Resources folder](#r-cmp-014-resources-folder) | Implemented |
| [R-CMP-015 Installed copy or CD copy](#r-cmp-015-installed-copy-or-cd-copy) | Implemented |

## Requirements

### R-CMP-001 Finding the player's data

The original data MUST be found in this order: command line, environment, settings, the user data folder, the development tree; explicit and existing locations MUST win.

*Rationale:* Works without `--data` (plan item 1.14).

- Implements: [F-CMP-001](../../definition/compatibility.md#f-cmp-001-play-from-the-players-own-cd)
- Status: implemented
- Verified by: `settings::original_data_is_found_in_order`; `cli::environment_supplies_data_and_flags_override_it`
- Reverse-engineered: yes
- See: [Settings and data discovery](../../implementation/architecture/settings.md)

### R-CMP-002 Original data by name in any layout

Original resources MUST be found by name whatever the layout of the player's copy.

*Rationale:* CD copies differ.

- Implements: [F-CMP-001](../../definition/compatibility.md#f-cmp-001-play-from-the-players-own-cd)
- Status: implemented
- Verified by: `content::vfs::original_data_is_found_by_name_in_any_layout`
- Reverse-engineered: yes
- See: [CD layout](../../reference/original-game/cd-layout.md)

### R-CMP-003 PE resources

Resources MUST be read from the original DLLs by type and name, as data; non-PE and truncated files MUST be errors.

*Rationale:* Original meshes and textures live in DLL resources.

- Implements: [F-CMP-002](../../definition/compatibility.md#f-cmp-002-original-formats-read-faithfully)
- Status: implemented
- Verified by: `formats::pe::finds_resources_by_type_and_name`, `name_ids_are_normalised`, `rejects_non_pe_data`, `truncated_files_are_errors`, `missing_file_is_an_error`
- Reverse-engineered: yes
- See: [PE resource container](../original-formats/pe-resources.md)

### R-CMP-004 XMSH meshes

XMSH meshes MUST be read with their LODs, signed edge loops, untextured polygons and polygon flags, mirrored into engine handedness with outward winding, triangulated and grouped; overlapping overlay faces MUST be layered by depth bias.

*Rationale:* All 297 meshes (verified, Stage 0); DEV-023.

- Implements: [F-CMP-002](../../definition/compatibility.md#f-cmp-002-original-formats-read-faithfully)
- Status: implemented
- Verified by: `formats::xmsh::parses_a_hand_built_mesh`, `polygon_flags_are_read`, `signed_edges_give_the_vertex_loop`, `rejects_bad_signature_and_bad_references`; `model::xmsh_is_triangulated_grouped_and_mirrored`, `mirrored_triangles_keep_outward_winding`, `later_overlapping_face_is_layered_on_top`, `overlay_flag_beats_file_order`; MT-1-31
- Reverse-engineered: yes
- See: [XMSH meshes](../original-formats/xmsh.md)

### R-CMP-005 XTXR textures

XTXR textures MUST be decoded through palette 0 with RGB555 expanded to full range.

*Rationale:* Original textures (DEV-005).

- Implements: [F-CMP-002](../../definition/compatibility.md#f-cmp-002-original-formats-read-faithfully)
- Status: implemented
- Verified by: `formats::xtxr::decodes_pixels_through_palette_zero`, `expands_rgb555_to_full_range`, `rejects_bad_data`
- Reverse-engineered: yes
- See: [XTXR textures](../original-formats/xtxr.md)

### R-CMP-006 TGA images and colour key

TGA images (16, 24 and 32 bit, RLE, colour-mapped) MUST decode; the cockpit colour key MUST become transparent black.

*Rationale:* Cockpit art (DEV-017).

- Implements: [F-CMP-002](../../definition/compatibility.md#f-cmp-002-original-formats-read-faithfully)
- Status: implemented
- Verified by: `formats::tga::decodes_16_bit_with_colour_key_and_flips_rows`, `decodes_24_and_32_bit_bgr`, `decodes_rle`, `decodes_colour_mapped_images`, `rejects_unsupported_and_truncated`
- Reverse-engineered: yes
- See: [TGA images and cockpits](../original-formats/tga.md)

### R-CMP-007 Audio

WAV in PCM 8/16, MS ADPCM and IMA ADPCM MUST decode.

*Rationale:* Original sound effects and music.

- Implements: [F-CMP-002](../../definition/compatibility.md#f-cmp-002-original-formats-read-faithfully)
- Status: implemented
- Verified by: `formats::wav::pcm16_round_trips_through_to_wav_bytes`, `pcm8_is_recentred`, `ms_adpcm_block_decodes`, `ima_adpcm_block_decodes`, `adpcm_step_clamps_and_keeps_delta_minimum`, `rejects_bad_files`
- Reverse-engineered: yes
- See: [Audio](../original-formats/audio.md)

### R-CMP-008 INI rules and language files

Original INI files MUST be read (Latin-1, comments, duplicates, warnings for ignored lines) and original language files MUST be read with their `~` line breaks.

*Rationale:* Balance data and text are plain files.

- Implements: [F-CMP-002](../../definition/compatibility.md#f-cmp-002-original-formats-read-faithfully)
- Status: implemented
- Verified by: `formats::ini::parses_sections_keys_and_comments`, `reports_lines_it_ignores`, `latin1_bytes_are_accepted`, `merge_overrides_keys_and_adds_sections`, `bad_numbers_and_bools_are_none`; `formats::langfile::keys_text_and_line_breaks`
- Reverse-engineered: yes
- See: [INI rule files](../original-formats/ini.md)

### R-CMP-009 XSPT sprites

XSPT sprite animations SHOULD be decoded and used for the original explosions.

*Rationale:* Faithful explosions (DEV-034 until then).

- Implements: [F-CMP-002](../../definition/compatibility.md#f-cmp-002-original-formats-read-faithfully)
- Status: planned
- Verified by: Planned.
- See: [XSPT sprites](../original-formats/xspt.md)

!!! question "Open"
    The RLE pixel runs are not decoded (unknown).

### R-CMP-010 Smacker video

Smacker video MUST be decoded in-crate for the original cut-scenes and MFD talking heads.

*Rationale:* Faithful presentation.

- Implements: [F-CMP-002](../../definition/compatibility.md#f-cmp-002-original-formats-read-faithfully)
- Status: implemented (app 0.12.0 – 0.13.1, main `5163a57`, `644b218`, `319a31b`, `747d1e9`) — the decoder, the cut-scenes, and the talking-head clips inside `mfdanim.mps`
- Verified by: `formats::smacker::tests::small_and_big_trees_decode_with_the_cache`, `palettes_decode_explicit_kept_and_copied_entries`, `a_tiny_file_decodes_a_solid_frame`, `audio_dpcm_8_bit_mono`; `formats::mpsrc::tests::a_pack_lists_and_returns_its_members`; by hand against all 11 of the reference CD's videos with the ignored `real_smacker_file_decodes` (`OI_SMK`); MT-6-07, MT-6-08
- See: [Smacker video](../original-formats/smacker.md), [Architecture › Formats](../../implementation/architecture/formats.md#smacker-formatssmacker)

!!! note "As built (app 0.12.0)"
    `formats::smacker` decodes `SMK2` and `SMK4`: the four big Huffman trees (built from two small trees each, with the three-slot escape cache reset every frame), mono, full (with version 4's two half-resolution modes), skip and solid 4 × 4 blocks, delta palettes, and DPCM or raw audio, 8- or 16-bit, mono or stereo, on up to seven tracks. Written from the format's public description; `SMACKW32.DLL` is never used. Bad data returns an error.

!!! note "Talking heads (main `319a31b`, app 0.13.1)"
    The MFD talking heads are 113 Smacker clips bundled in `mfdanim.mps`, an **MPSRC pack** ([format](../original-formats/mpsrc.md)), now read by `formats::mpsrc`; a member is addressed as `original:mfdanim.mps#<member>` and decodes like any Smacker file. The comms panel (app 0.13.1) can show one as a voice's `face`. Still open: which clip belongs to which speaker (the `at_`, `as_`, `e_`, `w_` prefixes) is *unknown*, so the `interceptor` voices have no faces yet.

### R-CMP-011 Bad data never panics

Every reader MUST return an error with a readable message on bad, truncated or out-of-bounds data, never panic.

*Rationale:* Modded and damaged data must not crash the game.

- Implements: [F-CMP-002](../../definition/compatibility.md#f-cmp-002-original-formats-read-faithfully), [F-MOD-002](../../definition/modding-toolkit.md#f-mod-002-readable-rule-files-that-explain-mistakes)
- Status: implemented
- Verified by: `formats::out_of_bounds_is_an_error_not_a_panic`, `errors_have_readable_messages`, `formats::xmsh::truncated_mesh_is_an_error`, `mesh_without_lods_is_invalid`
- Reverse-engineered: yes
- See: [Error handling](../../learning/rust/errors.md)

### R-CMP-012 No original content

No original asset may enter the repository or a published build in any form; tests MUST use synthetic data built in the test; original binaries MUST be read as data only and never executed.

*Rationale:* The player supplies the data (project rule).

- Implements: [F-CMP-003](../../definition/compatibility.md#f-cmp-003-no-original-content-in-the-project)
- Status: implemented
- Verified by: Review against the project rules; the `.gitignore` safety net.
- Reverse-engineered: yes
- See: [Testing policy](../../learning/best-practices/testing-policy.md)

### R-CMP-013 Facts and deviations marked

Facts derived from the original data MUST be marked verified, derived or unknown, and every knowing difference from the original MUST be recorded as a deviation.

*Rationale:* An undocumented deviation looks like a bug.

- Implements: [F-CMP-004](../../definition/compatibility.md#f-cmp-004-deviations-on-the-record)
- Status: implemented
- Verified by: Review.
- Reverse-engineered: yes
- See: [Deviations](../../implementation/deviations/index.md)

### R-CMP-014 Resources folder

Content bundled with the game MUST live under `resources/`:
- The default campaign's pack MUST be `resources/default/`, with all its assets in open formats (YAML, OBJ, PNG/TGA, WAV), generated by `tools/default_assets` or written by hand.
- `resources/xcom/` and `resources/xcom-cd/` MUST be git-ignored, MUST contain nothing in the repository, and are where a player places original data.
- The engine MUST load packs from `resources/` (sub-folders with a `pack.yaml`; the original-data folders have none and are skipped) and from `packs/`, which holds the bundled mods such as `interceptor`.
- A mod MUST still override resources, as packs did before.
- The app bundle MUST ship `resources/default` and `packs`, and never `resources/xcom*`.

Rationale: it makes explicit what the game provides and what a mod or the player adds; a root `game/` folder suggested the game was included.

- Implements: [F-CMP-005](../../definition/compatibility.md#f-cmp-005-a-resources-folder-that-says-what-ships-with-the-game)
- Status: implemented
- Verified by: `cli::tests::content_dirs_are_resources_then_packs`; `scripts/check.sh` refuses tracked files under `resources/xcom*` (and the old `game/`).
- See: [Getting started → Folder layout](../../user-guide/getting-started.md#folder-layout), [Settings and data discovery](../../implementation/architecture/settings.md)

### R-CMP-015 Installed copy or CD copy

When no path is given (`--data`, `OPENINTERCEPTOR_DATA`, `settings.yaml` `paths.original`, `<user data>/original`), a development checkout MUST look for original data in `resources/xcom/` (an installed copy) first, then `resources/xcom-cd/` (a CD copy). Both layouts MUST work: original files are found by file name anywhere under the folder.

Rationale: an installed copy is the more likely thing a player still has.

- Implements: [F-CMP-005](../../definition/compatibility.md#f-cmp-005-a-resources-folder-that-says-what-ships-with-the-game)
- Status: implemented
- Verified by: `settings::tests::original_data_is_found_in_order` (an empty `resources/xcom` is skipped; once it holds files it wins over `resources/xcom-cd`).
- See: [Settings and data discovery](../../implementation/architecture/settings.md)

## Revision history

| Version | Date | Change |
|---|---|---|
| 1.0.0 | 2026-11-11 | Bootstrap: R-CMP-001 – R-CMP-013 (13 requirements; 11 reverse-engineered from the code) |
| 1.2.0 | 2026-11-11 | R-CMP-014 resources folder; R-CMP-015 installed copy or CD copy |
| 1.3.0 | 2026-11-11 | R-CMP-014 and R-CMP-015 listed in the status table (implemented, app 0.4.2); R-CMP-001 Verified by: `settings::resolution_order_prefers_explicit_and_existing` renamed `original_data_is_found_in_order` |
| 1.16.0 | 2026-11-11 | R-CMP-010 implemented (app 0.12.0): the in-crate decoder, verified against the reference CD's eleven videos; the MPSRC container of the talking heads (`mfdanim.mps`) read (main `319a31b`). No requirement text changed |
