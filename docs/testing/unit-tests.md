# Unit tests

Every module's tests, as of main `cf374a7`: **146 tests**, all passing (`cargo test`). Tests
live next to the code in `#[cfg(test)] mod tests` blocks and use **synthetic data built in the
test** (fake DLLs, meshes, WAVs, temp pack folders) — never original game files.

`platform::linux` has a test too (joystick event mapping); it is compiled and run only on Linux,
so it is not in the count from macOS.

Not unit-tested, by design, and covered by [manual tests](manual-tests.md) instead: the
`hud::draw` painting, the `Renderer`'s draw calls and shaders, macroquad device reading, sound
playback and the macOS gamepad shim — they need a window, a GPU, speakers or a controller.

## Summary

| Module | Tests | Covers |
|---|---|---|
| `formats` | 4 | Little-endian reads, bounds errors instead of panics (incl. offset overflow), NUL-terminated names, error messages |
| `formats::pe` | 5 | Resource lookup by type and name, quoted/upper-cased names, non-PE and truncated files, missing file |
| `formats::xmsh` | 6 | Hand-built mesh: header, LOD, signed edge loops, untextured polygons, bad signature/edge/vertex/degenerate polygon, no LOD, truncation |
| `formats::xtxr` | 3 | RGB555 expansion, palette-0 decoding, bad size/signature/truncation |
| `formats::tga` | 4 | 16-bit colour key → transparent black and row flip, 24/32-bit BGR, RLE, unsupported types, truncation |
| `formats::wav` | 6 | PCM16 round trip, PCM8 recentring, MS ADPCM block, IMA ADPCM block, step clamping, bad files |
| `formats::ini` | 5 | Sections/keys/comments/duplicates, warnings for ignored lines, bad numbers, merge layering, Latin-1 |
| `formats::obj` | 3 | Positions/uvs/normals/faces/materials/negative indices, bad indices and short faces, MTL parsing |
| `model` | 5 | XMSH → engine model (mirroring, triangulation, groups, colours, bounds), outward winding after mirroring, LOD by distance, OBJ conversion (scale, v flip, materials, face normals), empty OBJ |
| `content` | 4 | Loading packs/rules/scenarios/models from temp folders, skipping packs that need the CD, scenario replacement, PNG/TGA decoding, model scaling, empty pack dirs |
| `content::pack` | 5 | Manifest parsing and validation, dependency ordering, missing/duplicate/cyclic dependencies, discovery |
| `content::rules` | 3 | Typed definitions with original-section inheritance, variants, defaults, sounds, hostility; later-pack override and delete; every error reported |
| `content::scenario` | 3 | Objects and nav points, `player = choose`, error messages naming the section |
| `content::vfs` | 3 | Asset reference parsing and `..` normalisation, refusing escaping paths, original data found by name in any layout, digit-named resources |
| `sim` | 8 | Generational handles, deterministic RNG, scenario spawning and leaders, scenario errors, axis convention, edge-control clearing, unconsumed-press carry-over, match speed |
| `sim::flight` | 9 | Helpers (turn factor, damping, approach), max speed along the nose, throttle steps/reverse, pitch/yaw/roll directions and speed falloff, afterburner, VCT and drift damping, recharge, static objects |
| `sim::combat` | 10 | Beams (energy, cooldown, hits on the aft shield), link modes, no-energy, lock and homing missile kill, lock loss and dumbfire expiry, proximity fuse on a near miss, empty racks, damage layers, rotate_towards, collisions with ships and a station mesh |
| `sim::ai` | 6 | Steering, lead point, wingman closing on its slot, enemy hunting and damaging, wing orders, target and idle behaviours |
| `sim::targeting` | 4 | Cycling with wrap-around, nearest hostile, most-central-in-sights, nothing to target |
| `sim::collision` | 3 | Segment–sphere, segment–triangle, shapes with meshes in model space and the broad phase |
| `input` | 8 | Original default layout, one-shot commands and Shift chords, gamepad sticks/buttons/presses and dead zone, mouse flight direction, chase keys, bindings.ini override and round trip, view names, chord parsing |
| `input::gamepad` | 2 | Button names and bits round trip, axis normalisation |
| `render` | 2 | Billboard and streak geometry, colour helpers |
| `render::camera` | 3 | Projection to screen and behind-camera, rotation, camera-relative model matrices |
| `render::effects` | 3 | Events create sparks/explosions/debris that age and expire, explosion curve, jitter range |
| `render::gpu` | 2 | Per-texture chunks with local u16 indices, splitting huge groups |
| `render::starfield` | 1 | Same seed same sky, unit directions, uniform hemispheres |
| `hud` | 5 | Layout scaling, radar placement (front/rear, mirroring), gauges, MFD pages, message expiry |
| `audio` | 2 | Distance attenuation, event → sound and volume choices |
| `app::devices` | 2 | Key names match the bindings vocabulary |
| `app::flight` | 4 | Camera pose for every view, chase orbit, targeting and wing-order commands, pilot messages from events |
| `app::launcher` | 3 | Menu entries and wrapping, ship choice only when choosing, text wrapping |
| `app::viewer` | 4 | Orbit pose and clamping, entries for every class, wheel steps |
| `check` | 2 | The sample pack passes; broken definitions become errors |
| `cli` | 4 | Every option, environment variable precedence, unknown/incomplete options, packs folder discovery |

## Test names

**`formats`**: `errors_have_readable_messages`, `names_stop_at_the_first_nul`, `out_of_bounds_is_an_error_not_a_panic`, `reads_little_endian_numbers`

**`formats::pe`**: `finds_resources_by_type_and_name`, `missing_file_is_an_error`, `name_ids_are_normalised`, `rejects_non_pe_data`, `truncated_files_are_errors`

**`formats::xmsh`**: `mesh_without_lods_is_invalid`, `parses_a_hand_built_mesh`, `rejects_bad_signature_and_bad_references`, `signed_edges_give_the_vertex_loop`, `truncated_mesh_is_an_error`, `untextured_polygons_have_no_texture`

**`formats::xtxr`**: `decodes_pixels_through_palette_zero`, `expands_rgb555_to_full_range`, `rejects_bad_data`

**`formats::tga`**: `decodes_16_bit_with_colour_key_and_flips_rows`, `decodes_24_and_32_bit_bgr`, `decodes_rle`, `rejects_unsupported_and_truncated`

**`formats::wav`**: `adpcm_step_clamps_and_keeps_delta_minimum`, `ima_adpcm_block_decodes`, `ms_adpcm_block_decodes`, `pcm16_round_trips_through_to_wav_bytes`, `pcm8_is_recentred`, `rejects_bad_files`

**`formats::ini`**: `bad_numbers_and_bools_are_none`, `latin1_bytes_are_accepted`, `merge_overrides_keys_and_adds_sections`, `parses_sections_keys_and_comments`, `reports_lines_it_ignores`

**`formats::obj`**: `parses_materials`, `parses_positions_uvs_normals_and_faces`, `rejects_bad_indices_and_short_faces`

**`model`**: `empty_obj_has_zero_bounds`, `lod_selection_by_distance`, `mirrored_triangles_keep_outward_winding`, `obj_models_use_materials_scale_and_flip_v`, `xmsh_is_triangulated_grouped_and_mirrored`

**`content`**: `empty_pack_dirs_are_an_error`, `loads_original_free_packs_rules_scenarios_and_models`, `png_and_tga_pack_images_decode`, `scaling_scales_everything`

**`content::pack`**: `detects_missing_duplicate_and_cyclic_dependencies`, `discovers_pack_folders`, `orders_dependencies_first_and_keeps_order_otherwise`, `parses_a_manifest`, `rejects_bad_manifests`

**`content::rules`**: `builds_typed_definitions_with_inheritance`, `later_packs_override_and_delete`, `reports_every_error_and_keeps_going`

**`content::scenario`**: `choose_means_the_launcher_picks`, `errors_name_the_section`, `parses_objects_and_nav_points`

**`content::vfs`**: `original_data_is_found_by_name_in_any_layout`, `parses_asset_references`, `refuses_paths_that_escape_the_pack`

**`sim`**: `generational_handles_detect_reuse`, `match_speed_copies_the_targets_speed`, `random_is_deterministic_and_in_range`, `scenario_errors_are_reported`, `scenario_spawns_player_objects_and_leaders`, `ship_axes_follow_the_engine_convention`, `step_advances_time_and_clears_edge_controls`, `unconsumed_presses_carry_over_held_inputs_do_not`

**`sim::flight`**: `afterburner_boosts_and_drains_energy`, `energy_and_shields_recharge_up_to_capacity`, `full_throttle_reaches_max_speed_along_the_nose`, `helpers`, `pitching_up_raises_the_nose_and_rates_fall_with_speed`, `static_objects_do_not_move`, `throttle_steps_clamp_and_reverse`, `vct_slides_and_drift_is_damped`, `yaw_right_and_roll_right_turn_the_expected_way`

**`sim::combat`**: `beams_fire_cost_energy_respect_cooldown_and_hit`, `beams_need_energy`, `damage_layers_and_aft_shield`, `empty_racks_report_and_cycle`, `fighters_bounce_off_each_other_and_stations`, `link_modes_cycle_and_change_how_many_fire`, `lock_builds_in_the_cone_and_missile_homes_in`, `lock_is_lost_outside_the_cone_and_dumbfire_works`, `proximity_fuse_fires_on_a_near_miss`, `rotate_towards_limits_the_turn`

**`sim::ai`**: `enemy_hunts_and_damages_a_hostile`, `lead_point_anticipates_motion`, `orders_switch_wingmen_to_attack`, `steering_turns_towards_points`, `target_drones_circle_and_idle_ships_sit`, `wingman_closes_on_its_slot`

**`sim::targeting`**: `cycles_forwards_and_backwards_with_wraparound`, `in_sights_picks_the_most_central`, `nearest_hostile_ignores_friends`, `nothing_to_target`

**`sim::collision`**: `segment_sphere_hits_misses_and_inside`, `segment_triangle_hits_inside_only`, `shapes_use_mesh_in_model_space`

**`input`**: `bindings_file_overrides_and_round_trips`, `chase_camera_keys`, `chord_parsing_and_dead_zone`, `default_keys_follow_the_original_layout`, `gamepad_sticks_buttons_and_presses`, `mouse_flight_only_when_enabled`, `one_shot_commands_and_shift_chords`, `view_names_parse`

**`input::gamepad`**: `axes_are_normalised`, `buttons_round_trip_names_and_bits`

**`render`**: `billboards_and_streaks_build_two_triangles`, `colour_helpers_clamp`

**`render::camera`**: `model_matrices_are_camera_relative`, `points_ahead_project_to_the_centre_and_behind_is_none`, `rotation_turns_the_view`

**`render::effects`**: `events_create_effects_that_expire`, `explosion_curve_grows_and_fades`, `jitter_stays_in_range`

**`render::gpu`**: `groups_become_chunks_with_local_indices`, `huge_groups_are_split`

**`render::starfield`**: `same_seed_same_sky_and_unit_directions`

**`hud`**: `gauges_sweep_and_read_the_ship`, `layout_scales_by_height_and_centres`, `messages_expire_and_keep_the_newest`, `mfd_pages_cycle_and_describe`, `radar_places_contacts_by_angle`

**`audio`**: `attenuation_falls_with_distance`, `events_choose_sounds_and_volumes`

**`app::devices`**: `key_names_match_the_bindings_vocabulary`, `known_names_are_real_keycodes`

**`app::flight`**: `chase_orbit_responds_and_resets`, `commands_target_and_order_wingmen`, `events_become_pilot_messages`, `views_place_the_camera`

**`app::launcher`**: `menu_lists_viewer_and_scenarios_and_wraps`, `scenarios_start_with_the_chosen_ship_only_when_choosing`, `wrapping_text`

**`app::viewer`**: `a_still_wheel_does_not_zoom`, `orbit_input_clamps`, `orbit_looks_at_the_origin_from_its_distance`, `sample_entries_list_every_class`

**`check`**: `broken_definitions_are_errors`, `the_sample_pack_passes`

**`cli`**: `environment_supplies_data_and_flags_override_it`, `errors_for_unknown_or_incomplete_options`, `finds_the_source_tree_packs`, `parses_every_option`

## Running

```sh
cargo test                     # all
cargo test sim::               # one module
cargo test proximity           # tests whose name contains "proximity"
cargo test -- --nocapture      # show println! output
```
