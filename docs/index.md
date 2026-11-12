# OpenInterceptor

OpenInterceptor is an open-source, moddable **space-combat simulator engine**, written in Rust,
for macOS and Linux first (Windows kept buildable).

Its reference content is *X-COM: Interceptor* (MicroProse, 1998), the fourth game in the X-COM
series, after *UFO Defense* (*Enemy Unknown*), *Terror from the Deep* and *Apocalypse*. In the same way that
[OpenXcom](https://openxcom.org) and [OpenApoc](https://www.openapoc.org) play their games, the
engine reads the data from **the player's own copy** (installed or the CD) and plays it. No original content
is in the repository, ever.

!!! abstract "Goals, in priority order"
    1. **Modding first.** The engine should make it easy to build a custom space simulator or
       campaign: new ships, weapons, stations, missions and a tech tree, written as data packs
       and scripts. The most likely use is a campaign that uses **none** of the original assets.
       *Interceptor* is the worked example that proves the engine is complete.
    2. **Be an example of good Rust practice** for a 3D application or game engine: clear
       layering, small dependencies, modules commented for a Rust beginner, and unit tests for
       all code. See [Learn Rust with this codebase](learning/rust/index.md) and
       [Best practices](learning/best-practices/index.md).
    3. **Play X-COM: Interceptor** from the player's data on current systems.

    The same order is in the repository's `CLAUDE.md` project rules.

**Site version 0.22.0**; Definition and Specification **1.16.0** ([revision history](reference/revision-history.md)).

## Sections

| Section | What it holds |
|---|---|
| [Definition](definition/index.md) | What the player or modder gets, and why: features, and the campaigns that define "done" |
| [Specification](specification/index.md) | How it behaves: requirements, the mod formats and the original data formats |
| [Implementation](implementation/index.md) | How it is built: plan, architecture, status, deviations, testing, build, workflows |
| [User guide](user-guide/index.md) | Installing, running and playing |
| [Modding](modding/index.md) | Building packs and campaigns: every rule and scenario key |
| [Learning](learning/index.md) | Rust and engine practice taught from this codebase |
| [Reference](reference/index.md) | The original game, and the histories of this site, the definition and the specification |

## Where things are

| Branch / folder | Holds |
|---|---|
| `main` | The engine: `Cargo.toml`, `src/`, the default campaign in `resources/default/`, bundled mods in `packs/`, the console page in `web/console/`, the gate `scripts/check.sh` |
| `docs` (this site) | Plan, architecture, original formats, modding guide, tests and status |
| `resources/xcom/`, `resources/xcom-cd/` (git-ignored) | The developer's own original data: an installed copy (tried first) or a CD copy. Never committed |
| `dist/` (git-ignored) | Local builds that may contain the player's data. Never shared |

## Status

Stage 0 (foundation and format analysis) is **complete**. Stage 1 (the flight simulator testbed)
is **complete**, signed off on 2026-11-10 — every original ship, station and cockpit flies or
renders from the player's original data, an original-free pack plays without it, any original asset can be
replaced by a mod file (with `--extract` producing editable copies), and cockpits can be HD
repaints or 3D models. Stage 2 (combat, objects and behaviours) is **complete** in app 0.3.0,
which also carries Stage 3 part 1 (missions). App **0.3.1** delivers the playtest fixes of the
[Stage 3 interlude](implementation/plan/stage-3-interlude.md): a drift readout and flight-path
marker, shot physics, visibility (rim light, distant dots, brightness), the `killed` mission
result and a keys overlay (`?`). App **0.4.0** adds the first **tablet control console** (C1:
pair a tablet on the local network, live telemetry, every action as a button —
[Tablet console](user-guide/console.md)); **0.4.1** fixes a start-up shader error; **0.4.2**
moves the default campaign to `resources/default`, with the player's optional original data in
`resources/xcom` (installed copy) or `resources/xcom-cd` (CD copy). See [implementation status](implementation/status.md) and the
[plan](implementation/plan/index.md).

## How work flows: Rapid Prototyping

The project follows the **Rapid Prototyping** workflow ([Workflows](implementation/workflows/index.md#rapid-prototyping)).
Each user request becomes, in the same pass and before any code, a **feature** (`F-…`) on the
[Definition](definition/index.md), then **requirements** (`R-…`) citing it on the
[Specification](specification/index.md), then issues queued in `sio` citing those requirements.
There are no approval gates; the definition and specification are a permanent record that moves
ahead of the code. When the prototype is complete they seed the formal definition and
specification of a production version, built through the SpecItOut fast track.

Definition and Specification **1.0.0** (2026-11-11): 79 features and 168
requirements, those for the code already built marked *reverse-engineered*; **1.1.0** marks the
interlude's feedback items implemented; **1.2.0** adds the `resources/` folder (F-CMP-005);
**1.3.0** marks console C1 and the `resources/` layout implemented and records protocol v1;
**1.4.0** – **1.6.0** add the console heartbeat, collisions with impact and Simple physics;
**1.7.0** plans the Mod Designer, Cutscene Designer and game bundling and records the console
pairing fix and keep-awake ([change history](meta/change-history.md)). Until `sio` supports
a `prototype` kind the project runs as `kind: implementation` with `spec` pointing at
`docs/specification` (DEV-042).

## Licence and repository

The engine is **MIT** (`LICENSE`). The repository is public at
[github.com/bbogovich/OpenInterceptor](https://github.com/bbogovich/OpenInterceptor); issues are
GitHub Issues, reached through the `sio` client. *X-COM* and *Interceptor* are trademarks of their owners; this project is
not affiliated with or endorsed by them.
