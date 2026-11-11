---
spec_version: 1.0.0
status: current
updated: 2026-11-11
---

# Platforms requirements

Requirements for the macOS, Linux and Windows builds, packaging and dependencies.

Features: [Platforms](../../definition/platforms.md). How to read a requirement: [Specification](../index.md#reading-a-requirement).

| Requirement | Status |
|---|---|
| [R-PLT-001 macOS app](#r-plt-001-macos-app) | Implemented |
| [R-PLT-002 Linux](#r-plt-002-linux) | Implemented |
| [R-PLT-003 Windows](#r-plt-003-windows) | Planned |
| [R-PLT-004 Release packages](#r-plt-004-release-packages) | Planned |
| [R-PLT-005 One dependency](#r-plt-005-one-dependency) | Implemented |
| [R-PLT-006 Unsafe only at boundaries](#r-plt-006-unsafe-only-at-boundaries) | Implemented |

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

## Revision history

| Version | Date | Change |
|---|---|---|
| 1.0.0 | 2026-11-11 | Bootstrap: R-PLT-001 – R-PLT-006 (6 requirements; 4 reverse-engineered from the code) |
