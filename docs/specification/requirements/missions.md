---
spec_version: 1.0.0
status: current
updated: 2026-11-11
---

# Missions requirements

Requirements for missions as data, objectives and outcomes, the debrief, chatter, environments, hazards and mission balance. The normative file format is [Format: missions, environments, events](../design/format-missions.md).

Features: [Missions](../../definition/missions.md). How to read a requirement: [Specification](../index.md#reading-a-requirement).

| Requirement | Status |
|---|---|
| [R-MSN-001 Mission roles](#r-msn-001-mission-roles) | Implemented |
| [R-MSN-002 Arrivals and waves](#r-msn-002-arrivals-and-waves) | Implemented |
| [R-MSN-003 Objectives, timer and outcomes](#r-msn-003-objectives-timer-and-outcomes) | Implemented |
| [R-MSN-004 Mission mistakes found on load](#r-msn-004-mission-mistakes-found-on-load) | Implemented |
| [R-MSN-005 Debrief](#r-msn-005-debrief) | Implemented |
| [R-MSN-006 Radio chatter](#r-msn-006-radio-chatter) | Implemented |
| [R-MSN-007 Voices](#r-msn-007-voices) | Planned |
| [R-MSN-008 Battle environments](#r-msn-008-battle-environments) | Planned |
| [R-MSN-009 Hazards](#r-msn-009-hazards) | Planned |
| [R-MSN-010 Simulator missions](#r-msn-010-simulator-missions) | Planned |
| [R-MSN-011 Base defence balance](#r-msn-011-base-defence-balance) | Planned |
| [R-MSN-012 Logistics missions](#r-msn-012-logistics-missions) | Implemented |

## Requirements

### R-MSN-001 Mission roles

A mission MUST declare roles with class lists, counts per difficulty level, faction, AI, behaviour, `protect`, `state`, hull, systems and speed; at start roles MUST be placed, counted and faced at the player.

*Rationale:* Missions as data.

- Implements: [F-MSN-001](../../definition/missions.md#f-msn-001-missions-as-data)
- Status: implemented
- Verified by: `content::mission::a_full_mission_parses`, `mission::roles_are_placed_counted_and_faced_at_the_player`; MT-3-01 – MT-3-09
- Reverse-engineered: yes
- See: [Format: missions, environments, events](../design/format-missions.md), [Missions (modding)](../../modding/missions.md)

### R-MSN-002 Arrivals and waves

A role MUST be able to arrive at a position relative to another (`at: {distance, bearing, from, spread}`) and when a condition holds (`arrive:`); a role without `arrive:` MUST enter at the start; waves MUST repeat a role's arrival.

*Rationale:* One place per group (DEV-040).

- Implements: [F-MSN-001](../../definition/missions.md#f-msn-001-missions-as-data)
- Status: implemented
- Verified by: `mission::arrivals_waves_chatter_and_a_win`
- Reverse-engineered: yes
- See: [Missions (modding)](../../modding/missions.md)

### R-MSN-003 Objectives, timer and outcomes

A mission MUST evaluate win, draw and lose conditions and optional objectives in the battle context; lose MUST beat win in the same tick; a timer MUST end the mission with its stated result; outcomes MUST be lists of checked actions with battle or campaign scope.

*Rationale:* Clear, testable results.

- Implements: [F-MSN-001](../../definition/missions.md#f-msn-001-missions-as-data)
- Status: implemented
- Verified by: `mission::lose_beats_win_and_the_timer_ends_it`; `script::action::actions_parse_and_check`; `script::battle::ids_match_and_roles_are_checked_names`
- Reverse-engineered: yes
- See: [Missions (architecture)](../../implementation/architecture/mission.md), [Conditions (script)](../../implementation/architecture/script.md)

### R-MSN-004 Mission mistakes found on load

Mistakes in a mission file (unknown role, bad timer result, unknown action) MUST be reported on load with where they are.

*Rationale:* Modders find errors before flying.

- Implements: [F-MSN-001](../../definition/missions.md#f-msn-001-missions-as-data), [F-MOD-002](../../definition/modding-toolkit.md#f-mod-002-readable-rule-files-that-explain-mistakes)
- Status: implemented
- Verified by: `content::mission::mistakes_are_reported_with_where`
- Reverse-engineered: yes
- See: [Missions (modding)](../../modding/missions.md)

### R-MSN-005 Debrief

At the end of a mission the debrief MUST show the result, its text, kills, losses, time, optional objectives and the campaign actions, and MUST offer a retry. The player jumping out MUST count as `retreat`.

*Rationale:* Closure and feedback.

- Implements: [F-MSN-002](../../definition/missions.md#f-msn-002-debrief)
- Status: implemented
- Verified by: MT-3-01 – MT-3-11 (the debrief in every mission case)
- Reverse-engineered: yes
- See: [Missions (architecture)](../../implementation/architecture/mission.md)

!!! question "Open"
    Salvage and score are applied by the strategy layer (Stage 4).

### R-MSN-006 Radio chatter

A mission MUST be able to play chatter lines when their conditions become true.

*Rationale:* Atmosphere and information.

- Implements: [F-MSN-003](../../definition/missions.md#f-msn-003-radio-chatter-and-voices)
- Status: implemented
- Verified by: `mission::arrivals_waves_chatter_and_a_win`
- Reverse-engineered: yes
- See: [Missions (modding)](../../modding/missions.md)

### R-MSN-007 Voices

Chatter and the computer voice SHOULD play recorded or original voice samples.

*Rationale:* Atmosphere.

- Implements: [F-MSN-003](../../definition/missions.md#f-msn-003-radio-chatter-and-voices), [F-CPN-007](../../definition/campaigns/features.md#f-cpn-007-presentation)
- Status: planned
- Verified by: Planned.
- See: [Stage 6](../../implementation/plan/stage-6-presentation.md)

### R-MSN-008 Battle environments

A battle MUST take place in one of the environments `space`, `orbit`, `pocket` or a surface, with a backdrop chosen from its map location.

*Rationale:* Variety; surface missions.

- Implements: [F-MSN-004](../../definition/missions.md#f-msn-004-environments-and-hazards)
- Status: planned
- Verified by: Planned.
- See: [Format: missions, environments, events](../design/format-missions.md)

### R-MSN-009 Hazards

A mission MUST be able to declare hazards: asteroids, black-hole pull, minefields and ion storms.

*Rationale:* Variety.

- Implements: [F-MSN-004](../../definition/missions.md#f-msn-004-environments-and-hazards)
- Status: planned
- Verified by: Planned.
- See: [Format: missions, environments, events](../design/format-missions.md)

### R-MSN-010 Simulator missions

The player MUST be able to set up a simulator mission on a setup screen and fly it.

*Rationale:* Original simulator.

- Implements: [F-MSN-005](../../definition/missions.md#f-msn-005-simulator-missions)
- Status: planned
- Verified by: Planned.
- See: [Stage 3](../../implementation/plan/stage-3-missions.md)

### R-MSN-011 Base defence balance

In every base-defence mission, at every difficulty including the lowest, the base MUST survive long enough for the player to engage more than one attacker.

*Rationale:* User request (F-MSN-006).

- Implements: [F-MSN-006](../../definition/missions.md#f-msn-006-base-defence-the-player-can-win)
- Status: planned
- Verified by: Planned (interlude item F.4; soak runs and a manual test).
- See: [Missions (modding)](../../modding/missions.md), [Manual tests (MT-3-01, MT-3-05)](../../implementation/testing/manual-tests.md)

!!! question "Open"
    A measurable threshold (for example a minimum time to destroy the base at the lowest level, measured with `--soak`), and which values change (base shields and hull, attacker counts and arrivals, point defence). The changes are pack data.

### R-MSN-012 Logistics missions

The `convoy_ambush` and `construction_defence` missions MUST be playable: protect the convoy against attackers; defend a facility shown in its construction-stage state.

*Rationale:* Logistics in battle.

- Implements: [F-MSN-007](../../definition/missions.md#f-msn-007-logistics-missions)
- Status: implemented
- Verified by: MT-3-02, MT-3-03
- Reverse-engineered: yes
- See: [Logistics](../design/logistics.md)

!!! question "Open"
    Generation from the strategy layer is R-STR-019.

## Revision history

| Version | Date | Change |
|---|---|---|
| 1.0.0 | 2026-11-11 | Bootstrap: R-MSN-001 – R-MSN-012 (12 requirements; 7 reverse-engineered from the code) |
