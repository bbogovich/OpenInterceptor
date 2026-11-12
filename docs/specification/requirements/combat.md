---
spec_version: 1.15.0
status: current
updated: 2026-11-11
---

# Combat requirements

Requirements for weapons, damage, systems and power, special equipment, targeting, AI pilots, stations and the look of damage.

Features: [Combat](../../definition/combat.md). How to read a requirement: [Specification](../index.md#reading-a-requirement).

| Requirement | Status |
|---|---|
| [R-CMB-001 Beam fire and energy](#r-cmb-001-beam-fire-and-energy) | Implemented |
| [R-CMB-002 Bolt travel](#r-cmb-002-bolt-travel) | Implemented |
| [R-CMB-003 Missile lock](#r-cmb-003-missile-lock) | Implemented |
| [R-CMB-004 Missile guidance and damage modes](#r-cmb-004-missile-guidance-and-damage-modes) | Implemented |
| [R-CMB-005 Mines](#r-cmb-005-mines) | Implemented |
| [R-CMB-006 Layered damage](#r-cmb-006-layered-damage) | Implemented |
| [R-CMB-007 Ship systems and repair](#r-cmb-007-ship-systems-and-repair) | Implemented |
| [R-CMB-008 Power settings](#r-cmb-008-power-settings) | Implemented |
| [R-CMB-009 Tractor and capture](#r-cmb-009-tractor-and-capture) | Implemented |
| [R-CMB-010 Cloak](#r-cmb-010-cloak) | Implemented |
| [R-CMB-011 ECM](#r-cmb-011-ecm) | Implemented |
| [R-CMB-012 Point defence](#r-cmb-012-point-defence) | Implemented |
| [R-CMB-013 Probes](#r-cmb-013-probes) | Implemented |
| [R-CMB-014 Targeting](#r-cmb-014-targeting) | Implemented |
| [R-CMB-015 Wingman orders](#r-cmb-015-wingman-orders) | Implemented |
| [R-CMB-016 Behaviours as data](#r-cmb-016-behaviours-as-data) | Implemented |
| [R-CMB-017 AI missile discipline](#r-cmb-017-ai-missile-discipline) | Implemented |
| [R-CMB-018 Station modules](#r-cmb-018-station-modules) | Implemented |
| [R-CMB-019 Model sets and damage looks](#r-cmb-019-model-sets-and-damage-looks) | Implemented |
| [R-CMB-020 Debris and explosions](#r-cmb-020-debris-and-explosions) | Implemented |
| [R-CMB-021 Shot physics](#r-cmb-021-shot-physics) | Implemented |

## Requirements

### R-CMB-001 Beam fire and energy

Each beam mount MUST have a cooldown of `1 / rate_of_fire`; a group MUST fire only if there is energy for the whole group, otherwise report `NoEnergy` and hold for 0.5 s; link modes `Linked` (pairs), `Full` (all) and `Unlinked` (one at a time) MUST cycle.

*Rationale:* The original's gunnery model.

- Implements: [F-CMB-001](../../definition/combat.md#f-cmb-001-beam-weapons)
- Status: implemented
- Verified by: `sim::combat::beams_fire_cost_energy_respect_cooldown_and_hit`, `beams_need_energy`, `link_modes_cycle_and_change_how_many_fire`; MT-1-11
- Reverse-engineered: yes
- See: [Simulation](../../implementation/architecture/sim.md)

### R-CMB-002 Bolt travel

A bolt MUST live for `range / speed`, start at its muzzle with its streak never reaching behind the muzzle, and inherit the firing ship's velocity plus the muzzle's turn velocity (R-CMB-021). Its streak MUST be drawn along its velocity *relative to the viewer*, so shots from a drifting craft streak straight out of its guns.

*Rationale:* Shots must look as if they leave the gun.

- Implements: [F-CMB-001](../../definition/combat.md#f-cmb-001-beam-weapons), [F-CMB-011](../../definition/combat.md#f-cmb-011-shot-physics)
- Status: implemented
- Verified by: `sim::combat::a_new_bolt_streak_never_reaches_behind_the_muzzle`, `a_bolt_leaves_with_the_muzzles_velocity_then_flies_straight`; MT-1-34, MT-3-14
- Reverse-engineered: yes
- See: [Simulation](../../implementation/architecture/sim.md)

### R-CMB-003 Missile lock

A lock MUST build while the target stays inside `flight.lock_angle` (10° by default, DEV-013) and within the missile's reach for the missile's `lock_time`, at a speed scaled by the sensors' efficiency and halved while the target's ECM is on; leaving the cone MUST reset it.

*Rationale:* The original's missile model.

- Implements: [F-CMB-002](../../definition/combat.md#f-cmb-002-missiles-and-mines)
- Status: implemented
- Verified by: `sim::combat::lock_builds_in_the_cone_and_missile_homes_in`, `lock_is_lost_outside_the_cone_and_dumbfire_works`; MT-1-12
- Reverse-engineered: yes
- See: [Simulation](../../implementation/architecture/sim.md)

### R-CMB-004 Missile guidance and damage modes

Missiles MUST support the guidance kinds homing, dumbfire, sidestep, parallax decoys, cloaked wait, cluster and doppelganger, and the damage modes normal, bypass shields and enveloping. The proximity fuse MUST fire on a near miss (inside `proximity_radius` and moving away); a hit inside the target's shape MUST deal impact damage; empty racks MUST be reported and skipped.

*Rationale:* Every missile of both campaigns.

- Implements: [F-CMB-002](../../definition/combat.md#f-cmb-002-missiles-and-mines)
- Status: implemented
- Verified by: `sim::guidance::homing_hits_and_dumbfire_misses_an_offset_target`, `sidestep_still_arrives`, `parallax_decoys_lure_enemy_missiles`, `cloaked_wait_sleeps_until_prey_comes_near`, `cluster_splits_into_submunitions`, `doppelganger_launches_a_decoy_ship_that_expires`, `damage_modes`; `sim::combat::proximity_fuse_fires_on_a_near_miss`, `empty_racks_report_and_cycle`; MT-2-24
- Reverse-engineered: yes
- See: [Missiles and mines](../../modding/weapons.md), [Deviations (DEV-010, DEV-036)](../../implementation/deviations/index.md)

### R-CMB-005 Mines

A mine MUST stay in place and detonate with an area blast when an enemy comes within its trigger range.

*Rationale:* Original and default-campaign weapons.

- Implements: [F-CMB-002](../../definition/combat.md#f-cmb-002-missiles-and-mines)
- Status: implemented
- Verified by: `sim::ai::mines_go_off_when_an_enemy_comes_near`; MT-2-24
- Reverse-engineered: yes
- See: [Missiles and mines](../../modding/weapons.md)

### R-CMB-006 Layered damage

Damage MUST strike the fore or aft shield (by the side hit), then armour, then hull; hull at zero MUST destroy the ship and make every ship targeting or towing it let go; a shield facing reaching zero MUST say "FORE/AFT SHIELD DOWN".

*Rationale:* The original's damage model.

- Implements: [F-CMB-003](../../definition/combat.md#f-cmb-003-damage-and-ship-systems)
- Status: implemented
- Verified by: `sim::combat::damage_layers_and_aft_shield`; MT-1-13
- Reverse-engineered: yes
- See: [Simulation](../../implementation/architecture/sim.md)

### R-CMB-007 Ship systems and repair

Each system MUST have hit points and a condition: full effect above half, half effect at or below half, stopped at zero until repaired back to 25 %, destroyed for the battle below `−destroy_below × hit_points`. Damage past the armour MUST strike a random system; repair MUST take the worst system first. A ship whose engines and manoeuvring are out MUST drift disabled.

*Rationale:* The original's system damage; thresholds are engine values (DEV-029).

- Implements: [F-CMB-003](../../definition/combat.md#f-cmb-003-damage-and-ship-systems)
- Status: implemented
- Verified by: `sim::systems::damage_knocks_systems_out_and_repairs_bring_them_back`, `systems_can_be_destroyed_beyond_repair`, `disabled_when_engines_and_manoeuvring_are_out`; `sim::ai::disabled_ships_drift`; MT-2-03
- Reverse-engineered: yes
- See: [Simulation](../../implementation/architecture/sim.md), [Rulesets](../../modding/rulesets.md)

### R-CMB-008 Power settings

Beam and shield charge settings A–E and switched-on equipment MUST share `power.budget` steps; raising one setting MUST take from the other when the budget is full; cycling MUST wrap to A; the pilot MUST be able to transfer energy and move shield points fore and aft within caps; recharge MUST follow the settings.

*Rationale:* The original's power management, as charge settings (DEV-024).

- Implements: [F-CMB-004](../../definition/combat.md#f-cmb-004-power-management)
- Status: implemented
- Verified by: `sim::systems::raising_takes_power_from_the_other_setting`, `cycling_wraps_to_a`, `balance_caps_and_message`, `shield_balance_moves_points_forward_and_recharge_follows_power`, `requests_change_power_and_balance`; `sim::flight::energy_and_shields_recharge_up_to_capacity`; MT-2-01, MT-2-02
- Reverse-engineered: yes
- See: [Simulation](../../implementation/architecture/sim.md)

### R-CMB-009 Tractor and capture

The tractor MUST need power, a fitting and a disabled target in range; while it is on, beams MUST be off-line; a ship holding a disabled target when it jumps MUST capture it. An object of a `towable` class (a cargo container) MUST be towable even when not disabled; tractoring one mounted on a hub MUST tear it off, and a ship jumping while it holds one MUST capture it.

*Rationale:* Original capture of disabled craft; grappler tugs stealing cargo (the user's convoy design, app 0.9.2).

- Implements: [F-CMB-005](../../definition/combat.md#f-cmb-005-special-equipment)
- Status: implemented (`towable` app 0.9.2, main `0d1c522`)
- Verified by: `sim::systems::tractor_needs_power_a_fitting_and_a_target_in_range`, `tractor_holds_a_disabled_target_and_captures_it_on_jump`; `sim::ai::tugs_tow_disabled_enemies_away`; `sim::tests::a_tug_tears_a_container_off_and_prefers_a_disabled_ship`; MT-2-04, MT-4-26
- Reverse-engineered: yes
- See: [Simulation](../../implementation/architecture/sim.md)

### R-CMB-010 Cloak

A cloaked ship MUST vanish from enemy sensors beyond detection range; firing beams or a missile MUST drop the cloak with "CLOAK DROPPED".

*Rationale:* Original equipment; no invisible attacks (DEV-025).

- Implements: [F-CMB-005](../../definition/combat.md#f-cmb-005-special-equipment)
- Status: implemented
- Verified by: `sim::systems::cloaked_ships_vanish_from_enemy_sensors_beyond_range`; MT-2-05
- Reverse-engineered: yes
- See: [Simulation](../../implementation/architecture/sim.md)

### R-CMB-011 ECM

While ECM is on, incoming missiles MUST lose their target and enemy lock MUST build at half speed.

*Rationale:* Original equipment.

- Implements: [F-CMB-005](../../definition/combat.md#f-cmb-005-special-equipment)
- Status: implemented
- Verified by: `sim::systems::ecm_makes_incoming_missiles_lose_their_target`; MT-2-06
- Reverse-engineered: yes
- See: [Simulation](../../implementation/architecture/sim.md)

### R-CMB-012 Point defence

A ship with point defence MUST fire at the nearest hostile missile (decoys included) within `range` when cooled down and with energy, roll `chance` on the world's seeded generator, and spend `energy` per shot.

*Rationale:* Capital ships and stations defend against missiles (values derived, DEV-038).

- Implements: [F-CMB-005](../../definition/combat.md#f-cmb-005-special-equipment)
- Status: implemented
- Verified by: `sim::combat::point_defence_shoots_down_hostile_missiles`; MT-2-31
- Reverse-engineered: yes
- See: [Stations, modules and probes](../../modding/stations.md)

### R-CMB-013 Probes

A sensor probe MUST extend its side's radar to 8 km around it; a jamming probe MUST halve hostile radar range and missile-lock speed within 2.5 km; the radar MUST hide cloaked ships.

*Rationale:* Original probes given battle effects (derived, DEV-039).

- Implements: [F-CMB-005](../../definition/combat.md#f-cmb-005-special-equipment)
- Status: implemented
- Verified by: `sim::probes_extend_radar_and_jam_their_enemies`; MT-2-32
- Reverse-engineered: yes
- See: [Stations, modules and probes](../../modding/stations.md)

### R-CMB-014 Targeting

The pilot MUST be able to cycle targets forwards and backwards with wrap-around, select the nearest hostile (ignoring friends and protected objectives) and select the most central ship in the sights.

*Rationale:* Original targeting keys.

- Implements: [F-CMB-006](../../definition/combat.md#f-cmb-006-targeting)
- Status: implemented
- Verified by: `sim::targeting::cycles_forwards_and_backwards_with_wraparound`, `nearest_hostile_ignores_friends`, `in_sights_picks_the_most_central`, `nothing_to_target`; `sim::protected_objectives_are_not_picked_as_the_nearest_enemy`; MT-1-10
- Reverse-engineered: yes
- See: [Simulation](../../implementation/architecture/sim.md)

### R-CMB-015 Wingman orders

Wingmen MUST hold formation slots and obey: formation (ignore my target), attack my target, fire at will, evasive (10 s), retreat (flee and jump out) and status report, given from a comms menu.

*Rationale:* Original wingman commands.

- Implements: [F-CMB-007](../../definition/combat.md#f-cmb-007-wingmen-and-orders)
- Status: implemented
- Verified by: `sim::ai::wingman_closes_on_its_slot`, `orders_switch_wingmen_to_attack`, `evasive_ends_and_retreat_jumps_out`; `app::flight::commands_target_and_order_wingmen`; MT-1-14, MT-2-08
- Reverse-engineered: yes
- See: [Simulation](../../implementation/architecture/sim.md)

### R-CMB-022 Wingmen keep their distance

A wingman's formation slot MUST come from a named formation (`formations:`, R-MOD-047): the mission role's `formation:`, else `flight.formation` (default `vee`); wingmen of every wingman role MUST take successive slots of it. The slot MUST lie at least `flight.formation_gap` (default 3) × the two ships' combined radii from the leader; a closer slot from data is pushed out along its own direction. The wing order *Wingmen: next formation* (Shift+O) MUST move the player's wingmen into the campaign's next formation, in list order, wrapping.

Every moving AI pilot — wingmen and enemies alike — MUST avoid collisions **predictively**: it MUST project each other object's path against its own (constant velocities) to their closest approach within `AVOID_HORIZON` (2.5 s; 1 s for static objects and for its own target, so that firing passes are not broken off), and when the miss would be under 1.5 × the combined radii + 10 m (static objects: the combined radii + 30 m), or the other is already inside twice the combined radii and closing, it MUST steer to widen the miss and ease off if the danger is ahead. A leader MUST ignore its own wingmen (they keep clear of it); static objects and turrets MUST NOT avoid.

*Rationale:* The user's request (see F-CMB-007): wingmen collided with the player and kept pushing.

- Implements: [F-CMB-007](../../definition/combat.md#f-cmb-007-wingmen-and-orders)
- Status: implemented
- Verified by: `sim::ai::formation_slots_keep_a_gap_from_the_leader`, `sim::ai::a_wingman_steers_clear_of_the_leader`, `sim::ai::collision_avoidance_predicts_the_closest_approach`, `sim::ai::tests::the_formation_order_cycles_the_campaigns_formations`; `mission::tests::wingmen_take_successive_slots_of_their_formation`; `content::formation::tests::shapes_place_followers_behind_and_beside_the_leader`, `custom_slots_then_a_vee_and_errors`; MT-3-62, MT-3-63, MT-4-01
- See: [Formations (modding)](../../modding/reference.md#formations), [Simulation › AI](../../implementation/architecture/sim.md#ai-simai)

!!! note "Formations (MD1, app 0.6.6, main `6d390d7`)"
    Slots come from `formations:` pack data (`content::formation`) instead of the fixed side offsets: shapes `vee`, `line_abreast`, `echelon`, `box`, `diamond` or a custom `slots:` list, in units of `spacing`. Both campaigns ship the five shapes in `rules/formations.yaml`. An unknown formation is a load error.

!!! note "From user feedback (app 0.6.7, main `c0e2516`)"
    The user's playtest reported wingmen and enemies ramming each other and the player. Avoidance was reactive and for wingmen only; it is now predictive and for every moving AI pilot (`ai::avoid_collision`). The default `guard_station` was given `hit_points: {per_level: [11000, 8000, 8000, 8000]}` and `defend_station` two raiders at cadet, keeping R-MSN-011's 150 s undefended promise with the AI wasting fewer runs.

### R-CMB-016 Behaviours as data

AI pilots MUST be driven by behaviour definitions: states using the built-in manoeuvres (idle, circle, patrol, attack, evade, retreat, turret, tow, escort, travel, mine), transitions on conditions, skill (aim, reaction), fire discipline, turret arcs, target preferences (`prefer`) and systems use (cloak, ECM, power). Any definition MUST accept difficulty `variants` and `{per_level: …}` values.

*Rationale:* Modding first; the original's AI is code, so its behaviours are derived (DEV-031, DEV-032).

- Implements: [F-CMB-008](../../definition/combat.md#f-cmb-008-enemies-driven-by-data-with-difficulty-levels)
- Status: implemented
- Verified by: `content::behaviour::parses_states_transitions_and_skill`, `reports_mistakes`, `the_builtin_attack_is_valid`, `target_preferences_parse_and_reject_unknown_selectors`; `sim::ai::transitions_switch_states_on_conditions`, `aim_skill_decides_accuracy`, `turrets_fire_off_the_nose`, `escorts_hold_formation_until_a_hostile_comes_close`; `sim::preferences_pull_the_ai_past_a_nearer_target`, `sim::ai_pilots_switch_cloak_ecm_and_power_by_condition`; MT-2-20, MT-2-27, MT-2-33
- Reverse-engineered: yes
- See: [Format: behaviours and scripting](../design/format-behaviour.md), [Behaviours and conditions](../../modding/behaviours.md)

!!! note "Added (app 0.9.2): `escort`, `travel`, `tow` `escape`"
    `escort` flies in formation on the leader the mission gives (`escort:` on the role) and attacks a hostile within `range` of the leader or itself; `travel` flies straight on for `range`, then jumps; `tow` takes a towable object when no disabled ship is near, skips a prize another ship is towing, and hauls it `escape` (default 2.5 km) before jumping.

### R-CMB-017 AI missile discipline

AI pilots SHOULD launch missiles only when closing on their target, staggered between pilots, each pilot drawing from its own random sequence.

*Rationale:* Missiles from every enemy at once overwhelmed the player.

- Implements: [F-CMB-008](../../definition/combat.md#f-cmb-008-enemies-driven-by-data-with-difficulty-levels)
- Status: implemented
- Verified by: None recorded.
- Reverse-engineered: yes
- See: [Missions (architecture)](../../implementation/architecture/mission.md)

!!! question "Open"
    No unit test names this behaviour; one should be added.

### R-CMB-018 Station modules

Modules MUST mount at a hub's attachment points (OBJ `slot_<name>` groups, original manipulators by index), follow the hub, provide turrets and shield generators, and fall with the hub, leaving wrecks in their `destroyed` look. Any ship class MAY be a hub: a module MUST NOT collide with its own hub or with another module of the same hub, a craft's collision avoidance MUST ignore its own modules, and the modules still mounted MUST leave by hyperspace with their hub.

*Rationale:* Base assaults and defences.

- Implements: [F-CMB-009](../../definition/combat.md#f-cmb-009-stations-with-modules)
- Status: implemented
- Verified by: `sim::station_modules_mount_follow_and_fall_with_their_hub`, `sim::tests::cargo_modules_count_until_torn_off_destroyed_or_jumped_away_with_the_hub` (a moving hub keeps its speed; modules jump with it), `formats::obj::slot_groups_become_attachment_points`; MT-2-22, MT-2-23, MT-4-26
- Reverse-engineered: yes
- See: [Stations, modules and probes](../../modding/stations.md), [Deviations (DEV-035)](../../implementation/deviations/index.md)

!!! question "Open"
    Damage zones on hubs are not built (Stage 2 deferred list).

### R-CMB-019 Model sets and damage looks

A model set MUST define the intact model, texture swaps that accumulate with damage levels (by hull fraction), and named states that may swap the texture, the mesh and the debris set; the viewer MUST show every look.

*Rationale:* Visible damage; construction stages.

- Implements: [F-CMB-010](../../definition/combat.md#f-cmb-010-damage-looks-debris-and-explosions), [F-STR-008](../../definition/strategy-logistics.md#f-str-008-facilities-under-construction)
- Status: implemented
- Verified by: `content::looks::swaps_accumulate_with_damage_and_state`, `texture_keys_ignore_folders_case_and_extensions`; `app::viewer::looks_cover_damage_states_and_debris`; MT-2-25
- Reverse-engineered: yes
- See: [Format: objects and models](../design/format-objects.md), [Looks](../../modding/looks.md)

!!! note "Playtest (app 0.9.1): size"
    A model set's `scale` scales its collision shape with it: the default campaign's dreadnought is set to 1.25. The default fighters were rebuilt larger (maximum radius 1.5 / 1.8 / 2.4 m for the Comet, Meteor and Thunderbolt, from 0.9 / 1.1 / 1.6) after the user found them too hard to hit — a content change, no requirement text changed.

!!! question "Open"
    Slot size classes, overlay groups and `lod<n>_*` files are not built (Stage 2 deferred list).

### R-CMB-020 Debris and explosions

Destruction MUST spawn the object's debris set (pieces with offsets, speed and spin ranges, random picks, lifetimes, seeded) and its effect (fireball size, duration, colour, optional shock wave, sound).

*Rationale:* Feedback on kills; engine fireballs stand in for sprites (DEV-034).

- Implements: [F-CMB-010](../../definition/combat.md#f-cmb-010-damage-looks-debris-and-explosions)
- Status: implemented
- Verified by: `content::looks::debris_and_effects_parse_with_defaults`; `render::effects::events_create_effects_that_expire`, `explosion_curve_grows_and_fades`; MT-1-13, MT-2-25
- Reverse-engineered: yes
- See: [Looks](../../modding/looks.md)

!!! question "Open"
    XSPT sprite explosions, shield flashes, lights and debris collisions are deferred.

### R-CMB-021 Shot physics

A ruleset option MUST choose how shots move, and the default MUST be physics: a shot MUST leave the muzzle with the craft's velocity plus the craft's turn velocity at the muzzle (angular velocity × muzzle offset) plus its own muzzle speed, and once fired MUST move independently of the craft's orientation.

The option is `flight: {bolt_frame: world | ship}`. `world` (the default) is the physics above. With `ship`, a shot MUST move in its owner's frame for `flight.bolt_carry` (a time, default 0.3 s), turning with the nose, and then fly straight with its last velocity.

*Rationale:* User request (F-CMB-011); decision D1.

- Implements: [F-CMB-011](../../definition/combat.md#f-cmb-011-shot-physics)
- Status: implemented
- Verified by: `sim::combat::a_bolt_leaves_with_the_muzzles_velocity_then_flies_straight`, `with_bolt_frame_ship_a_bolt_turns_with_the_nose_while_carried`; `content::rules::builds_typed_definitions_with_inheritance`; MT-3-14
- See: [Simulation](../../implementation/architecture/sim.md), [Mod file reference (`flight`)](../../modding/reference.md#flight)

## Revision history

| Version | Date | Change |
|---|---|---|
| 1.0.0 | 2026-11-11 | Bootstrap: R-CMB-001 – R-CMB-021 (21 requirements; 20 reverse-engineered from the code) |
| 1.1.0 | 2026-11-11 | R-CMB-021 implemented (app 0.3.1): `flight.bolt_frame` (`world` default, `ship` with `bolt_carry`), open note resolved. R-CMB-002: bolts also carry the muzzle's turn velocity; streaks drawn relative to the viewer |
| 1.5.0 | 2026-11-11 | R-CMB-022 wingmen keep their distance and avoid collisions |
| 1.13.0 | 2026-11-11 | R-CMB-022 revised: slots from named formations with `formation:` / `flight.formation` and the Shift+O next-formation order (MD1, app 0.6.6); predictive collision avoidance for every moving AI pilot (app 0.6.7, from user feedback) |
| 1.15.0 | 2026-11-11 | From the user's convoy design and playtest (app 0.9.1 – 0.9.2): R-CMB-009 revised — `towable` objects, torn off their hub by a tractor, captured on the jump; R-CMB-016 — the `escort` and `travel` manoeuvres and `tow` `escape`; R-CMB-018 revised — modules on any hub, no collisions within one craft, modules jump with their hub; R-CMB-019 note — the dreadnought's scale and the larger default fighters |
