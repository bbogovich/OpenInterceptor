# Implementation plan

An iterative, staged plan. **Each stage ends with a playable build of the testbed**: a launcher
listing live test environments (scenarios) that show the stage's features and let a tester
playtest one component at a time. Later stages add scenarios; none are removed.

| Stage | Version | Delivers | Status |
|---|---|---|---|
| [0 Foundation](stage-0-foundation.md) | 0.1.0 | Repository, SpecItOut, docs site, format analysis, feasibility | Done |
| [1 Flight simulator](stage-1-flight.md) | 0.2.0 | Packs and rules, model loading, rendering, cockpits and HUD, controls, flight model, weapons, targeting, wingmen, testbed | In progress |
| [2 Enemies and combat](stage-2-combat.md) | 0.3.0 | Per-class AI, full damage and power model, special systems, missiles' behaviours, mines, probes, station turrets, effects, sound | Planned |
| [3 Missions and scripting](stage-3-missions.md) | 0.4.0 | Mission definitions, objectives, triggers, comms, hyperspace, debrief; scripting language | Planned |
| [4 Strategy layer](stage-4-strategy.md) | 0.5.0 | Star map, bases, research, manufacturing, economy, enemy strategy | Planned |
| [5 Presentation](stage-5-presentation.md) | 0.6.0 | Menus, Smacker video, music, settings, saves | Planned |
| [6 Modding toolkit](stage-6-modding.md) | 0.7.0 | Validator, exporter, a full original-free sample campaign, guide | Planned |
| [7 Platforms and release](stage-7-release.md) | 1.0.0 | macOS app, Linux packages, Windows, release process | Planned |

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
| Modding first | Each feature is reachable from a pack before it is called done; the `sample` pack exercises it without original data. |
| Tests before done | Unit tests for every module; manual cases for what only a playtest shows ([testing policy](../best-practices/testing-policy.md)). |

## Checks for every stage

- `scripts/check.sh` (unit tests, release build, `--check` on all packs, asset-tracking guard).
- The stage's [manual test cases](../testing/manual-tests.md), recorded as passed.
