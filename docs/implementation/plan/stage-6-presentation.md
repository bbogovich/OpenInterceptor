# Stage 6 — Presentation

**Version 0.7.0. Status: in progress** — app 0.11.0 – 0.16.14 (main `1f183d1` … `de30c2a`): 6.2 and 6.3 done; 6.1, 6.4, 6.6, 6.8 and MD2 partly built; 6.5, 6.7 and MD3 not started. The campaigns look and sound finished.
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
| 6.7 | `interceptor`: screens laid out with the original wrapper art at 640×480; music sets; talking heads | `packs/interceptor` | **Partial**: music sets (0.11.0), a palette skin (0.11.1), films (0.12.0 – 0.13.0); wrapper screens not mapped; talking heads not mapped to speakers. #52 |
| 6.8 | `default`: the art-deco skin, chapter title cards and recaps, synthesised music and stings (generator tools for UI art) | `resources/default`, `tools` | **Partial** (0.11.0 – 0.13.1): art-deco skin and panel, title cards, synthesised music and stingers, the intro cutscene (captions as text keys since 0.16.2), CRT portraits; no recaps, no `good`/`bad` stingers. #52 |

## Mod and cutscene designer (MD2, MD3)

From the [mod designer, cutscene designer and game bundling](../../specification/design/designer.md) design (decisions D1–D5
made 2026-11-11): the engine side of cutscenes, which the Cutscene Designer (MD6, Stage 7) edits.

| # | Item | Module | Status | Requirements |
|---|---|---|---|---|
| MD2 | **Cutscenes**: the `cutscenes/<id>.yaml` format and its validation; the in-game player (timeline, centripetal Catmull-Rom paths and slerp, keyframes with the speed override, scripted and autopilot entities with the `aggressive` and `harmless` flags, formations, timed actions, cameras fixed and attached, cuts, sounds, captions), deterministic on the 60 Hz sim with Simple physics; played as the campaign intro, from events (`play_cutscene`) and as mission intro and outro; skip with Esc, the console or pad Start | `cutscene` (new), `app` | **Partial** (0.13.0, `content::cutscene`, `app::cutscene`): scripted entities, fire/missile/destroy, cameras, instant cuts, named sounds, captions (text keys since 0.16.2); intro and events; skip with Esc, Enter, a click or the pad's Start, A or B (0.16.2, `app::skip_pressed`). Not built: autopilot, formations, rotation keys, chase/capture/jump, sound files, mission intros/outros, console skip. #65 | [R-MOD-033 – R-MOD-045](../../specification/requirements/modding.md#r-mod-033-cutscene-files) |
| MD3 | **Video cutscenes**: video files through the platform shim (AVFoundation on macOS, Media Foundation on Windows, GStreamer loaded with `dlopen` on Linux when installed), with image sequences plus WAV as the fallback everywhere. The Linux path is evaluated on the [dependency page](../../learning/best-practices/dependencies.md) first. Complements 6.3 (Smacker, the original's films) | `platform`, `app` | **Not started**: needs the dependency/FFI evaluation first. #66 | [R-MOD-046](../../specification/requirements/modding.md#r-mod-046-video-cutscenes) |

**Exit**: every screen of both campaigns skinned; video and music working; manual
presentation tests recorded.
