# Testbed

`src/app/`: the interactive program — a small state machine of screens sharing one
`AppContext`. Every stage adds scenarios here so its features can be play-tested. Plan:
Stage 1.

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
    FlightSession --> Launcher: Esc menu → L
    FlightSession --> FlightSession: Esc menu → R, or Enter after death
    Launcher --> [*]: Esc
```

- `app::run(options)` loads `Content`, creates the `Renderer`, loads sounds (async), reads the
  bindings, then loops: the current screen's `frame` returns a `Next`, then
  `next_frame().await`. A start-up failure shows an error screen instead of crashing.
- **Launcher** (`launcher.rs`): the model viewer, the asset browser, then every scenario
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
- **Flight session** (`flight.rs`): see the frame loop on the
  [overview](index.md#logic-flow-one-frame-of-a-flight-session). Views F1–F8; after death the
  camera orbits the wreck; Esc opens a menu (Esc resume, R restart, L launcher); P pauses. The
  player's cockpit may have 2D art, a 3D model, or both; the HUD uses the cockpit's design
  space in the cockpit view and the default glass layout in the other views.

### Scenarios shipped

| Id | Pack | What it tests |
|---|---|---|
| `flight-test` | interceptor | **Stage 1 exit scenario**: player (any hull), X-COM base, two Lightning wingmen, one of every alien and pirate type flying practice circles; nav points `base`, `aliens`, `pirates` (N cycles). |
| `combat-test` | interceptor | The same arena (and nav points) with every alien and pirate hostile. |
| `gunnery-range` | interceptor | Six drones circling ahead; beams, link modes and missile locks. |
| `station-tour` | interceptor | Every station type in a row, a Patton cruiser and a freighter. |
| `sample-arena` | sample | No original data: Dart fighter flown from a **3D cockpit**, two wingmen, ring station, three raiders. |

## Key Symbols

| Symbol | What it is |
|---|---|
| `AppContext` | `content`, `renderer`, `sounds`, `bindings`, `devices`. |
| `Next` | `Stay`, `Launcher`, `Viewer`, `Assets`, `Fly { scenario, ship }`, `Quit`. |
| `assets::AssetBrowser`, `assets::Tab`, `assets::entries(&Content, Tab)` | The asset browser; `entries` (pure) lists assets with their override pack. |
| `launcher::Menu` | Pure menu state: `entries`, `selected`, `ship`, `playable`; `move_selection`, `move_ship`, `choosing`, `activate`. |
| `viewer::Orbit`, `viewer::wheel_steps` | Orbit camera maths (`pose`, `update`) and mouse-wheel steps. |
| `flight::FlightSession` | World, player, view, MFD page, chase orbit, nav, pause/menu flags, accumulator, cockpit texture, 3D cockpit model and offset, design space, instruments. |
| `flight::view_pose(View, pos, rot, radius, &ChaseOrbit)` | Camera placement per view (pure). |
| `flight::apply_command`, `flight::event_messages` | Targeting/wing-order commands and pilot messages (pure). |
| `flight::MAX_FRAME` | 0.25 s cap on time fed to the simulation per frame. |

## API Reference

- `app::run(Options).await`.
- `FlightSession::start(&mut AppContext, scenario_id, ship: Option<&str>) -> Result<FlightSession, String>`,
  `session.frame(&mut ctx) -> Next`, `session.view`.
- `Launcher::new(&ctx).frame(&mut ctx)`, `ModelViewer::new(&ctx).frame(&mut ctx)`,
  `AssetBrowser::new(&ctx).frame(&mut ctx).await`.

## Debugging

- `openinterceptor --data <cd> --scenario flight-test --ship firestar --view chase` jumps
  straight into a scenario.
- Scenario errors (unknown class, faction or leader) are shown in orange on the launcher
  and printed to standard error.
