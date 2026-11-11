# Testbed

`src/app/`: the interactive program — a small state machine of screens sharing one
`AppContext`. Every stage adds scenarios here so its features can be play-tested. Plan:
Stage 1; Stage 2 added the controls menu, the campaign switch, the wingman comms menu, system
and hyperspace messages, and `--screenshot`; Stage 3 the missions, the debrief, radio voices
(app 0.6.2) and the **simulator** (app 0.6.3).

## Architecture Overview

```mermaid
stateDiagram-v2
    [*] --> Launcher
    [*] --> FlightSession: --scenario id
    [*] --> ModelViewer: --model name
    Launcher --> FlightSession: Enter on a scenario (Next::Fly)
    Launcher --> FlightSession: Enter on a mission (Next::Mission)
    [*] --> FlightSession: --mission id
    Launcher --> ModelViewer: Enter on Model viewer (Next::Viewer)
    ModelViewer --> Launcher: Esc
    Launcher --> AssetBrowser: Enter on Asset browser (Next::Assets)
    AssetBrowser --> Launcher: Esc
    Launcher --> Controls: Enter on Controls (Next::Controls)
    Controls --> Launcher: Esc (save bindings.yaml)
    Launcher --> Simulator: Enter on Simulator (Next::Simulator)
    Simulator --> FlightSession: Enter on Fly (Next::Simulate, a modified mission)
    Simulator --> Launcher: Esc
    FlightSession --> FlightSession: Enter after a simulator debrief (Next::Simulate, same setup)
    Launcher --> Launcher: Tab or D (Next::Reload) reloads Content for another campaign or difficulty
    FlightSession --> Launcher: Esc menu → L
    FlightSession --> FlightSession: Esc menu → K (controls menu inside the session)
    FlightSession --> FlightSession: Esc menu → R, or Enter after death
    Launcher --> [*]: Esc
```

- `app::run(options)` loads `Content`, creates the `Renderer`, loads sounds (async), reads the
  bindings, then loops: the current screen's `frame` returns a `Next`, then
  `next_frame().await`. A start-up failure shows an error screen instead of crashing.
- **`--screenshot <png>`** (with `--scenario` or `--model`) runs the starting screen for
  `--frames` frames (default 120), saves what was drawn as an opaque PNG and exits — a way to
  check a cockpit or model on several window sizes without playing.
- **Launcher** (`launcher.rs`): the header names the loaded campaign and difficulty level;
  **Tab** (pad **Y**) switches to the next campaign (`launcher::next_campaign`) and **D** (pad
  **X**) to the next difficulty level (`launcher::next_difficulty`), each returning
  `Next::Reload { campaign, difficulty }`: the app reloads `Content` (the ruleset is rebuilt at
  that level), saves the choice in `settings.yaml` (`game.campaign`, `game.difficulty`), and shows any
  load error on the launcher. Entries: the model viewer, the asset browser, **Controls**, the **Simulator** (app 0.6.3), every scenario, then every **mission** (*Mission: name*, its briefing as the description; [Missions](mission.md)). The list scrolls to keep the selection near the middle (`Menu::first_visible`, "^ more" / "v more"). Scenarios are
  sorted by `order`; the first scenario is selected at start, found by searching the entries
  (`Entry::Scenario`) rather than by a fixed index, so adding an entry above the scenarios does
  not move the start. For
  `player: choose` scenarios, Left/Right picks among playable ship classes, previewed as a
  turning model with name, description and stats. Keyboard, mouse (click to select, click
  again to start) and gamepad (D-pad, A/Start, Back to quit). Skipped-pack warnings and start
  errors are shown.
- **Model viewer** (`viewer.rs`): every ship class's model, then every original mesh. Orbit
  with arrows or mouse drag, zoom with +/− or the wheel, PgUp/PgDn (or B/N) to change model,
  `L` to cycle LOD (auto, 0–3), Space to stop spinning, Esc back. The camera is fitted to each
  model's radius and never enters it; `viewer::wheel_steps` turns wheel movement into zoom steps.
  The near plane follows the orbit (`viewer::near_plane`: just in front of the model, at least
  1% of the distance, never under 5 cm), so depth precision suits a 600 m station as well as a
  2 m debris chunk. Mesh-debugging keys: **W** cycles `Display::Solid` / `SolidWire` / `Wire`
  (each original face's outline as 3D lines, pulled 0.3% towards the camera over a solid
  model, plus a dot on every vertex of faces towards the camera); **[ / ]** select a face
  (Shift ×10), outlined and numbered, with `viewer::face_info` on the info line (index, corners,
  texture, flags, overlay layer, UVs); **I** numbers every face towards the camera; **O** toggles
  `Renderer::show_overlays` (restored on Esc). **D** (class entries) cycles the class's looks
  (`viewer::looks_of`: intact, each damage level, each named state, and the debris pieces pulled
  apart from the centre), named on the info line (`look_label`). `--model NAME` opens the viewer on a model
  (`viewer::find_entry`: ship id or mesh name, whole names, case-insensitive).
- **Asset browser** (`assets.rs`): every original texture, image (cockpits, HUD, menus,
  sprites) and sound, loaded exactly as the game loads them — so a pack override shows its
  replacement, and the entry is marked `[overridden by <pack>]`. Tab switches between
  Textures, Images and Sounds; Up/Down (Shift: ×10) or B/N move; Enter or Space plays a sound;
  Esc back. Images are drawn over a checkerboard so transparency is visible. Its `frame` is
  `async` because loading a sound for playback is (macroquad's API); the app loop `.await`s it.
- **Controls** (`controls.rs`): the controls menu component (`ControlsMenu`), used from the
  launcher and from the flight pause menu; see [Input](input.md#the-controls-menu).
- **Missions in a flight session**: `FlightSession::start_mission` builds the battle with
  `MissionRun::start`; after each tick `MissionRun::update` referees it and its lines go to the
  message display; when it ends, the battle freezes behind the **debrief** (result, text, kills,
  losses, time, optional objectives, campaign actions; long text wrapped). Enter (pad A) flies it
  again, Esc or L returns to the launcher; the pause menu's R also restarts the mission.
- **Radio voices** (app 0.6.2): the flight screen decides when a pilot speaks — wing orders
  and status requests (`acknowledge`, `status`), a wingman lost (`down`, said by another),
  chatter clips (`MissionRun::voices`) and the computer voice at the debrief
  (`mission_accomplished` / `mission_failed`). See [Audio](audio.md).
- **Flight session** (`flight.rs`): see the frame loop on the
  [overview](index.md#logic-flow-one-frame-of-a-flight-session). Views F1–F8; after death the
  camera orbits the wreck; Esc opens a menu (Esc resume, R restart, K controls, L launcher; it
  points to `?` for the keys instead of listing them); P pauses; `?` (or Shift+F1) opens the
  **keys overlay** (`draw_help`, `help_open`), which pauses the battle and lists every action by
  group with its current keys, in as many columns as the screen needs. The player's cockpit may have 2D art, a 3D model, or both; the HUD uses the cockpit's
  design space in the cockpit view and the default glass layout in the other views. A 3D
  cockpit on a window narrower than its design widens the field of view (`cockpit_fov`), and
  its anchored instruments are drawn on the dashboard in perspective (`draw_anchored`).
  Stage 2 additions:
    - **Cockpit layout editor** (F12 in the cockpit view; the battle pauses): `LayoutEditor`
      holds the selected instrument, the drag in progress and a status message; `edit_layout`
      handles Tab, 2D drags and nudges, 3D ray picks on the cockpit model's top LOD, resizing
      by wheel or `+`/`-`, and **S**, which `save_layout` writes as an addon
      `<user data>/mods/<campaign>-cockpit-<id>/` (`pack.yaml` + `rules/cockpit-layout.yaml`).
      See [Cockpits](../../modding/cockpits.md#the-cockpit-layout-editor-f12).
    - **Wingman comms menu** (`]`, pad LB + left stick click): the six `WING_MENU` entries
      (fire at will, attack my target, ignore my target, evasive, retreat, status report),
      chosen with digits 1–6 or the d-pad and A; B or `]` closes it. While open, the d-pad and A
      belong to the menu, not to throttle, MFD and afterburner. A status report puts one line
      per wingman on the message line.
    - **Messages** (`event_messages`): system damage and repair ("… DAMAGED: HALF EFFICIENCY",
      "… INOPERATIVE: REPAIRS UNDERWAY", "… BACK ON-LINE", "… REPAIRED", "… DESTROYED"),
      "ENGINES AND THRUSTERS OUT: SHIP DISABLED", "TARGET DISABLED", "… JUMPED OUT",
      "… CAPTURED", "HYPERJUMP COMPLETE". `L` repeats the last message.
    - **Jumped-out end screen**: "JUMPED TO HYPERSPACE", with the captured prizes listed;
      Enter flies again, Esc returns to the menu.

    Stage 3 interlude additions:

    - **Tablet console** (C1, app 0.4.0): when the console is on (`--console`,
      `console.enabled`), `app::start_console` starts a `remote::server::ConsoleServer` into
      `AppContext::console` (a taken port is reported and the game carries on without it).
      Each frame `FlightSession::console_frame` merges the console's input into `RawInput`,
      sends `welcome` and the `catalogue` (again when the player's craft class changes) and
      20 Hz `telemetry`. Until a console pairs, and while the menu is open, the bottom-left
      corner shows `CONSOLE <url>   CODE <code>`. See [Remote console](remote.md).

### The simulator (app 0.6.3)

`app::simulator` (Stage 3.9, R-MSN-010): a setup screen in front of any mission of the loaded
campaign. Rows, chosen with Up/Down and changed with Left/Right; Enter flies; Esc returns to the
launcher:

| Row | Values | Effect on the copy (`simulator::apply`) |
|---|---|---|
| Mission | every mission, in launcher order | The mission to copy. Its briefing is shown below the rows. |
| Ship | the playable ships | The player's craft; "(the mission's own)" when the mission names `player.class`. |
| Environment | the mission's own (or open space), then every environment | `environment` replaced. |
| Asteroids | off / on | 40 objects of the campaign's asteroid class in a 4.5 km sphere 4.5 km ahead, spin 6 °/s. |
| Minefield | off / on | 20 mines of the campaign's mine class in a 2.5 km sphere 4.5 km ahead. |
| Black hole | off / on | A black hole 9 km ahead and 800 m below, `pull` 12, `radius` 450 m. |
| Ion storm | off / on | Radar at 40 %. |
| Enemy force | ×0.5, ×1, ×1.5, ×2 | Every enemy role's count range and every wave of an enemy role scaled and rounded, never below 1. Friendly and objective roles are unchanged. |
| Fly | — | Starts the copy. |

- **Hazard classes** come from the campaign itself (`simulator::hazard_classes`): the asteroid
  class is the first object class whose id contains `asteroid`, the mine class the first of
  `kind: mine`. A campaign without one shows "(none in this campaign)" on that row and the
  toggle does nothing — the simulator never assumes a class a mod may not have.
- **Refusal**: `simulator::ship_allowed` refuses a ship with `atmospheric: false` in an
  environment with `physics: atmosphere`; the reason is shown under the rows and nothing starts.
- **Flying**: Fly returns `Next::Simulate { mission, ship }` with the modified `MissionDef`
  (boxed: it is large); the app starts it with `FlightSession::start_custom`, which keeps the
  copy in `FlightSession::custom` so that "again" from the debrief (or R in the pause menu)
  returns `Next::Simulate` with the same setup instead of the original mission.
- The difficulty level is the launcher's (D); the mission's own values per level apply before
  the force multiplier.
- With the defaults (own environment, no hazards, ×1) the copy equals the mission (tested).
- `SimSetup` is plain data and `apply` and `ship_allowed` are pure, so the logic is unit-tested
  without a window; the drawing is a manual test (MT-3-51 – MT-3-54).

### Scenarios shipped

| Id | Pack | What it tests |
|---|---|---|
| `flight-test` | interceptor | **Stage 1 exit scenario**: player (any hull), X-COM base, two Lightning wingmen, one of every alien and pirate type flying practice circles; nav points `base`, `aliens`, `pirates` (N cycles). |
| `combat-test` | interceptor | The same arena (and nav points) with every alien and pirate hostile. |
| `gunnery-range` | interceptor | Six drones circling ahead; beams, link modes and missile locks. |
| `station-tour` | interceptor | Every station type in a row, a Patton cruiser and a freighter. |
| `systems-trial` | interceptor | Stage 2: power controls, a disabled Wraith (`destroyed_systems`) to tow with the tractor beam and capture by jumping out, a cloaking Shade, a Harbinger that tries to tow you; wingman orders. |
| `rocket-vs-saucer` | default | No original data: choice of Guard rocket flown from the **3D cockpit** `guard`, two wingmen, the Guard station, a flight of Dominion saucers. The basic dogfight. |
| `dominion-raid` | default | A Dreadnought disc, raider saucers and robot sentinels attack the Guard station; three wingmen. |
| `black-comet-ambush` | default | Escort a Prospector hauler past a cosmium asteroid while Black Comet hot-rods attack; neutral Senate and Cartel ships nearby. |
| `systems-trial` | default | The default campaign's systems trial: a crippled raider saucer to tow and capture (Meteor or Thunderbolt), power controls, the Thunderbolt's cloak ("invisibility screen"), a Grappler that tries to drag the prize off. |
| `duel-<enemy>` | both | You against one enemy type (default: scout saucer, raider saucer, dreadnought, robot sentinel, grappler, hot-rod; interceptor: wraith, spectre, phantasm, shade, harbinger, drone, pirate Lightning, Firestar and Avenger) — how it flies and fights at each difficulty. |
| `mixed-dogfight` | both | Several factions at once, everyone hostile to someone. |
| `escort` | interceptor | Escort an X-COM freighter against Wraiths and a Spectre. |
| `debris-showcase` | both | Every class at 2 % hull in a row: damage textures at once, then one shot each shows explosion, shock wave and debris. Nothing shoots back. |
| `base-assault` | interceptor | An alien base with plasma gun pods and shield pods; two wingmen. |
| `outpost-strike` | default | A Dominion outpost with death-ray pods and a shield pod: shields drop with the pod, the pods die with the hub. |
| `probe-net` | both | A friendly sensor probe 7 km ahead shows enemies on your radar early; a hostile jamming probe on the way halves your radar range and slows locks until you shoot it down. |
| `construction-defence` | default | A research station at 60 % (`state: build_60`, protected, no guns yet) and a manufacturing frame at 25 %; Dominion raiders try to wreck them. |
| `convoy-ambush` | default | Escort two cargo clippers (protected) from the mining facility while Black Comet hot-rods and a saucer pounce; a lost clipper loses its cargo. |
| `hazard-asteroids` | both | Stage 3.2 hazards. Default: 45 cosmium asteroids in a 4 km sphere with two scout saucers hiding among them. Interceptor: 50 asteroids in an **ion storm** (radar at 40 %) with two Wraiths. Asteroids show on radar only within range. |
| `hazard-black-hole` | both | A dogfight at the edge of a black hole (`pull: 12`, `radius: 450`): the pull grows as you near it, the core swallows what crosses it, the AI turns away. Default: two raider saucers; interceptor: a Wraith and a Spectre. |
| `hazard-minefield` | default | 24 Dominion space mines in a 50 % **ion storm** (they show up late) between you and a Prospector hauler to reach. |
| `env-orbit` | default | Stage 3.3: the `orbit` environment — a blue planet in the sky — with two scout saucers. |
| `env-pocket` | default | The `pocket` environment: a 12 km arena under a purple planet, two hot-rods. |
| `surface-moon` | default | Stage 3.4 – 3.6: `moon_surface` — low gravity, thin air, craters, a violet sky with stars; two saucers. Landing, crashing, ALT/VS and the horizon. |
| `surface-hills` | default | `dusk_hills` — full gravity and thick air, rolling hills, a warm dusk sky and fog, the ceiling at 7 km; two hot-rods on wings. |
| `surface-colony` | default | Stage 3.5 ground installations: defend a Guard colony (protected) set down on the moon from two raider saucers. |
| `surface-strike` | default | Three Dominion flak batteries (ground turrets) in the hills: knock them out; they track low fliers. |

The Stage 1 `sample-arena` scenario (pack `sample`) was retired when the `sample` pack became
the `default` campaign; `rocket-vs-saucer` takes its place in the manual tests.

## Key Symbols

| Symbol | What it is |
|---|---|
| `AppContext` | `content`, `renderer`, `sounds`, `bindings`, `devices`. |
| `Next` | `Stay`, `Launcher`, `Viewer`, `Assets`, `Controls`, `Reload { campaign, difficulty }`, `Fly { scenario, ship }`, `Mission { mission, ship }`, and since app 0.6.3 `Simulator` (the setup screen) and `Simulate { mission: Box<MissionDef>, ship }` (fly a modified mission), `Quit`. |
| `launcher::Entry` | `ModelViewer`, `AssetBrowser`, `Controls`, `Simulator`, `Scenario(i)`, `Mission(i)`. |
| `simulator::Simulator`, `SimSetup`, `FORCES` | The setup screen (`new`, `frame`); the choices (indices and switches); the force multipliers `[0.5, 1, 1.5, 2]`. |
| `simulator::apply(&MissionDef, &SimSetup, &Ruleset) -> MissionDef` | The modified copy (pure, tested). |
| `simulator::hazard_classes(&Ruleset)`, `simulator::ship_allowed(&Ruleset, ShipClassId, Option<&str>)` | The campaign's asteroid and mine classes; why a ship may not fly in an environment (`None` = allowed). |
| `flight::FlightSession::start_custom(ctx, MissionDef, ship)` | Starts a simulator mission and remembers it for "again". |
| `launcher::next_campaign(&Content)`, `next_difficulty(&Content) -> Option<String>` | The campaign / difficulty level after the current one (wrapping), if there is another. |
| `controls::ControlsMenu` | The controls menu component (`new`, `frame`). |
| `assets::AssetBrowser`, `assets::Tab`, `assets::entries(&Content, Tab)` | The asset browser; `entries` (pure) lists assets with their override pack. |
| `launcher::Menu` | Pure menu state: `entries`, `selected`, `ship`, `playable`; `move_selection`, `move_ship`, `choosing`, `activate`. |
| `viewer::Look`, `looks_of`, `look_label` | The D key's looks of a model set: `Intact`, `Damage(i)`, `State(i)`, `Debris`. |
| `viewer::Orbit`, `viewer::wheel_steps` | Orbit camera maths (`pose`, `update`) and mouse-wheel steps. |
| `flight::FlightSession` | World, player, view, MFD page, chase orbit, nav, pause/menu flags, accumulator, cockpit texture, 3D cockpit model and offset, design space, instruments. |
| `flight::view_pose(View, pos, rot, radius, &ChaseOrbit)` | Camera placement per view (pure). |
| `flight::apply_command`, `flight::event_messages` | Targeting/wing-order commands and pilot messages (pure). |
| `flight::WING_MENU` | The wingman menu entries `(label, Command)`, in the original's order. |
| `flight::cockpit_fov(design_aspect, aspect)` | Vertical field of view for a 3D cockpit: 60°, widened on screens narrower than the design (pure, tested). |
| `flight::MAX_FRAME` | 0.25 s cap on time fed to the simulation per frame. |

## API Reference

- `app::run(Options).await`.
- `FlightSession::start(&mut AppContext, scenario_id, ship: Option<&str>) -> Result<FlightSession, String>`,
  `FlightSession::start_custom(&mut AppContext, MissionDef, ship: Option<&str>)`,
  `Simulator::new().frame(&mut ctx) -> Next`,
  `session.frame(&mut ctx) -> Next`, `session.view`.
- `Launcher::new(&ctx).frame(&mut ctx)`, `ModelViewer::new(&ctx).frame(&mut ctx)`,
  `AssetBrowser::new(&ctx).frame(&mut ctx).await`.

## Debugging

- `openinterceptor --data <cd> --scenario flight-test --ship firestar --view chase` jumps
  straight into a scenario; add `--campaign default` (or `interceptor`) to pick the campaign.
- `openinterceptor --no-original --scenario rocket-vs-saucer --ship comet --screenshot shot.png`
  saves the cockpit view after 120 frames. The window opens at 1280×960 (4:3), so this shows
  the widened 3D-cockpit field of view; compare with a manual run in a 16:9 window (MT-2-14).
- Scenario errors (unknown class, faction or leader) are shown in orange on the launcher
  and printed to standard error.
- `openinterceptor --campaign default --soak rocket-vs-saucer --ship comet --difficulty cadet`
  plays the scenario headless and prints the player's fate (`src/soak.rs`): how long a passive
  ship survives, or whether an AI-flown one wins (`--soak-pilot ai`), per difficulty level.
  `--soak-pilot idle` leaves the player's craft sitting still: in a defence mission it measures
  how long the defended object holds on its own (`SoakPilot::Idle`).
- A layout saved by F12 that seems ignored: the addon targets the campaign that was loaded when
  it was saved; check `--list` shows `<campaign>-cockpit-<id>` and that it is not disabled.
- A simulator row that does nothing: the campaign has no asteroid class (no id containing
  `asteroid`) or no `kind: mine` class; the row says "(none in this campaign)". A Fly that does
  nothing: read the orange line under the rows (a ship that cannot enter an atmosphere).
- `openinterceptor --mission defend_station --ship comet` starts a mission directly;
  `--soak defend_station --soak-pilot ai --seconds 400` plays it headless and prints the result.
