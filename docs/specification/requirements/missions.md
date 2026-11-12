---
spec_version: 1.15.0
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
| [R-MSN-007 Voices](#r-msn-007-voices) | Implemented |
| [R-MSN-008 Battle environments](#r-msn-008-battle-environments) | Implemented |
| [R-MSN-009 Hazards](#r-msn-009-hazards) | Implemented |
| [R-MSN-010 Simulator missions](#r-msn-010-simulator-missions) | Implemented |
| [R-MSN-011 Base defence balance](#r-msn-011-base-defence-balance) | Implemented |
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

A role MUST be able to arrive at a position relative to another (`at: {distance, bearing, from, spread}`) and when a condition holds (`arrive:`); a role without `arrive:` MUST enter at the start; waves MUST repeat a role's arrival. A wave with `every: <time>` MUST come again that long after it last came, for as long as its `when` holds. A role with `escort: <role>` MUST fly in its formation on the newest live member of that role.

*Rationale:* One place per group (DEV-040). Repeating waves and escorts: the supply-convoy battle's tugs come every two minutes, each with its own squadron (the user's design, app 0.9.2).

- Implements: [F-MSN-001](../../definition/missions.md#f-msn-001-missions-as-data)
- Status: implemented (`every:` and `escort:` app 0.9.2, main `0d1c522`)
- Verified by: `mission::arrivals_waves_chatter_and_a_win`; `sim::ai::tests::escorts_hold_formation_until_a_hostile_comes_close`; MT-4-26
- Reverse-engineered: yes
- See: [Missions (modding)](../../modding/missions.md)

### R-MSN-003 Objectives, timer and outcomes

A mission MUST evaluate win, draw and lose conditions and optional objectives in the battle context; lose MUST beat win in the same tick; a timer MUST end the mission with its stated result; outcomes MUST be lists of checked actions with battle or campaign scope. The results are `win`, `lose`, `draw`, `retreat` and `killed`. The engine MUST set `killed` when the player's craft is destroyed, checked before the mission's own conditions; `killed` MUST take the mission's `lose` outcomes unless the mission gives `outcomes: {killed: …}`.

*Rationale:* Clear, testable results; the debrief must say what actually happened (the player's death is not the objective failing).

- Implements: [F-MSN-001](../../definition/missions.md#f-msn-001-missions-as-data), [F-MSN-006](../../definition/missions.md#f-msn-006-base-defence-the-player-can-win)
- Status: implemented
- Verified by: `mission::lose_beats_win_and_the_timer_ends_it`, `the_players_death_is_its_own_result_with_the_lose_outcomes`; `script::action::actions_parse_and_check`; `script::battle::ids_match_and_roles_are_checked_names`; `sim::tests::cargo_modules_count_until_torn_off_destroyed_or_jumped_away_with_the_hub` (`cargo(role)`); MT-4-27
- Reverse-engineered: yes

!!! note "Added (app 0.9.2): cargo"
    The battle query `cargo(role)` gives the share (0..1) of the role's cargo containers still aboard; a mission's `cargo: <role>` puts that share at the end into the debrief and the campaign's battle report, where a flown convoy escort delivers it (R-STR-019).
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

At the end of a mission the debrief MUST show the result, its text, kills, losses, time, optional objectives and the campaign actions, and MUST offer a retry. The player jumping out MUST count as `retreat`. For `killed` the headline MUST be "YOUR CRAFT WAS DESTROYED" and the text MUST be the mission's `debrief: {killed: …}` if given, otherwise none (never the `lose` text, which describes the objective).

When the result is decided the verdict MUST be shown at once ("MISSION ACCOMPLISHED", "MISSION FAILED", "YOUR CRAFT WAS DESTROYED" or "MISSION ENDED") and the battle MUST keep running, so its end can be watched; the debrief MUST appear `flight.debrief_delay` (rules data, a time, default 30 s) after the result, with a countdown, and Enter MUST show it at once. A `retreat` (the player leaving) MUST show the debrief at once.

*Rationale:* Closure and feedback.

- Implements: [F-MSN-002](../../definition/missions.md#f-msn-002-debrief)
- Status: implemented
- Verified by: `mission::the_players_death_is_its_own_result_with_the_lose_outcomes`; `app::flight::tests::the_debrief_waits_for_the_delay_after_the_result`; MT-3-01 – MT-3-11 (the debrief in every mission case), MT-3-17, MT-3-64
- Reverse-engineered: yes
- See: [Missions (architecture)](../../implementation/architecture/mission.md)

!!! note "From user feedback (app 0.7.4, main `9295707`)"
    The user's playtest asked to see the battle end: the result is announced (verdict and computer voice) at once, and the battle runs on for `flight.debrief_delay` before the debrief. `flight::debrief_due` decides when; the wait is skipped with Enter.

!!! note "Campaign battles (app 0.7.1)"
    In a campaign battle the debrief returns to the strategy screen (Enter or Esc), which applies salvage, score, money and flags (R-STR-003, R-STR-010).

### R-MSN-006 Radio chatter

A mission MUST be able to play chatter lines when their conditions become true.

*Rationale:* Atmosphere and information.

- Implements: [F-MSN-003](../../definition/missions.md#f-msn-003-radio-chatter-and-voices)
- Status: implemented
- Verified by: `mission::arrivals_waves_chatter_and_a_win`
- Reverse-engineered: yes
- See: [Missions (modding)](../../modding/missions.md)

### R-MSN-007 Voices

Chatter and the computer voice SHOULD play recorded or original voice samples. As built:

- A pack MUST be able to define radio voice sets as data, `voices: {<set>: {<line>: clip or [clips]}}`, and a ship class MUST be able to name one with `voice: <set>`; an unknown set MUST be a rules error.
- When the player gives a wing order (fire at will, attack my target, ignore my target, evasive, retreat), one of the player's living wingmen whose voice has the line MUST play an `acknowledge` clip; a status request MUST play a `status` clip; when one of the player's wingmen is destroyed, *another* MUST play a `down` clip. With several clips for a line, one is chosen each time.
- A mission chatter entry MAY name a clip with `voice:`; it MUST play when the line is said. Voice clips and chatter voices MAY be `cue:<Name>` references, resolved through the sound cue table like `sounds:` (app 0.6.7).
- When the mission's result is decided (since app 0.7.4 that is before the debrief, R-MSN-005), the named sound `mission_accomplished` MUST play for a win and `mission_failed` for any other result (the computer voice).
- A ship without a voice, a voice without the line, or a campaign without these sounds MUST stay silent without error.

The `default` campaign MUST ship the voice set `guard_pilot` (synthesised radio clips, used by the Comet, Meteor, Thunderbolt and Victory) and the two mission sounds; the `interceptor` campaign maps the computer voice from the original's cue table (`Comp_MissionAccomplished`, `Comp_MissionFailed`).

*Rationale:* Atmosphere, and the original's radio and computer voice. Voices are data, so a mod can give its pilots voices without code.

- Implements: [F-MSN-003](../../definition/missions.md#f-msn-003-radio-chatter-and-voices), [F-CPN-007](../../definition/campaigns/features.md#f-cpn-007-presentation)
- Status: implemented (app 0.6.2, main `5bdb47f`)
- Verified by: `app::flight::tests::a_wingman_answers_in_its_own_voice`, `content::rules::tests::builds_typed_definitions_with_inheritance` (voice sets and clips); MT-3-47 – MT-3-50
- See: [Radio voices (modding)](../../modding/voices.md), [Audio](../../implementation/architecture/audio.md)

!!! question "Open"
    The `interceptor` campaign's **pilot** chatter clips (the numbered `FPilot1xx` and `MPilotxxx` files) are not mapped to voice lines: which line each clip says is not known (*unknown*), and a wrong mapping would put the wrong words in a wingman's mouth. The user is asked to identify the lines by listening (for example in the launcher's asset browser, Sounds tab) so that `packs/interceptor` can map them to `acknowledge`, `status`, `down` and later lines. Also open: `--check` does not load voice clips. (Resolved in app 0.6.7, main `65141dc`: `cue:` references now resolve inside `voices:` and chatter `voice:`.)

### R-MSN-008 Battle environments

A pack MUST be able to define battle environments as data, `environments: {<id>: {physics, arena, gravity, air_density, ceiling, terrain, sky, backdrop}}`, and a mission or scenario MUST be able to name one with `environment: <id>`. Without it the battle MUST take place in open space, exactly as before.

- `physics: space` (the default) MUST leave the campaign's flight model unchanged; `physics: atmosphere` MUST apply atmospheric flight (R-FLT-011) over the environment's `terrain` (R-FLT-012), which it then MUST name.
- `arena`, if given, MUST replace `flight.arena_radius` for battles in that environment (a small "pocket").
- `backdrop: {planet: {color, size, direction}}` MUST draw a lit planet disc of that colour and angular size (degrees) in that direction (orbit).
- `backdrop: {nebula: {color, second_color, density, seed}}` MUST surround the battle with soft clouds shading between the two colours, as thick as `density` (0..1), with the stars dimmed behind them; `seed` MUST choose the pattern, the same for the same seed (a pocket inside a nebula; app 0.9.1).
- `sky: {top, horizon, fog, fog_distance, stars}` MUST replace black space with a sky gradient from `top` (zenith) to `horizon`, fade distant objects into `fog` (default: the horizon colour) by `fog_distance`, and dim the starfield to `stars` (0–1).
- An object class of `kind: ground` (a ground installation: a colony, an outpost, an anti-air battery) MUST be static and MUST be set down on the terrain wherever it is placed, at the ground height plus a quarter of its collision radius. Anti-air is a turret on the ground: a `kind: ground` class with beams, a `turret_arc` and a turret behaviour. The environment MUST be set before anything is placed, so this holds for scenario objects and mission roles alike; in space a ground object stays where it is placed.
- Unknown environment ids in a mission MUST be reported by `--check`; a scenario with one MUST fail to build with a message naming it. An atmosphere without a terrain, or naming an unknown terrain, MUST be a rules error.

The `default` campaign MUST ship `orbit`, `pocket` (with a nebula backdrop), `moon_surface` and `dusk_hills`, and the ground installations `guard_colony` and `dominion_flak` (an anti-air battery); the `interceptor` campaign stays in space, as the original.

*Rationale:* Variety, and the surface missions of the default campaign. Environments are pack data so a mod can add worlds without code.

- Implements: [F-MSN-004](../../definition/missions.md#f-msn-004-environments-and-hazards)
- Status: implemented (app 0.6.0, main `af003f6`; ground installations app 0.6.1, main `de417df`)
- Verified by: `content::environment::tests::environments_and_terrains_parse_and_report_mistakes`, `content::environment::nebula_tests::nebula_clouds_vary_stay_in_range_and_repeat_for_a_seed`, `sim::env::tests::ground_installations_are_set_down_on_the_terrain`; MT-3-37 – MT-3-40, MT-3-45, MT-3-46, MT-4-24
- See: [Format: missions, environments, events](../design/format-missions.md), [Environments and terrain](../../modding/environments.md)

!!! question "Open"
    Ground installations and anti-air batteries were resolved in 1.11.0 (app 0.6.1). Still open: choosing the environment and backdrop from the strategy map's location (`location`, `backdrop: location`, `environment: body`), which needs the strategy layer (Stage 4); scattered objects and detail textures are under R-FLT-012.

### R-MSN-009 Hazards

A mission or scenario MUST be able to declare `hazards:`. Each hazard is optional, and all are deterministic for a given seed.

- `asteroids: {class, count, radius, center, spin}` (defaults 40, 5 km, origin, 4 °/s): `count` objects of the given class (an object such as `cosmium_asteroid` or `asteroid`), placed uniformly at random in the sphere `radius` around `center`. None is closer than 300 m to a craft already placed. Each turns at a random rate up to `spin` degrees per second. Being objects, they collide, block shots and can be destroyed. Asteroids are *scenery*: they MUST show on the radar only within radar range (other objects that never move, such as stations, still always show).
- `minefield: {class, count, radius, center, faction}` (defaults 20, 2 km): `count` mines of a mine class (`kind: mine`, such as `space_mine`), placed the same way, belonging to `faction` (default: the class's own).
- `black_hole: {position, pull, radius}` (defaults 15 m/s², 400 m): every craft and missile that can move is accelerated towards `position`.
  - The acceleration is `pull` m/s² at 1 km, growing with the inverse square of the distance and capped at 20 × `pull`.
  - A craft that comes within `radius` metres is destroyed.
  - It MUST be drawn as a black core (a sphere of `radius`) with a slowly turning accretion ring of glows.
  - AI pilots (other than idle objects) MUST turn away and burn out when inside 4 × `radius`, or within 10 × `radius` while heading in.
- `ion_storm: {sensors}` (default 40 %): radar range is multiplied by `sensors` (a fraction, for example `40%`), missile locks build at half speed, and the pilot is told `ION STORM: SENSORS AT n%` when the battle starts.

An unknown class or faction in a hazard MUST be an error: `--check` reports it for missions, and a scenario fails to build with it.

*Rationale:* Variety, and the original's battles in asteroid belts and next to a black hole.

- Implements: [F-MSN-004](../../definition/missions.md#f-msn-004-environments-and-hazards)
- Status: implemented (app 0.5.1, main `70d68e2`)
- Verified by: `content::hazards::tests::hazards_parse_with_defaults_and_report_mistakes`, `sim::hazards::tests::asteroid_fields_and_minefields_are_scattered_clear_of_craft`, `a_black_hole_pulls_harder_when_near_and_swallows_what_crosses_its_radius`, `ai_pilots_turn_away_from_a_black_hole`, `an_ion_storm_shortens_radar_range`; MT-3-33 – MT-3-36
- See: [Format: missions, environments, events](../design/format-missions.md), [Missions › Hazards](../../modding/missions.md#hazards), [Deviations (DEV-045)](../../implementation/deviations/index.md)

### R-MSN-010 Simulator missions

The player MUST be able to set up a simulator mission on a setup screen and fly it. As built, the setup screen (the launcher's *Simulator* entry) MUST offer:

- the **mission** (any mission of the loaded campaign) and the **ship** (any playable ship, unless the mission fixes the player's class);
- the **environment**: the mission's own, or any environment of the campaign;
- **asteroids**, a **minefield**, a **black hole** and an **ion storm**, each on or off. Asteroids and mines MUST use the campaign's own classes (the first class whose id contains `asteroid`; the first `kind: mine` class); when the campaign has none, the row MUST say so and add nothing;
- the **enemy force**, ×0.5, ×1, ×1.5 or ×2, scaling every enemy role's count range and every enemy wave, rounded and never below one. Friendly and objective roles MUST be unchanged.

The choices MUST produce a modified copy of the mission that is flown by the ordinary mission code; with the defaults (own environment, no hazards, ×1) the copy MUST equal the mission. A ship with `atmospheric: false` MUST be refused in an environment with `physics: atmosphere`, with the reason shown. Flying again from the debrief MUST fly the same setup. The difficulty level is the launcher's.

*Rationale:* The original's simulator; practice, and trying a mission in other conditions without editing it.

- Implements: [F-MSN-005](../../definition/missions.md#f-msn-005-simulator-missions), [F-TST-004](../../definition/testbed-tools.md#f-tst-004-debugging-tools-for-missions-and-campaigns)
- Status: implemented (app 0.6.3, main `1d13b59`, `c3fb959`)
- Verified by: `app::simulator::tests::a_setup_changes_environment_hazards_and_enemy_force_only`, `ships_that_cannot_enter_an_atmosphere_are_refused_there`; `app::launcher::tests::menu_lists_viewer_and_scenarios_and_wraps`; MT-3-51 – MT-3-54
- See: [User guide › Simulator](../../user-guide/simulator.md), [Testbed › The simulator](../../implementation/architecture/testbed.md#the-simulator-app-063)

!!! question "Open"
    The `interceptor` campaign's own simulator mission types (R-CPN-005) are separate mission definitions still to be written (plan item 3.11); the simulator screen flies whatever missions a campaign defines.

### R-MSN-011 Base defence balance

In every base-defence mission, at every difficulty including the lowest, the base MUST survive long enough for the player to engage more than one attacker. Measured with `--soak <mission> --soak-pilot idle` (the player's craft sits still and does nothing): at the lowest level an undefended base or station MUST hold for at least 150 s.

*Rationale:* User request (F-MSN-006).

- Implements: [F-MSN-006](../../definition/missions.md#f-msn-006-base-defence-the-player-can-win)
- Status: implemented
- Verified by: `soak::tests::at_cadet_an_undefended_station_holds_for_150_seconds`; MT-3-01, MT-3-05, MT-3-12, MT-3-17
- See: [Missions (modding)](../../modding/missions.md), [Manual tests (MT-3-01, MT-3-05)](../../implementation/testing/manual-tests.md)

!!! note "Finding (app 0.3.1)"
    Measured with the idle pilot, `xcom_base_defense` at beginner still had 2 390 of 6 000 hull after 400 s, and the AI pilot won in 75 s with the base untouched. On veteran the mission ended at 15–18 s because the *player's* craft was destroyed by elerium torpedoes, while the debrief showed the base-lost text. The base was not too weak; the result was mislabelled. The fix is the `killed` result (R-MSN-003, R-MSN-005), not a change of pack values. Default campaign, `defend_station`, idle pilot: cadet still running at 400 s; ranger lose at 140 s; space ace killed at 8 s.

### R-MSN-012 Logistics missions

The `convoy_ambush` and `construction_defence` missions MUST be playable: protect the convoy against attackers; defend a facility shown in its construction-stage state. In `convoy_ambush` the enemy MUST be after the **cargo**: it wins by capturing a supply vessel or every cargo container, the convoy wins when its vessels have left with cargo aboard, and a battle in which a vessel never reaches its jump point MUST end in a draw after a time limit, delivering what is still aboard.

*Rationale:* Logistics in battle. The convoy battle was redesigned by the user after playtesting (app 0.9.2): a battle about cargo, not about killing raiders.

- Implements: [F-MSN-007](../../definition/missions.md#f-msn-007-logistics-missions)
- Status: implemented (the convoy design: app 0.9.2, main `0d1c522`, `146195b`)
- Verified by: MT-3-02, MT-3-03, MT-4-26, MT-4-27
- Reverse-engineered: yes
- See: [Logistics](../design/logistics.md), [Missions › The convoy ambush](../../modding/missions.md#the-convoy-ambush-app-092)

!!! note "As built (app 0.9.2)"
    Two clippers with twelve cargo pods and a point-defence laser each (R-STR-017) run about ten minutes to their jump point; grappler tugs come every two minutes from 140 s, each with a saucer squadron in formation on it (breaking off within 1.5 km), tear pods off and haul them 3 km before jumping out; raiders (`freighter_raider`) go for the clippers. Lose: `captured_any(clippers) or destroyed_all(clippers) or cargo(clippers) == 0`; win: `count(clippers) == 0 and cargo(clippers) > 0`; a 15-minute timer ends it in a draw.

!!! question "Open"
    Generating `construction_defence` from the strategy layer is R-STR-019 / R-STR-023 (the convoy ambush is generated since app 0.9.2).

## Revision history

| Version | Date | Change |
|---|---|---|
| 1.0.0 | 2026-11-11 | Bootstrap: R-MSN-001 – R-MSN-012 (12 requirements; 7 reverse-engineered from the code) |
| 1.1.0 | 2026-11-11 | R-MSN-011 implemented (app 0.3.1) with a measurable threshold (an undefended station holds 150 s at the lowest level, `--soak-pilot idle`); open note resolved. R-MSN-003: the `killed` result, set by the engine, with the `lose` outcomes by default. R-MSN-005: the `killed` headline and debrief text |
| 1.11.0 | 2026-11-11 | R-MSN-009 hazards rewritten as built and implemented (app 0.5.1): asteroid fields and minefields as scattered objects (asteroids are radar scenery), the black hole's 1/d² pull capped at 20 × `pull`, its look and the AI's escape, the ion storm's radar factor, half-speed locks and start message; defaults stated. R-MSN-008 battle environments rewritten as built and implemented (app 0.6.0): `environments:` data, `environment: <id>`, `physics`, `arena`, `backdrop.planet`, `sky`; ground installations (`kind: ground`, set down on the terrain) and anti-air batteries as ground turrets (app 0.6.1); Open note keeps the map location |
| 1.12.0 | 2026-11-11 | R-MSN-007 voices rewritten as built and implemented (app 0.6.2): `voices:` sets, ship `voice:`, the `acknowledge`, `status` and `down` lines, chatter `voice:`, `mission_accomplished` / `mission_failed`; Open note: the `interceptor` pilot clips are not mapped until their lines are identified. R-MSN-010 simulator rewritten as built and implemented (app 0.6.3): the setup rows, the modified copy, the atmospheric refusal, flying again |
| 1.13.0 | 2026-11-11 | R-MSN-005: the verdict at once, then the battle runs on for `flight.debrief_delay` (default 30 s) before the debrief; Enter skips the wait; a retreat shows it at once (app 0.7.4, from user feedback). R-MSN-007: the computer voice plays when the result is decided; `cue:` names resolve in voices and chatter (app 0.6.7) |
| 1.15.0 | 2026-11-11 | From the user's playtest and convoy design (app 0.9.1 – 0.9.2): R-MSN-002 revised — repeating waves (`every:`) and escorts (`escort:`); R-MSN-003 note — `cargo(role)` and the mission key `cargo:`; R-MSN-008 revised — the nebula backdrop, the `pocket` uses it; R-MSN-012 revised — the convoy ambush is about the cargo (capture by tugs, the draw after a time limit), with an As-built note |
