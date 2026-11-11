# Stage 4 — Strategy layer

**Version 0.5.0. Status: in progress** — MD1 formations in app 0.6.6 (main `6d390d7`); the campaign context in 0.6.7 (`65141dc`); the core (strategy data, the campaign runtime, saves) in **0.7.0** (`77d6c9f`); the default campaign's strategy data and the strategy screen in **0.7.1** (`10f7fce`); fitting in **0.7.2** (`94d66c1`); lane travel in **0.7.3** (`076203e`). The real-time strategy game around the battles: the
interactive star map, bases, research, the economy, ships and loadouts, pilots — with
functional screens and saves. Story and the rival's planning come in Stage 5.

Design: [strategy layer](../../specification/design/format-strategy.md), [interface](../../specification/design/format-interface.md).

## Work items

| # | Item | Module | Status |
|---|---|---|---|
| 4.1 | Campaign clock: dates, time speeds, auto-pause; the `campaign` and `starts` definitions; starting a new game with a chosen campaign and difficulty | `campaign` (new) | **Done** (0.7.0 – 0.7.1): dates, pause and four speeds, auto-pause on notices; `start:`; new game at the launcher's difficulty. GitHub #21. |
| 4.2 | Star map: generated (classes, bodies, names, hidden black holes; seeded) and fixed maps; free and lane travel; endurance and stranding | `campaign::map` | **Partial** (0.7.0, 0.7.3): fixed and generated maps (classes, names, hidden black holes, seeded); free and **lane** travel (Dijkstra routes, saved, drawn). Open: bodies, endurance and stranding; no unit test of generated maps. #22. |
| 4.3 | Squadrons, convoys and probes on flight plans; sensor coverage; contact detection; mission generation when they meet | `campaign` | **Partial** (0.7.0): raids as contacts with sensor detection, interception and the battle hand-off; unopposed strikes. Open: convoys, probes and squadrons on flight plans; generated missions (a raid flies its fixed mission). #23. |
| 4.4 | Economy: resources, income and expense expressions, markets (buy/sell), factories and queues with priorities, deliveries and losses, storage | `campaign::economy` | **Partial** (0.7.0): income and expense expressions, market buy/sell, factory queues (first job first), deliveries. Open: resources, priorities, losses, storage limits; **manufacture charges no money** (defect). #24. |
| 4.5 | Research: download and labs models, discovery by prerequisites, salvage and capture, unlocks | `campaign::research` | **Partial** (0.7.0): download and labs, prerequisites, unlocks, salvage by raid. Open: capture, events; **`salvaged(x)` compares names with ids** (defect). #25. |
| 4.6 | Bases: types, placement rules, templates, module construction (build times and construction states), limits, effects, destruction consequences, dismantling | `campaign::bases` | **Partial** (0.7.0): types, `max_distance` placement, templates, construction with build times, limits, effects (`production_rate`, `sensor_range`, `hangar`), a module lost to an unopposed raid. Open: dismantling, `on_destroyed`, repair, other effects. #26. |
| 4.7 | Ships and equipment: hulls, items, slots and fitting rules, power draw, stock designs, purchase and manufacture of craft | `campaign` | **Partial** (0.7.0, 0.7.2): hulls and items as data; craft bought or manufactured; **fitting** best-available from the store, flown in campaign battles (`rounds:`). Open: choosing a fit, fitting rules, power draw, stock designs. #27. |
| 4.8 | Pilots: generation, ranks, experience, training, injury and healing, bravery, recruiting, salaries, voices | `campaign::pilots` | **Partial** (0.7.0): generation, ranks by experience, injury and healing, survival, recruiting, salaries. Open: training, bravery, voices per pilot. #28. |
| 4.9 | Patron funding and evaluations; score and ratings; periodic reports | `campaign` | **Partial** (0.7.0): scores, the score-driven stipend, the monthly report. Open: evaluations and ratings, a report screen. #29. |
| 4.10 | Screens (functional, plain skin): star map, bases with the 3D base view, research, hangar, market, pilots, reports, messages, briefing/debriefing; battle hand-off and return | `app::strategy` (new) | **Partial** (0.7.1): one functional screen — map, eight tabs, clock, banners, battle hand-off and return; launcher entries and `--strategy`. Open: the 3D base view, reports, briefing, encyclopaedia; themed look (Stage 6). #30. |
| 4.11 | **Saves**: the full campaign state plus the mod list and versions; load refuses missing mods | `campaign::save` | **Done** (0.7.0): the whole state as YAML with packs and versions; refuses missing packs and mismatched rules. Versions are recorded, not compared; logistics state joins when built. #31. |
| 4.12 | `interceptor`: map generation tables, economy, the 71 research topics with authored unlocks, 68 modules with effects, items, pilots, salvage, funding evaluation, score names | `packs/interceptor` | **Open**: not started. The original's research prerequisites are numeric tech ids that do not map to its research sections in file order; a tech-id table must be worked out first. #32. |
| 4.13 | `default`: the fixed six-system map, economy, both research trees, Guard modules and Dominion pods, items, pilots | `resources/default` | **Partial** (0.7.1): the Reach map, a 12-topic tree, items, hulls, nine module kinds, the Guard base, the Senate stipend, five raids, pilots, scores. Open: the Dominion's own tree and pods, logistics data; texts as text keys (DEV-047). #33. |
| 4.14 | **Logistics** (default campaign, [design](../../specification/design/logistics.md)): resources, components and deposits (campaign data); `facility_types` mining facility, research station, manufacturing center with placement conditions, build cost, time and three construction stages; the economy tick (extraction, recipes that start only when their inputs are in store, research, construction progress) | `campaign::economy` | **Open** (#34). |
| 4.15 | **Supply and demand**: the order book (main base ← manufacturing ← mining; research ← mining once per project), automatic government supply convoys on fixed lanes (several vessels, even cargo shares), the attack point chosen at random at launch, delivery and loss/capture rules; mission-generation hooks for ambushes and construction strikes | `campaign::logistics`, `campaign::convoys` | **Open** (#35). |
| 4.16 | **Survey ships** (a separate mission profile and craft; within the launching station's radar range; 25 % attack roll at launch; suitability per facility type), **research sites** and station-only tech branches (`lab:`, `site:`, `start_cost` — materials only to begin), **facility fighter pools** (built at main bases, transferred, movable) | `campaign` | **Open** (#36). |
| 4.17 | Facility, survey and supply-traffic screens (functional); `default`: resources, components, deposits, facility types, recipes, research sites and branches (the stage models, facility objects and supply craft exist since Stage 2) | `app::strategy`, `resources/default` | **Open** (#37). |

**Testbed adds** (not built yet): a strategy sandbox for each campaign with a debug clock, money and research
cheats (the original's own cheat list — `payday`, `knowitall`, `quickbase` — as debug actions),
and "jump to battle" from any contact.

Requested logistics for the default campaign (4.14–4.17) are an engine capability that the
`interceptor` campaign deliberately does not use (it stays faithful to the original; an addon
could add it). It is all YAML data and a campaign may leave it out. Decisions and the one open
question (cargo capacity): [Logistics](../../specification/design/logistics.md#decisions-2026-11-11).

## Mod and cutscene designer (MD1)

From the [mod designer, cutscene designer and game bundling](../../specification/design/designer.md) design (decisions D1–D5
made 2026-11-11). The design places MD1 at "Stage 3 / 4"; it is planned here, in Stage 4. It
comes first because it **also serves wingmen**: battles use it before any cutscene does.

| # | Item | Module | Status |
|---|---|---|---|
| MD1 | **Shared formations** — **Done** (app 0.6.6, main `6d390d7`; GitHub #64): `formations:` in both campaigns, `formation:` on mission roles, successive slots, Shift+O; cutscene use with MD2: `formations:` pack data (vee, line abreast, echelon, box, diamond, custom), used by wingmen first; replaces today's per-scenario slot offsets; keeps the `flight.formation_gap` spacing ([R-CMB-022](../../specification/requirements/combat.md#r-cmb-022-wingmen-keep-their-distance)). Later used by cutscene formations (MD2) | `content`, `sim::ai` | [R-MOD-047](../../specification/requirements/modding.md#r-mod-047-shared-formations), [R-MOD-039](../../specification/requirements/modding.md#r-mod-039-formations-in-cutscenes) |

**Exit**: both campaigns' strategy layers run for 6 game months without errors, with economy,
research and bases behaving as specified; save/load round-trips (unit tests on synthetic
campaigns; manual long-run test).
