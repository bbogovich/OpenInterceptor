---
spec_version: 1.14.0
status: current
updated: 2026-11-11
---

# Strategy & logistics

The real-time strategy game around the battles, and the default campaign's supply-chain logistics.

Requirements for these features: [Strategy & logistics requirements](../specification/requirements/strategy-logistics.md).

| Feature | Status |
|---|---|
| [F-STR-001 Strategy layer](#f-str-001-strategy-layer) | In progress |
| [F-STR-002 Economy, research and bases](#f-str-002-economy-research-and-bases) | In progress |
| [F-STR-003 Ships, equipment and pilots](#f-str-003-ships-equipment-and-pilots) | In progress |
| [F-STR-004 Saved games](#f-str-004-saved-games) | Implemented |
| [F-STR-005 Supply-chain logistics](#f-str-005-supply-chain-logistics) | In progress |
| [F-STR-006 Surveys](#f-str-006-surveys) | In progress |
| [F-STR-007 Supply convoys worth escorting](#f-str-007-supply-convoys-worth-escorting) | In progress |
| [F-STR-008 Facilities under construction](#f-str-008-facilities-under-construction) | In progress |
| [F-STR-009 Research stations and fighter pools](#f-str-009-research-stations-and-fighter-pools) | In progress |

## Features

### F-STR-001 Strategy layer

**What.** A real-time strategy game around the battles: a star map with a campaign clock, squadrons and sensors, and battles generated when forces meet.

**Why.** The game is a space-combat *strategy* game; battles need a cause.

- **Status:** in-progress. Built (app 0.7.0 – 0.7.3): the campaign clock, fixed and generated maps with lane travel, raids detected by sensors and intercepted into battles, and the strategy screen; remaining: convoys and probes on flight plans, generated missions, endurance, the 3D base view.
- **Source:** Plan items 4.1–4.3.
- **Specified by:** [R-STR-001](../specification/requirements/strategy-logistics.md#r-str-001-campaign-clock), [R-STR-002](../specification/requirements/strategy-logistics.md#r-str-002-star-map-and-travel), [R-STR-003](../specification/requirements/strategy-logistics.md#r-str-003-contacts-and-mission-generation), [R-STR-010](../specification/requirements/strategy-logistics.md#r-str-010-strategy-screens)

### F-STR-002 Economy, research and bases

**What.** Money and resources, markets and factories, research with discoveries and salvage, bases built from modules.

**Why.** The original's strategy systems, as data.

- **Status:** in-progress. Built (app 0.7.0 – 0.7.1): income and expense expressions, market, factory queues, deliveries, research (download and labs, prerequisites, unlocks, salvage), base types, modules with limits and effects, the monthly report and score; remaining: resources, storage, evaluations, capture, dismantling.
- **Source:** Plan items 4.4–4.6 and 4.9.
- **Specified by:** [R-STR-004](../specification/requirements/strategy-logistics.md#r-str-004-economy), [R-STR-005](../specification/requirements/strategy-logistics.md#r-str-005-research), [R-STR-006](../specification/requirements/strategy-logistics.md#r-str-006-bases), [R-STR-009](../specification/requirements/strategy-logistics.md#r-str-009-funding-score-and-reports)

### F-STR-003 Ships, equipment and pilots

**What.** Hulls fitted with equipment, bought or built; pilots with ranks, experience, injuries and salaries.

**Why.** The original's hangar and roster.

- **Status:** in-progress. Built (app 0.7.0 – 0.7.2): hulls and items as data, craft bought or manufactured, fitting from the store flown in campaign battles, pilots with stats, ranks, experience, injury, recruiting and salaries; remaining: fitting rules, designs, training, bravery.
- **Source:** Plan items 4.7 and 4.8.
- **Specified by:** [R-STR-007](../specification/requirements/strategy-logistics.md#r-str-007-ships-and-equipment), [R-STR-008](../specification/requirements/strategy-logistics.md#r-str-008-pilots)

### F-STR-004 Saved games

**What.** The whole campaign can be saved and loaded, together with the list of mods it needs.

**Why.** Long campaigns; mods change between sessions.

- **Status:** implemented. Built (app 0.7.0): the whole campaign saved as YAML with its pack list; saves needing missing packs are refused. The logistics state joined in app 0.9.0.
- **Source:** Plan item 4.11.
- **Specified by:** [R-STR-011](../specification/requirements/strategy-logistics.md#r-str-011-saves)

### F-STR-005 Supply-chain logistics

**What.** In the default campaign, mining facilities, research stations and manufacturing centers are built in star systems, and demand pulls materials and components along a supply chain carried by government supply ships.

**Why.** User request for the default campaign: both sides race to mine the belts, and escort and raid missions get a strategic cause.

- **Status:** in-progress. Built (app 0.9.0): resources, components and recipes as data; mining facilities, manufacturing centers and research stations; demand-driven orders from base hull jobs through manufacturing to mining; automatic government supply convoys along the lanes; saves. Remaining: flown convoy ambushes and construction strikes (auto-resolved today), captured cargo and depots, third-party convoys, facility upgrades.
- **Source:** User request (logistics design, decisions 2026-11-11); plan items 4.14–4.17.
- **Specified by:** [R-STR-012](../specification/requirements/strategy-logistics.md#r-str-012-logistics-is-optional-data), [R-STR-013](../specification/requirements/strategy-logistics.md#r-str-013-demand-driven-orders), [R-STR-014](../specification/requirements/strategy-logistics.md#r-str-014-extraction-and-recipes), [R-STR-015](../specification/requirements/strategy-logistics.md#r-str-015-government-supply-ships), [R-STR-019](../specification/requirements/strategy-logistics.md#r-str-019-logistics-mission-generation), [R-STR-020](../specification/requirements/strategy-logistics.md#r-str-020-cargo-losses-and-depots), [R-STR-027](../specification/requirements/strategy-logistics.md#r-str-027-third-party-convoys), [R-MOD-014](../specification/requirements/modding.md#r-mod-014-campaign-context)

### F-STR-006 Surveys

**What.** Survey ships visit systems within a base's radar range and report deposits, features, research sites and which facilities a system suits. A survey ship can be attacked.

**Why.** The player chooses where to build from what is known.

- **Status:** in-progress. Built (app 0.9.0): survey ships from a base within its radar range, the attack roll at launch, a report of deposits and research sites; suitability shown on the Supply tab. Remaining: a survey craft and a battle when one is attacked (auto-resolved today).
- **Source:** Logistics decisions (2026-11-11, revised); plan item 4.16.
- **Specified by:** [R-STR-021](../specification/requirements/strategy-logistics.md#r-str-021-survey-ships)

### F-STR-007 Supply convoys worth escorting

**What.** Supply ships travel as convoys of several large vessels, each carrying a share of the cargo, so a convoy that loses some ships still delivers what survives; enemies attack them along the route.

**Why.** Escort missions stay interesting and playable: single ships would be destroyed quickly.

- **Status:** in-progress. Built (app 0.9.0): convoys of several vessels (data) with even shares, the ambush point chosen at launch, surviving shares delivered. Remaining: the large supply-vessel model, flying the ambush and escorting, cargo capacity.
- **Source:** User decision (2026-11-11).
- **Specified by:** [R-STR-016](../specification/requirements/strategy-logistics.md#r-str-016-supply-convoys), [R-STR-017](../specification/requirements/strategy-logistics.md#r-str-017-supply-vessel-model), [R-STR-018](../specification/requirements/strategy-logistics.md#r-str-018-supply-convoy-attack-position), [R-STR-026](../specification/requirements/strategy-logistics.md#r-str-026-supply-cargo-capacity)

### F-STR-008 Facilities under construction

**What.** A facility takes time to build and is seen in three construction stages, on the map and in battle; the enemy may strike it before it is finished.

**Why.** Construction is a vulnerable period worth defending.

- **Status:** in-progress. Built: the construction-stage looks (Stage 2); facilities take their build time (app 0.9.0). Remaining: stages shown on the map and in battle, strikes on facilities under construction.
- **Source:** Logistics design; construction-stage looks built in Stage 2; plan items 4.14 and 3.13.
- **Specified by:** [R-CMB-019](../specification/requirements/combat.md#r-cmb-019-model-sets-and-damage-looks), [R-STR-022](../specification/requirements/strategy-logistics.md#r-str-022-construction-stages), [R-STR-023](../specification/requirements/strategy-logistics.md#r-str-023-construction-attacks)

### F-STR-009 Research stations and fighter pools

**What.** Research stations unlock tech branches ordinary bases cannot, at sites that meet their criteria; fighters built at main bases can be assigned to facilities to defend them and escort their convoys.

**Why.** Gives facilities a purpose and a defence.

- **Status:** in-progress. Built (app 0.9.0): research stations on research sites with station-only topics and one-off start costs. Remaining: facility fighter pools.
- **Source:** Logistics decisions (2026-11-11); plan item 4.16.
- **Specified by:** [R-STR-024](../specification/requirements/strategy-logistics.md#r-str-024-research-stations-and-sites), [R-STR-025](../specification/requirements/strategy-logistics.md#r-str-025-facility-fighter-pools)

## Revision history

| Version | Date | Change |
|---|---|---|
| 1.0.0 | 2026-11-11 | Bootstrap: F-STR-001 – F-STR-009 (9 features) |
| 1.13.0 | 2026-11-11 | F-STR-001 – F-STR-003 in progress and F-STR-004 implemented (app 0.7.0 – 0.7.3) |
| 1.14.0 | 2026-11-11 | F-STR-005, F-STR-006, F-STR-007 and F-STR-009 in progress; F-STR-008 progress (app 0.9.0, logistics) |
