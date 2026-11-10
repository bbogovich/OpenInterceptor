# Stage 3 — Missions, environments and the condition language

**Version 0.4.0. Status: planned.** Missions become data: roles, waves, objectives and
outcomes, in space and — new to this engine — on planet surfaces with atmospheric flight.

Design: [missions and events](../design/format-missions.md),
[behaviours and scripting](../design/format-behaviour.md).

## Work items

| # | Item | Module |
|---|---|---|
| 3.1 | The **condition and action language**: lexer, parser, type checker, interpreter; units; contexts (object, battle, campaign); load-time validation with file/section/key messages | `script` |
| 3.2 | Mission types: roles (from situation, class lists per difficulty, player craft), arrivals and waves, objectives (win/draw/lose, optional), outcomes as action lists, timers, hazards (asteroids, black-hole pull, minefields, ion storm) | `mission` (new), `sim` |
| 3.3 | Battle environments: `space`, `orbit`, `pocket`; backdrops from the map location | `render`, `sim` |
| 3.4 | **Atmospheric flight**: drag, gravity, lift, hover, ground collision and landing, ceiling escape; per-ship `DragArea`, `Lift`, `Hover`, `AtmosphereMaxSpeed`, `Atmospheric` | `sim::flight` |
| 3.5 | **Terrain**: heightfield from an image or a generator (craters, hills), texturing with detail maps, scattered objects, sky gradient, fog; ground installations (`Kind = ground`) and anti-air turrets | `render::terrain` (new), `sim` |
| 3.6 | HUD for atmosphere: altitude, vertical speed, horizon | `hud` |
| 3.7 | Wingman comms menu and orders; radio chatter by trigger; voices | `sim`, `audio`, `hud` |
| 3.8 | Hyperspace out with countdown; retreat outcomes; debrief screen (results, salvage, score) | `mission`, `app` |
| 3.9 | Simulator missions with a setup screen | `app` |
| 3.10 | **Tier 2 script language decision** (Lua via `mlua` recommended): dependency evaluation recorded on the dependency page, then hooks (`State = script`, `Do = script(...)`) and the sandboxed API | `script` |
| 3.11 | `interceptor`: the 27 mission types, the simulator types and the final mission's two parts as mission definitions (compositions from the guide, else *derived* estimates) | `packs/interceptor` |
| 3.12 | `default`: the 11 generated mission types (2 on surfaces) and chapter missions 1, 2, 4, 5 playable from the testbed; terrain and sky assets generated | `packs/default`, `tools` |

**Testbed adds**: a mission browser (every mission type of both campaigns, with difficulty and
force choices), a terrain flight sandbox (atmospheric handling on several worlds), and a
condition-language console.

**Exit**: every mission type winnable and losable in the testbed; atmospheric flight feels
distinct from space (manual test); the language fully unit-tested (parser, types, evaluation,
error messages).
