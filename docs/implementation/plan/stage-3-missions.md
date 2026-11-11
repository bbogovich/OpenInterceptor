# Stage 3 — Missions, environments and the condition language

**Version 0.4.0. Status: in progress** — part 1 (missions as data) shipped in app 0.3.0, main
`d7553f5`, 2026-11-11. Missions become data: roles, waves, objectives and
outcomes, in space and — new to this engine — on planet surfaces with atmospheric flight.

Design: [missions and events](../../specification/design/format-missions.md),
[behaviours and scripting](../../specification/design/format-behaviour.md).

## Work items

| # | Item | Module | Status |
|---|---|---|---|
| 3.1 | The **condition and action language**: lexer, parser, type checker, interpreter; units; contexts (object, battle, campaign); load-time validation with file/section/key messages | `script` | **Partial:** conditions (Stage 2) and **actions** done (checked calls with battle/campaign scope); the **battle context** done; the campaign context comes with Stage 4. |
| 3.2 | Mission types: roles (from situation, class lists per difficulty, player craft), arrivals and waves, objectives (win/draw/lose, optional), outcomes as action lists, timers, hazards (asteroids, black-hole pull, minefields, ion storm) | `mission` (new), `sim` | **Done except hazards:** roles (class lists, counts per level, faction, AI, behaviour, protect, state, hull, systems, speed), arrivals (`at:`, `arrive:`; DEV-040), waves, objectives, optional objectives, timers, outcomes. Hazards planned. |
| 3.3 | Battle environments: `space`, `orbit`, `pocket`; backdrops from the map location | `render`, `sim` | Planned. |
| 3.4 | **Atmospheric flight**: drag, gravity, lift, hover, ground collision and landing, ceiling escape; per-ship `DragArea`, `Lift`, `Hover`, `AtmosphereMaxSpeed`, `Atmospheric` | `sim::flight` | Planned. |
| 3.5 | **Terrain**: heightfield from an image or a generator (craters, hills), texturing with detail maps, scattered objects, sky gradient, fog; ground installations (`kind: ground`) and anti-air turrets | `render::terrain` (new), `sim` | Planned. |
| 3.6 | HUD for atmosphere: altitude, vertical speed, horizon | `hud` | Planned. |
| 3.7 | Wingman comms menu and orders; radio chatter by trigger; voices | `sim`, `audio`, `hud` | **Partial:** wingman comms menu and orders (Stage 2); radio chatter by condition (missions). Voices planned. |
| 3.8 | Hyperspace out with countdown; retreat outcomes; debrief screen (results, salvage, score) | `mission`, `app` | **Partial:** hyperspace out with countdown (Stage 2) — the player jumping out is the `retreat` result; the **debrief screen** (result, text, kills, losses, time, optional objectives, campaign actions). Salvage and score are applied by the strategy layer (Stage 4). |
| 3.9 | Simulator missions with a setup screen | `app` | Planned. |
| 3.10 | **Tier 2 script language decision** (Lua via `mlua` recommended): dependency evaluation recorded on the dependency page, then hooks (`do: script`, `do: ["script(...)"]`) and the sandboxed API | `script` | Planned. |
| 3.11 | `interceptor`: the 27 mission types, the simulator types and the final mission's two parts as mission definitions (compositions from the guide, else *derived* estimates) | `packs/interceptor` | **Started:** 5 of 27 — `xcom_base_defense`, `defend_freighter_convoy`, `deep_space_encounter`, `attack_alien_outpost`, `destroy_alien_probe`; original names, briefings and debriefs by `@orig` key; compositions derived (DEV-041). |
| 3.12 | `default`: the 11 generated mission types (2 on surfaces) and chapter missions 1, 2, 4, 5 playable from the testbed; terrain and sky assets generated | `resources/default`, `tools` | **Started:** 4 missions — `defend_station`, `convoy_ambush`, `construction_defence`, `intercept_scouts` — with text in `text/en/missions.yaml`. |
| 3.13 | **Logistics missions** (default campaign, [design](../../specification/design/logistics.md#mission-types-tied-to-logistics-stage-3)): `convoy_ambush` (generated when a supply convoy reaches the attack point chosen at random at its launch; protect it; cargo lost if destroyed, stored by the enemy if captured) and `construction_defence` (from 50 % construction, at most three per location; the facility in its construction-stage state); roles from the strategy situation; auto-resolve when the player does not fly; the campaign context queries they need | `mission`, `script` | **Partial:** `construction_defence` and `convoy_ambush` exist as missions flown from the launcher; generation from the strategy layer is Stage 4. |

**Testbed adds** (so far: missions on the launcher with scrolling, `--mission`, `--soak <mission>`): a mission browser (every mission type of both campaigns, with difficulty and
force choices), a terrain flight sandbox (atmospheric handling on several worlds), and a
condition-language console.

**Exit**: every mission type winnable and losable in the testbed; atmospheric flight feels
distinct from space (manual test); the language fully unit-tested (parser, types, evaluation,
error messages).
