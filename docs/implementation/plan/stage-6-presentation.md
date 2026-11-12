# Stage 6 — Presentation

**Version 0.7.0. Status: in progress** — app 0.11.0 – 0.16.14 (main `1f183d1` … `de30c2a`): 6.2 and 6.3 done; 6.1, 6.4, 6.6, 6.8 and MD2 partly built; 6.5, 6.7, 6.9 and MD3 not started. The campaigns look and sound finished.
Progress: [Implementation status › Stage 6](../status.md#stage-6-progress).

Design: [interface, text and media](../../specification/design/format-interface.md).

## Work items

| # | Item | Module | Status |
|---|---|---|---|
| 6.1 | Screen layouts and skins: design spaces, nine-slice panels, buttons, fonts, custom panels bound to queries | `app::ui` | **Partial** (0.11.1, 0.14.2, 0.16.0): skins (colours, nine-slice panel, font) on the launcher and strategy screen; long menu option values shrink to fit (`menu::fit_size`, 0.14.2); **screen layouts** as data for the strategy screen (`screens.starmap`: design space, five widgets, stretched to the window) and immediate-mode **buttons, lists and modal prompts** (`app::widgets`) (0.16.0, `ac7ae62`). The layout's `background` is drawn since 0.16.2 (`6b45fcb`). Open: layouts for the other screens, custom panels bound to queries, skin sounds. #48 |
| 6.2 | Main menu, options (graphics, audio, controls), difficulty, load/save, credits, ending screens; campaign and mod selection (pack switches in the launcher) | `app` | **Done** (0.14.0, main `a18a7b4`): the main menu (Continue, New campaign, Battles & tools, Options, Credits, Quit), options with Left/Right (campaign, difficulty, physics, music, effects), the credits roll or film, `campaign.menu` / `campaign.credits`, `--credits`; endings' films (0.12.1). Since 0.16.14 (`de30c2a`): **save slots** 1 – 3 from the strategy screen's *Save* and a **Load game** page (the automatic save and the slots with their age), the **ending screen** (title, text, the campaign's record; Credits or Main menu), options **Full screen** (`graphics.fullscreen`, `--windowed` overrides) and **Controls**. #48 |
| 6.3 | Smacker video decoder (in-crate) for the original's cut-scenes and MFD talking heads; image-sequence video for mods | `formats::smacker`, `app` | **Done** for the original's films (0.12.0; verified on all 11 CD videos); image sequences are MD3. #49 |
| 6.4 | Interactive music: states, segments, switches, stingers; the original's loop sets; CD audio tracks from ripped files | `audio` | **Partial** (0.11.0, 0.16.2): states, segments, stingers, the loop sets, synthesised default music; the `good` and `bad` stingers play since 0.16.2 (a battle won or lost, research complete, the ending — `interceptor` only: the default music has none); no `switch:` data, `.wtl` tables or CD audio. #49 |
| 6.5 | Voices and chatter playback; computer voice | `audio` | Built in 3.7 (app 0.6.2); speakers on the comms panel in 6.6. #50 |
| 6.6 | Cockpit additions: comms panel, custom instruments | `hud` | **Partial** (0.13.1, 0.15.1): the comms panel (`app::comms`) with voices' `name`/`face` and chatter `speaker`; MPSRC packs read (`mfdanim.mps`); the **ship status display** (`ship_status`: the craft from above tinted by hull, fore and aft arcs of three shield bands — a user request, R-HUD-005; 0.15.1, `bf963b4`), in X-COM cockpits A – C (placed by eye) and in place of gauge 3 in the default cockpit and glass HUD. Open: custom instruments bound to queries; X-COM faces unmapped. #52 |
| 6.7 | `interceptor`: screens laid out with the original wrapper art at 640×480; music sets; talking heads | `packs/interceptor` | **Partial**: music sets (0.11.0), a palette skin (0.11.1), films (0.12.0 – 0.13.0); wrapper screens not mapped (now specified as 6.9: the `interceptor` screen set); talking heads not mapped to speakers. #52 |
| 6.8 | `default`: the art-deco skin, chapter title cards and recaps, synthesised music and stings (generator tools for UI art) | `resources/default`, `tools` | **Partial** (0.11.0 – 0.13.1): art-deco skin and panel, title cards, synthesised music and stingers, the intro cutscene (captions as text keys since 0.16.2), CRT portraits; no recaps, no `good`/`bad` stingers. #52 |
| 6.9 | **Original-style strategy screens**: a campaign's screen set of full screens and dialogs built from engine widgets, the `interceptor` set after the original, the `default` set with generated art, the single-grid screen kept as the testbed layout — [below](#69-original-style-strategy-screens) | `app::ui`, `app::screens` (new), `content::screens`, `packs/interceptor`, `resources/default`, `tools` | **Not started** (specified in 1.19.0, user request 2026-11-12). #112 |

## 6.9 Original-style strategy screens

**Status: not started** — specified in Definition and Specification 1.19.0 from the user's
request (2026-11-12): *"Critically missing from the game and requirements is the strategy UI
from Interceptor. The gridded layout currently present is fine for testing, but we need to
support the interface style the original game has. It broke up functionality across multiple
screens as opposed to a single grid like currently present."* — *"there are additional screens,
these are just a subset."*

Requirements: [Strategy interface](../../specification/requirements/interface.md) (R-UI-001 – R-UI-040); features
[F-UI-001 – F-UI-006](../../definition/interface.md); format:
[Screen sets](../../specification/design/format-interface.md#screen-sets-screens-and-widgets).
Parent issue #112.

A campaign defines a **screen set**: full screens and dialogs, each a background plus engine
**widgets** placed by rectangle in its design space (640 × 480 for `interceptor`, its art read by
`original:WrappersDLL.dll#<id>` reference), linked to one another by actions. The engine owns each
screen's function; the set owns the layout. A set may be partial — a screen it lacks opens the
built-in version — so the `interceptor` set grows as the `WrappersDLL` mapping proceeds and no
step waits on it. Every step works in the `default` campaign too, and adds unit tests for its
logic and manual tests MT-6-xx for what must be played.

| # | Step | Delivers | Requirements | Issue |
|---|---|---|---|---|
| 6.9.1 | Framework | `screens.<id>` sets, the campaign `interface` keys, navigation and dialogs, the core widgets (button, toggle and radio groups, label, value, list, text, image, frame), uniform scaling with bands, keyboard and gamepad focus, validation, the partial-set fallback, custom screens, interface sounds, `original:…@[x, y, w, h]` image parts | R-UI-001 – R-UI-009, R-UI-012, R-UI-014 – R-UI-017, R-UI-019, R-UI-021, R-UI-039, R-UI-040 | #98 |
| 6.9.2 | CSD | The round `map` with its hover box, `minimap`, `globe` and `lamp`s, `clock`, date, money, speeds, the screen buttons | [R-UI-010](../../specification/requirements/interface.md#r-ui-010-map-minimap-globe-and-clock), [R-UI-023](../../specification/requirements/interface.md#r-ui-023-csd-screen) | #99 |
| 6.9.3 | Dialogs | Interceptor Orders, Launch Mission, notices and Alien Transmission, event choices | [R-UI-004](../../specification/requirements/interface.md#r-ui-004-dialogs), [R-UI-024](../../specification/requirements/interface.md#r-ui-024-orders-dialog), [R-UI-029](../../specification/requirements/interface.md#r-ui-029-launch-mission-dialog), [R-UI-030](../../specification/requirements/interface.md#r-ui-030-notice-transmission-and-choice-dialogs) | #100 |
| 6.9.4 | Bases | The 3D base view (`model_view`) with slots, overlays, rotate pad and module dial | [R-UI-011](../../specification/requirements/interface.md#r-ui-011-model-views), [R-UI-025](../../specification/requirements/interface.md#r-ui-025-bases-screen) | #101 |
| 6.9.5 | Purchase / manufacture | `market` (categories, item list with quantities, cargo and hangar space) and `factory` (queue) | [R-UI-026](../../specification/requirements/interface.md#r-ui-026-purchase-screen), [R-UI-027](../../specification/requirements/interface.md#r-ui-027-manufacture-screen) | #102 |
| 6.9.6 | Pilots | `portraits` sockets, the stat panel, docked ships with pilot assignment | [R-UI-013](../../specification/requirements/interface.md#r-ui-013-portraits-and-grid-views), [R-UI-028](../../specification/requirements/interface.md#r-ui-028-pilots-screen) | #103 |
| 6.9.7 | Research | Topics, current work, progress and rate | [R-UI-033](../../specification/requirements/interface.md#r-ui-033-research-screen) | #104 |
| 6.9.8 | Hangar | Equip ship: the craft in 3D, slots and items, pilot | [R-UI-034](../../specification/requirements/interface.md#r-ui-034-hangar-screen) | #105 |
| 6.9.9 | Reports | Four report panels by period | [R-UI-035](../../specification/requirements/interface.md#r-ui-035-reports-screen) | #106 |
| 6.9.10 | Encyclopaedia | Categories, entries, model or picture, text, stats | [R-UI-036](../../specification/requirements/interface.md#r-ui-036-encyclopaedia-screen) | #107 |
| 6.9.11 | Briefing / debriefing | The formation `grid_view` and text; recovery, items, score, rating; return to the launching screen | [R-UI-031](../../specification/requirements/interface.md#r-ui-031-briefing-screen), [R-UI-032](../../specification/requirements/interface.md#r-ui-032-debriefing-screen) | #108 |
| 6.9.12 | Options | Keyboard configuration, audio, graphics, the layout switch; messages, simulator and title screens | [R-UI-037](../../specification/requirements/interface.md#r-ui-037-options-screen), [R-UI-038](../../specification/requirements/interface.md#r-ui-038-messages-simulator-and-title-screens) | #109 |
| 6.9.13 | Default-campaign art | The `default` set in art-deco style, art generated by `tools/default_assets` | [R-UI-022](../../specification/requirements/interface.md#r-ui-022-the-default-campaigns-screen-set) | #110 |
| 6.9.14 | Testbed-layout switch | `testbed_layout`, the Options switch per campaign, the fallback for packs without a set | [R-UI-018](../../specification/requirements/interface.md#r-ui-018-two-layouts-per-campaign) | #111 |

The `interceptor` screens are laid out in each step from the `WrappersDLL.dll` mapping as it
becomes available, each fact marked verified / derived / unknown
([R-UI-020](../../specification/requirements/interface.md#r-ui-020-interceptor-screens-follow-the-original)); nothing from the original
is copied ([R-UI-021](../../specification/requirements/interface.md#r-ui-021-original-art-by-reference-only)).

**Exit (6.9):** `interceptor` opens on its CSD screen set; `default` plays start to finish in its
own set with no original data; the testbed layout is one Options switch away; a manual pass
recorded.

## Mod and cutscene designer (MD2, MD3)

From the [mod designer, cutscene designer and game bundling](../../specification/design/designer.md) design (decisions D1–D5
made 2026-11-11): the engine side of cutscenes, which the Cutscene Designer (MD6, Stage 7) edits.

| # | Item | Module | Status | Requirements |
|---|---|---|---|---|
| MD2 | **Cutscenes**: the `cutscenes/<id>.yaml` format and its validation; the in-game player (timeline, centripetal Catmull-Rom paths and slerp, keyframes with the speed override, scripted and autopilot entities with the `aggressive` and `harmless` flags, formations, timed actions, cameras fixed and attached, cuts, sounds, captions), deterministic on the 60 Hz sim with Simple physics; played as the campaign intro, from events (`play_cutscene`) and as mission intro and outro; skip with Esc, the console or pad Start | `cutscene` (new), `app` | **Partial** (0.13.0, `content::cutscene`, `app::cutscene`): scripted entities, fire/missile/destroy, cameras, instant cuts, named sounds, captions (text keys since 0.16.2); intro and events; skip with Esc, Enter, a click or the pad's Start, A or B (0.16.2, `app::skip_pressed`). Not built: autopilot, formations, rotation keys, chase/capture/jump, sound files, mission intros/outros, console skip. #65 | [R-MOD-033 – R-MOD-045](../../specification/requirements/modding.md#r-mod-033-cutscene-files) |
| MD3 | **Video cutscenes**: video files through the platform shim (AVFoundation on macOS, Media Foundation on Windows, GStreamer loaded with `dlopen` on Linux when installed), with image sequences plus WAV as the fallback everywhere. The Linux path is evaluated on the [dependency page](../../learning/best-practices/dependencies.md) first. Complements 6.3 (Smacker, the original's films) | `platform`, `app` | **Not started**: needs the dependency/FFI evaluation first. #66 | [R-MOD-046](../../specification/requirements/modding.md#r-mod-046-video-cutscenes) |

**Exit**: every screen of both campaigns skinned, as a screen set (6.9); video and music working; manual
presentation tests recorded.
