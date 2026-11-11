---
spec_version: 1.11.0
status: current
updated: 2026-11-11
---

# Missions

Battles with a purpose: roles, waves, objectives, outcomes, the debrief, environments and balance.

Requirements for these features: [Missions requirements](../specification/requirements/missions.md).

| Feature | Status |
|---|---|
| [F-MSN-001 Missions as data](#f-msn-001-missions-as-data) | Implemented |
| [F-MSN-002 Debrief](#f-msn-002-debrief) | Implemented |
| [F-MSN-003 Radio chatter and voices](#f-msn-003-radio-chatter-and-voices) | In progress |
| [F-MSN-004 Environments and hazards](#f-msn-004-environments-and-hazards) | Implemented |
| [F-MSN-005 Simulator missions](#f-msn-005-simulator-missions) | Planned |
| [F-MSN-006 Base defence the player can win](#f-msn-006-base-defence-the-player-can-win) | Implemented |
| [F-MSN-007 Logistics missions](#f-msn-007-logistics-missions) | In progress |

## Features

### F-MSN-001 Missions as data

**What.** A mission is a data file: the forces taking part, when and where they arrive, waves, objectives, a timer, outcomes and chatter.

**Why.** Modding first: new missions without code.

- **Status:** implemented
- **Source:** Plan items 3.1 and 3.2; DEV-040.
- **Specified by:** [R-MSN-001](../specification/requirements/missions.md#r-msn-001-mission-roles), [R-MSN-002](../specification/requirements/missions.md#r-msn-002-arrivals-and-waves), [R-MSN-003](../specification/requirements/missions.md#r-msn-003-objectives-timer-and-outcomes), [R-MSN-004](../specification/requirements/missions.md#r-msn-004-mission-mistakes-found-on-load)

### F-MSN-002 Debrief

**What.** After a mission the player sees the result, kills, losses, time and optional objectives, and can retry. Jumping out counts as a retreat.

**Why.** Closure and feedback for each battle.

- **Status:** implemented
- **Source:** Plan item 3.8.
- **Specified by:** [R-MSN-005](../specification/requirements/missions.md#r-msn-005-debrief)

### F-MSN-003 Radio chatter and voices

**What.** Pilots talk on the radio when things happen; later, with voices.

**Why.** Atmosphere and information.

- **Status:** in-progress
- **Source:** Plan item 3.7 (chatter done; voices planned).
- **Specified by:** [R-MSN-006](../specification/requirements/missions.md#r-msn-006-radio-chatter), [R-MSN-007](../specification/requirements/missions.md#r-msn-007-voices)

### F-MSN-004 Environments and hazards

**What.** Battles happen in open space, in orbit over a planet, in small pockets and above planet surfaces with ground installations, with hazards such as asteroid fields, black-hole pull, minefields and ion storms. Environments and hazards are pack data that a mission or scenario names.

**Why.** Variety, and the surface missions of the default campaign.

- **Status:** implemented — hazards in app 0.5.1, environments in 0.6.0, ground installations in 0.6.1. Choosing the environment from the strategy map's location comes with Stage 4.
- **Source:** Plan items 3.2 (hazards), 3.3–3.5.
- **Specified by:** [R-MSN-008](../specification/requirements/missions.md#r-msn-008-battle-environments), [R-MSN-009](../specification/requirements/missions.md#r-msn-009-hazards)

### F-MSN-005 Simulator missions

**What.** The player sets up and flies practice battles from a setup screen.

**Why.** The original has simulator missions.

- **Status:** planned
- **Source:** Plan item 3.9.
- **Specified by:** [R-MSN-010](../specification/requirements/missions.md#r-msn-010-simulator-missions)

### F-MSN-006 Base defence the player can win

**What.** In a base defence the player has time to engage more than one attacker before the base can fall, on every difficulty.

**Why.** User request: "Base destroyed too easily, player doesn't have time to engage more than one fighter before the aliens successfully destroy the base even on beginner."

- **Status:** implemented (app 0.3.1). Measured, the base was not falling too fast: on the harder levels the *player's* craft was destroyed and the debrief showed the base-lost text. The player's death is now its own result, `killed`.
- **Source:** User request (feedback); interlude item F.4.
- **Specified by:** [R-MSN-003](../specification/requirements/missions.md#r-msn-003-objectives-timer-and-outcomes), [R-MSN-005](../specification/requirements/missions.md#r-msn-005-debrief), [R-MSN-011](../specification/requirements/missions.md#r-msn-011-base-defence-balance)

### F-MSN-007 Logistics missions

**What.** Convoy-ambush and construction-defence missions put the supply chain into battles.

**Why.** Gives escort and raid missions a strategic cause.

- **Status:** in-progress
- **Source:** Plan item 3.13 (flown from the launcher; generation from the strategy layer is Stage 4).
- **Specified by:** [R-MSN-012](../specification/requirements/missions.md#r-msn-012-logistics-missions), [R-STR-019](../specification/requirements/strategy-logistics.md#r-str-019-logistics-mission-generation)

## Revision history

| Version | Date | Change |
|---|---|---|
| 1.0.0 | 2026-11-11 | Bootstrap: F-MSN-001 – F-MSN-007 (7 features) |
| 1.1.0 | 2026-11-11 | F-MSN-006 implemented (app 0.3.1); also specified by R-MSN-003 and R-MSN-005 (the `killed` result) |
| 1.11.0 | 2026-11-11 | F-MSN-004 implemented: hazards (app 0.5.1), environments (0.6.0), ground installations (0.6.1) |
