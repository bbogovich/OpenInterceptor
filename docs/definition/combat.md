---
spec_version: 1.1.0
status: current
updated: 2026-11-11
---

# Combat

Weapons, damage, ship systems, special equipment, targeting, wingmen and the enemies' behaviour.

Requirements for these features: [Combat requirements](../specification/requirements/combat.md).

| Feature | Status |
|---|---|
| [F-CMB-001 Beam weapons](#f-cmb-001-beam-weapons) | Implemented |
| [F-CMB-002 Missiles and mines](#f-cmb-002-missiles-and-mines) | Implemented |
| [F-CMB-003 Damage and ship systems](#f-cmb-003-damage-and-ship-systems) | Implemented |
| [F-CMB-004 Power management](#f-cmb-004-power-management) | Implemented |
| [F-CMB-005 Special equipment](#f-cmb-005-special-equipment) | Implemented |
| [F-CMB-006 Targeting](#f-cmb-006-targeting) | Implemented |
| [F-CMB-007 Wingmen and orders](#f-cmb-007-wingmen-and-orders) | Implemented |
| [F-CMB-008 Enemies driven by data, with difficulty levels](#f-cmb-008-enemies-driven-by-data-with-difficulty-levels) | Implemented |
| [F-CMB-009 Stations with modules](#f-cmb-009-stations-with-modules) | Implemented |
| [F-CMB-010 Damage looks, debris and explosions](#f-cmb-010-damage-looks-debris-and-explosions) | In progress |
| [F-CMB-011 Shot physics](#f-cmb-011-shot-physics) | Implemented |

## Features

### F-CMB-001 Beam weapons

**What.** Beam weapons draw energy, fire at their own rate and can be linked to fire singly, in pairs or all together.

**Why.** The original's gunnery model; energy makes power management matter.

- **Status:** implemented
- **Source:** Plan item 1.6.
- **Specified by:** [R-CMB-001](../specification/requirements/combat.md#r-cmb-001-beam-fire-and-energy), [R-CMB-002](../specification/requirements/combat.md#r-cmb-002-bolt-travel)

### F-CMB-002 Missiles and mines

**What.** Missiles need a lock and home in; special missiles sidestep, decoy, wait cloaked, split or copy a ship; mines blow up when an enemy comes near.

**Why.** Every weapon of both campaigns must fight as described.

- **Status:** implemented
- **Source:** Plan items 1.6 and 2.10.
- **Specified by:** [R-CMB-003](../specification/requirements/combat.md#r-cmb-003-missile-lock), [R-CMB-004](../specification/requirements/combat.md#r-cmb-004-missile-guidance-and-damage-modes), [R-CMB-005](../specification/requirements/combat.md#r-cmb-005-mines)

### F-CMB-003 Damage and ship systems

**What.** Hits strike the shield facing the attacker, then armour, then hull. Damage that gets through knocks out ship systems, which damage control repairs; a craft without engines and manoeuvring drifts disabled.

**Why.** The original's damage model; disabled ships can be captured.

- **Status:** implemented
- **Source:** Plan item 2.5.
- **Specified by:** [R-CMB-006](../specification/requirements/combat.md#r-cmb-006-layered-damage), [R-CMB-007](../specification/requirements/combat.md#r-cmb-007-ship-systems-and-repair)

### F-CMB-004 Power management

**What.** The pilot shares a power budget between beams, shields and switched-on equipment, transfers energy and balances the shields fore and aft.

**Why.** A piloting function of the original game.

- **Status:** implemented
- **Source:** Plan item 2.6; DEV-024.
- **Specified by:** [R-CMB-008](../specification/requirements/combat.md#r-cmb-008-power-settings)

### F-CMB-005 Special equipment

**What.** Tractor beam (tow and capture), cloak, ECM, point defence and sensor and jamming probes work in battle, for the player and for AI pilots.

**Why.** All are equipment of the original game; probes and point defence give both campaigns tactical depth.

- **Status:** implemented
- **Source:** Plan items 2.7 and 2.10.
- **Specified by:** [R-CMB-009](../specification/requirements/combat.md#r-cmb-009-tractor-and-capture), [R-CMB-010](../specification/requirements/combat.md#r-cmb-010-cloak), [R-CMB-011](../specification/requirements/combat.md#r-cmb-011-ecm), [R-CMB-012](../specification/requirements/combat.md#r-cmb-012-point-defence), [R-CMB-013](../specification/requirements/combat.md#r-cmb-013-probes)

### F-CMB-006 Targeting

**What.** The pilot cycles targets, picks the nearest hostile or the one in the sights.

**Why.** A piloting function of the original game.

- **Status:** implemented
- **Source:** Plan item 1.7.
- **Specified by:** [R-CMB-014](../specification/requirements/combat.md#r-cmb-014-targeting)

### F-CMB-007 Wingmen and orders

**What.** Wingmen fly in formation and obey orders from a comms menu: attack my target, fire at will, evasive, retreat, status report.

**Why.** A piloting function of the original game.

- **Status:** implemented
- **Source:** Plan items 1.8 and 2.16; DEV-018 (resolved).
- **Specified by:** [R-CMB-015](../specification/requirements/combat.md#r-cmb-015-wingman-orders)

### F-CMB-008 Enemies driven by data, with difficulty levels

**What.** Every enemy type flies a behaviour written as data, with skill, fire discipline and use of its systems, and is harder or easier by difficulty level.

**Why.** Modding first: a modder writes new enemies without code. The original's AI is code, so its behaviours are derived.

- **Status:** implemented
- **Source:** Plan items 2.8 and 2.9; DEV-031, DEV-032.
- **Specified by:** [R-CMB-016](../specification/requirements/combat.md#r-cmb-016-behaviours-as-data), [R-CMB-017](../specification/requirements/combat.md#r-cmb-017-ai-missile-discipline)

### F-CMB-009 Stations with modules

**What.** Stations carry turrets, shield generators and other modules at attachment points; destroying the hub takes the modules with it.

**Why.** Base assaults and defences of both campaigns.

- **Status:** implemented
- **Source:** Plan item 2.11.
- **Specified by:** [R-CMB-018](../specification/requirements/combat.md#r-cmb-018-station-modules)

### F-CMB-010 Damage looks, debris and explosions

**What.** A craft looks more damaged as it is hurt, breaks into debris and explodes; stations can be shown under construction.

**Why.** Feedback on what the guns are doing; construction stages serve logistics.

- **Status:** in-progress
- **Source:** Plan items 2.2–2.4 (XSPT sprite explosions and model-set extras deferred).
- **Specified by:** [R-CMB-019](../specification/requirements/combat.md#r-cmb-019-model-sets-and-damage-looks), [R-CMB-020](../specification/requirements/combat.md#r-cmb-020-debris-and-explosions)

### F-CMB-011 Shot physics

**What.** Shots leave the gun with the craft's own motion, so while the craft turns they still appear to fire forwards in a straight line relative to the pilot. Once fired, a shot is independent of where the craft points.

**Why.** User request: "Weapons, when fired, should start with the same angular momentum as the craft so that they appear to fire forwards in a straight line relative to the player." Decision D1 (2026-11-11): a rules option, physics by default.

- **Status:** implemented (app 0.3.1)
- **Source:** User request (feedback); decision D1; interlude item F.2.
- **Specified by:** [R-CMB-002](../specification/requirements/combat.md#r-cmb-002-bolt-travel), [R-CMB-021](../specification/requirements/combat.md#r-cmb-021-shot-physics)

## Revision history

| Version | Date | Change |
|---|---|---|
| 1.0.0 | 2026-11-11 | Bootstrap: F-CMB-001 – F-CMB-011 (11 features) |
| 1.1.0 | 2026-11-11 | F-CMB-011 implemented (app 0.3.1) |
