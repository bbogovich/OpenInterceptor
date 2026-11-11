---
spec_version: 1.0.0
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

## Revision history

| Version | Date | Change |
|---|---|---|
| 1.0.0 | 2026-11-11 | Bootstrap: F-CMP-001 – F-CMP-004 (4 features) |
