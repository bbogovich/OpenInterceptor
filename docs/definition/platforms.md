---
spec_version: 1.7.0
status: current
updated: 2026-11-11
---

# Platforms

Where OpenInterceptor runs and how it is shipped, including (planned) a creator's own bundled game.

Requirements for these features: [Platforms requirements](../specification/requirements/platforms.md).

| Feature | Status |
|---|---|
| [F-PLT-001 macOS](#f-plt-001-macos) | Implemented |
| [F-PLT-002 Linux](#f-plt-002-linux) | In progress |
| [F-PLT-003 Windows](#f-plt-003-windows) | Planned |
| [F-PLT-004 Release packages](#f-plt-004-release-packages) | Planned |
| [F-PLT-005 Game bundling](#f-plt-005-game-bundling) | Planned |

## Features

### F-PLT-001 macOS

**What.** OpenInterceptor runs on macOS as an app, with gamepad support.

**Why.** First platform.

- **Status:** implemented
- **Source:** Plan item 1.19; Stage 8.
- **Specified by:** [R-CON-005](../specification/requirements/control-console.md#r-con-005-gamepad), [R-PLT-001](../specification/requirements/platforms.md#r-plt-001-macos-app), [R-PLT-005](../specification/requirements/platforms.md#r-plt-005-one-dependency), [R-PLT-006](../specification/requirements/platforms.md#r-plt-006-unsafe-only-at-boundaries)

### F-PLT-002 Linux

**What.** OpenInterceptor runs on Linux with joystick support.

**Why.** First-class platform.

- **Status:** in-progress
- **Source:** Plan item 1.10; Stage 8 (built and type-checked, not yet run on Linux, MT-1-21).
- **Specified by:** [R-CON-005](../specification/requirements/control-console.md#r-con-005-gamepad), [R-PLT-002](../specification/requirements/platforms.md#r-plt-002-linux), [R-PLT-005](../specification/requirements/platforms.md#r-plt-005-one-dependency), [R-PLT-006](../specification/requirements/platforms.md#r-plt-006-unsafe-only-at-boundaries)

### F-PLT-003 Windows

**What.** Windows is kept buildable and gets XInput gamepads.

**Why.** Reach; the original's platform.

- **Status:** planned
- **Source:** Stage 8.
- **Specified by:** [R-PLT-003](../specification/requirements/platforms.md#r-plt-003-windows), [R-PLT-005](../specification/requirements/platforms.md#r-plt-005-one-dependency)

### F-PLT-004 Release packages

**What.** Versioned releases: a macOS app, Linux packages and a Windows build, containing only the engine and original-free packs.

**Why.** Players need something to download.

- **Status:** planned
- **Source:** Stage 8.
- **Specified by:** [R-PLT-001](../specification/requirements/platforms.md#r-plt-001-macos-app), [R-PLT-004](../specification/requirements/platforms.md#r-plt-004-release-packages)

### F-PLT-005 Game bundling

**What.** A creator can release a complete game: the engine plus chosen packs, named and branded as their own, starting straight in their campaign, with developer tools hidden — downloaded and played with nothing to install. Players' own mods still load on top. Nothing original can ever be bundled. The build produces two artifacts, the game and the designer.

**Why.** Modding first: a custom campaign can become a game in its own right.

- **Status:** planned
- **Source:** The user's request, 2026-11-11, quoted in full on the [design page](../specification/design/designer.md): "Build system needs to accommodate bundling specific mods/packs into the distributable or application bundle so that users can release a complete game without requiring manual download and installation of packs and mods to their local folders." Decision D5, 2026-11-11 (two build artifacts). Plan item MD7.
- **Specified by:** [R-PLT-007](../specification/requirements/platforms.md#r-plt-007-product-file), [R-PLT-008](../specification/requirements/platforms.md#r-plt-008-bundled-game-builds), [R-PLT-009](../specification/requirements/platforms.md#r-plt-009-nothing-original-is-bundled), [R-PLT-010](../specification/requirements/platforms.md#r-plt-010-validate-before-bundling), [R-MOD-029](../specification/requirements/modding.md#r-mod-029-provenance-of-imported-assets)

## Revision history

| Version | Date | Change |
|---|---|---|
| 1.0.0 | 2026-11-11 | Bootstrap: F-PLT-001 – F-PLT-004 (4 features) |
| 1.7.0 | 2026-11-11 | F-PLT-005 game bundling (planned) |
