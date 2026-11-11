# Stage 4 — Strategy layer

**Version 0.5.0. Status: planned.** The real-time strategy game around the battles: the
interactive star map, bases, research, the economy, ships and loadouts, pilots — with
functional screens and saves. Story and the rival's planning come in Stage 5.

Design: [strategy layer](../design/format-strategy.md), [interface](../design/format-interface.md).

## Work items

| # | Item | Module |
|---|---|---|
| 4.1 | Campaign clock: dates, time speeds, auto-pause; the `campaign` and `starts` definitions; starting a new game with a chosen campaign and difficulty | `campaign` (new) |
| 4.2 | Star map: generated (classes, bodies, names, hidden black holes; seeded) and fixed maps; free and lane travel; endurance and stranding | `campaign::map` |
| 4.3 | Squadrons, convoys and probes on flight plans; sensor coverage; contact detection; mission generation when they meet | `campaign` |
| 4.4 | Economy: resources, income and expense expressions, markets (buy/sell), factories and queues with priorities, deliveries and losses, storage | `campaign::economy` |
| 4.5 | Research: download and labs models, discovery by prerequisites, salvage and capture, unlocks | `campaign::research` |
| 4.6 | Bases: types, placement rules, templates, module construction (build times and construction states), limits, effects, destruction consequences, dismantling | `campaign::bases` |
| 4.7 | Ships and equipment: hulls, items, slots and fitting rules, power draw, stock designs, purchase and manufacture of craft | `campaign` |
| 4.8 | Pilots: generation, ranks, experience, training, injury and healing, bravery, recruiting, salaries, voices | `campaign::pilots` |
| 4.9 | Patron funding and evaluations; score and ratings; periodic reports | `campaign` |
| 4.10 | Screens (functional, plain skin): star map, bases with the 3D base view, research, hangar, market, pilots, reports, messages, briefing/debriefing; battle hand-off and return | `app::strategy` (new) |
| 4.11 | **Saves**: the full campaign state plus the mod list and versions; load refuses missing mods | `campaign::save` |
| 4.12 | `interceptor`: map generation tables, economy, the 71 research topics with authored unlocks, 68 modules with effects, items, pilots, salvage, funding evaluation, score names | `packs/interceptor` |
| 4.13 | `default`: the fixed six-system map, economy, both research trees, Guard modules and Dominion pods, items, pilots | `packs/default` |

**Testbed adds**: a strategy sandbox for each campaign with a debug clock, money and research
cheats (the original's own cheat list — `payday`, `knowitall`, `quickbase` — as debug actions),
and "jump to battle" from any contact.

**Exit**: both campaigns' strategy layers run for 6 game months without errors, with economy,
research and bases behaving as specified; save/load round-trips (unit tests on synthetic
campaigns; manual long-run test).
