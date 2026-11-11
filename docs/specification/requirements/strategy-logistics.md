---
spec_version: 1.0.0
status: current
updated: 2026-11-11
---

# Strategy & logistics requirements

Requirements for the strategy layer and logistics. The normative designs are [Format: strategy layer](../design/format-strategy.md) and [Logistics](../design/logistics.md).

Features: [Strategy & logistics](../../definition/strategy-logistics.md). How to read a requirement: [Specification](../index.md#reading-a-requirement).

| Requirement | Status |
|---|---|
| [R-STR-001 Campaign clock](#r-str-001-campaign-clock) | Planned |
| [R-STR-002 Star map and travel](#r-str-002-star-map-and-travel) | Planned |
| [R-STR-003 Contacts and mission generation](#r-str-003-contacts-and-mission-generation) | Planned |
| [R-STR-004 Economy](#r-str-004-economy) | Planned |
| [R-STR-005 Research](#r-str-005-research) | Planned |
| [R-STR-006 Bases](#r-str-006-bases) | Planned |
| [R-STR-007 Ships and equipment](#r-str-007-ships-and-equipment) | Planned |
| [R-STR-008 Pilots](#r-str-008-pilots) | Planned |
| [R-STR-009 Funding, score and reports](#r-str-009-funding-score-and-reports) | Planned |
| [R-STR-010 Strategy screens](#r-str-010-strategy-screens) | Planned |
| [R-STR-011 Saves](#r-str-011-saves) | Planned |
| [R-STR-012 Logistics is optional data](#r-str-012-logistics-is-optional-data) | Planned |
| [R-STR-013 Demand-driven orders](#r-str-013-demand-driven-orders) | Planned |
| [R-STR-014 Extraction and recipes](#r-str-014-extraction-and-recipes) | Planned |
| [R-STR-015 Government supply ships](#r-str-015-government-supply-ships) | Planned |
| [R-STR-016 Supply convoys](#r-str-016-supply-convoys) | Planned |
| [R-STR-017 Supply vessel model](#r-str-017-supply-vessel-model) | Planned |
| [R-STR-018 Supply convoy attack position](#r-str-018-supply-convoy-attack-position) | Planned |
| [R-STR-019 Logistics mission generation](#r-str-019-logistics-mission-generation) | Planned |
| [R-STR-020 Cargo losses and depots](#r-str-020-cargo-losses-and-depots) | Planned |
| [R-STR-021 Survey ships](#r-str-021-survey-ships) | Planned |
| [R-STR-022 Construction stages](#r-str-022-construction-stages) | Planned |
| [R-STR-023 Construction attacks](#r-str-023-construction-attacks) | Planned |
| [R-STR-024 Research stations and sites](#r-str-024-research-stations-and-sites) | Planned |
| [R-STR-025 Facility fighter pools](#r-str-025-facility-fighter-pools) | Planned |
| [R-STR-026 Supply cargo capacity](#r-str-026-supply-cargo-capacity) | Planned |
| [R-STR-027 Third-party convoys](#r-str-027-third-party-convoys) | Planned |

## Requirements

### R-STR-001 Campaign clock

The strategy layer MUST run on a campaign clock with dates, time speeds and auto-pause, start from the campaign's `starts` definition, and start a new game with a chosen campaign and difficulty.

*Rationale:* Real-time strategy.

- Implements: [F-STR-001](../../definition/strategy-logistics.md#f-str-001-strategy-layer)
- Status: planned
- Verified by: Planned.
- See: [Format: strategy layer](../design/format-strategy.md)

### R-STR-002 Star map and travel

The star map MUST be generated (classes, bodies, names, hidden black holes; seeded) or fixed, with free or lane travel, endurance and stranding.

*Rationale:* Both campaigns' maps.

- Implements: [F-STR-001](../../definition/strategy-logistics.md#f-str-001-strategy-layer)
- Status: planned
- Verified by: Planned.
- See: [Format: strategy layer](../design/format-strategy.md)

### R-STR-003 Contacts and mission generation

Squadrons, convoys and probes MUST move on flight plans; sensor coverage MUST decide detection; a mission MUST be generated when hostile forces meet.

*Rationale:* Battles with a cause.

- Implements: [F-STR-001](../../definition/strategy-logistics.md#f-str-001-strategy-layer)
- Status: planned
- Verified by: Planned.
- See: [Format: strategy layer](../design/format-strategy.md)

### R-STR-004 Economy

The economy MUST model resources, income and expense expressions, markets, factories with prioritised queues, deliveries and losses, and storage.

*Rationale:* The original's economy, as data.

- Implements: [F-STR-002](../../definition/strategy-logistics.md#f-str-002-economy-research-and-bases)
- Status: planned
- Verified by: Planned.
- See: [Format: strategy layer](../design/format-strategy.md)

### R-STR-005 Research

Research MUST support the download and labs models, discovery by prerequisites, salvage and capture, and unlocks.

*Rationale:* Tech tree.

- Implements: [F-STR-002](../../definition/strategy-logistics.md#f-str-002-economy-research-and-bases)
- Status: planned
- Verified by: Planned.
- See: [Format: strategy layer](../design/format-strategy.md)

### R-STR-006 Bases

Bases MUST have types, placement rules, templates, module construction with build times and construction states, limits, effects, destruction consequences and dismantling.

*Rationale:* The original's bases.

- Implements: [F-STR-002](../../definition/strategy-logistics.md#f-str-002-economy-research-and-bases)
- Status: planned
- Verified by: Planned.
- See: [Format: strategy layer](../design/format-strategy.md)

### R-STR-007 Ships and equipment

Hulls, items, slots and fitting rules, power draw and stock designs MUST be data; craft MUST be bought or manufactured.

*Rationale:* Hangar.

- Implements: [F-STR-003](../../definition/strategy-logistics.md#f-str-003-ships-equipment-and-pilots)
- Status: planned
- Verified by: Planned.
- See: [Format: strategy layer](../design/format-strategy.md)

### R-STR-008 Pilots

Pilots MUST be generated with ranks, experience, training, injury and healing, bravery, recruiting, salaries and voices.

*Rationale:* Roster.

- Implements: [F-STR-003](../../definition/strategy-logistics.md#f-str-003-ships-equipment-and-pilots)
- Status: planned
- Verified by: Planned.
- See: [Format: strategy layer](../design/format-strategy.md)

### R-STR-009 Funding, score and reports

Patron funding and evaluations, score and ratings and periodic reports MUST be campaign data.

*Rationale:* The original's funding model.

- Implements: [F-STR-002](../../definition/strategy-logistics.md#f-str-002-economy-research-and-bases)
- Status: planned
- Verified by: Planned.
- See: [Format: strategy layer](../design/format-strategy.md)

### R-STR-010 Strategy screens

Functional screens MUST cover the star map, bases (with the 3D base view), research, hangar, market, pilots, reports, messages and briefing/debriefing, with hand-off to battle and return.

*Rationale:* Playable strategy layer before skins (Stage 6).

- Implements: [F-STR-001](../../definition/strategy-logistics.md#f-str-001-strategy-layer)
- Status: planned
- Verified by: Planned.
- See: [Format: interface, text and media](../design/format-interface.md)

### R-STR-011 Saves

A save MUST hold the full campaign state plus the mod list and versions; loading MUST refuse a save whose mods are missing. Logistics state (stores, orders, build progress, ships in flight with their attack rolls, survey results) MUST be saved.

*Rationale:* Long campaigns; changing mods.

- Implements: [F-STR-004](../../definition/strategy-logistics.md#f-str-004-saved-games)
- Status: planned
- Verified by: Planned (round-trip unit tests on synthetic campaigns).
- See: [Stage 4](../../implementation/plan/stage-4-strategy.md)

### R-STR-012 Logistics is optional data

Logistics MUST be entirely campaign data; a campaign without `facility_types`, `recipes` and `logistics` sections MUST behave as without logistics. The `interceptor` campaign MUST NOT use logistics.

*Rationale:* Modding first; `interceptor` stays faithful to the original.

- Implements: [F-STR-005](../../definition/strategy-logistics.md#f-str-005-supply-chain-logistics)
- Status: planned
- Verified by: Planned.
- See: [Logistics](../design/logistics.md)

### R-STR-013 Demand-driven orders

A main base's build queue MUST create orders for components to manufacturing centers; a manufacturing center's recipes MUST create orders for materials to mining facilities; a research station's start cost MUST create a one-off order. Each order MUST be filled by the nearest suitable facility by lane. Nothing MUST be shipped speculatively.

*Rationale:* A basic supply-and-demand simulation.

- Implements: [F-STR-005](../../definition/strategy-logistics.md#f-str-005-supply-chain-logistics)
- Status: planned
- Verified by: Planned.
- See: [Logistics](../design/logistics.md)

### R-STR-014 Extraction and recipes

Mining facilities MUST extract from deposits at their rate until reserves run out; a recipe MUST start only when its inputs are in that center's store, take its time, and leave its output in the store; storage limits MUST stop production when full.

*Rationale:* Production follows supply.

- Implements: [F-STR-005](../../definition/strategy-logistics.md#f-str-005-supply-chain-logistics)
- Status: planned
- Verified by: Planned.
- See: [Logistics](../design/logistics.md)

### R-STR-015 Government supply ships

When an order's cargo is ready, supply ships MUST be dispatched automatically along the fixed lanes; they are provided by the government and the player MUST NOT buy or manage them.

*Rationale:* Logistics decision (2026-11-11).

- Implements: [F-STR-005](../../definition/strategy-logistics.md#f-str-005-supply-chain-logistics)
- Status: planned
- Verified by: Planned.
- See: [Logistics](../design/logistics.md)

### R-STR-016 Supply convoys

A shipment MUST travel as a convoy of several supply vessels, each carrying an even share of the cargo; a convoy that is partly destroyed MUST deliver the surviving vessels' share.

*Rationale:* Escort missions stay interesting and playable; single ships would be destroyed quickly (decision 2026-11-11).

- Implements: [F-STR-007](../../definition/strategy-logistics.md#f-str-007-supply-convoys-worth-escorting)
- Status: planned
- Verified by: Planned.
- See: [Logistics](../design/logistics.md)

!!! question "Open"
    The number of vessels per convoy.

### R-STR-017 Supply vessel model

The default campaign MUST provide a supply vessel model that is reasonably large: a core vessel with several attached cargo containers.

*Rationale:* A convoy worth escorting (decision 2026-11-11).

- Implements: [F-STR-007](../../definition/strategy-logistics.md#f-str-007-supply-convoys-worth-escorting), [F-CPN-003](../../definition/campaigns/features.md#f-cpn-003-the-default-campaign-perils-of-the-void)
- Status: planned
- Verified by: Planned (generated by `tools/default_assets/`).
- See: [The default campaign](../../definition/campaigns/default-campaign.md)

!!! question "Open"
    Whether the existing cargo clipper and cargo pod become the core vessel and containers, or a new model replaces them.

### R-STR-018 Supply convoy attack position

A supply convoy's attack position along its route MUST be chosen at random when it launches.

*Rationale:* Logistics decision (2026-11-11).

- Implements: [F-STR-007](../../definition/strategy-logistics.md#f-str-007-supply-convoys-worth-escorting)
- Status: planned
- Verified by: Planned.
- See: [Logistics](../design/logistics.md)

!!! question "Open"
    Whether every supply convoy is attacked or only some, and at what chance: the user's answer gives the 25 % roll for survey ships only (R-STR-021).

### R-STR-019 Logistics mission generation

A `convoy_ambush` mission MUST be generated when an attacked convoy reaches its attack position, and a `construction_defence` mission when a strike on a facility under construction is due; when the player does not fly, the mission MUST be auto-resolved.

*Rationale:* Logistics in battle.

- Implements: [F-STR-005](../../definition/strategy-logistics.md#f-str-005-supply-chain-logistics), [F-MSN-007](../../definition/missions.md#f-msn-007-logistics-missions)
- Status: planned
- Verified by: Planned.
- See: [Logistics](../design/logistics.md)

### R-STR-020 Cargo losses and depots

A destroyed vessel's cargo MUST be lost; a captured vessel's cargo MUST go to the captor and be stored at a supply depot; destroying an enemy supply depot MUST let the player seize its stock.

*Rationale:* Logistics decision (2026-11-11).

- Implements: [F-STR-005](../../definition/strategy-logistics.md#f-str-005-supply-chain-logistics)
- Status: planned
- Verified by: Planned (depots in Stage 5).
- See: [Logistics](../design/logistics.md)

### R-STR-021 Survey ships

Surveys MUST be flown by survey ships, a mission profile and craft distinct from supply ships, launched from a base to a system within the base's current radar range. A survey MUST reveal the system's deposits and reserves, features, the research sites it qualifies for and its suitability for each facility type. When a survey ship launches, a 25 % roll MUST decide whether it will be attacked.

*Rationale:* Logistics decisions (2026-11-11, revised): survey and supply are different.

- Implements: [F-STR-006](../../definition/strategy-logistics.md#f-str-006-surveys)
- Status: planned
- Verified by: Planned.
- See: [Logistics](../design/logistics.md)

!!! question "Open"
    What the survey-ship craft is (the design has used the Sky-Eye probe object), where and when an attacked survey ship meets its attackers, and which mission is generated.

### R-STR-022 Construction stages

A facility under construction MUST be shown in its `build_25` state from 0 %, `build_60` from 60 % and `build_90` from 90 %, and intact on completion, on the strategy map and in battle.

*Rationale:* Thresholds confirmed (2026-11-11).

- Implements: [F-STR-008](../../definition/strategy-logistics.md#f-str-008-facilities-under-construction)
- Status: planned
- Verified by: Planned (the stage looks themselves are R-CMB-019, implemented).
- See: [Logistics](../design/logistics.md)

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
- Status: planned
- Verified by: Planned.
- See: [Logistics](../design/logistics.md)

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
    The value is TBD, set by tech-tree balance.

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
