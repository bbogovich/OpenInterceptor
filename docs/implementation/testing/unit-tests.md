# Unit tests

Every module's tests, as of main `bf7b7ce` (build 0.4.2): **292 tests** on macOS (293 on
Linux), run with `cargo test` by the gate. Tests
live next to the code in `#[cfg(test)] mod tests` blocks and use **synthetic data built in the
test** (fake DLLs, meshes, WAVs, temp pack folders) — never original game files.

`platform::linux` has a test too (joystick event mapping); it is compiled and run only on Linux,
so it is not in the count from macOS.

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
| `content` | 5 | Loading packs/rules/scenarios/models from temp folders, skipping packs that need original data, scenario replacement, **per-asset overrides** of a mesh, image and sound and their removal when the pack is disabled, PNG/TGA decoding, model scaling, empty pack dirs |
| `content::pack` | 7 | Manifest parsing and validation, **kind, parent, targets and format** (and their errors), **the active set for a chosen campaign** (parent chain, targeting addons, dependencies), dependency ordering, missing/duplicate/cyclic dependencies, discovery |
| `content::rules` | 6 | Typed definitions with original-section inheritance (YAML), variants, defaults, sounds, **voice sets and their clips**, hostility, flight model and arena defaults, **`bolt_frame`, `bolt_carry` and `lighting` (unset keys keep defaults)**, cockpits; **later packs merge, delete and edit lists**; **difficulty variants and per-level values**; every error reported; **spelling of original keys** (`HitPoints` → `hit_points`, `Side` → `faction`, beam `Range` in km); **sound cues resolve through the original table** |
| `content::equipment` | 3 | System kinds round trip and built-ins exist, `systems` definitions overriding built-ins, `power` parsing and validation (level counts, start levels) |
| `content::hazards` | 1 | `hazards:` parses with defaults (count, radius, `sensors` as a fraction); a hazard without `class`, or a position without three numbers, is an error |
| `content::environment` | 2 | Environments and terrains parse with defaults; mistakes reported (unknown `physics`, an atmosphere without a terrain, both or neither of `generate` / `heightmap`, an unknown generator); generated heightfields span their `height` range, are the same for the same seed and interpolate between grid points |
| `content::looks` | 3 | Debris sets and effects parse with defaults; texture swaps accumulate with damage levels and state; texture keys ignore folders, case and extensions |
| `content::formation` | 2 | Every shape places followers behind and beside the leader in the leader's frame, scaled by `spacing`; custom slots then a vee behind them; `shape` with `slots`, or an unknown shape, is an error |
| `content::strategy` | 3 | Dates count days across months and leap years; a strategy layer builds from YAML with every name checked (a misspelt expression name, an unknown base type and an unknown unlock are errors); **the original's numeric tech ids become research prerequisites** through `tech_ids` (an unlisted id is left out; `tech_required` gates an item) |
| `script::campaign` | 1 | Query ids match the table; campaign ids and level names are constants; a misspelt name is an error |
| `campaign` | 11 | A new campaign has its start; research follows prerequisites and unlocks and stops at completion; modules, limits, bases (one per system), purchases, deliveries, manufacture and selling; monthly money flows and costs; raids detected, intercepted and fought; same seed same campaign; **the default campaign runs six months** (solvent, research, battles, a save round trip); fitting from the store and the loadout; lane routes (Dijkstra); **the testbed cheats** (payday, quickbase, knowitall, jump to battle); **manufacture paid per unit as it starts, and salvage recorded by raid id** |
| `campaign::logistics` | 2 | **Survey, build, mine, make and deliver on demand** (a mine refused before the survey, a station refused off a site; a hull built once its plating arrives; a reserve drawn down); **station research waits for its start materials, and convoys lose vessels** (every convoy ambushed at 50 % loss; orders follow until the start cost is met) |
| `campaign::save` | 1 | A save round-trips and the game continues identically; a save needing a missing pack, or with an index out of range, is refused |
| `formats::png` | 2 | A PNG (encoded by the `image` crate since `2ed1149`) has the signature and chunks and **decodes back to the same pixels**; downscaling averages blocks |
| `app::strategy` | 2 | The map fits its rectangle, centred; money has thousands separators |
| `content::mission` | 2 | A full mission parses (roles, class lists, counts, arrivals, waves, objectives, optional, timer, outcomes, chatter, debrief); mistakes are reported with where (unknown role, bad timer result, unknown action) |
| `content::text` | 1 | Keys resolve, plain values pass through, imports are listed |
| `content::schema` | 4 | Good files pass; **mistakes reported with line and key path** (unknown key with suggestion, wrong type, bad unit); quantities with units and did-you-mean suggestions; JSON Schema export well formed |
| `content::files` | 1 | Every file schema builds and exports |
| `content::behaviour` | 4 | States, transitions and skill parsed (units, per-state parameters); mistakes reported (no `do`, unknown manoeuvre or state, bad condition); the built-in attack behaviour is valid; **target preferences parse and reject unknown selectors** |
| `content::scenario` | 3 | Objects and nav points (incl. `destroyed_systems`, `hull`), `player: choose`, **errors name the line and path** |
| `content::vfs` | 4 | Asset reference parsing and `..` normalisation, refusing escaping paths, **override path naming**, original data found by name in any layout, digit-named resources, resource and file listing |
| `settings` | 2 | settings.yaml parse/save round trip incl. `packs.disabled`, `game.campaign`, `game.difficulty` **`graphics.brightness` (clamped)** and **`console.enabled` / `console.port` (an out-of-range port is ignored)**, and the legacy settings.ini reader; **original data found in order** (command line, environment, settings, user data folder, then the checkout's `resources/xcom` before `resources/xcom-cd`, an empty folder skipped; relative paths) |
| `sim` | 14 | Generational handles, **body velocity reads drift in the ship's own axes**, deterministic RNG, scenario spawning and leaders, scenario errors, axis convention, edge-control clearing, unconsumed-press carry-over, match speed sets a speed hold; **target preferences pull the AI past a nearer target**, **station modules mount, follow and fall with their hub**; **probes extend radar and jam their enemies**, **AI pilots switch cloak, ECM and power by condition**, **protected objectives are not picked as the nearest enemy** |
| `sim::flight` | 15 | Helpers (turn factor, damping, approach, approach_vec); **newtonian**: acceleration to the cap and coasting, half throttle, reverse thrust, brake stops exactly, turning keeps the velocity, speed hold and drift cancelling, afterburner cap and bleed-off, VCT thrust; pitch/yaw/roll directions and speed falloff; **assisted** model end to end; recharge; static objects; arena edge |
| `sim::combat` | 14 | Beams (energy, cooldown, hits on the aft shield), **bolt streak never trails behind the muzzle**, **a bolt leaves with the muzzle's velocity (ω × r) then flies straight**, **with `bolt_frame: ship` it turns with the nose while carried**, link modes, no-energy, lock and homing missile kill, lock loss and dumbfire expiry, proximity fuse on a near miss, empty racks, damage layers, rotate limits, collisions; **point defence shoots down hostile missiles** |
| `sim::ai` | 15 | Steering, lead point, wingman closing on its slot, enemy hunting and damaging, wing orders, evasive ending and retreat jumping out, **aim skill decides accuracy**, **transitions switch states on conditions**, **turrets fire off the nose**, **tugs tow disabled enemies away**, disabled ships drift, target and idle behaviours; **mines go off when an enemy comes near**; **predictive collision avoidance** (head-on: break and slow; passing wide or beyond the horizon: nothing); **the formation order cycles the campaign's formations** |
| `sim::systems` | 14 | **Power**: raising takes from the other setting, cycling wraps to A, balance caps and message, requests change power and balance, shield balance moves points and recharge follows power; **damage**: knocked out and repaired back, destroyed beyond repair, disabled when engines and manoeuvring are out; **hyperspace** countdown, abort and departure; **tractor** hold and capture on jump, and its refusals (power, fitting, range); **cloak** hides from enemy sensors beyond range; **ECM** makes missiles lose their target; **the console's sliders set power levels and balance outright**, within the budget |
| `sim::guidance` | 7 | Homing hits and dumbfire misses an offset target; sidestep still arrives; parallax decoys lure enemy missiles; cloaked wait sleeps until prey comes near; cluster splits into submunitions; doppelganger launches a decoy ship that expires; damage modes |
| `mission` | 5 | Roles are placed, counted and faced at the player; arrivals, waves, chatter and a win; lose beats win and the timer ends it; **the player's death is its own result (`killed`) with the lose outcomes**; **wingmen take successive slots of their formation** across roles |
| `sim::targeting` | 4 | Cycling with wrap-around, nearest hostile, most-central-in-sights, nothing to target |
| `sim::hazards` | 4 | Asteroid fields and minefields are scattered in their sphere, clear of craft, deterministically, asteroids as scenery; a black hole pulls harder when near (capped) and swallows what crosses its radius; AI pilots turn away from it; an ion storm shortens radar range |
| `sim::env` | 5 | Drag, gravity and lift act in an atmosphere and not in space (hover cancels gravity, lift carries the weight at the `lift` speed); AI pilots pull up near the ground; gentle touchdowns land, hard ones crash with damage, and the ceiling is an exit; ground installations are set down on the terrain; **craft spawned below 300 m over a ground are lifted to 300 m** |
| `script` | 3 | Units and tokens; evaluation with precedence and short-circuit; **errors found when loading** (unknown names, argument counts, types) |
| `script::object` | 1 | Query ids match the table; system names match the kinds; level names as constants |
| `script::battle` | 1 | Query ids match the table; role names are checked names |
| `script::action` | 1 | Actions parse and are checked (names, argument kinds, battle or campaign scope) |
| `sim::collision` | 3 | Segment–sphere, segment–triangle, shapes with meshes in model space and the broad phase |
| `input` | 14 | **The keys help lists every action with its current keys**, Original default layout incl. brake, **power and equipment keys**, one-shot commands and Shift chords, **a Shift chord does not trigger the plain held key**, gamepad sticks/buttons/presses and dead zone, **pad modifier layer reaches every power control**, **every action reachable from the keyboard and flight actions from the pad**, **stick layouts and mouse inversion**, mouse flight, chase keys, bindings.ini override and round trip, view names, chord parsing |
| `input::editor` | 5 | Rows grouped with headers skipped, capturing a key rebinds and takes it from other actions, capturing plain pad presses and chords, clear / reset / settings, pretty names |
| `input::gamepad` | 2 | Button names and bits round trip, axis normalisation |
| `render` | 4 | Billboard and streak geometry, colour helpers, overlay depth bias pasted into the shader; **uniforms shared by both lit shaders match exactly, with an explicit precision** (the `Light` link fix, 0.4.1) |
| `render::camera` | 4 | Projection to screen and behind-camera, rotation, near plane changes depth only, camera-relative model matrices |
| `render::effects` | 3 | Events create sparks/explosions/debris that age and expire, explosion curve, jitter range |
| `render::gpu` | 3 | Per-texture chunks with local u16 indices and the overlay flag, splitting huge groups, engine glow points |
| `render::starfield` | 1 | Same seed same sky, unit directions, uniform hemispheres |
| `render::terrain` | 2 | Terrain meshes face up (counter-clockwise from above) and are coloured from low to high by height (the X-mirrored mesh too); **every placed horizon tile matches the folded ground height** |
| `hud` | 7 | Layout fitting any design space (4:3 and widescreen), radar placement (front/rear, mirroring), gauges, MFD pages (incl. Systems), **power readout and system lamps**, message expiry; **the message log reports what is new since a count** (the console's events, kept-lines limit) |
| `hud::layout` | 4 | Anchor corners lie on the surface with world up; rays pick the nearest surface; 2D dragging and anchor scaling; saved layouts are valid rules |
| `hud::draw` | 4 | Lamp grids follow the panel shape; shield rings go green, amber, red, then blink; **drift reads signed whole metres per second**; **the pitch ladder sits ahead at its angles, the horizon level whatever the camera's pitch** |
| `hud::glyphs` | 1 | Text sizes snap down onto the glyph ladder |
| `audio` | 2 | Distance attenuation, event → sound and volume choices (incl. `system_damage`, switch sounds, `hyperspace`) |
| `app::assets` | 2 | Tab cycling; empty lists without original data |
| `app::devices` | 2 | Key names match the bindings vocabulary |
| `app::flight` | 8 | Camera pose for every view, **3D cockpit field of view widens only on narrow screens**, chase orbit, targeting and wing-order commands, pilot messages from events (incl. system, disabled, jump and capture messages), **a wingman answers in its own voice** (only the player's wingmen, only with the line, never the lost one); **a campaign loadout refits the player's class in this battle only**; **the debrief waits for the delay after the result** |
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
| `remote::protocol` | 6 | Console messages decode (incl. `power`), unknown actions and types are refused, throttle clamped; **the catalogue lists every action and marks missing equipment unavailable**; **telemetry reports drift in body axes and the target's rotation relative to the player**; **`model` messages are normalised to radius 1 with a colour per group** (average colour ignores transparent pixels); **`contacts` are in the player's axes** (a craft dead ahead after a 90° yaw), relation and target flag, and `target {id}` decodes (bad ids refused); `event`; **themes layer per cockpit and become CSS variables** (cockpit override over the base, `--font`, `--corner`, an empty theme sends none) |
| `remote::server` | 3 | **Only local-network peers are served** (IPv4 private, link-local, loopback; IPv6 loopback, unique-local, link-local, IPv4-mapped); pairing codes are six digits and vary with the seed; **end to end over a real localhost socket**: page served, 404, handshake accept key, unpaired action dropped, wrong code denied, pairing, action relayed, broadcast received; **`welcome` answered at once by the server** (0.4.7). Plus one ignored helper, `serve_for_a_browser` (not counted) |
| `soak` | 2 | The default campaign's difficulty scales enemy lethality (headless battle runs); **at cadet an undefended station holds for 150 s** (idle pilot) |

## Test names

Generated from the `#[test]` functions in `src/` at main `bf7b7ce` (read with grep, not a build).

**`app::assets`**: `no_original_data_means_empty_lists`, `tabs_cycle`

**`app::devices`**: `key_names_match_the_bindings_vocabulary`, `known_names_are_real_keycodes`

**`app::flight`**: `a_wingman_answers_in_its_own_voice`, `chase_orbit_responds_and_resets`, `cockpit_fov_widens_only_on_narrow_screens`, `commands_target_and_order_wingmen`, `events_become_pilot_messages`, `views_place_the_camera`

**`app::launcher`**: `menu_lists_viewer_and_scenarios_and_wraps`, `scenarios_start_with_the_chosen_ship_only_when_choosing`, `wrapping_text`

**`app::simulator`**: `a_setup_changes_environment_hazards_and_enemy_force_only`, `ships_that_cannot_enter_an_atmosphere_are_refused_there`

**`app::viewer`**: `a_still_wheel_does_not_zoom`, `default_entries_list_every_class`, `display_modes_cycle`, `entries_are_found_by_ship_id_or_mesh_name`, `face_info_describes_a_face`, `looks_cover_damage_states_and_debris`, `near_plane_moves_out_to_the_model`, `orbit_input_clamps`, `orbit_looks_at_the_origin_from_its_distance`

**`audio`**: `attenuation_falls_with_distance`, `events_choose_sounds_and_volumes`

**`check`**: `broken_definitions_are_errors`, `the_default_campaign_passes`

**`cli`**: `content_dirs_are_resources_then_packs`, `environment_supplies_data_and_flags_override_it`, `errors_for_unknown_or_incomplete_options`, `parses_every_option`

**`content`**: `empty_pack_dirs_are_an_error`, `loads_original_free_packs_rules_scenarios_and_models`, `pack_files_override_original_assets`, `png_and_tga_pack_images_decode`, `scaling_scales_everything`

**`content::behaviour`**: `parses_states_transitions_and_skill`, `reports_mistakes`, `target_preferences_parse_and_reject_unknown_selectors`, `the_builtin_attack_is_valid`

**`content::equipment`**: `kinds_round_trip_and_builtins_exist`, `power_section_parses_and_validates`, `system_definitions_override_builtins`

**`content::environment`**: `environments_and_terrains_parse_and_report_mistakes`, `generated_heightfields_span_their_range_are_deterministic_and_interpolate`

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

**`script::action`**: `actions_parse_and_check`

**`script::battle`**: `ids_match_and_roles_are_checked_names`

**`script::object`**: `ids_match_the_table_and_system_names_match_the_kinds`

**`settings`**: `original_data_is_found_in_order`, `parses_and_round_trips`

**`sim`**: `ai_pilots_switch_cloak_ecm_and_power_by_condition`, `body_velocity_reads_drift_in_the_ships_own_axes`, `generational_handles_detect_reuse`, `match_speed_copies_the_targets_speed`, `preferences_pull_the_ai_past_a_nearer_target`, `probes_extend_radar_and_jam_their_enemies`, `protected_objectives_are_not_picked_as_the_nearest_enemy`, `random_is_deterministic_and_in_range`, `scenario_errors_are_reported`, `scenario_spawns_player_objects_and_leaders`, `ship_axes_follow_the_engine_convention`, `station_modules_mount_follow_and_fall_with_their_hub`, `step_advances_time_and_clears_edge_controls`, `unconsumed_presses_carry_over_held_inputs_do_not`

**`sim::ai`**: `aim_skill_decides_accuracy`, `disabled_ships_drift`, `enemy_hunts_and_damages_a_hostile`, `evasive_ends_and_retreat_jumps_out`, `lead_point_anticipates_motion`, `mines_go_off_when_an_enemy_comes_near`, `orders_switch_wingmen_to_attack`, `steering_turns_towards_points`, `target_drones_circle_and_idle_ships_sit`, `transitions_switch_states_on_conditions`, `tugs_tow_disabled_enemies_away`, `turrets_fire_off_the_nose`, `wingman_closes_on_its_slot`

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
