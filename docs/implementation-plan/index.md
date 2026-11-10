# Implementation plan

An iterative, staged plan. **Each stage ends with a playable build of the testbed**: a launcher
listing live test environments (scenarios) that show the stage's features and let a tester
playtest one component at a time. Later stages add scenarios; none are removed.

| Stage | Version | Delivers | Status |
|---|---|---|---|
| [0 Foundation](stage-0-foundation.md) | 0.1.0 | Repository, SpecItOut, docs site, format analysis, feasibility | Done |
| [1 Flight simulator](stage-1-flight.md) | 0.2.0 | Packs and rules, model loading, rendering, cockpits and HUD, controls, flight model, weapons, targeting, wingmen, testbed | Done (signed off 2026-11-10) |
| [2 Combat, objects and behaviours](stage-2-combat.md) | 0.3.0 | Mod kinds and `parent`; model sets (damage, construction states, debris); full damage, power and special systems; behaviours with difficulty variants; missiles, mines, probes, stations; effects, sound | In progress |
| [3 Missions, environments and conditions](stage-3-missions.md) | 0.4.0 | Condition/action language; mission types; space and **atmospheric/ground** environments with terrain; comms; hyperspace; debrief; Tier 2 script decision | Planned |
| [4 Strategy layer](stage-4-strategy.md) | 0.5.0 | Interactive star map, squadrons and sensors, economy, research, bases and modules, ships and loadouts, pilots, functional screens, saves | Planned |
| [5 Campaigns](stage-5-campaign.md) | 0.6.0 | Events, arcs, rival plans, strategist, dynamic relations, endings: **both campaigns playable start to finish** | Planned |
| [6 Presentation](stage-6-presentation.md) | 0.7.0 | Screen layouts and skins, menus, video, interactive music, voices | Planned |
| [7 Modding toolkit and Remastered](stage-7-modding.md) | 0.8.0 | Full validator, recipes (upscale, mesh patch), `interceptor-remastered`, modding guide | Planned |
| [8 Platforms and release](stage-8-release.md) | 1.0.0 | macOS app, Linux packages, Windows, release process | Planned |

## Three campaigns

The plan delivers three campaign mods, designed in [Game design and mod format](../design/index.md):

| Campaign | Grows in stages | Complete |
|---|---|---|
| [`interceptor`](../design/interceptor-campaign.md): the full original campaign from the player's CD — the primary test case | 2–6 | Playable start to finish at Stage 5; presentation at Stage 6 |
| [`default`](../design/default-campaign.md): *Perils of the Void*, a 1950s film-serial campaign with original content — runs without a CD | 2–7 | Playable start to finish at Stage 5; finished at Stage 7 |
| [`interceptor-remastered`](../design/remastered.md): extends `interceptor` with locally generated upscaled textures and an enhanced Lightning | 7 | Stage 7 |

Every stage's features must work in both `interceptor` and `default` before the stage is done
(the project's modding-first rule), so neither campaign can fall behind.

## Feasibility

**Confident.** Every asset type stages 1–2 need was decoded and checked against the data: all
297 meshes parse; hulls and a saucer rendered correctly with their own textures; textures,
cockpit images, sound effects and music loops are understood; all balance data is plain INI.
The remaining unknowns (XSPT pixel runs, a few mesh fields, `mfdanim.mps`, the CD audio codec)
have fallbacks and do not block any stage. The original simulator is far simpler than an id
Tech 3 game: no levels, no skeletal animation, no scripting VM to reproduce.

## Ground rules

| Rule | In practice |
|---|---|
| The player supplies the game | Original data is read from the player's own CD copy at run time. |
| No original content in the repository | `game/`, `dist/` and every original asset type are git-ignored; the gate fails if one is tracked. |
| Engine, not game | Nothing *Interceptor*-specific is hard-coded: it lives in the `interceptor` pack as rules and scenarios. |
| Original programs are data | `Interceptor.exe` and the DLLs are parsed, never executed. |
| Modding first | Each feature is reachable from a pack before it is called done; the original-free `default` campaign (the `sample` pack in Stage 1) exercises it without original data. |
| Tests before done | Unit tests for every module; manual cases for what only a playtest shows ([testing policy](../best-practices/testing-policy.md)). |

## Checks for every stage

- `scripts/check.sh` (unit tests, release build, `--check` on all packs, asset-tracking guard).
- The stage's [manual test cases](../testing/manual-tests.md), recorded as passed.
