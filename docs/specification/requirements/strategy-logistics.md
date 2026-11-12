---
spec_version: 1.14.0
status: current
updated: 2026-11-11
---

# Strategy & logistics requirements

Requirements for the strategy layer and logistics. The normative designs are [Format: strategy layer](../design/format-strategy.md) and [Logistics](../design/logistics.md).

Features: [Strategy & logistics](../../definition/strategy-logistics.md). How to read a requirement: [Specification](../index.md#reading-a-requirement).

| Requirement | Status |
|---|---|
| [R-STR-001 Campaign clock](#r-str-001-campaign-clock) | Implemented |
| [R-STR-002 Star map and travel](#r-str-002-star-map-and-travel) | Planned (partly met) |
| [R-STR-003 Contacts and mission generation](#r-str-003-contacts-and-mission-generation) | Planned (partly met) |
| [R-STR-004 Economy](#r-str-004-economy) | Planned (partly met) |
| [R-STR-005 Research](#r-str-005-research) | Planned (partly met) |
| [R-STR-006 Bases](#r-str-006-bases) | Planned (partly met) |
| [R-STR-007 Ships and equipment](#r-str-007-ships-and-equipment) | Planned (partly met) |
| [R-STR-008 Pilots](#r-str-008-pilots) | Planned (partly met) |
| [R-STR-009 Funding, score and reports](#r-str-009-funding-score-and-reports) | Planned (partly met) |
| [R-STR-010 Strategy screens](#r-str-010-strategy-screens) | Planned (partly met) |
| [R-STR-011 Saves](#r-str-011-saves) | Implemented |
| [R-STR-012 Logistics is optional data](#r-str-012-logistics-is-optional-data) | Implemented |
| [R-STR-013 Demand-driven orders](#r-str-013-demand-driven-orders) | Implemented |
| [R-STR-014 Extraction and recipes](#r-str-014-extraction-and-recipes) | Planned (partly met) |
| [R-STR-015 Government supply ships](#r-str-015-government-supply-ships) | Implemented |
| [R-STR-016 Supply convoys](#r-str-016-supply-convoys) | Implemented |
| [R-STR-017 Supply vessel model](#r-str-017-supply-vessel-model) | Planned |
| [R-STR-018 Supply convoy attack position](#r-str-018-supply-convoy-attack-position) | Implemented |
| [R-STR-019 Logistics mission generation](#r-str-019-logistics-mission-generation) | Planned (partly met) |
| [R-STR-020 Cargo losses and depots](#r-str-020-cargo-losses-and-depots) | Planned (partly met) |
| [R-STR-021 Survey ships](#r-str-021-survey-ships) | Planned (partly met) |
| [R-STR-022 Construction stages](#r-str-022-construction-stages) | Planned |
| [R-STR-023 Construction attacks](#r-str-023-construction-attacks) | Planned |
| [R-STR-024 Research stations and sites](#r-str-024-research-stations-and-sites) | Implemented |
| [R-STR-025 Facility fighter pools](#r-str-025-facility-fighter-pools) | Planned |
| [R-STR-026 Supply cargo capacity](#r-str-026-supply-cargo-capacity) | Planned |
| [R-STR-027 Third-party convoys](#r-str-027-third-party-convoys) | Planned |

## Requirements

### R-STR-001 Campaign clock

The strategy layer MUST run on a campaign clock with dates (Gregorian, from the campaign's `start.date`), time speeds and auto-pause, start from the campaign's `start` definition, and start a new game with a chosen campaign and difficulty. As built:

- Time passes only through `Campaign::advance`, in steps of at most one hour, so the same orders from the same state give the same game.
- The screen offers pause and four speeds (1 hour, 6 hours, 1 day and 5 days of campaign time per real second). The clock MUST stop by itself whenever the campaign reports a notice (construction or research finished, a contact detected, a raid striking unopposed, the monthly report) or an interception reaching its contact.
- A new game starts at the launcher's difficulty level with a seed from the system clock; the start places the starting bases (finished), craft, pilots (who take the craft in order), known research and stock.
- A campaign without `start:` MUST have no strategy layer.

*Rationale:* Real-time strategy.

- Implements: [F-STR-001](../../definition/strategy-logistics.md#f-str-001-strategy-layer)
- Status: implemented (app 0.7.0 – 0.7.1, main `77d6c9f`, `10f7fce`)
- Verified by: `campaign::tests::a_new_campaign_has_its_start`, `same_seed_same_campaign`, `the_default_campaign_runs_six_months`; `content::strategy::tests::dates_count_days_across_months_and_leap_years`; `app::launcher::tests::menu_lists_viewer_and_scenarios_and_wraps`; MT-4-02, MT-4-03
- See: [Strategy layer (modding)](../../modding/strategy.md#start), [Campaign (architecture)](../../implementation/architecture/campaign.md), [User guide › Campaign](../../user-guide/campaign.md)

!!! note "As built (app 0.7.0 – 0.7.1)"
    The design's `starts` definition is the single section `start:` (date, money, map, bases, craft, pilots, research, stock); one start per campaign (DEV-046). The difficulty is chosen in the launcher, not on a new-game screen.

### R-STR-002 Star map and travel

The star map MUST be generated (classes, bodies, names, hidden black holes; seeded) or fixed, with free or lane travel, endurance and stranding.

*Rationale:* Both campaigns' maps.

- Implements: [F-STR-001](../../definition/strategy-logistics.md#f-str-001-strategy-layer)
- Status: planned — partly met (app 0.7.0 – 0.7.3, main `77d6c9f`, `10f7fce`, `076203e`)
- Verified by: `content::strategy::tests::a_strategy_layer_builds_with_checked_names`; `campaign::tests::a_new_campaign_has_its_start` (a hidden system beyond sensor range), `lane_maps_route_along_lanes`; `app::strategy::tests::the_map_fits_its_rectangle_centred`; MT-4-02, MT-4-11
- See: [Strategy layer › maps](../../modding/strategy.md#maps-and-star_classes)

!!! note "As built (app 0.7.0 – 0.7.1)"
    `maps: {<id>: {kind: fixed | generated, …}}` with `star_classes`. Fixed maps list systems (`name`, `pos`, `class`, `hidden`) and `lanes`; generated maps scatter `systems: [min, max]` systems in a `size` rectangle, at least a little apart, with star classes by `weight`, names from `names` (then numbered), `black_holes` hidden systems, and the campaign's seed unless the map fixes its own. Hidden systems appear when a base's sensor range reaches them. The default campaign's map `reach` is fixed: six systems, seven lanes and the hidden dark moon. **Lane travel** (app 0.7.3): on a `travel: lanes` map raids enter at the free system farthest from their target and follow the shortest lane route (Dijkstra, `Campaign::lane_route`); interceptions fly the lanes to the system nearest the contact, then straight to it, and home along the lanes; routes are saved and the map draws each contact's way ahead.

!!! question "Open"
    Not built: **bodies** (planets per system); **endurance and stranding**. Generated maps have no unit test yet.

### R-STR-003 Contacts and mission generation

Squadrons, convoys and probes MUST move on flight plans; sensor coverage MUST decide detection; a mission MUST be generated when hostile forces meet.

*Rationale:* Battles with a cause.

- Implements: [F-STR-001](../../definition/strategy-logistics.md#f-str-001-strategy-layer)
- Status: planned — partly met (app 0.7.0 – 0.7.1, main `77d6c9f`, `10f7fce`)
- Verified by: `campaign::tests::raids_are_detected_intercepted_and_fought`, `the_default_campaign_runs_six_months`; MT-4-04, MT-4-05
- See: [Strategy layer › raids](../../modding/strategy.md#raids), [Raids and battles](../../modding/strategy.md#raids-and-battles)

!!! note "As built (app 0.7.0 – 0.7.1)"
    Enemy flights are **raids** (`raids: {<id>: {mission, faction, every, chance, requires, target, speed}}`): each is rolled every `every` while `requires` holds; a success launches a **contact** that heads for one of the player's bases or any system — from well outside the map in a straight line, or on a lane map from the farthest free system along the lanes (R-STR-002). A contact inside a base's sensor range (the hub's `sensor_range` plus `sensor_range` module effects) is **detected** and stays so. The player sends a base's ready craft to **intercept**; when they meet it, the battle is the raid's `mission` (its composition comes from the mission, difficulty and variants), with the first craft's hull as the player's ship, or the player breaks off. The debrief returns to the campaign (R-STR-010). A contact that reaches its target unopposed destroys the target base's newest finished module and scores `undefended`.

!!! question "Open"
    Not built: convoys, probes and the player's own squadrons on flight plans; missions generated from the forces that meet (the battle is the raid's fixed mission); the rival's planning (Stage 5); auto-resolve when the player does not fly (a break-off leaves the contact flying).

### R-STR-004 Economy

The economy MUST model resources, income and expense expressions, markets, factories with prioritised queues, deliveries and losses, and storage.

*Rationale:* The original's economy, as data.

- Implements: [F-STR-002](../../definition/strategy-logistics.md#f-str-002-economy-research-and-bases)
- Status: planned — partly met (app 0.7.0 – 0.9.0, main `77d6c9f`, `10f7fce`, `c64d178`, `ff9d8fc`)
- Verified by: `campaign::tests::modules_bases_purchases_and_manufacture`, `money_flows_monthly_with_costs`, `manufacture_is_paid_per_unit_and_salvage_names_the_raid_by_id`, `the_default_campaign_runs_six_months`; `script::campaign::tests::ids_match_and_names_are_checked`; `campaign::logistics::tests::survey_build_mine_make_and_deliver_on_demand`; MT-4-06, MT-4-15, MT-4-19
- See: [Strategy layer › items and hulls](../../modding/strategy.md#items-and-hulls), [incomes and expenses](../../modding/strategy.md#incomes-and-expenses)

!!! note "As built (app 0.7.0 – 0.7.1)"
    `incomes` and `expenses` are amounts written as expressions in the campaign context (`script::campaign`, 16 queries), paid every `every`; module maintenance, hull upkeep and pilot salaries are charged at each 30-day month. The **market** buys items and hulls at `cost` (delivered to the base after `economy.delivery`) and sells items at `sell_price`. **Factories** are a base's `production_rate` (module effects) working through its queue, first job first; a finished hull becomes a craft at the base.

!!! note "As built (app 0.8.0 – 0.9.0)"
    **Manufacture is paid per unit as it starts** (app 0.9.0, `ff9d8fc`; it was free until then): the job waits while the money is short. **Resources** exist with logistics (R-STR-012 – R-STR-014): mined resources and manufactured components, and hulls built from components instead of money. `economy.cost_scale` multiplies every cost, price, upkeep and maintenance (app 0.8.0, for the original's thousands).

!!! question "Open"
    Not built: prioritised queues; delivery losses of purchases; storage limits at bases (`size` and the `storage` effect are read but not enforced); markets per seller.

### R-STR-005 Research

Research MUST support the download and labs models, discovery by prerequisites, salvage and capture, and unlocks.

*Rationale:* Tech tree.

- Implements: [F-STR-002](../../definition/strategy-logistics.md#f-str-002-economy-research-and-bases)
- Status: planned — partly met (app 0.7.0 – 0.9.0, main `77d6c9f`, `10f7fce`, `c64d178`, `ff9d8fc`)
- Verified by: `campaign::tests::research_follows_prerequisites_and_unlocks`, `manufacture_is_paid_per_unit_and_salvage_names_the_raid_by_id`, `the_default_campaign_runs_six_months`; `content::strategy::tests::a_strategy_layer_builds_with_checked_names`, `original_tech_ids_become_research_prerequisites`; `campaign::logistics::tests::station_research_waits_for_its_start_materials_and_convoys_lose_vessels`; MT-4-07, MT-4-17, MT-4-19, MT-4-23
- See: [Strategy layer › research](../../modding/strategy.md#research-and-research_model)

!!! note "As built (app 0.7.0 – 0.7.1)"
    `research: {<id>: {name, size, requires, unlocks, description}}` and `research_model: {mode: download | labs, rate, switching: keep_progress | lose_progress}`. Each base works on one topic at `rate` work units an hour (an expression evaluated for that base). A topic is available while its `requires` holds; `unlocks` (`items.x`, `hulls.x`, `modules.x`, `base_types.x`) stay unavailable until it is known. Every name in an expression and every unlock is checked when the rules load. `salvage` outcomes mark the raid salvaged for `salvaged(x)`.

!!! note "As built (app 0.8.0 – 0.9.0)"
    **Salvage records the raid's id** (app 0.9.0, `ff9d8fc`), so `salvaged(<raid id>)` holds after a salvage and the default campaign's `saucer_hulls` and `robot_brains` can be started (until then the display name was recorded). **The original's numeric prerequisites** (app 0.8.0): with a `tech_ids: {<number>: <topic>}` table, a topic's `prereq1` – `prereq3` and an item's, hull's or module's `tech_required`, `tech_required2` become `research(x) and …` when no `requires:` is written; numbers not in the table are left out. **Station-only topics** (`lab`, `site`, `start_cost`; R-STR-024) are researched at research stations and never offered to bases.

!!! question "Open"
    Not built: capture; discovery by events; `requires_any`; the original's per-item salvage tables.

### R-STR-006 Bases

Bases MUST have types, placement rules, templates, module construction with build times and construction states, limits, effects, destruction consequences and dismantling.

*Rationale:* The original's bases.

- Implements: [F-STR-002](../../definition/strategy-logistics.md#f-str-002-economy-research-and-bases)
- Status: planned — partly met (app 0.7.0 – 0.7.1, main `77d6c9f`, `10f7fce`)
- Verified by: `campaign::tests::modules_bases_purchases_and_manufacture`, `a_new_campaign_has_its_start`; MT-4-06
- See: [Strategy layer › base_modules and base_types](../../modding/strategy.md#base_modules-and-base_types)

!!! note "As built (app 0.7.0 – 0.7.1)"
    `base_types` (cost, build time, `max_bases`, `limits` per module group, `template`, the hub's `sensor_range`, `hangar` and `quarters`, `max_distance` placement) and `base_modules` (group, cost, build time, maintenance, `effects`, `requires`). One base per system. A module is under construction until its build time has passed (it does not count or take effect until then); the screen shows *building, n days* or *working*. Effects acted on: `production_rate`, `sensor_range`, `hangar`; `research_rate` is available to the research-rate expression through `module_count`. An unopposed raid destroys the target base's newest finished module.

!!! question "Open"
    Not built: dismantling; `on_destroyed` consequences (losing craft, pilots or stock); repair; the `quarters`, `storage`, `defence` and `income` effects (read, not acted on); placement expressions; base-type templates of "best available" modules; the 3D base view.

### R-STR-007 Ships and equipment

Hulls, items, slots and fitting rules, power draw and stock designs MUST be data; craft MUST be bought or manufactured.

*Rationale:* Hangar.

- Implements: [F-STR-003](../../definition/strategy-logistics.md#f-str-003-ships-equipment-and-pilots)
- Status: planned — partly met (app 0.7.0 – 0.7.2, main `77d6c9f`, `10f7fce`, `94d66c1`)
- Verified by: `campaign::tests::modules_bases_purchases_and_manufacture`, `craft_are_fitted_from_the_store_and_fly_their_loadout`; `app::flight::tests::a_campaign_loadout_refits_the_players_class_in_this_battle_only`; MT-4-06, MT-4-12
- See: [Strategy layer › items and hulls](../../modding/strategy.md#items-and-hulls)

!!! note "As built (app 0.7.0 – 0.7.1)"
    `hulls: {<ship class id>: {cost, prod_time, upkeep, speed, slots, requires}}` (a hull must have a ship class of the same id) and `items: {<id>: {name, kind, cost, sell_price, prod_time, size, requires}}`. Craft are bought (delivered) or manufactured at a base. **Fitting** (app 0.7.2): `Campaign::fit_best` fills each hull slot with the costliest item of that `kind` in the base's store (a missile slot takes the item's `rounds`, a new item key), returning the old fit to the store first; the Hangar tab's **F** fits the chosen craft. A campaign battle carries `Campaign::loadout` (`Next::CampaignBattle`), and `flight::apply_loadout` refits the player's class with the fitted beams and missile racks — items matched to beams and missiles of the same id — in that battle's own copy of the rules (`Arc::make_mut`), leaving the shared content untouched.

!!! note "As built (app 0.9.0)"
    With logistics, a hull's `components` make a main base build it from components delivered by the supply chain instead of paying its `cost` (R-STR-013).

!!! question "Open"
    Not built: choosing items slot by slot (fitting is *best available* only); fitting rules (`fits`); power draw; stock designs; effects of slot kinds other than `beam` and `missile` in battle.

### R-STR-008 Pilots

Pilots MUST be generated with ranks, experience, training, injury and healing, bravery, recruiting, salaries and voices.

*Rationale:* Roster.

- Implements: [F-STR-003](../../definition/strategy-logistics.md#f-str-003-ships-equipment-and-pilots)
- Status: planned — partly met (app 0.7.0 – 0.7.1, main `77d6c9f`, `10f7fce`)
- Verified by: `campaign::tests::a_new_campaign_has_its_start`, `raids_are_detected_intercepted_and_fought`, `money_flows_monthly_with_costs`; MT-4-08
- See: [Strategy layer › pilots](../../modding/strategy.md#pilots-and-scores)

!!! note "As built (app 0.7.0 – 0.7.1)"
    `pilots: {stats, ranks, salary, recruit_fee, pool, names, heal_days, survive}`. Pilots are generated with stats rolled in their ranges and a name from the list; rank follows experience (`ranks`, ascending). A battle's leader gains its kills and kills + 1 experience; every pilot in the party a mission. A leader shot down (`killed`) ejects injured for `heal_days` with chance `survive`, else dies; the craft is lost. A pool of recruits is offered each month for `recruit_fee`; living pilots are paid `salary` monthly. A craft has one pilot; a craft is sent on an interception only with a ready pilot.

!!! question "Open"
    Not built: training, bravery (fleeing), stat effects in battle, pilot voices by pilot, injuries by health levels.

### R-STR-009 Funding, score and reports

Patron funding and evaluations, score and ratings and periodic reports MUST be campaign data.

*Rationale:* The original's funding model.

- Implements: [F-STR-002](../../definition/strategy-logistics.md#f-str-002-economy-research-and-bases)
- Status: planned — partly met (app 0.7.0 – 0.7.1, main `77d6c9f`, `10f7fce`)
- Verified by: `campaign::tests::money_flows_monthly_with_costs`, `raids_are_detected_intercepted_and_fought`, `the_default_campaign_runs_six_months`; MT-4-09
- See: [Strategy layer › pilots and scores](../../modding/strategy.md#pilots-and-scores)

!!! note "As built (app 0.7.0 – 0.7.1)"
    `scores: {<name>: points}` give mission outcomes' `score(<name>)` their points; the campaign keeps the total and the month's score (`score`, `month_score` queries). Funding is an ordinary income whose amount may use them — the default campaign's Senate stipend is `400000 + 60000 * bases + month_score * 200` every 30 days. Every 30 days a **monthly report** (a notice that stops the clock) gives the month's score, the total, the month's costs by kind and the funds.

!!! question "Open"
    Not built: evaluations (`evaluations:`) and ratings; funding withdrawn by patrons; a report screen (the report is a banner and a log line).

### R-STR-010 Strategy screens

Functional screens MUST cover the star map, bases (with the 3D base view), research, hangar, market, pilots, reports, messages and briefing/debriefing, with hand-off to battle and return.

*Rationale:* Playable strategy layer before skins (Stage 6).

- Implements: [F-STR-001](../../definition/strategy-logistics.md#f-str-001-strategy-layer)
- Status: planned — partly met (app 0.7.1 – 0.9.0, main `10f7fce`, `8e96338`, `ff9d8fc`)
- Verified by: `app::strategy::tests::the_map_fits_its_rectangle_centred`, `money_has_thousands_separators`; `app::launcher::tests::menu_lists_viewer_and_scenarios_and_wraps`; MT-4-02 – MT-4-10, MT-4-13 – MT-4-18
- See: [User guide › Campaign](../../user-guide/campaign.md), [Campaign (architecture)](../../implementation/architecture/campaign.md#the-battle-hand-off), [Format: interface, text and media](../design/format-interface.md)

!!! note "As built (app 0.7.1)"
    One functional, plain-skinned screen (`app::strategy`): the star map (systems, lanes, bases with sensor ranges, detected contacts, craft in flight; click a system to choose it) and eight tabs — Bases (and new bases), Research, Build, Market, Factory, Hangar (craft, pilots, recruits), Contacts (intercept) and Messages — with a clock line (date, speed, funds, score). Keys: Space, 1 – 4, Tab / Shift+Tab, Up/Down, Enter, Left/Right (current base), S (sell), F (fit, app 0.7.2), F5 (save), Esc (save and leave). An interception that meets its contact stops the clock with a fly-or-break-off banner; flying it hands the screen to the app (`Next::CampaignBattle`, `AppContext::campaign`), and the debrief returns to the map (`Next::Strategy`, `AppContext::battle_report`). The launcher leads with *Campaign: continue* and *Campaign: new game* when the campaign has a strategy layer; `--strategy` opens a new campaign.

!!! note "As built (app 0.9.0)"
    A ninth tab, **Supply** (`Tab::Logistics`): survey the chosen system, build each facility type there (the row gives the cost and why it is not allowed), the facilities with their stores or build time and research projects, *Research <topic> here* on a free research station, the current base's store, and a count of open orders, convoys and survey ships. The map draws facilities (diamonds: grey while building, orange when working), convoys (gold dots) and survey ships (sky-blue rings). Testbed cheats on this screen: R-TST-010.

!!! question "Open"
    Not built: the 3D base view, a report screen, the briefing before an interception (the mission's own briefing is shown in flight as usual), the encyclopaedia; the Supply tab is plain text, without a supply-traffic view. Texts are plain English in the default campaign's strategy files, not text keys. The look is Stage 6.

### R-STR-011 Saves

A save MUST hold the full campaign state plus the mod list and versions; loading MUST refuse a save whose mods are missing. Logistics state (stores, orders, build progress, ships in flight with their attack rolls, survey results) MUST be saved.

*Rationale:* Long campaigns; changing mods.

- Implements: [F-STR-004](../../definition/strategy-logistics.md#f-str-004-saved-games)
- Status: implemented (app 0.7.0 – 0.9.0, main `77d6c9f`, `10f7fce`, `ff9d8fc`)
- Verified by: `campaign::save::tests::a_save_round_trips_and_refuses_missing_packs`; `campaign::tests::the_default_campaign_runs_six_months` (a six-month game round-trips, with the default campaign's logistics block); MT-4-10, MT-4-18
- See: [Campaign (architecture) › Saves](../../implementation/architecture/campaign.md#saves), [Stage 4](../../implementation/plan/stage-4-strategy.md)

!!! note "As built (app 0.7.0 – 0.9.0)"
    `campaign::save` writes the whole `Campaign` as YAML (format version 1): the campaign id, the packs with their versions, then every field — stores, queues, build progress, contacts, deliveries, craft in flight, pilots, flags, messages — and reads it back equal (the test also continues both games 300 hours and compares). Loading refuses a save whose packs are not all loaded, naming them, and a save whose indices do not fit the loaded rules ("was a mod changed?"). One save slot per campaign, `<user data>/saves/<campaign>.yaml`, written on F5 and on leaving the map. **Logistics** (app 0.9.0): a `logistics:` block with the facilities (type, system, ready time, store, running lines, research project), open orders, convoys (cargo, destination, position, route, vessels, distance to a pending ambush), survey ships (with their attack roll), the surveyed systems, the deposits' reserves and the bases' stores, indices checked; a save with facilities is refused by a campaign without logistics.

!!! question "Open"
    Pack **versions** are recorded but not compared: a save from an older version of a loaded mod is accepted if its indices still fit. No unit test round-trips a save with logistics **in flight** (the six-month test's automatic player builds no facilities); MT-4-18 covers it by hand.

### R-STR-012 Logistics is optional data

Logistics MUST be entirely campaign data; a campaign without `facility_types`, `recipes` and `logistics` sections MUST behave as without logistics. The `interceptor` campaign MUST NOT use logistics.

*Rationale:* Modding first; `interceptor` stays faithful to the original.

- Implements: [F-STR-005](../../definition/strategy-logistics.md#f-str-005-supply-chain-logistics)
- Status: implemented (app 0.9.0, main `ff9d8fc`)
- Verified by: `campaign::logistics::tests::survey_build_mine_make_and_deliver_on_demand`, `station_research_waits_for_its_start_materials_and_convoys_lose_vessels` (synthetic campaigns); `campaign::tests` (campaigns without logistics play as before); `check::tests::the_default_campaign_passes`
- See: [Strategy layer › Logistics](../../modding/strategy.md#logistics), [Logistics](../design/logistics.md)

!!! note "As built (app 0.9.0)"
    `content::logistics::LogisticsDef` is built only when at least one `facility_types` entry exists; `StrategyDef::logistics` is then `Some`, else `None` and every logistics key is ignored. The sections: `resources`, `components`, `recipes`, `facility_types`, `research_sites`, `logistics`; on map systems `deposits` and `features`; on hulls `components`; on topics `lab`, `site`, `start_cost` — all in the rules schema. The default campaign's data is `resources/default/strategy/logistics.yaml`; the `interceptor` campaign defines no facility type.

### R-STR-013 Demand-driven orders

A main base's build queue MUST create orders for components to manufacturing centers; a manufacturing center's recipes MUST create orders for materials to mining facilities; a research station's start cost MUST create a one-off order. Each order MUST be filled by the nearest suitable facility by lane. Nothing MUST be shipped speculatively.

*Rationale:* A basic supply-and-demand simulation.

- Implements: [F-STR-005](../../definition/strategy-logistics.md#f-str-005-supply-chain-logistics)
- Status: implemented (app 0.9.0, main `ff9d8fc`), with the nearest facility measured in a straight line (DEV-049)
- Verified by: `campaign::logistics::tests::survey_build_mine_make_and_deliver_on_demand`, `station_research_waits_for_its_start_materials_and_convoys_lose_vessels`; MT-4-15, MT-4-17
- See: [Strategy layer › How the chain runs](../../modding/strategy.md#how-the-chain-runs), [Campaign › Logistics](../../implementation/architecture/campaign.md#logistics)

!!! note "As built (app 0.9.0)"
    Each hour, a base's unstarted hull jobs want their hull's `components`; what the base's store and the convoys already heading there do not cover is ordered (`Campaign::want`, which tops up rather than duplicates). A manufacturing center that must fill an order and lacks a recipe's inputs orders them from a mining facility with a deposit of each; a research station orders its project's `start_cost` until it arrives. The supplier is the **nearest ready facility of the right role by straight-line distance** between systems (`Campaign::supplier`), not by lane distance (DEV-049). With no suitable facility no order is placed, and the demand is asked again next hour. Goods move only to fill an order.

### R-STR-014 Extraction and recipes

Mining facilities MUST extract from deposits at their rate until reserves run out; a recipe MUST start only when its inputs are in that center's store, take its time, and leave its output in the store; storage limits MUST stop production when full.

*Rationale:* Production follows supply.

- Implements: [F-STR-005](../../definition/strategy-logistics.md#f-str-005-supply-chain-logistics)
- Status: planned — partly met (app 0.9.0, main `ff9d8fc`)
- Verified by: `campaign::logistics::tests::survey_build_mine_make_and_deliver_on_demand` (a reserve drawn down; a recipe run on delivered inputs); MT-4-15
- See: [Strategy layer › recipes](../../modding/strategy.md#recipes), [facility_types](../../modding/strategy.md#facility_types)

!!! note "As built (app 0.9.0)"
    A mining facility adds `rate` / 24 of each deposit of its system every hour, limited by the room left in its store (`storage`) and by the deposit's reserve (`unlimited` by default). A manufacturing center starts a recipe on a free line (`lines`) only when all inputs are in its store, uses them at the start, and adds `output` to the store after `time`; it starts runs only for orders it must fill.

!!! question "Open"
    `storage` is enforced for **mining facilities only**: a manufacturing center's and a research station's store has no limit yet.

### R-STR-015 Government supply ships

When an order's cargo is ready, supply ships MUST be dispatched automatically along the fixed lanes; they are provided by the government and the player MUST NOT buy or manage them.

*Rationale:* Logistics decision (2026-11-11).

- Implements: [F-STR-005](../../definition/strategy-logistics.md#f-str-005-supply-chain-logistics)
- Status: implemented (app 0.9.0, main `ff9d8fc`)
- Verified by: `campaign::logistics::tests::survey_build_mine_make_and_deliver_on_demand`; MT-4-15
- See: [Logistics](../design/logistics.md)

!!! note "As built (app 0.9.0)"
    When a source facility's store holds an order's amount, the amount is taken out and a convoy leaves for the order's base or facility along the shortest lane route (`Campaign::lane_route`; a system no lane reaches is one straight hop) at `logistics.convoy_speed`. The player has no order for convoys; the map shows them.

### R-STR-016 Supply convoys

A shipment MUST travel as a convoy of several supply vessels, each carrying an even share of the cargo; a convoy that is partly destroyed MUST deliver the surviving vessels' share.

*Rationale:* Escort missions stay interesting and playable; single ships would be destroyed quickly (decision 2026-11-11).

- Implements: [F-STR-007](../../definition/strategy-logistics.md#f-str-007-supply-convoys-worth-escorting)
- Status: implemented (app 0.9.0, main `ff9d8fc`)
- Verified by: `campaign::logistics::tests::station_research_waits_for_its_start_materials_and_convoys_lose_vessels` (every convoy ambushed at 50 % loss; deliveries shrink and orders follow until the cost is met); MT-4-16
- See: [Strategy layer › logistics](../../modding/strategy.md#logistics)

!!! note "As built (app 0.9.0)"
    The number of vessels is campaign data, `logistics.vessels` (default 3; the default campaign 3) — answering the former Open question. On arrival a convoy delivers its cargo × surviving vessels ÷ `vessels`; what is missing is ordered again by the next hour's demand.

### R-STR-017 Supply vessel model

The default campaign MUST provide a supply vessel model that is reasonably large: a core vessel with several attached cargo containers.

*Rationale:* A convoy worth escorting (decision 2026-11-11).

- Implements: [F-STR-007](../../definition/strategy-logistics.md#f-str-007-supply-convoys-worth-escorting), [F-CPN-003](../../definition/campaigns/features.md#f-cpn-003-the-default-campaign-perils-of-the-void)
- Status: planned
- Verified by: Planned (generated by `tools/default_assets/`).
- See: [The default campaign](../../definition/campaigns/default-campaign.md)

!!! question "Open"
    Whether the existing cargo clipper and cargo pod become the core vessel and containers, or a new model replaces them. Convoys are dots on the strategy map until their battles are flown (R-STR-019).

### R-STR-018 Supply convoy attack position

A supply convoy's attack position along its route MUST be chosen at random when it launches.

*Rationale:* Logistics decision (2026-11-11).

- Implements: [F-STR-007](../../definition/strategy-logistics.md#f-str-007-supply-convoys-worth-escorting)
- Status: implemented (app 0.9.0, main `ff9d8fc`)
- Verified by: `campaign::logistics::tests::station_research_waits_for_its_start_materials_and_convoys_lose_vessels`; MT-4-16
- See: [Campaign › Logistics](../../implementation/architecture/campaign.md#logistics)

!!! note "As built (app 0.9.0)"
    At launch one roll against `logistics.ambush_chance` decides whether the convoy will be attacked (default 30 %; the default campaign 30 %), and a second roll picks the point as a fraction of the route's length (`Convoy::ambush_in`, saved). This answers the former Open question: only some convoys are attacked, at a campaign-defined chance.

### R-STR-019 Logistics mission generation

A `convoy_ambush` mission MUST be generated when an attacked convoy reaches its attack position, and a `construction_defence` mission when a strike on a facility under construction is due; when the player does not fly, the mission MUST be auto-resolved.

*Rationale:* Logistics in battle.

- Implements: [F-STR-005](../../definition/strategy-logistics.md#f-str-005-supply-chain-logistics), [F-MSN-007](../../definition/missions.md#f-msn-007-logistics-missions)
- Status: planned — partly met (app 0.9.0, main `ff9d8fc`): the auto-resolve of a convoy ambush
- Verified by: `campaign::logistics::tests::station_research_waits_for_its_start_materials_and_convoys_lose_vessels` (auto-resolved losses); MT-4-16
- See: [Logistics](../design/logistics.md)

!!! note "As built (app 0.9.0)"
    At the attack position the ambush is **always auto-resolved**: each vessel is lost with `logistics.ambush_loss` (default 35 %; the convoy has no escort), and a notice names the cargo and the losses.

!!! question "Open"
    Not built: generating and flying `convoy_ambush` (the mission exists in the default campaign; the hookup from the map does not), construction strikes and `construction_defence` (R-STR-023).

### R-STR-020 Cargo losses and depots

A destroyed vessel's cargo MUST be lost; a captured vessel's cargo MUST go to the captor and be stored at a supply depot; destroying an enemy supply depot MUST let the player seize its stock.

*Rationale:* Logistics decision (2026-11-11).

- Implements: [F-STR-005](../../definition/strategy-logistics.md#f-str-005-supply-chain-logistics)
- Status: planned — partly met (app 0.9.0, main `ff9d8fc`): destroyed vessels' cargo is lost
- Verified by: `campaign::logistics::tests::station_research_waits_for_its_start_materials_and_convoys_lose_vessels`; MT-4-16
- See: [Logistics](../design/logistics.md)

!!! question "Open"
    Not built: capture, captured cargo, enemy supply depots and seizing their stock (Stage 5 with the rival's logistics).

### R-STR-021 Survey ships

Surveys MUST be flown by survey ships, a mission profile and craft distinct from supply ships, launched from a base to a system within the base's current radar range. A survey MUST reveal the system's deposits and reserves, features, the research sites it qualifies for and its suitability for each facility type. When a survey ship launches, a 25 % roll MUST decide whether it will be attacked.

*Rationale:* Logistics decisions (2026-11-11, revised): survey and supply are different.

- Implements: [F-STR-006](../../definition/strategy-logistics.md#f-str-006-surveys)
- Status: planned — partly met (app 0.9.0, main `ff9d8fc`)
- Verified by: `campaign::logistics::tests::survey_build_mine_make_and_deliver_on_demand` (survey first; the report names the deposit); MT-4-13
- See: [Strategy layer › The player's orders](../../modding/strategy.md#the-players-orders)

!!! note "As built (app 0.9.0)"
    `Campaign::survey(def, base, system)` launches a survey ship (separate from convoys) from a base to a system within that base's sensor range, not yet surveyed. At launch a roll against `logistics.survey_attack_chance` (default 25 %) marks it attacked. It flies straight at `logistics.survey_speed`; on arrival an attacked ship is lost half the time, else the system is surveyed and a notice lists its deposits with their daily rates and the research sites it meets. Suitability per facility type is the Supply tab's reason on each *Build* row (R-STR-010).

!!! question "Open"
    The report does not give reserves or features by name; the survey ship has no craft or battle model, and an attack is auto-resolved (lost on a fixed 50 % roll, not campaign data) instead of generating a mission.

### R-STR-022 Construction stages

A facility under construction MUST be shown in its `build_25` state from 0 %, `build_60` from 60 % and `build_90` from 90 %, and intact on completion, on the strategy map and in battle.

*Rationale:* Thresholds confirmed (2026-11-11).

- Implements: [F-STR-008](../../definition/strategy-logistics.md#f-str-008-facilities-under-construction)
- Status: planned
- Verified by: Planned (the stage looks themselves are R-CMB-019, implemented).
- See: [Logistics](../design/logistics.md)

!!! question "Open"
    App 0.9.0 builds facilities over their `time` (grey on the map until finished) and reads `facility_types.<id>.hub`, but shows no construction stages and no battle at a facility.

### R-STR-023 Construction attacks

The enemy MAY strike a facility under construction from 50 % progress, at most three times per location; both numbers MUST be campaign data.

*Rationale:* Logistics decision (2026-11-11).

- Implements: [F-STR-008](../../definition/strategy-logistics.md#f-str-008-facilities-under-construction)
- Status: planned
- Verified by: Planned.
- See: [Logistics](../design/logistics.md)

### R-STR-024 Research stations and sites

A research topic or branch MAY be restricted to a facility type (`lab:`) and to systems meeting a research site's criteria (`site:`); its `start_cost` MUST be delivered once to begin the project, after which research MUST run without further supply.

*Rationale:* Logistics decision (2026-11-11).

- Implements: [F-STR-009](../../definition/strategy-logistics.md#f-str-009-research-stations-and-fighter-pools)
- Status: implemented (app 0.9.0, main `ff9d8fc`)
- Verified by: `campaign::logistics::tests::station_research_waits_for_its_start_materials_and_convoys_lose_vessels`, `survey_build_mine_make_and_deliver_on_demand` (a research station refused off a site); MT-4-17
- See: [Strategy layer › Research stations](../../modding/strategy.md#research-stations-and-station-only-topics)

!!! note "As built (app 0.9.0)"
    `research_sites: {<id>: {features: […]}}` match systems whose `features` include all the tags. A topic with `lab:` is researched only by a facility of that type, with `site:` only in a system meeting that site, and is not offered to bases; `Campaign::start_station_research` checks both and the topic's `requires`. The station orders the `start_cost`, uses it once on arrival, then adds `research_rate` an hour with no further supply. Research stations themselves may be built only on a surveyed research site.

### R-STR-025 Facility fighter pools

Fighters MUST be built at main bases and MAY be transferred to a facility's pool, separate from base hangars; pool fighters MUST defend the facility's system and escort its convoys, and MAY be moved between facilities.

*Rationale:* Logistics decision (2026-11-11).

- Implements: [F-STR-009](../../definition/strategy-logistics.md#f-str-009-research-stations-and-fighter-pools)
- Status: planned
- Verified by: Planned.
- See: [Logistics](../design/logistics.md)

### R-STR-026 Supply cargo capacity

A supply vessel's cargo capacity MUST be campaign data.

*Rationale:* Balance belongs to the campaign.

- Implements: [F-STR-007](../../definition/strategy-logistics.md#f-str-007-supply-convoys-worth-escorting)
- Status: planned
- Verified by: Planned.
- See: [Logistics](../design/logistics.md)

!!! question "Open"
    The value is TBD, set by tech-tree balance. In app 0.9.0 a convoy carries a whole order, whatever its size.

### R-STR-027 Third-party convoys

Prospector convoys MUST fly expeditions (base → system under investigation → pause → base); Cartel convoys MUST fly to random destinations, including empty or uncharted stars; escorting or attacking them MUST change relations.

*Rationale:* A living map (default campaign, Stage 5).

- Implements: [F-STR-005](../../definition/strategy-logistics.md#f-str-005-supply-chain-logistics), [F-CPN-006](../../definition/campaigns/features.md#f-cpn-006-story-rivals-and-relations)
- Status: planned
- Verified by: Planned.
- See: [Logistics](../design/logistics.md)

## Revision history

| Version | Date | Change |
|---|---|---|
| 1.0.0 | 2026-11-11 | Bootstrap: R-STR-001 – R-STR-027 (27 requirements; 0 reverse-engineered from the code) |
| 1.13.0 | 2026-11-11 | R-STR-001 – R-STR-011 as built (app 0.7.0 – 0.7.3, main `77d6c9f`, `10f7fce`, `94d66c1`, `076203e`): R-STR-001 (`start:` replaces `starts`) and R-STR-011 implemented; R-STR-002 – R-STR-010 partly met (status stays planned), each with an As-built note, Verified-by lines and an Open note of what remains (endurance, choosing a fit, logistics, evaluations, the 3D base view; known issues: manufacture charges no money, `salvaged` compares names with ids). R-STR-012 – R-STR-027 stay planned |
| 1.14.0 | 2026-11-11 | Logistics, the X-COM data and fixes as built (app 0.8.0 – 0.9.0, main `c64d178`, `8e96338`, `ff9d8fc`): R-STR-011 (logistics state saved), R-STR-012, R-STR-013 (nearest supplier in a straight line, DEV-049), R-STR-015, R-STR-016 (vessels: `logistics.vessels`), R-STR-018 (attack chance: `logistics.ambush_chance`) and R-STR-024 implemented; R-STR-014 (storage only at mines), R-STR-019 (ambushes auto-resolved, not flown), R-STR-020 (losses, no capture or depots) and R-STR-021 (no survey craft; attacks auto-resolved) partly met; R-STR-004, R-STR-005 (`tech_ids`, station-only topics), R-STR-007 (hull components) and R-STR-010 (the Supply tab) As-built notes; the two known issues (manufacture free, `salvaged` by name) fixed. R-STR-017, R-STR-022, R-STR-023, R-STR-025 – R-STR-027 stay planned with Open notes |
