# Unit tests

Every module's tests, as of main `d7553f5` (build 0.3.0): **269 tests** on macOS (270 on
Linux), run with `cargo test` by the gate. Tests
live next to the code in `#[cfg(test)] mod tests` blocks and use **synthetic data built in the
test** (fake DLLs, meshes, WAVs, temp pack folders) — never original game files.

`platform::linux` has a test too (joystick event mapping); it is compiled and run only on Linux,
so it is not in the count from macOS.

Not unit-tested, by design, and covered by [manual tests](manual-tests.md) instead: the
`hud::draw` painting, the `Renderer`'s draw calls and shaders, macroquad device reading, sound
playback, the 3D cockpit pass, anchored instruments drawn into textures, the asset browser's, controls menu's and layout editor's drawing and the macOS gamepad shim — they need a window, a GPU, speakers or a controller.

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
| `content` | 5 | Loading packs/rules/scenarios/models from temp folders, skipping packs that need the CD, scenario replacement, **per-asset overrides** of a mesh, image and sound and their removal when the pack is disabled, PNG/TGA decoding, model scaling, empty pack dirs |
| `content::pack` | 7 | Manifest parsing and validation, **kind, parent, targets and format** (and their errors), **the active set for a chosen campaign** (parent chain, targeting addons, dependencies), dependency ordering, missing/duplicate/cyclic dependencies, discovery |
| `content::rules` | 6 | Typed definitions with original-section inheritance (YAML), variants, defaults, sounds, hostility, flight model and arena defaults, cockpits; **later packs merge, delete and edit lists**; **difficulty variants and per-level values**; every error reported; **spelling of original keys** (`HitPoints` → `hit_points`, `Side` → `faction`, beam `Range` in km); **sound cues resolve through the original table** |
| `content::equipment` | 3 | System kinds round trip and built-ins exist, `systems` definitions overriding built-ins, `power` parsing and validation (level counts, start levels) |
| `content::looks` | 3 | Debris sets and effects parse with defaults; texture swaps accumulate with damage levels and state; texture keys ignore folders, case and extensions |
| `content::mission` | 2 | A full mission parses (roles, class lists, counts, arrivals, waves, objectives, optional, timer, outcomes, chatter, debrief); mistakes are reported with where (unknown role, bad timer result, unknown action) |
| `content::text` | 1 | Keys resolve, plain values pass through, imports are listed |
| `content::schema` | 4 | Good files pass; **mistakes reported with line and key path** (unknown key with suggestion, wrong type, bad unit); quantities with units and did-you-mean suggestions; JSON Schema export well formed |
| `content::files` | 1 | Every file schema builds and exports |
| `content::behaviour` | 4 | States, transitions and skill parsed (units, per-state parameters); mistakes reported (no `do`, unknown manoeuvre or state, bad condition); the built-in attack behaviour is valid; **target preferences parse and reject unknown selectors** |
| `content::scenario` | 3 | Objects and nav points (incl. `destroyed_systems`, `hull`), `player: choose`, **errors name the line and path** |
| `content::vfs` | 4 | Asset reference parsing and `..` normalisation, refusing escaping paths, **override path naming**, original data found by name in any layout, digit-named resources, resource and file listing |
| `settings` | 2 | settings.yaml parse/save round trip incl. `packs.disabled`, `game.campaign` and `game.difficulty`, and the legacy settings.ini reader; original-data resolution order (command line, environment, settings, user data folder, development tree, relative paths) |
| `sim` | 13 | Generational handles, deterministic RNG, scenario spawning and leaders, scenario errors, axis convention, edge-control clearing, unconsumed-press carry-over, match speed sets a speed hold; **target preferences pull the AI past a nearer target**, **station modules mount, follow and fall with their hub**; **probes extend radar and jam their enemies**, **AI pilots switch cloak, ECM and power by condition**, **protected objectives are not picked as the nearest enemy** |
| `sim::flight` | 15 | Helpers (turn factor, damping, approach, approach_vec); **newtonian**: acceleration to the cap and coasting, half throttle, reverse thrust, brake stops exactly, turning keeps the velocity, speed hold and drift cancelling, afterburner cap and bleed-off, VCT thrust; pitch/yaw/roll directions and speed falloff; **assisted** model end to end; recharge; static objects; arena edge |
| `sim::combat` | 12 | Beams (energy, cooldown, hits on the aft shield), **bolt streak never trails behind the muzzle**, link modes, no-energy, lock and homing missile kill, lock loss and dumbfire expiry, proximity fuse on a near miss, empty racks, damage layers, rotate limits, collisions; **point defence shoots down hostile missiles** |
| `sim::ai` | 13 | Steering, lead point, wingman closing on its slot, enemy hunting and damaging, wing orders, evasive ending and retreat jumping out, **aim skill decides accuracy**, **transitions switch states on conditions**, **turrets fire off the nose**, **tugs tow disabled enemies away**, disabled ships drift, target and idle behaviours; **mines go off when an enemy comes near** |
| `sim::systems` | 13 | **Power**: raising takes from the other setting, cycling wraps to A, balance caps and message, requests change power and balance, shield balance moves points and recharge follows power; **damage**: knocked out and repaired back, destroyed beyond repair, disabled when engines and manoeuvring are out; **hyperspace** countdown, abort and departure; **tractor** hold and capture on jump, and its refusals (power, fitting, range); **cloak** hides from enemy sensors beyond range; **ECM** makes missiles lose their target |
| `sim::guidance` | 7 | Homing hits and dumbfire misses an offset target; sidestep still arrives; parallax decoys lure enemy missiles; cloaked wait sleeps until prey comes near; cluster splits into submunitions; doppelganger launches a decoy ship that expires; damage modes |
| `mission` | 3 | Roles are placed, counted and faced at the player; arrivals, waves, chatter and a win; lose beats win and the timer ends it |
| `sim::targeting` | 4 | Cycling with wrap-around, nearest hostile, most-central-in-sights, nothing to target |
| `script` | 3 | Units and tokens; evaluation with precedence and short-circuit; **errors found when loading** (unknown names, argument counts, types) |
| `script::object` | 1 | Query ids match the table; system names match the kinds; level names as constants |
| `script::battle` | 1 | Query ids match the table; role names are checked names |
| `script::action` | 1 | Actions parse and are checked (names, argument kinds, battle or campaign scope) |
| `sim::collision` | 3 | Segment–sphere, segment–triangle, shapes with meshes in model space and the broad phase |
| `input` | 13 | Original default layout incl. brake, **power and equipment keys**, one-shot commands and Shift chords, **a Shift chord does not trigger the plain held key**, gamepad sticks/buttons/presses and dead zone, **pad modifier layer reaches every power control**, **every action reachable from the keyboard and flight actions from the pad**, **stick layouts and mouse inversion**, mouse flight, chase keys, bindings.ini override and round trip, view names, chord parsing |
| `input::editor` | 5 | Rows grouped with headers skipped, capturing a key rebinds and takes it from other actions, capturing plain pad presses and chords, clear / reset / settings, pretty names |
| `input::gamepad` | 2 | Button names and bits round trip, axis normalisation |
| `render` | 3 | Billboard and streak geometry, colour helpers, overlay depth bias pasted into the shader |
| `render::camera` | 4 | Projection to screen and behind-camera, rotation, near plane changes depth only, camera-relative model matrices |
| `render::effects` | 3 | Events create sparks/explosions/debris that age and expire, explosion curve, jitter range |
| `render::gpu` | 3 | Per-texture chunks with local u16 indices and the overlay flag, splitting huge groups, engine glow points |
| `render::starfield` | 1 | Same seed same sky, unit directions, uniform hemispheres |
| `hud` | 6 | Layout fitting any design space (4:3 and widescreen), radar placement (front/rear, mirroring), gauges, MFD pages (incl. Systems), **power readout and system lamps**, message expiry |
| `hud::layout` | 4 | Anchor corners lie on the surface with world up; rays pick the nearest surface; 2D dragging and anchor scaling; saved layouts are valid rules |
| `hud::draw` | 2 | Lamp grids follow the panel shape; shield rings go green, amber, red, then blink |
| `hud::glyphs` | 1 | Text sizes snap down onto the glyph ladder |
| `audio` | 2 | Distance attenuation, event → sound and volume choices (incl. `system_damage`, switch sounds, `hyperspace`) |
| `app::assets` | 2 | Tab cycling; empty lists without original data |
| `app::devices` | 2 | Key names match the bindings vocabulary |
| `app::flight` | 5 | Camera pose for every view, **3D cockpit field of view widens only on narrow screens**, chase orbit, targeting and wing-order commands, pilot messages from events (incl. system, disabled, jump and capture messages) |
| `app::launcher` | 3 | Menu entries (viewer, asset browser, controls, scenarios) and wrapping, ship choice only when choosing, **next campaign**, text wrapping |
| `app::viewer` | 9 | Orbit pose and clamping, entries for every class, wheel steps, **display-mode cycle, fitted near plane, face info line, finding a model by ship id or mesh name**; **looks cover damage levels, states and debris** |
| `check` | 2 | The **default campaign** passes; broken definitions become errors |
| `extract` | 3 | OBJ export round-trips through the mod loader (positions, triangles, uvs, materials, flat colours), PNG encoding round trip, a full extraction of a synthetic CD writes every override path (incl. cockpit cyan → alpha) |
| `cli` | 4 | Every option incl. `--extract`, `--view`, `--model`, `--campaign`, `--screenshot`, `--frames`, environment variable precedence, unknown/incomplete options, packs folder discovery |
| `soak` | 1 | The default campaign's difficulty scales enemy lethality (headless battle runs) |

## Test names

**`app::assets`**: `no_original_data_means_empty_lists`, `tabs_cycle`

**`app::devices`**: `key_names_match_the_bindings_vocabulary`, `known_names_are_real_keycodes`

**`app::flight`**: `chase_orbit_responds_and_resets`, `cockpit_fov_widens_only_on_narrow_screens`, `commands_target_and_order_wingmen`, `events_become_pilot_messages`, `views_place_the_camera`

**`app::launcher`**: `menu_lists_viewer_and_scenarios_and_wraps`, `scenarios_start_with_the_chosen_ship_only_when_choosing`, `wrapping_text`

**`app::viewer`**: `a_still_wheel_does_not_zoom`, `default_entries_list_every_class`, `display_modes_cycle`, `entries_are_found_by_ship_id_or_mesh_name`, `face_info_describes_a_face`, `looks_cover_damage_states_and_debris`, `near_plane_moves_out_to_the_model`, `orbit_input_clamps`, `orbit_looks_at_the_origin_from_its_distance`

**`audio`**: `attenuation_falls_with_distance`, `events_choose_sounds_and_volumes`

**`check`**: `broken_definitions_are_errors`, `the_default_campaign_passes`

**`cli`**: `environment_supplies_data_and_flags_override_it`, `errors_for_unknown_or_incomplete_options`, `finds_the_source_tree_packs`, `parses_every_option`

**`content`**: `empty_pack_dirs_are_an_error`, `loads_original_free_packs_rules_scenarios_and_models`, `pack_files_override_original_assets`, `png_and_tga_pack_images_decode`, `scaling_scales_everything`

**`content::behaviour`**: `parses_states_transitions_and_skill`, `reports_mistakes`, `target_preferences_parse_and_reject_unknown_selectors`, `the_builtin_attack_is_valid`

**`content::equipment`**: `kinds_round_trip_and_builtins_exist`, `power_section_parses_and_validates`, `system_definitions_override_builtins`

**`content::files`**: `every_schema_builds_and_exports`

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

**`hud`**: `gauges_sweep_and_read_the_ship`, `layout_scales_by_height_and_centres`, `messages_expire_and_keep_the_newest`, `mfd_pages_cycle_and_describe`, `power_and_system_readouts`, `radar_places_contacts_by_angle`

**`hud::draw`**: `lamp_grids_follow_the_panel_shape`, `shield_rings_go_green_amber_red_then_blink`

**`hud::glyphs`**: `sizes_snap_down_onto_the_ladder`

**`hud::layout`**: `corners_lie_on_the_surface_with_world_up`, `dragging_and_scaling`, `rays_pick_the_nearest_surface`, `saved_layouts_are_valid_rules`

**`input`**: `a_shift_chord_does_not_also_trigger_the_plain_held_key`, `bindings_file_overrides_and_round_trips`, `chase_camera_keys`, `chord_parsing_and_dead_zone`, `default_keys_follow_the_original_layout`, `every_action_is_reachable_from_keyboard_and_flight_actions_from_the_pad`, `gamepad_sticks_buttons_and_presses`, `mouse_flight_only_when_enabled`, `one_shot_commands_and_shift_chords`, `pad_modifier_layer_reaches_every_power_control`, `power_and_equipment_keys_follow_the_original`, `stick_layouts_and_mouse_inversion`, `view_names_parse`

**`input::editor`**: `capturing_a_key_rebinds_and_steals_it_from_others`, `capturing_pad_plain_presses_and_chords`, `clear_reset_and_settings`, `pretty_names`, `rows_are_grouped_and_headers_are_skipped`

**`input::gamepad`**: `axes_are_normalised`, `buttons_round_trip_names_and_bits`

**`mission`**: `arrivals_waves_chatter_and_a_win`, `lose_beats_win_and_the_timer_ends_it`, `roles_are_placed_counted_and_faced_at_the_player`

**`model`**: `empty_obj_has_zero_bounds`, `faces_record_their_corners_and_group`, `faces_that_only_touch_or_face_away_are_not_layered`, `later_overlapping_face_is_layered_on_top`, `lod_selection_by_distance`, `mirrored_triangles_keep_outward_winding`, `obj_models_use_materials_scale_and_flip_v`, `overlay_flag_beats_file_order`, `xmsh_is_triangulated_grouped_and_mirrored`

**`render`**: `billboards_and_streaks_build_two_triangles`, `colour_helpers_clamp`, `layer_bias_reaches_the_shader`

**`render::camera`**: `model_matrices_are_camera_relative`, `near_plane_changes_only_depth`, `points_ahead_project_to_the_centre_and_behind_is_none`, `rotation_turns_the_view`

**`render::effects`**: `events_create_effects_that_expire`, `explosion_curve_grows_and_fades`, `jitter_stays_in_range`

**`render::gpu`**: `engines_use_rear_attachments_or_the_back_centre`, `groups_become_chunks_with_local_indices`, `huge_groups_are_split`

**`render::starfield`**: `same_seed_same_sky_and_unit_directions`

**`script`**: `errors_are_found_when_loading`, `evaluates_with_precedence_and_short_circuits`, `units_and_tokens`

**`script::action`**: `actions_parse_and_check`

**`script::battle`**: `ids_match_and_roles_are_checked_names`

**`script::object`**: `ids_match_the_table_and_system_names_match_the_kinds`

**`settings`**: `parses_and_round_trips`, `resolution_order_prefers_explicit_and_existing`

**`sim`**: `ai_pilots_switch_cloak_ecm_and_power_by_condition`, `generational_handles_detect_reuse`, `match_speed_copies_the_targets_speed`, `preferences_pull_the_ai_past_a_nearer_target`, `probes_extend_radar_and_jam_their_enemies`, `protected_objectives_are_not_picked_as_the_nearest_enemy`, `random_is_deterministic_and_in_range`, `scenario_errors_are_reported`, `scenario_spawns_player_objects_and_leaders`, `ship_axes_follow_the_engine_convention`, `station_modules_mount_follow_and_fall_with_their_hub`, `step_advances_time_and_clears_edge_controls`, `unconsumed_presses_carry_over_held_inputs_do_not`

**`sim::ai`**: `aim_skill_decides_accuracy`, `disabled_ships_drift`, `enemy_hunts_and_damages_a_hostile`, `evasive_ends_and_retreat_jumps_out`, `lead_point_anticipates_motion`, `mines_go_off_when_an_enemy_comes_near`, `orders_switch_wingmen_to_attack`, `steering_turns_towards_points`, `target_drones_circle_and_idle_ships_sit`, `transitions_switch_states_on_conditions`, `tugs_tow_disabled_enemies_away`, `turrets_fire_off_the_nose`, `wingman_closes_on_its_slot`

**`sim::collision`**: `segment_sphere_hits_misses_and_inside`, `segment_triangle_hits_inside_only`, `shapes_use_mesh_in_model_space`

**`sim::combat`**: `a_new_bolt_streak_never_reaches_behind_the_muzzle`, `beams_fire_cost_energy_respect_cooldown_and_hit`, `beams_need_energy`, `damage_layers_and_aft_shield`, `empty_racks_report_and_cycle`, `fighters_bounce_off_each_other_and_stations`, `link_modes_cycle_and_change_how_many_fire`, `lock_builds_in_the_cone_and_missile_homes_in`, `lock_is_lost_outside_the_cone_and_dumbfire_works`, `point_defence_shoots_down_hostile_missiles`, `proximity_fuse_fires_on_a_near_miss`, `rotate_towards_limits_the_turn`

**`sim::flight`**: `afterburner_raises_the_cap_drains_energy_and_excess_bleeds_off`, `assisted_model_holds_throttle_speed_and_damps_drift`, `brake_stops_against_the_direction_of_travel`, `energy_and_shields_recharge_up_to_capacity`, `half_throttle_is_half_the_acceleration`, `helpers`, `newtonian_accelerates_to_the_cap_and_coasts`, `pitching_up_raises_the_nose_and_rates_fall_with_speed`, `reverse_thrust_slows_and_then_backs_up`, `speed_hold_matches_speed_and_cancels_drift`, `static_objects_do_not_move`, `the_arena_edge_halts_outward_motion`, `turning_does_not_steer_the_velocity`, `vct_thrusts_sideways`, `yaw_right_and_roll_right_turn_the_expected_way`

**`sim::guidance`**: `cloaked_wait_sleeps_until_prey_comes_near`, `cluster_splits_into_submunitions`, `damage_modes`, `doppelganger_launches_a_decoy_ship_that_expires`, `homing_hits_and_dumbfire_misses_an_offset_target`, `parallax_decoys_lure_enemy_missiles`, `sidestep_still_arrives`

**`sim::systems`**: `balance_caps_and_message`, `cloaked_ships_vanish_from_enemy_sensors_beyond_range`, `cycling_wraps_to_a`, `damage_knocks_systems_out_and_repairs_bring_them_back`, `disabled_when_engines_and_manoeuvring_are_out`, `ecm_makes_incoming_missiles_lose_their_target`, `hyperspace_counts_down_aborts_and_departs`, `raising_takes_power_from_the_other_setting`, `requests_change_power_and_balance`, `shield_balance_moves_points_forward_and_recharge_follows_power`, `systems_can_be_destroyed_beyond_repair`, `tractor_holds_a_disabled_target_and_captures_it_on_jump`, `tractor_needs_power_a_fitting_and_a_target_in_range`

**`sim::targeting`**: `cycles_forwards_and_backwards_with_wraparound`, `in_sights_picks_the_most_central`, `nearest_hostile_ignores_friends`, `nothing_to_target`

**`soak`**: `default_campaign_difficulty_scales_enemy_lethality`

## Running

```sh
cargo test                     # all
cargo test sim::               # one module
cargo test proximity           # tests whose name contains "proximity"
cargo test -- --nocapture      # show println! output
```
