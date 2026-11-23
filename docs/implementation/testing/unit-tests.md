# Unit tests

Every module's tests, as of main `bf7b7ce` (build 0.4.2): **292 tests** on macOS (293 on
Linux), run with `cargo test` by the gate. Tests
live next to the code in `#[cfg(test)] mod tests` blocks and use **synthetic data built in the
test** (fake DLLs, meshes, WAVs, temp pack folders) — never original game files.

`platform::linux` has a test too (joystick event mapping); it is compiled and run only on Linux,
so it is not in the count from macOS.

**Difficulty fix (app 0.22.36)** two more: `campaign::tests::a_save_names_its_difficulty_before_it_is_read`
and `campaign::tests::content_resolves_per_level_values_at_the_chosen_level`.

**#139 (app 0.22.34)** five more in `campaign::aliens::tests`:
`the_rival_starts_with_a_bases_that_queue_ships_and_outposts_that_pay` (and its save section),
`the_raid_planner_sends_raids_from_an_a_base_with_ships_queued`,
`new_bases_come_on_the_originals_triggers_and_are_b`, `the_rivals_outposts_build_its_plans_pieces` (since the follow-up: 60 technicians build a 1,200 technician-hour piece at the 20-hour update)
and `freighters_near_the_rivals_places_are_attacked`.

**#134 (app 0.22.32)** six more: `campaign::autoresolve::tests::ratings_follow_the_original_formulas`,
`rounds_come_at_five_hours_then_hourly_and_hits_go_shield_armour_hull`,
`a_won_battle_reports_its_kills_and_outcome_actions`,
`station_hits_take_modules_first_and_the_hull_by_quarters`,
`campaign::tests::battle_damage_stays_on_craft_mends_by_the_day_and_saves` (a flown battle's
damage, the save of a running battle, a day's repair, joining in real time, a craft lost with its
pilot) and `campaign::tests::interceptions_are_fought_off_screen_by_the_hour` (the default
campaign with the hourly model, end to end).

**#133 (app 0.22.28)** four more: `sim::combat::tests::hull_hits_and_collisions_hurt_the_pilot_until_out_cold_or_dead`,
`content::behaviour::tests::health_rolls_follow_the_original_generator`,
`campaign::tests::pilots_heal_train_and_are_paid_as_the_original` (health 4 → 8 in two days in the
bay; a 75-Int pilot's Accuracy 60 → 61 in 5 days; a commander with 96 Accuracy and a kill earns
4,000 + 2,350 / 12; below 0 dead) and `sim::ai::tests::alien_grades_rise_with_the_campaign_month`.

**#130 (app 0.22.24)** two more: `campaign::tests::a_seen_contact_may_turn_on_the_base_that_saw_it`
(unseen beyond 3,000 map units, seen inside, then heading for the base) and
`campaign::tests::generated_systems_draw_planets_from_their_class`.

**#121 (app 0.22.20)** two more, so **566**: `balance::tests::protection_follows_the_difficulty_and_station_pools`
and `soak::tests::default_scenarios_engage_the_players_friends`. **Changed:**
`balance::tests::energy_limits_sustained_fire_and_kill_times_follow` checks the start beam setting
and the pool (10 s of full fire, then the sustained rate).

**#120 (app 0.22.19)** seven more, so **563**: `sim::combat::tests::bolts_shoot_down_missiles_and_cloaks_cost_energy_shields_and_missiles`
(a 30-damage bolt destroys a 20-point missile 80 – 97 times in 100; a cloaked hit skips the shields;
no missile from cloak; −50 energy a second, no shield recharge, the cloak drops at 0),
`launcher_rate_and_start_throttle_come_from_the_rules`, `sim::guidance::tests::original_missiles_fly_straight_first_and_slow_in_hard_turns`,
`original_blasts_hit_everything_round_them_without_falloff`, `original_fuses_duds_and_lost_locks`,
`decoy_images_life_caps_and_doppelganger_pulls` and
`content::rules::tests::original_missile_flight_cloak_and_flight_keys_parse`.

**#119 follow-up (app 0.22.19)** two more, so **551**: `sim::ai::tests::the_original_break_off_ends_with_its_manoeuvre_by_range_or_chance`
(a break-off at 1,500 m ends with its first manoeuvre; at 800 m not before 5 s, then on a 25 %
roll; breaking off at another ship sends the call) and
`mission::tests::a_nova_battle_end_is_a_draw_unless_an_objective_decides` (a draw with no
objective; `win: battle_over and alive(patton)` wins). **Changed:** the Nova test also checks the
wingman voice line `wave33`.

**Parity M1, #119 (app 0.22.16)** three more, so **549**:
`content::behaviour::tests::retreat_chances_follow_rank_stats_and_clamps` (disarmed: rank 0 and
Bravery 100 → 40 %, rank 3 → held to 5 %, Bravery 0 → held to 85 %, no pilot → the band; Nova:
rank 0 with all stats 100 → 50 %, with all 0 → 90 %, rank 4 → held to 5 %; an unknown state is an
error), `sim::ai::tests::disarmed_pilots_and_a_nova_send_ships_home` (a ship with one mount left
never rolls; with none it flees after its 5 s; a Nova sends an armed ship home once) and
`sim::guidance::tests::a_nova_bomb_goes_off_on_its_fuse_and_ends_the_battle_a_minute_later` (no
detonation at 59 s and none on contact, one at 60 s, the battle over 60 s later, the warnings at
30 s and 88 s).

**Parity M1, #117 (app 0.22.14)** six more, so **537**: `content::equipment::tests::tachyon_table_and_recharge_while_burning_parse`
and in `sim::combat` `accumulator_shields_turn_ordinary_hits_into_shield` (40 on a raised 20-point bank:
no damage, the bank 30; held to 40; a bank at 0 takes it; a Permeator's halves do not accumulate),
`permeator_and_enveloping_split_their_damage_as_the_original` (per-side armour: a Permeator's 60 from
ahead leaves the fore shield 10 and the fore armour 20; an Enveloping 120 from behind runs 60 through
each chain), `tachyon_hits_skip_armour_and_hull_and_slow_the_repair` (50 at a 40-point bank: the bank 0,
the engine −10, armour and hull untouched; repair a tenth as fast until whole; a station is
untouched), `original_link_modes_fire_by_type_and_each_mount_on_its_own` (linked fires both lasers of a
laser–heavy–laser fit; full with 60 energy fires the laser and the heavy; the engine's rule fires
nothing with 55) and `energy_recharges_through_the_afterburner_when_the_rules_say_so`.

**#115 follow-up (app 0.22.14)** one more, so **531**: `sim::ai::tests::turret_defaults_and_pilot_accuracy_bands`
(an `aim: fixed` state with no interval or range gets 0.5 s and 2.5 km; with no record a turret's
accuracy lies in rand[50, 75) − rand[5, 15); with Accuracy 80, in 65 … 75 %).

**Parity M1, #115 (app 0.22.13)** four more, so **530**: in `sim::ai`
`fixed_turrets_lead_fully_and_miss_sideways_on_a_failed_roll` (the aim is the target plus 1 s of its
velocity at 1 km and a 1,000 m/s beam; at 1 % accuracy and 300 m every miss is within 20 m along one
target axis, none at 150 m; no shot outside the 85° arc),
`turrets_fire_on_their_interval_for_free_and_need_a_clear_line` (six shots in 3 s at a 0.5 s
interval with an empty energy pool; none with a ship of its own side in the line),
`gimballed_turrets_lead_at_two_thousand_with_no_error_in_front_only` (one second of lead at 2 km
whatever the beam's speed; no shot behind the mount) and
`missile_turrets_launch_after_the_lock_time_then_every_interval` (five launches, the first after 2 s,
at least 2 s apart; none outside the arc).

**#114 follow-up (app 0.22.13)** one more, so **526**: `sim::stations::tests::mines_with_no_range_go_off_on_contact` (a
`range: 0` mine stays put with a hostile 150 m off and goes off when the two 50 m hulls touch).
**Changed:** `hard_points_take_hits_whole_behind_the_shields` (was `…_only_with_the_shields_down`: a
1,500 hit on the 1,400-point connection point leaves it at −400 and the hull untouched; a tendril with
`hits: any` takes an enveloping hit and shows its 100 m explosion; hits on a fallen tendril reach the
hull) and `turrets_by_position_and_chance_and_minefields_by_level` (mine 0 at the unit X vector turned
by the ring, times the radius).

**Parity M1, #114 (app 0.22.11)** three more, so **519** `#[test]` functions:
`content::rules::tests::hard_points_placed_modules_and_minefields_parse` (`destroyed_when`, a hard
point, modules by `at`/`facing`/`chance` beside a slot, a minefield; a module with neither slot
nor at, an unknown mine and a bad `destroyed_when` are errors; absent keys change nothing) and in
`sim::stations` `hard_points_take_direct_hits_only_with_the_shields_down` (the pool takes a hit
while up; with it down a direct hit inside the 124 m sphere lowers the point, not the hull; a hit
outside it or an enveloping one strikes the hull; the fall is announced; all four down destroys
the station with hull to spare) and `turrets_by_position_and_chance_and_minefields_by_level` (five
turrets at 100 %, facing out; two at 0 %; a seeded 50 % varies; no mines on level 0; on level 4
8 mines 4 km out and 18 at 2 km round (0, 500, 0)).

**Parity M1, #118 (app 0.22.8)** four more, so **505** `#[test]` functions by a count of the
sources (Track B's merges included): in `sim::ai`
`the_original_steering_closes_a_small_error_within_two_ticks` (1° to the right: under 0.01° left
after two ticks with `steer_original`, more than 0.5° with the smooth law),
`pilot_records_set_accuracy_within_the_easy_level_bands` (no record 50 %; Accuracy 90 on
Beginner 90 % for the player's side and 50 % for others, 60 % on level 1, 90 % from level 2),
`grades_set_the_look_ahead_and_the_sprint` (the grade table; an object 180 m ahead seen by grade
0 — ¾ of top speed — and not by grade 2; a grade-1 pilot 5 km off asks for 106.25 % of top
speed, a grade-0 one on Beginner does not) and
`the_pilots_own_lock_fires_guided_missiles_in_its_window` (on the nose at 1 km: a homing launch
with no sensor lock and every mount firing; none within the 5 s interval; none once the lock
time has passed; the timer dropped off the cone and a new window back on it; break-off inside
250 m). **Changed:** `sim::flight::tests::ai_ships_drive_by_their_factor_with_no_cap` — a 200 m/s
request settles at 200 m/s, 250 is full throttle, 600 is held to 312.5 m/s, and the afterburner
neither burns nor drains energy for an AI ship under the drive.

**Parity M1, #113 follow-up (app 0.22.6)** one more, so **482**:
`sim::stations::tests::a_pool_without_generators_recharges_to_its_own_shields` (an ore plant's
1,000 pool from its `shields`, emptied by a 1,200 hit with 200 to the hull and never below 0,
back to 10 after 5 s); `content::rules::tests::fitted_equipment_and_hit_point_charts_set_protection`
now also checks that a pooled class with no modules takes its chart's `SD` as `shields`.

**Parity M1, #116 (app 0.22.6)** seven more, so **481** `#[test]` functions by a count of the
sources: `content::equipment::tests::efficiency_tables_follow_the_share_of_hit_points` (the
engine's steps at 60 / 50 / 49 / 24 / 16 / 14 % give 1 / 1 / 0.5 / 0.25 / 0.25 / 0 whatever
order they are written in; `linear`, `ignore`, `min_hit_points`, `disables_below`; an unknown word
is an error) and `repair_order_system_table_and_mounts_parse` (unknown items reported and dropped,
chances over 100 % an error, absent keys give today's rules); in `sim::systems`
`repair_follows_the_order_hull_included_at_two_a_second` (hull 50 → 52 in 1 s while the shield
generator waits; 4 a second with a damage control computer; the engine before the hull; a sensor
at 0 never repaired), `efficiency_steps_and_the_engine_failure_below_fifteen_percent` (engine
efficiency by hit points; flight control at 10 / 40 works at 0.25; at 16 % the ship flies, at 14 %
it is disabled, the cloak and afterburner drop, no thrust, and the hull is repaired before the
engine), `damaged_sensors_and_hyperdrives_change_nothing_when_set_to_ignore` (10 %: full
efficiency and the normal jump time), `the_system_table_strikes_mounts_and_carries_over` (30 on a
20-point mount wrecks one, the rest is lost; a wrecked mount's rate 0, at 40 % ½; a destroyed
launcher loses its rounds; a carrying shield hit leaves the generator destroyed at 0; flight
control floored at 1) and `the_system_table_rolls_follow_their_weights` (10,000 rolls land
50 % / 30 % within 3 points).

**Parity M1, #113 (app 0.22.4)** seven more, so **474** `#[test]` functions by a count of the
sources: `content::rules::tests::station_shield_pool_and_damage_model_keys` (`shield_pool`,
`shield_capacity`, `shielded_by_hub`, `damage_model` parse; a capacity implies a generator, none
gives the hub's `shields`; the pool's defaults are 2 per second and `drop`; absent keys keep two
banks and the ship model; a bad `on_generator_lost` is an error) and six in the new
`sim::stations`: `the_pool_starts_full_at_the_first_generators_capacity` (3,000, not 5,000),
`the_pool_recharges_two_points_a_second_up_to_its_maximum` (2,990 → 2,992 after 1 s, 3,000 and
no more after 10 s), `losing_a_generator_follows_the_next_and_losing_all_keeps_or_drops` (the
2,000 generator takes over and the pool comes down to it; `keep` holds 1,500 without recharge;
`drop` empties with "STATION SHIELDS DOWN"), `hits_from_fore_and_aft_drain_the_same_pool`,
`module_hits_are_taken_by_the_hubs_pool_while_it_is_up` (100 on a covered pod at pool 500 leaves
the pod whole and the pool 400; 600 at 400 puts 200 on the **hub's** hull; with the pool down the
pod takes its own armour and hull) and `station_hull_hits_skip_armour_systems_and_difficulty`
(a player's beam on Beginner: factor 1, exactly the damage on the hull, armour and systems
untouched, no `SystemChanged`; a ship still takes 1.15). **Changed:**
`content::rules::tests::fitted_equipment_and_hit_point_charts_set_protection` expects `HP1` (10),
not the sum of `HP1` – `HP4` (60).

**At main `9064f59` (app 0.20.0, Batch 4.2)** six more, so **449** run on macOS by the gate's
run: `campaign::logistics::tests::an_order_larger_than_a_convoy_leaves_as_several_convoys`
(`cargo_capacity: 1` with four vessels: an order of 10 leaves as convoys of 4, 4 and 2, all
arrive, and the totals count 3 convoys, 10 shipped and 10 delivered; without the key one convoy
carries any order), `construction_stages_follow_progress_and_stand_in_for_the_struck_site` (the
default stages and `stage_at` at 0, 59, 60 and 95 % — 0 / 60 / 90 % since app 0.20.1, `7345fc3`;
a ten-hour build is `build_25`, then `build_60` at 6.5 h, `build_90` at 9.5 h, with its type's `hub` as the stand-in class, and no stage
once built; a pack's own stages listed out of order are sorted),
`an_attacked_survey_ship_waits_at_the_site_then_reports_or_is_lost` (an attacked ship raises
`Event::Survey` and waits unsurveyed; one `survey_class` stands in; the held ship and the totals
round-trip through a save; flown and won it reports *after fighting off an attack* with
*suits: mine*; not flown with `survey_loss: 100%` it is lost and the system stays unknown),
`without_a_survey_mission_an_attack_is_settled_at_once` (no event; a 0 % loss reports at once,
*suits: plant, station*), `content::mission::tests::a_stand_in_replaces_the_first_protected_role_in_a_copy`
(the first protected role, not the first role; class, state and count replaced on a copy, later
protected roles and the original untouched; fields left out kept; no protected role is an
error), and `content::rules::tests::logistics_stand_ins_and_classes_are_checked` (a survey
mission without a protected role, an unknown defence mission, an unknown `hub` and an unknown
`survey_class` are each reported). Not unit-tested (manual MT-4-55, MT-4-56): the stage pips,
the stand-in seen in battle, the survey prompt and battle.

**At main `c28fc2c` (app 0.19.0, Batch 4.1)** five more, so **443** run on macOS by the gate's
run: `campaign::tests::stores_hold_deliveries_and_output_to_their_capacity` (a purchase counts
what is on its way; manufacture waits for room, unpaid; a delivery that finds no room is held,
announced once, and stored after a repair), `wrecked_modules_lose_what_they_held_until_repaired`
(a wrecked bay loses both craft, their pilots injured; a wrecked bunk kills one pilot; repair at
half cost and half build time; a wreck dismantled for nothing),
`templates_take_the_best_module_and_placement_decides_where` (a template's `sensor` group gives
the radar, then the better radar once researched; `base_distance(system) <= 3500` refuses the far
system), `a_whole_base_is_dismantled_with_its_craft_and_stock_moved` (the last base refused; craft
rehoused, stock moved, a delivery redirected, a raid's target cleared, the refund paid), and
`campaign::logistics::tests::placement_conditions_and_storage_limit_facilities` (a plant allowed
only in a nebula; a role's default rule parsed as a condition; a plant with 12 units of room never
holds more and orders nothing beyond it, yet builds three rockets in small shipments).
**Extended:** `campaign::save::tests::a_save_round_trips_and_refuses_missing_packs` (a wrecked
module and a held delivery round-trip), `campaign::tests::the_default_campaign_runs_a_year_with_its_story_and_stays_consistent`
(no store above its capacity), `script::campaign::tests::ids_match_and_names_are_checked`
(`base_distance`), and `campaign::logistics::tests::survey_build_mine_make_and_deliver_on_demand`
(the refusal now reads *does not suit*).

**At main `3ea5177` (app 0.18.1)** one more: `content::tests::evaluation_texts_are_resolved` (the 0.18.1 regression: `@keys` in evaluations' `do` lists resolved), so **438** run on macOS (441 `#[test]` functions less the Linux-only test and two ignored helpers). App 0.16.15 (`00c7da5`) added no test function; the default campaign's six-month test now also asserts that its evaluations reach the strategy layer.

**At main `5d05ba2` (app 0.18.0)** a count of the `#[test]` functions in `src/` (grep, not a
build) gives **437** run on macOS (440 less the Linux-only test and two ignored helpers). The 29
added since `de30c2a`: app **0.17.0** (`4c29f89`)
`content::screens::tests::widgets_and_actions_parse_for_the_screen_sets`; app **0.17.1**
(`deba9f5`) `content::strategy::tests::weekdays_follow_the_calendar`; **3.5** (`4a0bd03`) —
`content::environment::tests::detail_textures_and_scatter_parse_with_defaults`,
`bad_detail_and_scatter_values_say_what_is_wrong`, `scatter_is_deterministic_and_stays_inside_the_field`,
`scatter_respects_slope_height_and_alignment`, `render::gpu::tests::shared_vertex_grids_are_split_by_index_count`,
`render::terrain::tests::detail_coordinates_repeat_per_scale_and_meet_at_tile_edges`,
`steep_triangles_take_the_slope_texture`, `scatter_is_baked_into_cells_in_place`; **3.10**
(`040ca26`) — eleven in `script::stmt::tests` (`let_assignment_and_if_else_chains`,
`repeat_and_for_each_are_counted_loops`, `functions_take_typed_parameters_and_return_values`,
`recursion_is_refused_when_loading`, `the_budget_stops_a_runaway_hook`, `same_seed_same_result`,
`mistakes_are_load_errors_with_file_and_line`, `hook_references_are_checked`,
`file_structure_is_checked_when_loading`, `programs_list_and_rewrite_their_action_arguments`,
`every_context_query_fits_the_argument_buffer`),
`script::tests::script_lexing_keeps_lines_text_and_statement_ends`,
`content::behaviour::tests::scripted_states_compile_and_check_what_they_name`,
`content::tests::scripts_load_from_packs_mods_override_them_and_hooks_are_checked`,
`campaign::story::tests::an_event_script_counts_through_flags_and_sees_its_own_actions`,
`mission::tests::a_script_outcome_runs_in_the_battle_context`,
`sim::ai::tests::a_scripted_state_runs_its_script_every_interval`,
`a_runaway_script_is_stopped_and_reported_once` and
`soak::tests::the_default_campaigns_scripted_ace_taunts_and_turns_furious`. **Extended:**
`check::tests` (a missing detail texture or scatter model, `max_slope` out of range). **Not
unit-tested:** the look of detail textures (mipmapping through the GL context), drawing and
culling the scatter cells, and how the scripted ace plays — covered by manual tests
MT-3-67 – MT-3-70.

**At main `de30c2a` (app 0.16.14)** a count of the `#[test]` functions in `src/` (grep, not a
build) gives **408** run on macOS (411 less the Linux-only test and two ignored helpers). The 14
added since `ac7ae62`: app **0.16.1** (`f82a6ab`)
`content::rules::tests::fitted_equipment_and_hit_point_charts_set_protection`,
`content::strategy::tests::protection_comes_from_the_item_or_the_originals_keys`; app **0.16.2**
(`6b45fcb`) `campaign::tests::generated_maps_are_spaced_named_seeded_and_hide_black_holes`; app
**0.16.3** (`a45331a`) `campaign::logistics::tests::logistics_queries_answer_about_systems`,
`outcomes_capture_prizes_and_lose_facilities_safely`; app **0.16.4** (`d300642`)
`campaign::logistics::tests::construction_sites_are_struck_from_halfway_at_most_max_times`; app
**0.16.5** (`f227494`)
`mission::tests::the_condition_console_evaluates_against_the_battle_without_changing_it`; app
**0.16.6** (`1093f0a`)
`campaign::tests::requires_any_and_hidden_topics_wait_for_one_prerequisite_and_their_discovery`;
app **0.16.7** (`2c152a8`) `campaign::tests::pilots_train_up_to_their_maximum_and_can_be_dismissed`;
app **0.16.8** (`26f6c2a`) `campaign::tests::the_monthly_evaluation_rates_the_month_and_runs_its_actions`;
app **0.16.10** (`1bc200e`) `campaign::tests::quarters_limit_the_roster_and_modules_can_be_dismantled`;
app **0.16.11** (`6821147`) `campaign::tests::queues_reorder_and_cancel_and_deliveries_can_be_lost`;
app **0.16.12** (`01e2f2b`) `balance::tests::auto_resolve_odds_favour_the_stronger_side`; app
**0.16.13** (`52e9e7f`)
`campaign::story::tests::the_debugger_fires_events_and_the_testbed_plays_to_a_chapter`. App
0.16.9 and 0.16.14 added none. **Extended:** `campaign::save::tests::a_save_round_trips_and_refuses_missing_packs`
(a newer pack version loads with a notice, 0.16.2), `campaign::story::tests::mission_outcomes_move_standings`
(a change of stance becomes a notice, 0.16.2), `script::campaign::tests::ids_match_and_names_are_checked`
(the eight logistics queries, 0.16.3), `app::menu::tests::the_menu_offers_continue_only_with_a_save`
(Load game, `age_text`) and `options_show_only_what_can_change` (Full screen, Controls; 0.16.14).
**Not unit-tested:** the 0.16.9 fix (the rules' list of strategy globals lacked `evaluations`, so a
pack's ratings never reached the campaign — the unit test built the strategy layer directly);
`app::skip_pressed`, the stingers, `screens.<id>.background`, the `--check` voice, music and face
checks, the survey report's text, the cutscene text keys; the strike, auto-resolve, save-slot,
Debug and ending prompts and screens; `strategy::slot_path`; `graphics.fullscreen` in the settings
round trip; the storage limit on purchases and the `defence` effect — covered by manual tests
MT-3-66, MT-4-38 – MT-4-49, MT-5-11 – MT-5-13 and MT-6-21 – MT-6-27.

**At main `ac7ae62` (app 0.16.0)** a count of the `#[test]` functions in `src/` (grep, not a
build) gives **394** run on macOS (397 less the Linux-only test and two ignored helpers). The 9
added since `16a9954`: app **0.15.0** (`1ff754b`)
`balance::tests::energy_limits_sustained_fire_and_kill_times_follow` (and
`app::flight::tests::a_campaign_loadout_refits_the_players_class_in_this_battle_only` now checks
fitted shields and armour); main `bb04952` (#6, app 0.15.1)
`input::tests::every_action_has_a_one_sentence_description`,
`remote::protocol::tests::the_idle_catalogue_lists_every_action_with_its_description_and_all_available`,
`app::flight::tests::the_keys_help_shows_descriptions_only_when_the_columns_are_wide_enough`;
app **0.15.1** (`bf963b4`) `hud::draw::tests::ship_status_bands_follow_thirds_of_the_shield`;
app **0.16.0** (`ac7ae62`) `app::widgets::tests::text_wraps_between_words_and_cuts_only_giant_words`,
`lists_keep_the_selection_in_view`, `grids_and_hits`,
`content::screens::tests::a_layout_moves_only_what_it_names`. App 0.14.2 (`dcf9563`) added none.
**Not unit-tested:** `menu::fit_size`, `fitted:` on ship classes, `strategy::protection_of`, the
`HP1` – `HP4` hull sum, and the strategy screen's own logic (picking, zoom about the cursor,
the info panel's actions) — covered by manual tests MT-4-29 – MT-4-37 and MT-6-19.

**At main `16a9954` (app 0.14.1)** the count is **385** run on macOS: added
`sim::combat::tests::a_harmless_ships_bolts_hit_but_do_no_damage` (a cutscene's `harmless` ship's
bolts land but change neither shields, armour nor hull).

**At main `a18a7b4` (app 0.14.0)** the count is **384** run on macOS (387 less the Linux-only
test and two ignored helpers): the 5 added are `app::menu::tests::the_menu_offers_continue_only_with_a_save`,
`options_show_only_what_can_change`, `volumes_step_in_quarters_and_stop_at_the_ends`,
`app::credits::tests::the_roll_rises_from_below_and_ends_off_the_top` and
`content::front::tests::menu_and_credits_parse_with_defaults`.

**At main `747d1e9` (app 0.13.1)** a count of the `#[test]` functions in `src/` (grep, not a
build) gives **379** run on macOS (382 less the Linux-only test and two ignored helpers). The 13
added since `76bda5c`: app **0.11.0** (`1f183d1`)
`content::music::tests::sets_parse_and_the_state_follows_the_nearest_enemy`,
`audio::music_tests::segments_cycle_within_the_wanted_state`; app **0.11.1** (`2dd85dd`)
`content::skin::tests::skins_parse_colours_with_fallbacks`,
`app::ui::tests::nine_slices_keep_corners_and_stretch_the_rest`; app **0.12.0** (`5163a57`)
`formats::smacker::tests::small_and_big_trees_decode_with_the_cache`,
`palettes_decode_explicit_kept_and_copied_entries`, `a_tiny_file_decodes_a_solid_frame`,
`audio_dpcm_8_bit_mono`, and the **ignored** helper `real_smacker_file_decodes` (not counted:
`OI_SMK=<file> OI_SMK_OUT=/tmp/f.png cargo test --lib real_smacker -- --ignored --nocapture`,
run by hand against the player's own videos); app **0.12.1** (`563c93f`)
`campaign::story::tests::mission_outcomes_move_standings`; app **0.13.0** (`a59ffce`)
`content::cutscene::tests::catmull_rom_passes_through_keys_without_overshoot`,
`cutscenes_parse_and_check_their_references`; main `319a31b`
`formats::mpsrc::tests::a_pack_lists_and_returns_its_members`; app **0.13.1** (`747d1e9`)
`app::comms::tests::the_panel_lasts_the_clip_and_the_signal_moves` (and the voice-set test in
`content::rules` now checks `name`, `face` and `voice_of_clip`). Not unit-tested: drawing, video
timing and cutscene playback (manual tests MT-6-01 – MT-6-13).

**At main `76bda5c` (app 0.10.3)** a count of the `#[test]` functions in `src/` (grep, not a
build) gives **366** run on macOS (368 less the Linux-only test and the ignored helper). The 11
added since `2ed1149`: app **0.9.1** (`3b19dea`)
`content::environment::nebula_tests::nebula_clouds_vary_stay_in_range_and_repeat_for_a_seed`;
app **0.9.2** (`0d1c522`) `sim::tests::cargo_modules_count_until_torn_off_destroyed_or_jumped_away_with_the_hub`,
`a_tug_tears_a_container_off_and_prefers_a_disabled_ship`,
`sim::ai::tests::escorts_hold_formation_until_a_hostile_comes_close`,
`campaign::logistics::tests::an_ambushed_convoy_waits_for_its_escort_and_delivers_the_share_that_got_through`;
app **0.10.0** (`485d01e`) `campaign::story::tests::arcs_fire_in_order_with_choices_missions_and_cooldowns`,
`plans_grow_from_shipments_that_get_through_and_can_end_the_campaign`,
`relations_drift_and_the_strategist_answers_the_player`; app **0.10.1** (`66d0b5a`)
`script::action::tests::a_text_argument_keeps_its_commas`; app **0.10.3** (`76bda5c`)
`app::flight::tests::campaign_stances_decide_who_fights_whom`,
`campaign::tests::the_default_campaign_runs_a_year_with_its_story_and_stays_consistent` (the
year-long soak, with `campaign::autoplay_from`). App 0.10.2 (`8926d11`) added no test: the
X-COM story needs the player's data.

**At main `2ed1149` (after app 0.9.0)** a count of the `#[test]` functions in `src/` (grep, not a
build) gives **355** run on macOS (357 less the Linux-only test and the ignored helper). Added
since `9295707`: app **0.8.0** (`c64d178`)
`content::strategy::tests::original_tech_ids_become_research_prerequisites`; app **0.8.1**
(`8e96338`) `campaign::tests::testbed_cheats_do_what_they_say`; app **0.9.0** (`ff9d8fc`)
`campaign::logistics::tests::survey_build_mine_make_and_deliver_on_demand`,
`station_research_waits_for_its_start_materials_and_convoys_lose_vessels`,
`campaign::tests::manufacture_is_paid_per_unit_and_salvage_names_the_raid_by_id`; main `2ed1149`
replaced `formats::png::tests::checksums_match_known_values` and
`a_png_has_the_signature_chunks_and_stored_pixels` with `a_png_has_the_signature_and_decodes_back`
(the in-crate checksums are gone). `campaign::autoplay`, the automatic player the six-month test
uses, moved out of the test module (app 0.8.0) to serve `--strategy-soak`.

**At main `9295707` (app 0.7.4)** a count of the `#[test]` functions in `src/` (grep, not a
build) gives **351** run on macOS (353 less the Linux-only test and the ignored helper). The 26
added since app 0.6.4 are: app **0.6.6** (`6d390d7`) `content::formation::tests::shapes_place_followers_behind_and_beside_the_leader`,
`custom_slots_then_a_vee_and_errors`, `mission::tests::wingmen_take_successive_slots_of_their_formation`,
`sim::ai::tests::the_formation_order_cycles_the_campaigns_formations`; app **0.6.7** (`65141dc`,
`c0e2516`) `script::campaign::tests::ids_match_and_names_are_checked`,
`formats::png::tests::checksums_match_known_values`, `a_png_has_the_signature_chunks_and_stored_pixels`,
`downscaling_averages_blocks`, `render::terrain::tests::tiles_match_the_folded_ground_heights`,
`sim::ai::tests::collision_avoidance_predicts_the_closest_approach` (and the static-object lift in
`sim::env::tests::craft_spawned_low_over_a_ground_are_lifted_clear_of_it`, mirrored heights in
`generated_heightfields_span_their_range_are_deterministic_and_interpolate`, textures in
`remote::protocol::tests::model_messages_are_normalised_and_coloured_per_group`); app **0.7.0**
(`77d6c9f`) `content::strategy::tests::dates_count_days_across_months_and_leap_years`,
`a_strategy_layer_builds_with_checked_names`, `campaign::tests::a_new_campaign_has_its_start`,
`research_follows_prerequisites_and_unlocks`, `modules_bases_purchases_and_manufacture`,
`money_flows_monthly_with_costs`, `raids_are_detected_intercepted_and_fought`,
`same_seed_same_campaign`, `campaign::save::tests::a_save_round_trips_and_refuses_missing_packs`;
app **0.7.1** (`10f7fce`) `campaign::tests::the_default_campaign_runs_six_months`,
`app::strategy::tests::the_map_fits_its_rectangle_centred`, `money_has_thousands_separators` (and the
campaign entries in `app::launcher::tests::menu_lists_viewer_and_scenarios_and_wraps`); app
**0.7.2** (`94d66c1`) `campaign::tests::craft_are_fitted_from_the_store_and_fly_their_loadout`,
`app::flight::tests::a_campaign_loadout_refits_the_players_class_in_this_battle_only`; app **0.7.3**
(`076203e`) `campaign::tests::lane_maps_route_along_lanes`; app **0.7.4** (`9295707`)
`app::flight::tests::the_debrief_waits_for_the_delay_after_the_result`.

Since that count, tests named in the requirements' *Verified by* lines were added with apps
0.4.3 – 0.6.1 without the full list being regenerated; those named below are listed (the
summary rows of `hud`, `remote::protocol` and `sim::systems` include the 0.4.11 – 0.5.0 ones, and
the rows of `content::hazards`, `content::environment`, `sim::hazards`, `sim::env`,
`render::terrain` and `hud::draw` the 0.5.1 – 0.6.1 ones, and the rows of `app::flight`,
`app::simulator` and `sim::env` the 0.6.2 – 0.6.4 ones). A count of the `#[test]` functions in
`src/` at main `90653b9` (app 0.6.4), read with grep and not a build, gives **325** run on macOS
(327 less the Linux-only test and the ignored helper below; 321 at main `de417df`, app 0.6.1).
App **0.6.4** (main `90653b9`) adds
`sim::env::tests::craft_spawned_low_over_a_ground_are_lifted_clear_of_it`; app **0.6.3** (main
`1d13b59`) adds `app::simulator::tests::a_setup_changes_environment_hazards_and_enemy_force_only`
and `ships_that_cannot_enter_an_atmosphere_are_refused_there` (and updates the launcher's menu
tests for the Simulator entry); app **0.6.2** (main `5bdb47f`) adds
`app::flight::tests::a_wingman_answers_in_its_own_voice` (and voice sets in
`content::rules::tests::builds_typed_definitions_with_inheritance`). App **0.6.1** (main `de417df`) adds
`sim::env::tests::ground_installations_are_set_down_on_the_terrain`; app **0.6.0** (main
`af003f6`) adds `content::environment::tests::environments_and_terrains_parse_and_report_mistakes`,
`generated_heightfields_span_their_range_are_deterministic_and_interpolate`,
`sim::env::tests::drag_gravity_and_lift_act_in_air_and_not_in_space`,
`ai_pilots_pull_up_near_the_ground`, `gentle_touchdowns_land_hard_ones_crash_and_the_ceiling_is_an_exit`,
`render::terrain::tests::terrain_meshes_face_up_and_colour_by_height` and
`hud::draw::tests::the_pitch_ladder_sits_ahead_at_its_angles`; app **0.5.1** (main `70d68e2`) adds
`content::hazards::tests::hazards_parse_with_defaults_and_report_mistakes` and
`sim::hazards::tests::asteroid_fields_and_minefields_are_scattered_clear_of_craft`,
`a_black_hole_pulls_harder_when_near_and_swallows_what_crosses_its_radius`,
`ai_pilots_turn_away_from_a_black_hole`, `an_ion_storm_shortens_radar_range`. App **0.5.0** (main `0a1c340`) adds
`remote::protocol::tests::themes_layer_per_cockpit_and_become_css_variables`; app **0.4.15**
(main `e2b5012`) adds `remote::protocol::tests::contacts_are_in_the_players_axes_and_targets_decode`
and `hud::tests::the_message_log_reports_what_is_new_since_a_count`; app **0.4.14** (main
`e51eca9`) adds `remote::protocol::tests::model_messages_are_normalised_and_coloured_per_group`;
app **0.4.11** (main `db2f264`) adds `sim::systems::tests::sliders_set_power_levels_and_balance_outright`;
app 0.4.10 (main `4edc9d7`) adds `input::keys_held_when_a_battle_starts_are_ignored_until_released`;
app 0.4.8 (main `f1c8be8`) adds `remote::http::ranges_are_served_as_partial_content`, and app 0.4.7
(main `c42d773`) adds the **ignored** helper `remote::server::tests::serve_for_a_browser`, which
is not a unit test and is not counted: `cargo test serve_for_a_browser -- --ignored --nocapture`
serves the console page on `127.0.0.1:8790` for 20 s, answering like the launcher, for trying
the real page in a browser ([Remote console](../architecture/remote.md#debugging)).

Not unit-tested, by design, and covered by [manual tests](manual-tests.md) instead: the
`hud::draw` painting, the `Renderer`'s draw calls and shaders, macroquad device reading, sound
playback, the sky dome, planet disc, fog and terrain drawing, the black hole's look, the artificial horizon's drawing, the 3D cockpit pass, anchored instruments drawn into textures, the asset browser's, controls menu's and layout editor's drawing and the macOS gamepad shim — they need a window, a GPU, speakers or a controller.

## Summary

| Module | Tests | Covers |
|---|---|---|
| `formats` | 4 | Little-endian reads, bounds errors instead of panics (incl. offset overflow), NUL-terminated names, error messages |
| `formats::pe` | 5 | Resource lookup by type and name, quoted/upper-cased names, non-PE and truncated files, missing file |
| `formats::xmsh` | 7 | Hand-built mesh: header, LOD, signed edge loops, untextured polygons, **polygon flags** (overlay bit and undecoded bits kept), bad signature/edge/vertex/degenerate polygon, no LOD, truncation |
| `formats::xtxr` | 3 | RGB555 expansion, palette-0 decoding, bad size/signature/truncation |
| `formats::tga` | 5 | 16-bit colour key → transparent black and row flip, 24/32-bit BGR, RLE, colour-mapped (palettised) plain and RLE, bad palette index, unsupported types, truncation |
| `formats::wav` | 6 | PCM16 round trip, PCM8 recentring, MS ADPCM block, IMA ADPCM block, step clamping, bad files |
| `formats::ini` | 5 | Sections/keys/comments/duplicates, warnings for ignored lines, bad numbers, merge layering, Latin-1 |
| `formats::obj` | 4 | Positions/uvs/normals/faces/materials/negative indices, bad indices and short faces, MTL parsing; **`slot_` groups become attachment points** |
| `formats::yaml` | 6 | Mappings, sequences and scalars; block scalars and flows wrapped over lines; **unsupported or broken input is an error with a line** (tabs, anchors, aliases, tags, …); keys with colons and URLs stay whole; writing round-trips; empty and scalar documents |
| `formats::langfile` | 1 | Keys, text and `~` line breaks of the original language files |
| `model` | 9 | XMSH → engine model (mirroring, triangulation, groups, colours, bounds), outward winding after mirroring, **face table** (corner ranges, groups surviving empty-group removal), **overlay layers** (later face on top, overlay flag beats file order, touching or back-to-back faces not layered, OBJ duplicate face layered), LOD by distance, OBJ conversion, empty OBJ |
| `content` | 7 | Loading packs/rules/scenarios/models from temp folders, skipping packs that need original data, scenario replacement, **per-asset overrides** of a mesh, image and sound and their removal when the pack is disabled, PNG/TGA decoding, model scaling, empty pack dirs; **scripts load from packs, mods override them, and hooks are checked** (text keys resolved, a broken file and a missing function reported — app 0.18.0); **`@keys` in evaluations resolved** (app 0.18.1) |
| `content::pack` | 7 | Manifest parsing and validation, **kind, parent, targets and format** (and their errors), **the active set for a chosen campaign** (parent chain, targeting addons, dependencies), dependency ordering, missing/duplicate/cyclic dependencies, discovery |
| `content::rules` | 8 | Typed definitions with original-section inheritance (YAML), variants, defaults, sounds, **voice sets and their clips**, hostility, flight model and arena defaults, **`bolt_frame`, `bolt_carry` and `lighting` (unset keys keep defaults)**, cockpits; **later packs merge, delete and edit lists**; **difficulty variants and per-level values**; every error reported; **spelling of original keys** (`HitPoints` → `hit_points`, `Side` → `faction`, beam `Range` in km); **sound cues resolve through the original table**; **`fitted:` equipment and hit-point charts set protection** (a shield's `Special`, armour's `HitPoints`, a class's own value winning, inherited; the hull is `HP1` alone (app 0.22.4; `HP1` – `HP4` summed before); an unknown `fitted` slot is an error — app 0.16.1); **station shield pools, covered modules and the station damage model parse** (app 0.22.4) |
| `content::equipment` | 6 | System kinds round trip and built-ins exist, `systems` definitions overriding built-ins, `power` parsing and validation (level counts, start levels); **system `efficiency` tables, `repair_order`, `system_table` and `mounts` parse** (app 0.22.6) |
| `content::hazards` | 1 | `hazards:` parses with defaults (count, radius, `sensors` as a fraction); a hazard without `class`, or a position without three numbers, is an error |
| `content::environment` | 7 | Environments and terrains parse with defaults; mistakes reported (unknown `physics`, an atmosphere without a terrain, both or neither of `generate` / `heightmap`, an unknown generator); generated heightfields span their `height` range, are the same for the same seed and interpolate between grid points; **a nebula's clouds vary, stay in range and repeat for a seed** (app 0.9.1); **detail textures and scatter parse with defaults**, and **bad values say what is wrong** (count, scale, slope, height, align, draw distance, `slope_detail` without `detail`); **scatter is deterministic and stays inside the field**; **scatter respects slope, height and alignment** (app 0.18.0) |
| `content::looks` | 3 | Debris sets and effects parse with defaults; texture swaps accumulate with damage levels and state; texture keys ignore folders, case and extensions |
| `content::formation` | 2 | Every shape places followers behind and beside the leader in the leader's frame, scaled by `spacing`; custom slots then a vee behind them; `shape` with `slots`, or an unknown shape, is an error |
| `content::strategy` | 5 | Dates count days across months and leap years; a strategy layer builds from YAML with every name checked (a misspelt expression name, an unknown base type and an unknown unlock are errors); **the original's numeric tech ids become research prerequisites** through `tech_ids` (an unlisted id is left out; `tech_required` gates an item); **protection comes from the item or the original's keys** (`protection` first, then `special` for shields and `hit_points` for armour; other kinds none — app 0.16.1); weekdays follow the calendar (app 0.17.1) |
| `script::campaign` | 1 | Query ids match the table (29 queries since app 0.16.3, 30 with `base_distance` since 0.19.0); campaign ids and level names are constants; a misspelt name is an error |
| `campaign` | 22 | A new campaign has its start; research follows prerequisites and unlocks and stops at completion; modules, limits, bases (one per system), purchases, deliveries, manufacture and selling; monthly money flows and costs; raids detected, intercepted and fought; same seed same campaign; **the default campaign runs six months** (solvent, research, battles, a save round trip); fitting from the store and the loadout; lane routes (Dijkstra); **the testbed cheats** (payday, quickbase, knowitall, jump to battle); **manufacture paid per unit as it starts, and salvage recorded by raid id**; **the default campaign runs a year with its story and stays consistent** (app 0.10.3); **generated maps** are spaced, named in order then numbered, seeded and end in a hidden black hole (0.16.2); **`requires_any` and hidden topics** wait for one prerequisite and for `discover` (0.16.6); **pilots train** up to their stats' maximum at a daily cost, **and can be dismissed** (0.16.7); **the monthly evaluation** rates the month with the first rating that holds and runs its actions (0.16.8); **quarters limit the roster** and modules can be **dismantled** for a refund (0.16.10); **queues reorder and cancel**, and **deliveries can be lost** (0.16.11); **stores hold deliveries and output to their capacity**, **wrecked modules lose what they held until repaired**, **templates take the best module and placement decides where**, and **a whole base is dismantled with its craft and stock moved** (0.19.0) |
| `campaign::logistics` | 7 | **Survey, build, mine, make and deliver on demand** (a mine refused before the survey, a station refused off a site; a hull built once its plating arrives; a reserve drawn down); **station research waits for its start materials, and convoys lose vessels** (every convoy ambushed at 50 % loss; orders follow until the start cost is met); **an ambushed convoy waits for its escort and delivers the share that got through** (app 0.9.2); **the logistics queries** (`deposit` known only once surveyed, `feature`, `research_site`, `survey_known`, `owned`, `lane_distance` along lanes, `facility_progress`, `construction_attacks` — 0.16.3); **outcomes capture prizes and lose facilities safely** (`captured(<class>)` kept only with `capture`; `lose(<facility type>)` renumbers the other facility's orders — 0.16.3); **construction sites are struck from halfway, at most `max` times**, a strike not flown sets the build back and a lost defence destroys the site (0.16.4); **placement conditions and storage limit facilities** (0.19.0) |
| `campaign::story` | 6 | **Arcs fire in order** with choices, offered missions and cooldowns; **plans grow from shipments that get through** (not from intercepted ones) and can end the campaign; **relations drift and the strategist answers the player** (app 0.10.0); **mission outcomes move standings** (app 0.12.1), and a change of stance becomes a notice (0.16.2); **the debugger fires events** (fired count, the choice offered) **and the testbed plays to a chapter** (0.16.13); **an event script counts through flags and sees its own actions** (app 0.18.0) |
| `campaign::save` | 1 | A save round-trips and the game continues identically; a save needing a missing pack, or with an index out of range, is refused; a save from another version of a loaded pack loads with a notice (app 0.16.2); wrecked modules and held deliveries round-trip (0.19.0) |
| `formats::smacker` | 4 | **Small and big Huffman trees with the escape cache**; palettes (kept, copied, explicit entries); a hand-built one-frame file with a solid block; 8-bit mono DPCM audio (app 0.12.0). Plus one ignored helper, `real_smacker_file_decodes` (not counted) |
| `formats::mpsrc` | 1 | An MPSRC pack's table and members (case-insensitive), a missing member, a foreign file (main `319a31b`) |
| `content::music` | 1 | Sets parse; a missing state falls back to cruise; the state follows the nearest enemy (cruise, threat, battle); a set without states is an error (app 0.11.0) |
| `content::skin` | 1 | Colours (RGB and RGBA) with fallbacks for unset roles; the panel; an unknown role is an error (app 0.11.1) |
| `content::cutscene` | 2 | **Centripetal Catmull-Rom passes through its keys without overshoot**, holds at the ends; facing along the path; linear scalar keys; a cutscene parses (actions sorted, `destroy_at`, cuts, captions) and **bad entity or camera references are errors** (app 0.13.0) |
| `app::ui` | 1 | Nine-slice pieces keep the corners and stretch the rest (app 0.11.1) |
| `app::menu` | 3 | Continue only with a save; options rows only for what can change (campaign with more than one, physics where Simple is allowed); volumes step in quarters and stop at the ends (app 0.14.0); **Load game only when a slot holds a save**, `age_text` (just now, minutes, hours, days), and the Full screen and Controls rows (app 0.16.14) |
| `app::credits` | 1 | The roll rises from below the screen and ends when its last line leaves the top (app 0.14.0) |
| `content::front` | 1 | `campaign.menu` and `campaign.credits` parse, with defaults; the roll ends with the engine's credits (app 0.14.0) |
| `app::comms` | 1 | The panel lasts the clip plus 0.6 s, at least 2.5 s; the signal bars move and stay in range (app 0.13.1) |
| `formats::png` | 2 | A PNG (encoded by the `image` crate since `2ed1149`) has the signature and chunks and **decodes back to the same pixels**; downscaling averages blocks |
| `app::strategy` | 2 | The map fits its rectangle, centred; money has thousands separators |
| `app::widgets` | 3 | Word wrapping between words (a giant word cut, newlines kept); **lists keep the selection in view** (step, scroll to the last screenful, a shrinking list pulls the selection in, an empty list does nothing); grid cells and hit tests (the far edge is outside) — app 0.16.0 |
| `content::screens` | 2 | **A layout moves only the widgets it names**; an unknown widget is an error — app 0.16.0; widgets and actions parse for the screen sets (app 0.17.0) |
| `content::mission` | 2 | A full mission parses (roles, class lists, counts, arrivals, waves, objectives, optional, timer, outcomes, chatter, debrief); mistakes are reported with where (unknown role, bad timer result, unknown action) |
| `content::text` | 1 | Keys resolve, plain values pass through, imports are listed |
| `content::schema` | 4 | Good files pass; **mistakes reported with line and key path** (unknown key with suggestion, wrong type, bad unit); quantities with units and did-you-mean suggestions; JSON Schema export well formed |
| `content::files` | 1 | Every file schema builds and exports |
| `content::behaviour` | 6 | States, transitions and skill parsed (units, per-state parameters); mistakes reported (no `do`, unknown manoeuvre or state, bad condition); the built-in attack behaviour is valid; **target preferences parse and reject unknown selectors**; **scripted states compile and check what they name** (`call`, `every`, `fly`, `vars`; `goto` and `set_var` targets; context mistakes — app 0.18.0) |
| `content::scenario` | 3 | Objects and nav points (incl. `destroyed_systems`, `hull`), `player: choose`, **errors name the line and path** |
| `content::vfs` | 4 | Asset reference parsing and `..` normalisation, refusing escaping paths, **override path naming**, original data found by name in any layout, digit-named resources, resource and file listing |
| `settings` | 2 | settings.yaml parse/save round trip incl. `packs.disabled`, `game.campaign`, `game.difficulty` **`graphics.brightness` (clamped)** and **`console.enabled` / `console.port` (an out-of-range port is ignored)**, and the legacy settings.ini reader; **original data found in order** (command line, environment, settings, user data folder, then the checkout's `resources/xcom` before `resources/xcom-cd`, an empty folder skipped; relative paths) |
| `sim` | 16 | Generational handles, **body velocity reads drift in the ship's own axes**, deterministic RNG, scenario spawning and leaders, scenario errors, axis convention, edge-control clearing, unconsumed-press carry-over, match speed sets a speed hold; **target preferences pull the AI past a nearer target**, **station modules mount, follow and fall with their hub**; **probes extend radar and jam their enemies**, **AI pilots switch cloak, ECM and power by condition**, **protected objectives are not picked as the nearest enemy**; **cargo modules count until torn off, destroyed or jumped away with the hub** (a moving hub keeps its speed); **a tug tears a container off and prefers a disabled ship** (app 0.9.2) |
| `sim::flight` | 15 | Helpers (turn factor, damping, approach, approach_vec); **newtonian**: acceleration to the cap and coasting, half throttle, reverse thrust, brake stops exactly, turning keeps the velocity, speed hold and drift cancelling, afterburner cap and bleed-off, VCT thrust; pitch/yaw/roll directions and speed falloff; **assisted** model end to end; recharge; static objects; arena edge |
| `sim::combat` | 19 | Beams (energy, cooldown, hits on the aft shield), **bolt streak never trails behind the muzzle**, **a bolt leaves with the muzzle's velocity (ω × r) then flies straight**, **with `bolt_frame: ship` it turns with the nose while carried**, link modes, no-energy, lock and homing missile kill, lock loss and dumbfire expiry, proximity fuse on a near miss, empty racks, damage layers, rotate limits, collisions; **point defence shoots down hostile missiles**; **accumulator shields, the Permeator and Enveloping split, the Tachyon Pulser, the original's link modes, recharge while burning** (app 0.22.14, #117) |
| `sim::ai` | 28 | Steering, lead point, wingman closing on its slot, enemy hunting and damaging, wing orders, evasive ending and retreat jumping out, **aim skill decides accuracy**, **transitions switch states on conditions**, **turrets fire off the nose**, **tugs tow disabled enemies away**, disabled ships drift, target and idle behaviours; **mines go off when an enemy comes near**; **predictive collision avoidance** (head-on: break and slow; passing wide or beyond the horizon: nothing); **the formation order cycles the campaign's formations**; **escorts hold formation until a hostile comes close** (app 0.9.2); **a scripted state runs its script every interval** and **a runaway script is stopped and reported once** (app 0.18.0); **the original's steering, pilot accuracy bands, grades (look-ahead, sprint), the pilot's own missile lock and break-off** (app 0.22.8, #118); **the original's fixed, gimballed and missile turrets** (app 0.22.13, #115) |
| `sim::systems` | 19 | **Power**: raising takes from the other setting, cycling wraps to A, balance caps and message, requests change power and balance, shield balance moves points and recharge follows power; **damage**: knocked out and repaired back, destroyed beyond repair, disabled when engines and manoeuvring are out; **hyperspace** countdown, abort and departure; **tractor** hold and capture on jump, and its refusals (power, fitting, range); **cloak** hides from enemy sensors beyond range; **ECM** makes missiles lose their target; **the console's sliders set power levels and balance outright**, within the budget; **the original's repair order, per-system efficiency, engine failure below 15 %, sensors and hyperdrive unaffected, the system table with mounts, launchers, carry-over and its weights** (app 0.22.6, #116) |
| `sim::guidance` | 8 | Homing hits and dumbfire misses an offset target; sidestep still arrives; parallax decoys lure enemy missiles; cloaked wait sleeps until prey comes near; cluster splits into submunitions; doppelganger launches a decoy ship that expires; damage modes |
| `mission` | 7 | Roles are placed, counted and faced at the player; arrivals, waves, chatter and a win; lose beats win and the timer ends it; **the player's death is its own result (`killed`) with the lose outcomes**; **wingmen take successive slots of their formation** across roles; **the condition console evaluates against the battle without changing it** (a role count, an unknown role is an error, the mission's generator untouched — app 0.16.5); **a script outcome runs in the battle context** (checked against the mission's roles — app 0.18.0) |
| `sim::stations` | 10 | **A station's one shield pool** starts full at its first generator's capacity, recharges 2 per second to its maximum, follows the next generator and keeps or drops its points when the last falls, and takes fore and aft hits alike; **a pool without generators recharges to its own `shields`**; **covered modules** pass hits to the hub's pool and the overflow to the hub's hull; **`damage_model: station`** skips armour, systems and the difficulty factor (app 0.22.4, #113); **hard points and destruction by them; modules by position and chance; minefields by level** (app 0.22.11, #114); **mines go off on contact** (app 0.22.13) |
| `sim::targeting` | 4 | Cycling with wrap-around, nearest hostile, most-central-in-sights, nothing to target |
| `sim::hazards` | 4 | Asteroid fields and minefields are scattered in their sphere, clear of craft, deterministically, asteroids as scenery; a black hole pulls harder when near (capped) and swallows what crosses its radius; AI pilots turn away from it; an ion storm shortens radar range |
| `sim::env` | 5 | Drag, gravity and lift act in an atmosphere and not in space (hover cancels gravity, lift carries the weight at the `lift` speed); AI pilots pull up near the ground; gentle touchdowns land, hard ones crash with damage, and the ceiling is an exit; ground installations are set down on the terrain; **craft spawned below 300 m over a ground are lifted to 300 m** |
| `script` | 4 | Units and tokens; evaluation with precedence and short-circuit; **errors found when loading** (unknown names, argument counts, types); **script lexing keeps lines, text and statement ends** (app 0.18.0) |
| `script::object` | 1 | Query ids match the table; system names match the kinds; level names as constants |
| `script::battle` | 1 | Query ids match the table; role names are checked names |
| `script::action` | 2 | Actions parse and are checked (names, argument kinds, battle or campaign scope); **a text argument keeps its commas** (app 0.10.1) |
| `script::stmt` | 11 | Tier 2 scripts (app 0.18.0): `let`, assignment and `if`/`else if`/`else` chains; **`repeat` and `for each` are counted loops** (at most 1 000 rounds); functions take typed parameters and return values; **recursion is refused when loading** (with the chain); **the budget stops a runaway hook**; same seed, same result; **mistakes are load errors with file and line**; hook references are checked; file structure is checked when loading; programs list and rewrite their action arguments; every context's queries fit the argument buffer |
| `sim::collision` | 3 | Segment–sphere, segment–triangle, shapes with meshes in model space and the broad phase |
| `input` | 15 | **The keys help lists every action with its current keys**, Original default layout incl. brake, **power and equipment keys**, one-shot commands and Shift chords, **a Shift chord does not trigger the plain held key**, gamepad sticks/buttons/presses and dead zone, **pad modifier layer reaches every power control**, **every action reachable from the keyboard and flight actions from the pad**, **stick layouts and mouse inversion**, mouse flight, chase keys, bindings.ini override and round trip, view names, chord parsing; **every action has a one-sentence description** (app 0.15.1) |
| `input::editor` | 5 | Rows grouped with headers skipped, capturing a key rebinds and takes it from other actions, capturing plain pad presses and chords, clear / reset / settings, pretty names |
| `input::gamepad` | 2 | Button names and bits round trip, axis normalisation |
| `render` | 4 | Billboard and streak geometry, colour helpers, overlay depth bias pasted into the shader; **uniforms shared by both lit shaders match exactly, with an explicit precision** (the `Light` link fix, 0.4.1) |
| `render::camera` | 4 | Projection to screen and behind-camera, rotation, near plane changes depth only, camera-relative model matrices |
| `render::effects` | 3 | Events create sparks/explosions/debris that age and expire, explosion curve, jitter range |
| `render::gpu` | 4 | Per-texture chunks with local u16 indices and the overlay flag, splitting huge groups, engine glow points; **shared-vertex grids are split by index count** (every index of a 58 806-index grid drawn in chunks within `MAX_CHUNK_INDICES`, app 0.18.0) |
| `render::starfield` | 1 | Same seed same sky, unit directions, uniform hemispheres |
| `render::terrain` | 5 | Terrain meshes face up (counter-clockwise from above) and are coloured from low to high by height (the X-mirrored mesh too); **every placed horizon tile matches the folded ground height**; **detail coordinates repeat per `detail_scale` and meet at tile edges**; **steep triangles take the slope texture**; **scatter is baked into cells in place** (app 0.18.0) |
| `hud` | 7 | Layout fitting any design space (4:3 and widescreen), radar placement (front/rear, mirroring), gauges, MFD pages (incl. Systems), **power readout and system lamps**, message expiry; **the message log reports what is new since a count** (the console's events, kept-lines limit) |
| `hud::layout` | 4 | Anchor corners lie on the surface with world up; rays pick the nearest surface; 2D dragging and anchor scaling; saved layouts are valid rules |
| `hud::draw` | 5 | Lamp grids follow the panel shape; shield rings go green, amber, red, then blink; **drift reads signed whole metres per second**; **the pitch ladder sits ahead at its angles, the horizon level whatever the camera's pitch**; **the ship status display lights one band per third of a shield, and its hull tint runs green to red** (app 0.15.1) |
| `hud::glyphs` | 1 | Text sizes snap down onto the glyph ladder |
| `audio` | 3 | Distance attenuation, event → sound and volume choices (incl. `system_damage`, switch sounds, `hyperspace`); **music segments cycle within the wanted state** and a missing state plays cruise (app 0.11.0) |
| `app::assets` | 2 | Tab cycling; empty lists without original data |
| `app::devices` | 2 | Key names match the bindings vocabulary |
| `app::flight` | 10 | Camera pose for every view, **3D cockpit field of view widens only on narrow screens**, chase orbit, targeting and wing-order commands, pilot messages from events (incl. system, disabled, jump and capture messages), **a wingman answers in its own voice** (only the player's wingmen, only with the line, never the lost one); **a campaign loadout refits the player's class in this battle only**; **the debrief waits for the delay after the result**; **campaign stances decide who fights whom** (app 0.10.3); **the keys overlay shows descriptions only when its columns are wide enough** (app 0.15.1); a campaign loadout also sets fitted shields and armour (0.15.0) |
| `app::simulator` | 2 | **A setup changes only the environment, hazards and enemy force** (the defaults change nothing; enemies ×2, friends unchanged, never below one); **ships that cannot enter an atmosphere are refused there** |
| `app::launcher` | 3 | Menu entries (viewer, asset browser, controls, simulator, scenarios; starting on the first scenario) and wrapping, ship choice only when choosing, **next campaign**, text wrapping |
| `app::viewer` | 9 | Orbit pose and clamping, entries for every class, wheel steps, **display-mode cycle, fitted near plane, face info line, finding a model by ship id or mesh name**; **looks cover damage levels, states and debris** |
| `check` | 2 | The **default campaign** passes; broken definitions become errors |
| `extract` | 3 | OBJ export round-trips through the mod loader (positions, triangles, uvs, materials, flat colours), PNG encoding round trip, a full extraction of a synthetic CD writes every override path (incl. cockpit cyan → alpha) |
| `cli` | 4 | Every option incl. `--extract`, `--view`, `--model`, `--campaign`, `--screenshot`, `--frames`, environment variable precedence, unknown/incomplete options; **content folders are `resources/` then `packs/`, and `--packs` replaces only the mods folder** |
| `remote` | 1 | **Console input**: held actions last until released, taps fire once, the throttle is one-shot, the bindings see a console-held action like a held key, a console that leaves lets go |
| `remote::ws` | 3 | **SHA-1 and base64 match the published vectors** (FIPS 180 and RFC 4648; the RFC 6455 handshake example), masked frames decode (the RFC 6455 §5.7 examples), partial frames wait for more bytes, back-to-back frames, 16-bit lengths, unmasked server frames with 16- and 64-bit lengths; **protocol violations are errors** (unmasked, unknown opcode, too large) |
| `remote::json` | 3 | Objects, arrays, strings with escapes, numbers with exponents; **bad and hostile input is an error** (unclosed, trailing commas, trailing text, nesting deeper than 32); the writer's compact objects (numbers to 3 decimals, NaN as `null`) parse back |
| `remote::http` | 3 | A WebSocket upgrade parses and the parser waits for the blank line; bad requests are errors (incl. an oversized head) and responses are well formed; **byte ranges are served as `206 Partial Content`** (`a-b`, `a-`, `-n`, clamped end; no range or out of range gives the whole file with `Accept-Ranges`) |
| `remote::protocol` | 7 | Console messages decode (incl. `power`), unknown actions and types are refused, throttle clamped; **the catalogue lists every action and marks missing equipment unavailable**; **telemetry reports drift in body axes and the target's rotation relative to the player**; **`model` messages are normalised to radius 1 with a colour per group** (average colour ignores transparent pixels); **`contacts` are in the player's axes** (a craft dead ahead after a 90° yaw), relation and target flag, and `target {id}` decodes (bad ids refused); `event`; **themes layer per cockpit and become CSS variables** (cockpit override over the base, `--font`, `--corner`, an empty theme sends none); **the idle catalogue lists every action with its description, all available** (app 0.15.1) |
| `remote::server` | 3 | **Only local-network peers are served** (IPv4 private, link-local, loopback; IPv6 loopback, unique-local, link-local, IPv4-mapped); pairing codes are six digits and vary with the seed; **end to end over a real localhost socket**: page served, 404, handshake accept key, unpaired action dropped, wrong code denied, pairing, action relayed, broadcast received; **`welcome` answered at once by the server** (0.4.7). Plus one ignored helper, `serve_for_a_browser` (not counted) |
| `soak` | 3 | The default campaign's difficulty scales enemy lethality (headless battle runs); **at cadet an undefended station holds for 150 s** (idle pilot); **the default campaign's scripted ace taunts and turns furious** (headless, app 0.18.0) |
| `balance` | 2 | **Energy limits sustained fire**, and kill times follow effective hit points — app 0.15.0; **auto-resolve odds favour the stronger side** (more foes worse, more craft better, always 0..1 — app 0.16.12) |

## Test names

Generated from the `#[test]` functions in `src/` at main `bf7b7ce` (read with grep, not a build).

**`app::assets`**: `no_original_data_means_empty_lists`, `tabs_cycle`

**`app::devices`**: `key_names_match_the_bindings_vocabulary`, `known_names_are_real_keycodes`

**`app::flight`**: `a_wingman_answers_in_its_own_voice`, `chase_orbit_responds_and_resets`, `cockpit_fov_widens_only_on_narrow_screens`, `commands_target_and_order_wingmen`, `events_become_pilot_messages`, `views_place_the_camera`, `campaign_stances_decide_who_fights_whom`

**`app::launcher`**: `menu_lists_viewer_and_scenarios_and_wraps`, `scenarios_start_with_the_chosen_ship_only_when_choosing`, `wrapping_text`

**`app::simulator`**: `a_setup_changes_environment_hazards_and_enemy_force_only`, `ships_that_cannot_enter_an_atmosphere_are_refused_there`

**`app::viewer`**: `a_still_wheel_does_not_zoom`, `default_entries_list_every_class`, `display_modes_cycle`, `entries_are_found_by_ship_id_or_mesh_name`, `face_info_describes_a_face`, `looks_cover_damage_states_and_debris`, `near_plane_moves_out_to_the_model`, `orbit_input_clamps`, `orbit_looks_at_the_origin_from_its_distance`

**`audio`**: `attenuation_falls_with_distance`, `events_choose_sounds_and_volumes`

**`check`**: `broken_definitions_are_errors`, `the_default_campaign_passes`

**`cli`**: `content_dirs_are_resources_then_packs`, `environment_supplies_data_and_flags_override_it`, `errors_for_unknown_or_incomplete_options`, `parses_every_option`

**`content`**: `empty_pack_dirs_are_an_error`, `loads_original_free_packs_rules_scenarios_and_models`, `pack_files_override_original_assets`, `png_and_tga_pack_images_decode`, `scaling_scales_everything`

**`content::behaviour`**: `parses_states_transitions_and_skill`, `reports_mistakes`, `target_preferences_parse_and_reject_unknown_selectors`, `the_builtin_attack_is_valid`

**`content::equipment`**: `kinds_round_trip_and_builtins_exist`, `power_section_parses_and_validates`, `system_definitions_override_builtins`

**`content::environment`**: `environments_and_terrains_parse_and_report_mistakes`, `generated_heightfields_span_their_range_are_deterministic_and_interpolate`, `nebula_clouds_vary_stay_in_range_and_repeat_for_a_seed`

**`content::files`**: `every_schema_builds_and_exports`

**`content::hazards`**: `hazards_parse_with_defaults_and_report_mistakes`

**`content::looks`**: `debris_and_effects_parse_with_defaults`, `swaps_accumulate_with_damage_and_state`, `texture_keys_ignore_folders_case_and_extensions`

**`content::mission`**: `a_full_mission_parses`, `mistakes_are_reported_with_where`

**`content::pack`**: `detects_missing_duplicate_and_cyclic_dependencies`, `discovers_pack_folders`, `manifests_declare_kind_parent_targets_and_format`, `orders_dependencies_first_and_keeps_order_otherwise`, `parses_a_manifest`, `rejects_bad_manifests`, `the_active_set_follows_the_chosen_campaign`

**`content::rules`**: `builds_typed_definitions_with_inheritance`, `difficulty_variants_and_per_level_values`, `later_packs_merge_delete_and_edit_lists`, `reports_every_error_and_keeps_going`, `sound_cues_resolve_through_the_original_table`, `spelling_of_original_keys`

**`content::scenario`**: `choose_means_the_launcher_picks`, `errors_name_the_line_and_path`, `parses_objects_and_nav_points`

**`content::schema`**: `good_files_pass`, `json_schema_is_wellformed_enough`, `mistakes_are_reported_with_line_and_path`, `quantities_and_suggestions`

**`content::text`**: `keys_resolve_plain_values_pass_through_and_imports_are_listed`

**`content::vfs`**: `original_data_is_found_by_name_in_any_layout`, `override_paths_follow_the_convention`, `parses_asset_references`, `refuses_paths_that_escape_the_pack`

**`extract`**: `extraction_writes_override_paths`, `obj_export_round_trips_through_the_mod_loader`, `png_encoding_decodes_back`

**`formats`**: `errors_have_readable_messages`, `names_stop_at_the_first_nul`, `out_of_bounds_is_an_error_not_a_panic`, `reads_little_endian_numbers`

**`formats::ini`**: `bad_numbers_and_bools_are_none`, `latin1_bytes_are_accepted`, `merge_overrides_keys_and_adds_sections`, `parses_sections_keys_and_comments`, `reports_lines_it_ignores`

**`formats::langfile`**: `keys_text_and_line_breaks`

**`formats::obj`**: `parses_materials`, `parses_positions_uvs_normals_and_faces`, `rejects_bad_indices_and_short_faces`, `slot_groups_become_attachment_points`

**`formats::pe`**: `finds_resources_by_type_and_name`, `missing_file_is_an_error`, `name_ids_are_normalised`, `rejects_non_pe_data`, `truncated_files_are_errors`

**`formats::tga`**: `decodes_16_bit_with_colour_key_and_flips_rows`, `decodes_24_and_32_bit_bgr`, `decodes_colour_mapped_images`, `decodes_rle`, `rejects_unsupported_and_truncated`

**`formats::wav`**: `adpcm_step_clamps_and_keeps_delta_minimum`, `ima_adpcm_block_decodes`, `ms_adpcm_block_decodes`, `pcm16_round_trips_through_to_wav_bytes`, `pcm8_is_recentred`, `rejects_bad_files`

**`formats::xmsh`**: `mesh_without_lods_is_invalid`, `parses_a_hand_built_mesh`, `polygon_flags_are_read`, `rejects_bad_signature_and_bad_references`, `signed_edges_give_the_vertex_loop`, `truncated_mesh_is_an_error`, `untextured_polygons_have_no_texture`

**`formats::xtxr`**: `decodes_pixels_through_palette_zero`, `expands_rgb555_to_full_range`, `rejects_bad_data`

**`formats::yaml`**: `block_scalars_and_wrapped_flows`, `empty_and_scalar_documents`, `keys_with_colons_and_urls_stay_whole`, `mappings_sequences_and_scalars`, `unsupported_and_broken_input_is_an_error_with_a_line`, `writing_round_trips`

**`hud`**: `gauges_sweep_and_read_the_ship`, `layout_scales_by_height_and_centres`, `messages_expire_and_keep_the_newest`, `mfd_pages_cycle_and_describe`, `power_and_system_readouts`, `radar_places_contacts_by_angle`, `the_message_log_reports_what_is_new_since_a_count`

**`hud::draw`**: `drift_reads_signed_whole_metres_per_second`, `lamp_grids_follow_the_panel_shape`, `shield_rings_go_green_amber_red_then_blink`, `the_pitch_ladder_sits_ahead_at_its_angles`

**`hud::glyphs`**: `sizes_snap_down_onto_the_ladder`

**`hud::layout`**: `corners_lie_on_the_surface_with_world_up`, `dragging_and_scaling`, `rays_pick_the_nearest_surface`, `saved_layouts_are_valid_rules`

**`input`**: `a_shift_chord_does_not_also_trigger_the_plain_held_key`, `bindings_file_overrides_and_round_trips`, `chase_camera_keys`, `chord_parsing_and_dead_zone`, `default_keys_follow_the_original_layout`, `every_action_is_reachable_from_keyboard_and_flight_actions_from_the_pad`, `gamepad_sticks_buttons_and_presses`, `mouse_flight_only_when_enabled`, `one_shot_commands_and_shift_chords`, `keys_held_when_a_battle_starts_are_ignored_until_released`, `pad_modifier_layer_reaches_every_power_control`, `power_and_equipment_keys_follow_the_original`, `stick_layouts_and_mouse_inversion`, `the_keys_help_lists_every_action_with_its_current_keys`, `view_names_parse`

**`input::editor`**: `capturing_a_key_rebinds_and_steals_it_from_others`, `capturing_pad_plain_presses_and_chords`, `clear_reset_and_settings`, `pretty_names`, `rows_are_grouped_and_headers_are_skipped`

**`input::gamepad`**: `axes_are_normalised`, `buttons_round_trip_names_and_bits`

**`mission`**: `arrivals_waves_chatter_and_a_win`, `lose_beats_win_and_the_timer_ends_it`, `roles_are_placed_counted_and_faced_at_the_player`, `the_players_death_is_its_own_result_with_the_lose_outcomes`

**`model`**: `empty_obj_has_zero_bounds`, `faces_record_their_corners_and_group`, `faces_that_only_touch_or_face_away_are_not_layered`, `later_overlapping_face_is_layered_on_top`, `lod_selection_by_distance`, `mirrored_triangles_keep_outward_winding`, `obj_models_use_materials_scale_and_flip_v`, `overlay_flag_beats_file_order`, `xmsh_is_triangulated_grouped_and_mirrored`

**`platform::linux`**: `events_map_to_xinput_layout`

**`remote`**: `held_actions_last_until_released_and_taps_fire_once`

**`remote::http`**: `bad_requests_are_errors_and_responses_are_well_formed`, `parses_a_websocket_upgrade_and_waits_for_the_blank_line`, `ranges_are_served_as_partial_content`

**`remote::json`**: `bad_and_hostile_input_is_an_error`, `parses_objects_arrays_strings_and_numbers`, `writes_compact_objects_that_parse_back`

**`remote::protocol`**: `contacts_are_in_the_players_axes_and_targets_decode`, `decodes_console_messages_and_refuses_unknown_ones`, `model_messages_are_normalised_and_coloured_per_group`, `telemetry_reports_drift_and_the_targets_relative_rotation`, `the_catalogue_lists_every_action_and_marks_missing_equipment`, `themes_layer_per_cockpit_and_become_css_variables`

**`remote::server`**: `only_local_network_peers_are_served`, `pairing_codes_are_six_digits_and_vary_with_the_seed`, `serves_the_page_pairs_and_relays_actions_over_a_real_socket`; ignored helper: `serve_for_a_browser`

**`remote::ws`**: `frames_decode_masked_wait_for_more_bytes_and_encode`, `protocol_violations_are_errors`, `sha1_and_base64_match_the_published_test_vectors`

**`render`**: `billboards_and_streaks_build_two_triangles`, `colour_helpers_clamp`, `layer_bias_reaches_the_shader`, `uniforms_shared_by_both_lit_shaders_match_exactly`

**`render::camera`**: `model_matrices_are_camera_relative`, `near_plane_changes_only_depth`, `points_ahead_project_to_the_centre_and_behind_is_none`, `rotation_turns_the_view`

**`render::effects`**: `events_create_effects_that_expire`, `explosion_curve_grows_and_fades`, `jitter_stays_in_range`

**`render::gpu`**: `engines_use_rear_attachments_or_the_back_centre`, `groups_become_chunks_with_local_indices`, `huge_groups_are_split`

**`render::starfield`**: `same_seed_same_sky_and_unit_directions`

**`render::terrain`**: `terrain_meshes_face_up_and_colour_by_height`

**`script`**: `errors_are_found_when_loading`, `evaluates_with_precedence_and_short_circuits`, `units_and_tokens`

**`script::action`**: `actions_parse_and_check`, `a_text_argument_keeps_its_commas`

**`script::battle`**: `ids_match_and_roles_are_checked_names`

**`script::object`**: `ids_match_the_table_and_system_names_match_the_kinds`

**`settings`**: `original_data_is_found_in_order`, `parses_and_round_trips`

**`sim`**: `ai_pilots_switch_cloak_ecm_and_power_by_condition`, `body_velocity_reads_drift_in_the_ships_own_axes`, `generational_handles_detect_reuse`, `match_speed_copies_the_targets_speed`, `preferences_pull_the_ai_past_a_nearer_target`, `probes_extend_radar_and_jam_their_enemies`, `protected_objectives_are_not_picked_as_the_nearest_enemy`, `random_is_deterministic_and_in_range`, `scenario_errors_are_reported`, `scenario_spawns_player_objects_and_leaders`, `ship_axes_follow_the_engine_convention`, `station_modules_mount_follow_and_fall_with_their_hub`, `step_advances_time_and_clears_edge_controls`, `unconsumed_presses_carry_over_held_inputs_do_not`, `a_tug_tears_a_container_off_and_prefers_a_disabled_ship`, `cargo_modules_count_until_torn_off_destroyed_or_jumped_away_with_the_hub`

**`sim::ai`**: `aim_skill_decides_accuracy`, `disabled_ships_drift`, `enemy_hunts_and_damages_a_hostile`, `evasive_ends_and_retreat_jumps_out`, `lead_point_anticipates_motion`, `mines_go_off_when_an_enemy_comes_near`, `orders_switch_wingmen_to_attack`, `steering_turns_towards_points`, `target_drones_circle_and_idle_ships_sit`, `transitions_switch_states_on_conditions`, `tugs_tow_disabled_enemies_away`, `turrets_fire_off_the_nose`, `wingman_closes_on_its_slot`, `escorts_hold_formation_until_a_hostile_comes_close`

**`sim::collision`**: `segment_sphere_hits_misses_and_inside`, `segment_triangle_hits_inside_only`, `shapes_use_mesh_in_model_space`

**`sim::combat`**: `a_bolt_leaves_with_the_muzzles_velocity_then_flies_straight`, `a_new_bolt_streak_never_reaches_behind_the_muzzle`, `beams_fire_cost_energy_respect_cooldown_and_hit`, `beams_need_energy`, `damage_layers_and_aft_shield`, `empty_racks_report_and_cycle`, `fighters_bounce_off_each_other_and_stations`, `link_modes_cycle_and_change_how_many_fire`, `lock_builds_in_the_cone_and_missile_homes_in`, `lock_is_lost_outside_the_cone_and_dumbfire_works`, `point_defence_shoots_down_hostile_missiles`, `proximity_fuse_fires_on_a_near_miss`, `rotate_towards_limits_the_turn`, `with_bolt_frame_ship_a_bolt_turns_with_the_nose_while_carried`

**`sim::flight`**: `afterburner_raises_the_cap_drains_energy_and_excess_bleeds_off`, `assisted_model_holds_throttle_speed_and_damps_drift`, `brake_stops_against_the_direction_of_travel`, `energy_and_shields_recharge_up_to_capacity`, `half_throttle_is_half_the_acceleration`, `helpers`, `newtonian_accelerates_to_the_cap_and_coasts`, `pitching_up_raises_the_nose_and_rates_fall_with_speed`, `reverse_thrust_slows_and_then_backs_up`, `speed_hold_matches_speed_and_cancels_drift`, `static_objects_do_not_move`, `the_arena_edge_halts_outward_motion`, `turning_does_not_steer_the_velocity`, `vct_thrusts_sideways`, `yaw_right_and_roll_right_turn_the_expected_way`

**`sim::env`**: `ai_pilots_pull_up_near_the_ground`, `craft_spawned_low_over_a_ground_are_lifted_clear_of_it`, `drag_gravity_and_lift_act_in_air_and_not_in_space`, `gentle_touchdowns_land_hard_ones_crash_and_the_ceiling_is_an_exit`, `ground_installations_are_set_down_on_the_terrain`

**`sim::guidance`**: `cloaked_wait_sleeps_until_prey_comes_near`, `cluster_splits_into_submunitions`, `damage_modes`, `doppelganger_launches_a_decoy_ship_that_expires`, `homing_hits_and_dumbfire_misses_an_offset_target`, `parallax_decoys_lure_enemy_missiles`, `sidestep_still_arrives`

**`sim::hazards`**: `a_black_hole_pulls_harder_when_near_and_swallows_what_crosses_its_radius`, `ai_pilots_turn_away_from_a_black_hole`, `an_ion_storm_shortens_radar_range`, `asteroid_fields_and_minefields_are_scattered_clear_of_craft`

**`sim::systems`**: `balance_caps_and_message`, `cloaked_ships_vanish_from_enemy_sensors_beyond_range`, `cycling_wraps_to_a`, `damage_knocks_systems_out_and_repairs_bring_them_back`, `disabled_when_engines_and_manoeuvring_are_out`, `ecm_makes_incoming_missiles_lose_their_target`, `hyperspace_counts_down_aborts_and_departs`, `raising_takes_power_from_the_other_setting`, `requests_change_power_and_balance`, `shield_balance_moves_points_forward_and_recharge_follows_power`, `sliders_set_power_levels_and_balance_outright`, `systems_can_be_destroyed_beyond_repair`, `tractor_holds_a_disabled_target_and_captures_it_on_jump`, `tractor_needs_power_a_fitting_and_a_target_in_range`

**`sim::targeting`**: `cycles_forwards_and_backwards_with_wraparound`, `in_sights_picks_the_most_central`, `nearest_hostile_ignores_friends`, `nothing_to_target`

**`soak`**: `at_cadet_an_undefended_station_holds_for_150_seconds`, `default_campaign_difficulty_scales_enemy_lethality`

## Running

```sh
cargo test                     # all
cargo test sim::               # one module
cargo test proximity           # tests whose name contains "proximity"
cargo test -- --nocapture      # show println! output
```
