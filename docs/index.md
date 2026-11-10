# OpenInterceptor

OpenInterceptor is an open-source, moddable **space-combat simulator engine**, written in Rust,
for macOS and Linux first (Windows kept buildable).

Its reference content is *X-COM: Interceptor* (MicroProse, 1998), the third game in the X-COM
series after *UFO Defense* and *Apocalypse*. In the same way that
[OpenXcom](https://openxcom.org) and [OpenApoc](https://www.openapoc.org) play their games, the
engine reads the data from **the player's own copy** of the CD and plays it. No original content
is in the repository, ever.

!!! abstract "Goals, in priority order"
    1. **Modding first.** The engine should make it easy to build a custom space simulator or
       campaign: new ships, weapons, stations, missions and a tech tree, written as data packs
       and scripts. The most likely use is a campaign that uses **none** of the original assets.
       *Interceptor* is the worked example that proves the engine is complete.
    2. **Be an example of good Rust practice** for a 3D application or game engine: clear
       layering, small dependencies, modules commented for a Rust beginner, and unit tests for
       all code. See [Learn Rust with this codebase](learn-rust/index.md) and
       [Best practices](best-practices/index.md).
    3. **Play X-COM: Interceptor** from the player's data on current systems.

    The same order is in the repository's `CLAUDE.md` project rules.

## Where things are

| Branch / folder | Holds |
|---|---|
| `main` | The engine: `Cargo.toml`, `src/`, built-in packs in `packs/`, the gate `scripts/check.sh` |
| `docs` (this site) | Plan, architecture, original formats, modding guide, tests and status |
| `game/` (git-ignored) | The developer's own copy of the CD. Never committed |
| `dist/` (git-ignored) | Local builds that may contain the player's data. Never shared |

## Status

Stage 0 (foundation and format analysis) is **complete**. Stage 1 (the flight simulator testbed)
is **complete**, signed off on 2026-11-10 — every original ship, station and cockpit flies or
renders from the player's CD, an original-free pack plays without it, any original asset can be
replaced by a mod file (with `--extract` producing editable copies), and cockpits can be HD
repaints or 3D models. Stage 2 is **in progress** (build **0.2.2**): one campaign at a time —
the original `interceptor` or the original-free default campaign *Perils of the Void* — with
ship systems, power, damage and repair, tractor, cloak, ECM, hyperspace, wingman orders and an
in-game controls menu. See [implementation status](implementation-status.md) and the
[plan](implementation-plan/index.md).

## Specification

This is a SpecItOut **implementation** project with **no specification yet**
(`"specVersion": null` in `.specitout/sdd.json`). The intended specification project is
`OpenInterceptor-Specification`. Until it exists, pages here cite plan stages rather than
requirement IDs.

## Licence

The engine is **MIT**. *X-COM* and *Interceptor* are trademarks of their owners; this project is
not affiliated with or endorsed by them.
