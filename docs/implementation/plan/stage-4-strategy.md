# Stage 4 — Strategy layer

**Version 0.5.0. Status: in progress** — MD1 formations in app 0.6.6 (main `6d390d7`); the campaign context in 0.6.7 (`65141dc`); the core (strategy data, the campaign runtime, saves) in **0.7.0** (`77d6c9f`); the default campaign's strategy data and the strategy screen in **0.7.1** (`10f7fce`); fitting in **0.7.2** (`94d66c1`); lane travel in **0.7.3** (`076203e`); the `interceptor` strategy layer from the player's data in **0.8.0** (`c64d178`); the testbed cheats in **0.8.1** (`8e96338`); logistics in **0.9.0** (`ff9d8fc`); playtest changes in **0.9.1** (`3b19dea`) and flown supply-convoy battles in **0.9.2** (`0d1c522`, `146195b`); fitted shields and armour and `--balance` in **0.15.0** (`1ff754b`); the strategy screen rebuilt for the mouse in **0.16.0** (`ac7ae62`); the issue-audit gaps in **0.16.1 – 0.16.14** (`f82a6ab` … `de30c2a`) and the docs-review fixes in **0.16.15** (`00c7da5`); evaluation texts as text keys in **0.18.1** (`3ea5177`); issue hygiene (Batch 4.0) on 2026-11-20; bases and economy completeness (Batch 4.1) in **0.19.0** (`c28fc2c`). The real-time strategy game around the battles: the
interactive star map, bases, research, the economy, ships and loadouts, pilots — with
functional screens and saves. Story and the rival's planning come in Stage 5.

Design: [strategy layer](../../specification/design/format-strategy.md), [interface](../../specification/design/format-interface.md).

## Work items

| # | Item | Module | Status |
|---|---|---|---|
| 4.1 | Campaign clock: dates, time speeds, auto-pause; the `campaign` and `starts` definitions; starting a new game with a chosen campaign and difficulty | `campaign` (new) | **Done** (0.7.0 – 0.7.1): dates, pause and four speeds, auto-pause on notices; `start:`; new game at the launcher's difficulty. GitHub #21. |
| 4.2 | Star map: generated (classes, bodies, names, hidden black holes; seeded) and fixed maps; free and lane travel; endurance and stranding | `campaign::map` | **Partial** (0.7.0, 0.7.3): fixed and generated maps (classes, names, hidden black holes, seeded); free and **lane** travel (Dijkstra routes, saved, drawn). Open: bodies, endurance and stranding. Generated maps unit-tested since 0.16.2. #22. |
| 4.3 | Squadrons, convoys and probes on flight plans; sensor coverage; contact detection; mission generation when they meet | `campaign` | **Partial** (0.7.0): raids as contacts with sensor detection, interception and the battle hand-off; unopposed strikes; **auto-resolve** from both sides' strength (0.16.12, `01e2f2b`: `balance::win_chance`). Open: convoys, probes and squadrons on flight plans; generated missions (a raid flies its fixed mission). #23. |
| 4.4 | Economy: resources, income and expense expressions, markets (buy/sell), factories and queues with priorities, deliveries and losses, storage | `campaign::economy` | **Partial** (0.7.0, 0.9.0): income and expense expressions, market buy/sell, factory queues (first job first), deliveries; manufacture **paid per unit as it starts** (0.9.0 fix); resources and components with logistics (0.9.0). **Priorities** (*Move up*) and cancelling jobs, **purchase losses** (`economy.delivery_loss`) — 0.16.11 (`6821147`); **storage limits** on purchases (`storage` modules, item sizes) — 0.16.10 (`1bc200e`); since 0.16.15 (`00c7da5`) only a base type with hub `storage` limits its store (plus its `storage` modules). **Storage everywhere** (0.19.0, `c28fc2c`, Batch 4.1): purchases count what is on its way, manufacture of an item waits unpaid for room, a delivery without room waits at the base (`Delivery::held`, announced once); the default campaign's hub store of 120 and supply depot. **Done** except markets per seller, moved to Stage 5 (#46, D4). #24. |
| 4.5 | Research: download and labs models, discovery by prerequisites, salvage and capture, unlocks | `campaign::research` | **Partial** (0.7.0, 0.8.0, 0.9.0): download and labs, prerequisites, unlocks, salvage by raid **id** (0.9.0 fix); the original's numeric prerequisites through `tech_ids` (0.8.0); station-only topics (0.9.0). **Capture** (the `capture` outcome keeps the tractor's prizes as salvage, 0.16.3), **`requires_any`**, **hidden topics discovered by events** (`discover`, 0.16.6, `1093f0a`). Open: the original's per-item salvage tables. #25. |
| 4.6 | Bases: types, placement rules, templates, module construction (build times and construction states), limits, effects, destruction consequences, dismantling | `campaign::bases` | **Done** (0.7.0, 0.16.10, 0.19.0): types, templates, construction with build times, limits, effects (`production_rate`, `sensor_range`, `hangar`). **Dismantling** (`economy.dismantle` refund) and the **`quarters`**, **`storage`** and **`defence`** effects (0.16.10, `1bc200e`); dismantling refuses a hangar or quarters module whose craft or pilots would have nowhere to go (0.16.15). **Batch 4.1** (0.19.0, `c28fc2c`): a raid **wrecks** a module (no effects, no upkeep, keeps its slot) and `on_destroyed: [craft, pilots, stock]` loses what has no room; **repair** (`economy.repair`, 50 % of cost and build time); a wreck dismantled for nothing; **Dismantle base** (refund, craft rehoused or sold, stock moved or sold, deliveries and convoys redirected); template entries naming a **module group** (the best available); **`placement:`** conditions replacing `max_distance` (`base_distance(system)`). The 3D base view is 6.9 work (#101). #26. |
| 4.7 | Ships and equipment: hulls, items, slots and fitting rules, power draw, stock designs, purchase and manufacture of craft | `campaign` | **Partial** (0.7.0, 0.7.2, 0.15.0): hulls and items as data; craft bought or manufactured; **fitting** best-available from the store, flown in campaign battles (`rounds:`); **shields and armour** fitted and flown (the default campaign's own shields and armour, two research topics and a slot of each on every hull since 0.16.1, `f82a6ab`) (item `protection`, the `Loadout` struct, `fitted:` on ship classes reading the original's equipment, one shield and one armour slot on the X-COM hulls — 0.15.0, `1ff754b`). Open: choosing a fit slot by slot, fitting rules (`fits`), power draw, stock designs. #27. |
| 4.8 | Pilots: generation, ranks, experience, training, injury and healing, bravery, recruiting, salaries, voices | `campaign::pilots` | **Partial** (0.7.0): generation, ranks by experience, injury and healing, survival, recruiting, salaries. **Training** (`pilots.training`) and **dismissal** (0.16.7, `2c152a8`). Open: bravery, stat effects in battle, voices per pilot. #28. |
| 4.9 | Patron funding and evaluations; score and ratings; periodic reports | `campaign` | **Done** (0.7.0, 0.16.8 – 0.16.9): scores, the score-driven stipend, the monthly report; **evaluations** (`evaluations: [{rating, when, do}]`) rating each month and running its actions; the **Reports** list (`26f6c2a`, `d0c64e3`); `@keys` in `evaluations[].do` resolved since 0.18.1 (`3ea5177`). **#29 closed** 2026-11-20; the X-COM evaluation rules go on in #45, the finances screen in #106 (6.9.9). |
| 4.10 | Screens (functional, plain skin): star map, bases with the 3D base view, research, hangar, market, pilots, reports, messages, briefing/debriefing; battle hand-off and return | `app::strategy` (new) | **Done as the functional screen** (0.7.1 – 0.16.15; **#30 closed** 2026-11-20): one functional screen — map, ten lists (eleven with Reports, 0.16.8) (the **Supply** list and facility, convoy and survey markers in 0.9.0), clock, battle hand-off and return; launcher entries and `--strategy`. **Rebuilt for the mouse in 0.16.0** (`ac7ae62`): a top bar with buttons, a map that zooms, pans and picks, an info panel with action buttons, scrolling lists with per-row buttons, modal prompts; layout as pack data (`screens.starmap`, 6.1). Since 0.16.2 – 0.16.14: the **Reports** list (0.16.8), the strike prompt (0.16.4), *Auto-resolve* (0.16.12), the Debug list (0.16.13), save slots (0.16.14); the info panel shows up to six actions (0.16.1); the Hangar's pilot rows show each pilot's stats (0.16.15). It stays as the testbed layout. The leftovers are 6.9 work: the 3D base view #101, reports #106, the encyclopaedia #107, briefing #108, screens per list #98 – #105, the testbed-layout switch #111, the supply-traffic view #99; strategy texts as text keys go to Batch 4.5 (DEV-047). |
| 4.11 | **Saves**: the full campaign state plus the mod list and versions; load refuses missing mods | `campaign::save` | **Done** (0.7.0, 0.9.0): the whole state as YAML with packs and versions; refuses missing packs and mismatched rules; the logistics state (0.9.0). Versions compared since 0.16.2 (a changed pack version loads with a notice); save slots 1 – 3 and a Load game page since 0.16.14 (`de30c2a`, #48). #31. |
| 4.12 | `interceptor`: map generation tables, economy, the 71 research topics with authored unlocks, 68 modules with effects, items, pilots, salvage, funding evaluation, score names | `packs/interceptor` | **Done as derived** (0.8.0, `c64d178`; not play-tested): `tools/interceptor_strategy.py` generates `references.yaml` (the executable's tech-id table, *derived*, and `inherit:` references for 71 topics, 55 items, 34 X-COM modules); `campaign.yaml` authors the Frontier map, downlink research, hulls, module groups and effects, the X-COM base, funding, raids, pilots, scores, every estimate `DERIVED` (DEV-050). Engine keys `tech_ids`, `economy.cost_scale`, `economy.module_time`, `start.money_from`. Six-month soak: 4 of 71 topics. Open: play-testing and balance; unlocks are not authored (TechRequired gates instead); evaluations with Stage 5. #32. |
| 4.13 | `default`: the fixed six-system map, economy, both research trees, Guard modules and Dominion pods, items, pilots | `resources/default` | **Partial** (0.7.1, 0.9.0): the Reach map, a 12-topic tree plus two station-only topics, items, hulls, nine module kinds, the Guard base, the Senate stipend, five raids, pilots, scores; the logistics data (0.9.0). Since 0.16.1 four shield and armour items and the topics `force_fields`, `saucer_alloys`; 0.16.4 construction strikes; 0.16.7 training; 0.16.8 the Senate's evaluations. Open: the Dominion's own tree and pods; texts as text keys (DEV-047). #33. |
| 4.14 | **Logistics** (default campaign, [design](../../specification/design/logistics.md)): resources, components and deposits (campaign data); `facility_types` mining facility, research station, manufacturing center with placement conditions, build cost, time and three construction stages; the economy tick (extraction, recipes that start only when their inputs are in store, research, construction progress) | `campaign::economy` | **Partial** (0.9.0, `ff9d8fc`): resources, components, deposits and features (data); the three facility types with fixed placement rules per role (DEV-049), cost and build time; hourly extraction, recipes that start only on inputs in store, station research, construction time. **Batch 4.1** (0.19.0, `c28fc2c`): `placement:` **conditions** on facility types, each role's default rule a condition (`Role::default_placement`; DEV-049's placement part closed); **storage at manufacturing centers and research stations** — orders only what fits, a recipe starts only when its output fits. Open: the three construction stages shown (`hub` read, not used; Batch 4.2), facility upgrades. #34. |
| 4.15 | **Supply and demand**: the order book (main base ← manufacturing ← mining; research ← mining once per project), automatic government supply convoys on fixed lanes (several vessels, even cargo shares), the attack point chosen at random at launch, delivery and loss/capture rules; mission-generation hooks for ambushes and construction strikes | `campaign::logistics`, `campaign::convoys` | **Partial** (0.9.0): the order book (base hull jobs → manufacturing → mining; station start costs → mining), nearest supplier by straight-line distance (DEV-049); automatic convoys along the lanes, `logistics.vessels` each with an even share; the ambush roll and point at launch; auto-resolved vessel losses with the surviving share delivered. **Flown ambushes** (0.9.2, `0d1c522`, `146195b`; the user's design): with `logistics.ambush_mission` an ambushed convoy waits for the player — fly its escort (`convoy_ambush`) and the share of cargo still aboard (`cargo:`) is delivered, or let it fight through (`Campaign::convoy_result`). In battle the enemy captures pods and clippers with tugs; the campaign counts them as lost. **Construction strikes** (0.16.4, `d300642`): `logistics.construction_strikes` from `from` progress, at most `max` per location, `chance` a day; fly `construction_defence` or let it set the build back (`Event::Strike`, `Campaign::strike_result`). **Done** except depots and capture: captured cargo stored at enemy depots and seized back moved to Stage 5 (5.11, #46). **#35 closed** 2026-11-20; #19. |
| 4.16 | **Survey ships** (a separate mission profile and craft; within the launching station's radar range; 25 % attack roll at launch; suitability per facility type), **research sites** and station-only tech branches (`lab:`, `site:`, `start_cost` — materials only to begin), **facility fighter pools** (built at main bases, transferred, movable) | `campaign` | **Partial** (0.9.0): survey ships within the base's radar range, the attack roll at launch (`survey_attack_chance`), reports of deposits (with reserves since 0.16.2), features and sites; research sites and station-only topics (`lab`, `site`, `start_cost` once). Open: a survey craft and a battle when attacked (auto-resolved), **facility fighter pools**. #36. |
| 4.17 | Facility, survey and supply-traffic screens (functional); `default`: resources, components, deposits, facility types, recipes, research sites and branches (the stage models, facility objects and supply craft exist since Stage 2) | `app::strategy`, `resources/default` | **Partial** (0.9.0, 0.9.2): the Supply tab (survey, build, station research, stores, counts) and map markers; the default campaign's `strategy/logistics.yaml` (four resources, three components and recipes, the three facility types, two research sites, deposits in all six systems, hull components, two station-only topics). **The supply vessel** (0.9.2): the cargo clipper is the core vessel with twelve cargo pods and a point-defence laser as separate modules (`supply_pod`, `supply_pd`; R-STR-017). The supply-traffic view moved to the CSD screen (6.9.2, #99) on 2026-11-20. Open: **cargo capacity** (R-STR-026, decision D1, Batch 4.2) — #37, renamed *4.17 Supply convoy cargo capacity (R-STR-026)*. |

**Testbed adds** — **Done** (0.8.0 – 0.8.1, `c64d178`, `8e96338`; the Stage 5 debugger and start-at-chapter in 0.16.13, `52e9e7f`): every campaign's strategy screen is
its sandbox; the original's own cheats as debug keys — Ctrl+M `payday`, Ctrl+K `knowitall`,
Ctrl+B `quickbase` — and Ctrl+J "jump to battle" from the chosen (detected) contact
(`Campaign::cheat_*`); `--strategy-soak <days>` plays a strategy layer headless
(`campaign::autoplay`). MT-4-20, MT-4-21; GitHub #47 (R-TST-010, R-TST-011, partly met).

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

**Exit** (status at app 0.19.0, main `c28fc2c`: both strategy layers run six months headless —
`default` 14 of 16 topics and 47 battles, `interceptor` 4 of 71 and 1 battle, every store within
its capacity in both — and save/load round-trips in the six-month unit test; the manual long run
and the `interceptor` play-test are outstanding): both campaigns' strategy layers run for 6 game months without errors, with economy,
research and bases behaving as specified; save/load round-trips (unit tests on synthetic
campaigns; manual long-run test).

## Remaining work in batches

Written 2026-11-20 from the digest of the open Stage 4 issues; code checked at `6462799` (app 0.17.1).
Copied here from the code repository's `plan/stage-4-batches.md` on 2026-11-20 (Stage 3 closed in app
0.18.0); this section is now the checklist of record.

**Why batches.** Each batch groups issues that touch the same modules and data, so one session can load that
context once, finish the batch, and hand off. **Start every batch in a fresh session** (`/clear`) with the
batch's *start prompt*. Only this checklist and the issues carry state between batches.

### Batch protocol (every batch)

**Start**

- [ ] Fresh session. Read `CLAUDE.md`, this page's batch section, and the batch's issues (`gh issue view <n> --comments`).
- [ ] Check the vault for the batch's topic (`sio-vault context "<terms>" --project openinterceptor --brief`).
- [ ] Confirm the batch's **decisions** below are answered. If not, ask before coding.

**Work**

- [ ] Logic first, in `campaign`/`content`, with unit tests (synthetic data only). UI changes only where the batch says.
- [ ] Every new key works in the `default` campaign. The `interceptor` pack is mapped where the original has the data.
- [ ] `git add -A`, then `OPENINTERCEPTOR_DATA=resources/xcom-cd ./scripts/check.sh` → `checks: passed`.
- [ ] Bump the version (patch per change set, minor per feature batch). Commit with the attribution line. Push.

**Close**

- [ ] Docs pass: run one background docs agent with the commit list. It updates the Stage 4 plan page,
      Modding pages for new keys, architecture/campaign, requirements status, deviations, manual tests (MT-4-*)
      and the manual-test issues #88–#92. It also comments on the implementation issues.
- [ ] Close finished issues, or comment with what remains. Tick this checklist and add a one-line hand-off under the batch.
- [ ] Vault: one log entry and any captures (decisions, root causes).


### Batch 4.0 — issue hygiene (small; do first, can share a session with 4.1)

- [x] Close **#29** (4.9 done in 0.16.8–0.16.9). Pointers: X-COM evaluation rules → #45, finances screen → #106.
- [x] Close **#35** (construction strikes done in 0.16.4 `d300642`). Pointer: depots and captured cargo → #46.
- [x] Close **#30** with pointers: base view #101, reports #106, encyclopaedia #107, briefing #108, per-list screens
      #98–#105, testbed grid #111, traffic view → #99, text keys → Batch 4.5.
- [x] Correct stale comments:
    - #27: the `protection_of` test exists, and default shields and armour were added in 0.16.1.
    - #22: the generated-map test exists.
    - #36: reserves and features in the survey report are done.
- [x] Merge **#37**'s traffic view into #99. #37 keeps cargo capacity only.
- [x] Update the Stage 4 plan page statuses to match.

Hand-off: done 2026-11-20 — #29, #30, #35 closed with pointers; #22, #27, #36 corrected; #37 renamed to cargo capacity (traffic view → #99); plan statuses updated (site 0.27.0). Next: Batch 4.1 or 4.2.

### Batch 4.1 — bases and economy completeness (M; no blockers)

Issues: **#24** (storage part), **#34** (storage at facilities), **#26** (lifecycle).

- [x] Storage enforced on **deliveries and manufactured output**, not only on purchase (#24, R-STR-004).
- [x] Storage at **manufacturing centres and research stations**. Recipe output is checked against `ft.storage` (#34, R-STR-014).
- [x] **Dismantle a whole base**; **`on_destroyed`** consequences (craft, pilots, stock lost or moved); **repair** (#26, R-STR-006).
- [x] **Best-available base templates**: a template picks the best module researched so far, not a fixed list (#26).
- [x] **`placement:` expressions** for bases (#26) and facilities (#34). They replace `max_distance` and the fixed
      rules per role, and close DEV-049's placement part.
- [x] Save format: new state in `campaign/save.rs`, with a round-trip test.

Files: `src/campaign/mod.rs` (storage, buy/deliver, build_base, dismantle_module, hourly step),
`src/campaign/logistics.rs` (~320–380), `src/content/{strategy,logistics}.rs`, `src/campaign/save.rs`,
`resources/default/strategy/{economy,logistics}.yaml`, `packs/interceptor/strategy/`.
Done when: the gate passes; a soak of 180 days on both campaigns shows no stock above capacity; #34's storage
item and #26's lifecycle items are ticked on the issues.
Start prompt: *"Batch 4.1 of plan/stage-4-batches.md: bases and economy completeness. Follow the batch protocol."*

Hand-off: done 2026-11-20 — app 0.19.0 (main `c28fc2c`): storage on purchases, manufacture, deliveries and every facility; wrecks with `on_destroyed`, repair, Dismantle base; group templates; `placement:` conditions (`max_distance` removed — a breaking change for mods); 443 unit tests; `--strategy-soak 180`: `default` 14/16 topics, 47 battles, `interceptor` 4/71, 1 battle, every store within its capacity in both. Docs site 0.28.0, Spec 1.21.0 (R-STR-006, R-STR-014 implemented; MT-4-50 – MT-4-54). Open: the `interceptor` hub's cargo capacity (no hub `storage`; unknown). Next: Batch 4.2.

### Batch 4.2 — logistics leftovers (M; parts blocked by decisions D1, D3)

Issues: **#34** (R-STR-022), **#37** (R-STR-026), **#36** (survey), **#47** (logistics soak).

- [ ] **Construction stages shown** (R-STR-022): use `facility_types.hub` and the `build_25/60/90` states on the map
      marker and in the `construction_defence` battle (the struck facility's own type and stage). This also fixes
      the default mission always showing the same station.
- [ ] **Cargo capacity as data** (R-STR-026): convoys carry up to capacity and split larger orders. *Needs D1.*
- [ ] **Survey report suitability** for each facility type (R-STR-021).
- [ ] **Attacked survey ship**: the survey craft plus an auto-resolved or flown battle, instead of the fixed 50 % loss. *Needs D3.*
- [ ] **Logistics soak** (#47): autoplay builds facilities and the soak reports throughput and losses.

Files: `src/campaign/logistics.rs`, `src/content/logistics.rs`, `src/app/strategy.rs` (Supply tab, markers),
`src/app/flight.rs` (stage objects), `resources/default/strategy/logistics.yaml`, `tools/default_assets/`.
Done when: construction-stage visuals are confirmed by screenshots (manual cases MT-4-*); the logistics soak runs.
Start prompt: *"Batch 4.2 of plan/stage-4-batches.md: logistics leftovers. Follow the batch protocol."*

Hand-off: _

### Batch 4.3 — map, movement and contacts (L; gates CSD parity #99/#100; needs D5)

Issues: **#22**, **#23**, **#36** (fighter pools), **#47** (force-based soak battles). Order inside the batch:
endurance → flight plans → pools → generated missions → auto-resolve.

- [ ] **Bodies** in play and **endurance / stranding** per craft (#22, R-STR-002). *Needs D5.*
- [ ] **Squadrons on flight plans** (patrol, launch to a destination, return) and **probes as map objects** (#23, R-STR-003).
- [ ] **Facility fighter pools** (R-STR-025): built at main bases, transferred, defending a system and escorting
      convoys (#36). Reuses the squadron model.
- [ ] **Missions generated from the forces that meet**, replacing each raid's fixed `mission:` (#23).
- [ ] **Auto-resolve uses loadouts and pilots**, and losses damage craft (`balance::win_chance`; known small issue).
      Soak battles are resolved from forces (#47).
- [ ] Save format and round-trip tests for squadrons, flight plans and pools.

Files: `src/campaign/mod.rs` (contacts, interceptions, lane_route, autoplay), `src/campaign/logistics.rs`
(escorts), `src/content/{strategy,mission}.rs`, `src/balance.rs`, `src/campaign/save.rs`, the map and info panel in
`src/app/strategy.rs`. Split across two sessions if needed: **4.3a** (endurance and flight plans), then
**4.3b** (pools, generated missions, auto-resolve).
Start prompt: *"Batch 4.3a of plan/stage-4-batches.md: …"*

Hand-off: _

### Batch 4.4 — ships and pilots in battle (M–L; after Track A stat fidelity; gates #105/#103)

Issues: **#27**, **#28**.

- [ ] **Slot-by-slot fitting** (replacing `fit_best` as the only path), **`fits` rules**, **power draw**, **stock
      designs** (#27, R-STR-007).
- [ ] **Battle effects of the other slot kinds** (engine, sensors/targeting, ECM, cloak, tractor, VCT, damage
      control). `Loadout` grows beyond beam/missile/shield/armour. Uses Track A's readings of the INI keys.
- [ ] **Pilot stats in battle**: accuracy and flying feed `sim::ai` / handling; **bravery** (fleeing); **graded injuries** (#28, R-STR-008).
- [ ] **Rank thresholds** from `RankAdvancement*` (interceptor) and as data in default (see `notes/re/ini-files.md` §1.10).
- [ ] Per-pilot voices: deferred until #50 maps the clips (decision D7).

Files: `src/campaign/mod.rs` (Loadout, fit_best, pilots), `src/content/strategy.rs` (items, hulls, slots, pilots),
`src/app/flight.rs` (apply_loadout), `src/sim/ai.rs`, `src/balance.rs`, `packs/interceptor/strategy/campaign.yaml`,
`resources/default/strategy/{economy,pilots}.yaml`.
Start prompt: *"Batch 4.4 of plan/stage-4-batches.md: ships and pilots in battle. Follow the batch protocol."*

Hand-off: _

### Batch 4.5 — content and data (M; parts need D2 and RE)

Issues: **#33**, **#25**, **#32**; [DEV-047](../deviations/index.md).

- [ ] **Text keys for strategy texts** (DEV-047). Also removes the English-prefix matching for the research
      stinger and the Reports list (`app/strategy.rs` ~829, ~1424).
- [ ] **Default: the Dominion's own research tree and pods** (#33).
- [ ] **Salvage tables**: `*Goodies` percentages by difficulty for interceptor (`notes/re/ini-files.md` §1.11),
      and a per-item salvage table format for default. Replaces `salvage_per_kill` (#25, #45).
- [ ] **Interceptor research pace** set from D2, then play-test notes (#32). Close #32 or re-scope it.

Files: `resources/default/strategy/*.yaml`, `src/content/{text,strategy}.rs`, `src/app/strategy.rs`,
`src/campaign/story.rs`, `packs/interceptor/strategy/`, `tools/interceptor_strategy.py`.
Start prompt: *"Batch 4.5 of plan/stage-4-batches.md: content and data. Follow the batch protocol."*

Hand-off: _

### Moved out of Stage 4

- **Per-seller market** (#24 remainder) and **depots / captured cargo / seizure** (#35 remainder) → **Stage 5,
  #46 / #40** (needs relations and D4).

- **Screens** (#30 leftovers, 3D base view, traffic view) → **6.9** issues, after their logic batch.

### Order

```
4.0 hygiene ─┬─ 4.1 bases/economy ─┐
             └─ 4.2 logistics ──────┼─ 4.3a/b map & contacts ─ (6.9 #99/#100 CSD + dialogs)
   Track A stats ───────────────────┴─ 4.4 ships & pilots ──── (6.9 #105/#103 hangar, pilots)
                                       4.5 content (any time after 4.0; D2 for #32)
```

4.1 and 4.2 can run in parallel with Track A, since they touch different modules.

### Decisions needed (ask before the batch that needs them)

| # | Decision | Blocks | Suggested default |
|---|---|---|---|
| D1 | Supply convoy **cargo capacity** (R-STR-026; logistics.md open question 1) | 4.2 cargo | A capacity per vessel type in data; orders split into several convoys |
| D2 | **Interceptor research pace** (#32: 4 of 71 topics in 180 days) | 4.5 #32 | Calibrate so a normal campaign researches most topics in about 18–24 months (check against the Sybex guide) |
| D3 | The **survey ship's craft**, and where and in which mission an attacked survey ship fights (open question 3) | 4.2 survey battle | A light survey hull plus the existing `convoy_ambush` shape at the survey site; auto-resolved unless the player flies |
| D4 | What a **market seller** is (per faction or per station; relations set prices) | Stage 5 | Per faction, prices scaled by relation |
| D5 | **Bodies in play** and the **endurance model** (fuel per hull vs range per lane hop) | 4.3a | Range per hull in lane hops; stranded craft drift until recovered |
| D6 | Logistics open questions 2 and 4 (ambush chance, vessels per convoy): **confirm** the current data defaults | none (confirm) | Keep `ambush_chance` and `vessels` as they are |
| D7 | Interceptor **pilot voice mapping** (#50) | 4.4 per-pilot voices | Defer |
