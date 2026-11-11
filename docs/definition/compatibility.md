---
spec_version: 1.3.0
status: current
updated: 2026-11-11
---

# Compatibility (original data)

Playing *X-COM: Interceptor* from the player's own CD, read as data, without the project ever holding original content.

Requirements for these features: [Compatibility (original data) requirements](../specification/requirements/compatibility.md).

| Feature | Status |
|---|---|
| [F-CMP-001 Play from the player's own CD](#f-cmp-001-play-from-the-players-own-cd) | Implemented |
| [F-CMP-002 Original formats read faithfully](#f-cmp-002-original-formats-read-faithfully) | In progress |
| [F-CMP-003 No original content in the project](#f-cmp-003-no-original-content-in-the-project) | Implemented |
| [F-CMP-004 Deviations on the record](#f-cmp-004-deviations-on-the-record) | Implemented |
| [F-CMP-005 A resources folder that says what ships with the game](#f-cmp-005-a-resources-folder-that-says-what-ships-with-the-game) | Implemented |

## Features

### F-CMP-001 Play from the player's own CD

**What.** The engine finds the player's copy of the CD data and plays *X-COM: Interceptor* from it.

**Why.** Compatibility objective; the project never holds original content.

- **Status:** implemented
- **Source:** Plan items 1.1, 1.13 and 1.14.
- **Specified by:** [R-CMP-001](../specification/requirements/compatibility.md#r-cmp-001-finding-the-players-data), [R-CMP-002](../specification/requirements/compatibility.md#r-cmp-002-original-data-by-name-in-any-layout)

### F-CMP-002 Original formats read faithfully

**What.** Original meshes, textures, images, sounds, rules and text are read as data and shown as the original intended; sprites and video follow.

**Why.** Compatibility objective.

- **Status:** in-progress
- **Source:** Plan item 1.1; Stage 0 analysis; XSPT and Smacker outstanding (2.4, 6.3).
- **Specified by:** [R-CPN-012](../specification/requirements/campaigns.md#r-cpn-012-video-and-music), [R-MOD-019](../specification/requirements/modding.md#r-mod-019-sound-cue-table), [R-CMP-003](../specification/requirements/compatibility.md#r-cmp-003-pe-resources), [R-CMP-004](../specification/requirements/compatibility.md#r-cmp-004-xmsh-meshes), [R-CMP-005](../specification/requirements/compatibility.md#r-cmp-005-xtxr-textures), [R-CMP-006](../specification/requirements/compatibility.md#r-cmp-006-tga-images-and-colour-key), [R-CMP-007](../specification/requirements/compatibility.md#r-cmp-007-audio), [R-CMP-008](../specification/requirements/compatibility.md#r-cmp-008-ini-rules-and-language-files), [R-CMP-009](../specification/requirements/compatibility.md#r-cmp-009-xspt-sprites), [R-CMP-010](../specification/requirements/compatibility.md#r-cmp-010-smacker-video), [R-CMP-011](../specification/requirements/compatibility.md#r-cmp-011-bad-data-never-panics)

### F-CMP-003 No original content in the project

**What.** Nothing from the original game is committed or published in any form; original binaries are read as data, never executed.

**Why.** Legal and ethical: the player supplies the data.

- **Status:** implemented
- **Source:** Project rules.
- **Specified by:** [R-CMP-012](../specification/requirements/compatibility.md#r-cmp-012-no-original-content), [R-CON-021](../specification/requirements/control-console.md#r-con-021-console-theme-assets-are-original-free)

### F-CMP-004 Deviations on the record

**What.** Where the engine differs from the original, or uses its own value where the original's is unknown, the difference is documented and facts are marked verified, derived or unknown.

**Why.** An undocumented deviation looks like a bug.

- **Status:** implemented
- **Source:** Project rules; Deviations page.
- **Specified by:** [R-CMP-013](../specification/requirements/compatibility.md#r-cmp-013-facts-and-deviations-marked)

### F-CMP-005 A resources folder that says what ships with the game

**What.** Content bundled with the game lives in one `resources/` folder:
- `resources/default/` holds the default campaign and every one of its assets, in modern editable formats. It is the equivalent of the X-COM game's contents.
- `resources/xcom/` (an installed copy of X-COM: Interceptor) and `resources/xcom-cd/` (a copy of the CD) are empty, git-ignored places where a player adds their own original data for a personal build.

Mods (`packs/`, and the user's mods folder) override resources as before. The interceptor campaign is such a mod.

**Why.** Most players will not own X-COM: Interceptor, and those who do more likely have it installed than a CD copy. A `game/` folder at the root suggested the game itself was included. The new layout makes it explicit which assets the default campaign provides and what belongs to an installed mod.

- **Status:** implemented (app 0.4.2, main `bf7b7ce`)
- **Source:** User request, 2026-11-11: "Rather than a \"game\" folder in the repo root, which is misleading because it gives the impression that the game itself is there, it should be a \"resources\" folder that contains core assets bundled with the game for the default campaign in one subfolder, and gitignored xcom and xcom-cd subfolders that a user would manually add to for their personal build." "the default campaign's content in resources is the equivalent the xcom game contents, but with all files in modern editable formats."
- **Specified by:** [R-CMP-014](../specification/requirements/compatibility.md#r-cmp-014-resources-folder), [R-CMP-015](../specification/requirements/compatibility.md#r-cmp-015-installed-copy-or-cd-copy)

## Revision history

| Version | Date | Change |
|---|---|---|
| 1.0.0 | 2026-11-11 | Bootstrap: F-CMP-001 – F-CMP-004 (4 features) |
| 1.2.0 | 2026-11-11 | F-CMP-005 resources folder |
| 1.3.0 | 2026-11-11 | F-CMP-005 listed in the status table; implemented in app 0.4.2 |
