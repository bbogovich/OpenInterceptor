# Testbed

`src/app/`: the interactive program — a small state machine of screens sharing one
`AppContext`. Every stage adds scenarios here so its features can be play-tested. Plan:
Stage 1; Stage 2 added the controls menu, the campaign switch, the wingman comms menu, system
and hyperspace messages, and `--screenshot`.

## Architecture Overview

```mermaid
stateDiagram-v2
    [*] --> Launcher
    [*] --> FlightSession: --scenario id
    [*] --> ModelViewer: --model name
    Launcher --> FlightSession: Enter on a scenario (Next::Fly)
    Launcher --> ModelViewer: Enter on Model viewer (Next::Viewer)
    ModelViewer --> Launcher: Esc
    Launcher --> AssetBrowser: Enter on Asset browser (Next::Assets)
    AssetBrowser --> Launcher: Esc
    Launcher --> Controls: Enter on Controls (Next::Controls)
    Controls --> Launcher: Esc (save bindings.ini)
    Launcher --> Launcher: Tab (Next::Campaign) reloads Content for the next campaign
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
- **Launcher** (`launcher.rs`): the header names the loaded campaign; **Tab** (pad **Y**) switches
  to the next campaign (`launcher::next_campaign`), which returns `Next::Campaign(id)`: the app
  reloads `Content` with that campaign, saves it as `settings.ini [game] campaign`, and shows any
  load error on the launcher. Entries: the model viewer, the asset browser, **Controls**, then every scenario
  sorted by `order` (the first scenario is selected at start). For
  `player = choose` scenarios, Left/Right picks among playable ship classes, previewed as a
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
  `Renderer::show_overlays` (restored on Esc). `--model NAME` opens the viewer on a model
  (`viewer::find_entry`: ship id or mesh name, whole names, case-insensitive).
- **Asset browser** (`assets.rs`): every original texture, image (cockpits, HUD, menus,
  sprites) and sound, loaded exactly as the game loads them — so a pack override shows its
  replacement, and the entry is marked `[overridden by <pack>]`. Tab switches between
  Textures, Images and Sounds; Up/Down (Shift: ×10) or B/N move; Enter or Space plays a sound;
  Esc back. Images are drawn over a checkerboard so transparency is visible. Its `frame` is
  `async` because loading a sound for playback is (macroquad's API); the app loop `.await`s it.
- **Controls** (`controls.rs`): the controls menu component (`ControlsMenu`), used from the
  launcher and from the flight pause menu; see [Input](input.md#the-controls-menu).
- **Flight session** (`flight.rs`): see the frame loop on the
  [overview](index.md#logic-flow-one-frame-of-a-flight-session). Views F1–F8; after death the
  camera orbits the wreck; Esc opens a menu (Esc resume, R restart, K controls, L launcher); P
  pauses. The player's cockpit may have 2D art, a 3D model, or both; the HUD uses the cockpit's
  design space in the cockpit view and the default glass layout in the other views. A 3D
  cockpit on a window narrower than its design widens the field of view (`cockpit_fov`).
  Stage 2 additions:
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

The Stage 1 `sample-arena` scenario (pack `sample`) was retired when the `sample` pack became
the `default` campaign; `rocket-vs-saucer` takes its place in the manual tests.

## Key Symbols

| Symbol | What it is |
|---|---|
| `AppContext` | `content`, `renderer`, `sounds`, `bindings`, `devices`. |
| `Next` | `Stay`, `Launcher`, `Viewer`, `Assets`, `Controls`, `Campaign(id)`, `Fly { scenario, ship }`, `Quit`. |
| `launcher::next_campaign(&Content) -> Option<String>` | The campaign after the current one (wrapping), if there is another. |
| `controls::ControlsMenu` | The controls menu component (`new`, `frame`). |
| `assets::AssetBrowser`, `assets::Tab`, `assets::entries(&Content, Tab)` | The asset browser; `entries` (pure) lists assets with their override pack. |
| `launcher::Menu` | Pure menu state: `entries`, `selected`, `ship`, `playable`; `move_selection`, `move_ship`, `choosing`, `activate`. |
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
