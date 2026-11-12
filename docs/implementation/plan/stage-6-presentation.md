# Stage 6 — Presentation

**Version 0.7.0. Status: in progress** — app 0.11.0 – 0.13.1 (main `1f183d1` … `747d1e9`): 6.1, 6.3, 6.4, 6.6, 6.8 and MD2 partly or wholly built; 6.2, 6.5, 6.7 and MD3 not started. The campaigns look and sound finished.
Progress: [Implementation status › Stage 6](../status.md#stage-6-progress).

Design: [interface, text and media](../../specification/design/format-interface.md).

## Work items

| # | Item | Module | Status |
|---|---|---|---|
| 6.1 | Screen layouts and skins: design spaces, nine-slice panels, buttons, fonts, custom panels bound to queries | `app::ui` | **Partial** (0.11.1): skins (colours, nine-slice panel, font) on the launcher and strategy screen; no layouts, buttons, custom panels. #48 |
| 6.2 | Main menu, options (graphics, audio, controls), difficulty, load/save, credits, ending screens; campaign and mod selection (pack switches in the launcher) | `app` | **Not started** beyond the launcher's M/V volume keys (0.12.1) and endings' films. #48 |
| 6.3 | Smacker video decoder (in-crate) for the original's cut-scenes and MFD talking heads; image-sequence video for mods | `formats::smacker`, `app` | **Done** for the original's films (0.12.0; verified on all 11 CD videos); image sequences are MD3. #49 |
| 6.4 | Interactive music: states, segments, switches, stingers; the original's loop sets; CD audio tracks from ripped files | `audio` | **Partial** (0.11.0): states, segments, stingers, the loop sets, synthesised default music; no `switch:` data, `.wtl` tables or CD audio. #49 |
| 6.5 | Voices and chatter playback; computer voice | `audio` | Built in 3.7 (app 0.6.2); speakers on the comms panel in 6.6. #50 |
| 6.6 | Cockpit additions: comms panel, custom instruments | `hud` | **Partial** (0.13.1): the comms panel (`app::comms`) with voices' `name`/`face` and chatter `speaker`; MPSRC packs read (`mfdanim.mps`); no custom instruments; X-COM faces unmapped. #52 |
| 6.7 | `interceptor`: screens laid out with the original wrapper art at 640×480; music sets; talking heads | `packs/interceptor` | **Partial**: music sets (0.11.0), a palette skin (0.11.1), films (0.12.0 – 0.13.0); wrapper screens not mapped; talking heads not mapped to speakers. #52 |
| 6.8 | `default`: the art-deco skin, chapter title cards and recaps, synthesised music and stings (generator tools for UI art) | `resources/default`, `tools` | **Partial** (0.11.0 – 0.13.1): art-deco skin and panel, title cards, synthesised music and stingers, the intro cutscene, CRT portraits; no recaps. #52 |

## Mod and cutscene designer (MD2, MD3)

From the [mod designer, cutscene designer and game bundling](../../specification/design/designer.md) design (decisions D1–D5
made 2026-11-11): the engine side of cutscenes, which the Cutscene Designer (MD6, Stage 7) edits.

| # | Item | Module | Status | Requirements |
|---|---|---|---|---|
| MD2 | **Cutscenes**: the `cutscenes/<id>.yaml` format and its validation; the in-game player (timeline, centripetal Catmull-Rom paths and slerp, keyframes with the speed override, scripted and autopilot entities with the `aggressive` and `harmless` flags, formations, timed actions, cameras fixed and attached, cuts, sounds, captions), deterministic on the 60 Hz sim with Simple physics; played as the campaign intro, from events (`play_cutscene`) and as mission intro and outro; skip with Esc, the console or pad Start | `cutscene` (new), `app` | **Partial** (0.13.0, `content::cutscene`, `app::cutscene`): scripted entities, fire/missile/destroy, cameras, instant cuts, named sounds, captions; intro and events; Esc. Not built: autopilot, formations, rotation keys, chase/capture/jump, sound files, mission intros/outros, console/pad skip. #65 | [R-MOD-033 – R-MOD-045](../../specification/requirements/modding.md#r-mod-033-cutscene-files) |
| MD3 | **Video cutscenes**: video files through the platform shim (AVFoundation on macOS, Media Foundation on Windows, GStreamer loaded with `dlopen` on Linux when installed), with image sequences plus WAV as the fallback everywhere. The Linux path is evaluated on the [dependency page](../../learning/best-practices/dependencies.md) first. Complements 6.3 (Smacker, the original's films) | `platform`, `app` | **Not started**: needs the dependency/FFI evaluation first. #66 | [R-MOD-046](../../specification/requirements/modding.md#r-mod-046-video-cutscenes) |

**Exit**: every screen of both campaigns skinned; video and music working; manual
presentation tests recorded.
