---
spec_version: 1.7.0
status: current
updated: 2026-11-11
---

# Platforms requirements

Requirements for the macOS, Linux and Windows builds, packaging, dependencies and (planned) bundling a complete game.

Features: [Platforms](../../definition/platforms.md). How to read a requirement: [Specification](../index.md#reading-a-requirement).

| Requirement | Status |
|---|---|
| [R-PLT-001 macOS app](#r-plt-001-macos-app) | Implemented |
| [R-PLT-002 Linux](#r-plt-002-linux) | Implemented |
| [R-PLT-003 Windows](#r-plt-003-windows) | Planned |
| [R-PLT-004 Release packages](#r-plt-004-release-packages) | Planned |
| [R-PLT-005 One dependency](#r-plt-005-one-dependency) | Implemented |
| [R-PLT-006 Unsafe only at boundaries](#r-plt-006-unsafe-only-at-boundaries) | Implemented |
| [R-PLT-007 Product file](#r-plt-007-product-file) | Planned |
| [R-PLT-008 Bundled game builds](#r-plt-008-bundled-game-builds) | Planned |
| [R-PLT-009 Nothing original is bundled](#r-plt-009-nothing-original-is-bundled) | Planned |
| [R-PLT-010 Validate before bundling](#r-plt-010-validate-before-bundling) | Planned |

## Requirements

### R-PLT-001 macOS app

A macOS `.app` bundle with the stage build number MUST open from Finder and contain only the engine and original-free packs.

*Rationale:* First platform.

- Implements: [F-PLT-001](../../definition/platforms.md#f-plt-001-macos), [F-PLT-004](../../definition/platforms.md#f-plt-004-release-packages)
- Status: implemented
- Verified by: MT-1-30
- Reverse-engineered: yes
- See: [Packaging](../../implementation/build/packaging.md)

### R-PLT-002 Linux

The engine MUST build and run on Linux, with gamepads through the Linux joystick API.

*Rationale:* First-class platform.

- Implements: [F-PLT-002](../../definition/platforms.md#f-plt-002-linux)
- Status: implemented
- Verified by: `platform::linux` test (Linux only); MT-1-21 (not yet run)
- Reverse-engineered: yes
- See: [Platform layer](../../implementation/architecture/platform.md)

!!! question "Open"
    Built and type-checked only; not yet run on Linux.

### R-PLT-003 Windows

The engine MUST stay buildable on Windows and SHOULD support XInput gamepads.

*Rationale:* Reach.

- Implements: [F-PLT-003](../../definition/platforms.md#f-plt-003-windows)
- Status: planned
- Verified by: Planned.
- See: [Stage 8](../../implementation/plan/stage-8-release.md)

### R-PLT-004 Release packages

Releases MUST provide a macOS app, a Linux tarball with a `.desktop` file and a Windows build, versioned, with no player data in any published build.

*Rationale:* Distribution.

- Implements: [F-PLT-004](../../definition/platforms.md#f-plt-004-release-packages)
- Status: planned
- Verified by: Planned.
- See: [Stage 8](../../implementation/plan/stage-8-release.md)

### R-PLT-005 One dependency

The engine MUST depend only on `macroquad` (pinned); any other dependency MUST be evaluated on the dependency page before it is added.

*Rationale:* Every dependency is a liability (project rule).

- Implements: [F-PLT-001](../../definition/platforms.md#f-plt-001-macos), [F-PLT-002](../../definition/platforms.md#f-plt-002-linux), [F-PLT-003](../../definition/platforms.md#f-plt-003-windows)
- Status: implemented
- Verified by: Review of `Cargo.toml`.
- Reverse-engineered: yes
- See: [Dependency policy](../../learning/best-practices/dependencies.md)

### R-PLT-006 Unsafe only at boundaries

`unsafe` MUST appear only at FFI boundaries (platform shims) and the one documented access to macroquad's GL state, each with a `// SAFETY:` note.

*Rationale:* Teaching example and safety (project rule).

- Implements: [F-PLT-001](../../definition/platforms.md#f-plt-001-macos), [F-PLT-002](../../definition/platforms.md#f-plt-002-linux)
- Status: implemented
- Verified by: Review.
- Reverse-engineered: yes
- See: [FFI, repr(C) and unsafe](../../learning/rust/ffi-unsafe.md)

### R-PLT-007 Product file

A creator's project MUST be able to describe a complete game in a product file, `product.yaml`: name, version, bundle id and icon; splash image and intro cutscene; the packs to include, in load order; the default campaign; whether the testbed, model viewer and developer tools are shown (off for a release); licence and credit files.

*Rationale:* The user's request: "release a complete game without requiring manual download and installation of packs and mods".

- Implements: [F-PLT-005](../../definition/platforms.md#f-plt-005-game-bundling)
- Status: planned
- Verified by: Planned (MD7).
- See: [Mod designer: bundling a complete game](../design/designer.md#5-bundling-a-complete-game)

### R-PLT-008 Bundled game builds

The build MUST produce two artifacts from the one repository: the **game** and the **designer** (R-MOD-021). From a product file, the build tool (`tools/build_bundle.py --product product.yaml`, or the designer's "Package game") MUST build a product bundle from the game artifact that:

- contains the engine, `resources/default` when the product uses it, and exactly the listed packs;
- is named and branded as the product;
- starts directly in the product's default campaign;
- hides the testbed, model viewer and developer tools when the product says so;
- still loads the player's own mods from their user folder on top.

It MUST write the macOS `.app`, and the Linux and Windows packages once those exist (R-PLT-004).

*Rationale:* A creator ships a game in its own right; players install nothing by hand. The designer is a separate program (decision D5, 2026-11-11), not part of a product.

- Implements: [F-PLT-005](../../definition/platforms.md#f-plt-005-game-bundling)
- Status: planned
- Verified by: Planned (MD7).
- See: [Mod designer: bundling a complete game](../design/designer.md#5-bundling-a-complete-game), [Packaging](../../implementation/build/packaging.md)

### R-PLT-009 Nothing original is bundled

The bundler MUST refuse a pack with `requires_original`, and any file recorded as converted from original data (R-MOD-029). A pack built on `interceptor` MAY be shared as a mod, but MUST NOT be bundled into a product.

*Rationale:* No original content is ever distributed (project rule).

- Implements: [F-PLT-005](../../definition/platforms.md#f-plt-005-game-bundling), [F-CMP-003](../../definition/compatibility.md#f-cmp-003-no-original-content-in-the-project)
- Status: planned
- Verified by: Planned (MD7).
- See: [Mod designer: bundling a complete game](../design/designer.md#5-bundling-a-complete-game)

### R-PLT-010 Validate before bundling

The full validator MUST run on the product's packs and report no errors before a bundle is written.

*Rationale:* A released game starts without content errors.

- Implements: [F-PLT-005](../../definition/platforms.md#f-plt-005-game-bundling)
- Status: planned
- Verified by: Planned (MD7).
- See: [R-MOD-017](modding.md#r-mod-017-full-validator)

## Revision history

| Version | Date | Change |
|---|---|---|
| 1.0.0 | 2026-11-11 | Bootstrap: R-PLT-001 – R-PLT-006 (6 requirements; 4 reverse-engineered from the code) |
| 1.7.0 | 2026-11-11 | R-PLT-007 – R-PLT-010 (planned): game bundling — the product file, two build artifacts (game and designer, decision D5) and branded product builds, the provenance guard, validation first |
