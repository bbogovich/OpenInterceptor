# Stage 6 — Presentation

**Version 0.7.0. Status: planned.** The campaigns look and sound finished.

Design: [interface, text and media](../../specification/design/format-interface.md).

## Work items

| # | Item | Module |
|---|---|---|
| 6.1 | Screen layouts and skins: design spaces, nine-slice panels, buttons, fonts, custom panels bound to queries | `app::ui` |
| 6.2 | Main menu, options (graphics, audio, controls), difficulty, load/save, credits, ending screens; campaign and mod selection (pack switches in the launcher) | `app` |
| 6.3 | Smacker video decoder (in-crate) for the original's cut-scenes and MFD talking heads; image-sequence video for mods | `formats::smacker`, `app` |
| 6.4 | Interactive music: states, segments, switches, stingers; the original's loop sets; CD audio tracks from ripped files | `audio` |
| 6.5 | Voices and chatter playback; computer voice | `audio` |
| 6.6 | Cockpit additions: comms panel, custom instruments | `hud` |
| 6.7 | `interceptor`: screens laid out with the original wrapper art at 640×480; music sets; talking heads | `packs/interceptor` |
| 6.8 | `default`: the art-deco skin, chapter title cards and recaps, synthesised music and stings (generator tools for UI art) | `resources/default`, `tools` |

## Mod and cutscene designer (MD2, MD3)

From the [mod designer, cutscene designer and game bundling](../../specification/design/designer.md) design (decisions D1–D5
made 2026-11-11): the engine side of cutscenes, which the Cutscene Designer (MD6, Stage 7) edits.

| # | Item | Module | Requirements |
|---|---|---|---|
| MD2 | **Cutscenes**: the `cutscenes/<id>.yaml` format and its validation; the in-game player (timeline, centripetal Catmull-Rom paths and slerp, keyframes with the speed override, scripted and autopilot entities with the `aggressive` and `harmless` flags, formations, timed actions, cameras fixed and attached, cuts, sounds, captions), deterministic on the 60 Hz sim with Simple physics; played as the campaign intro, from events (`play_cutscene`) and as mission intro and outro; skip with Esc, the console or pad Start | `cutscene` (new), `app` | [R-MOD-033 – R-MOD-045](../../specification/requirements/modding.md#r-mod-033-cutscene-files) |
| MD3 | **Video cutscenes**: video files through the platform shim (AVFoundation on macOS, Media Foundation on Windows, GStreamer loaded with `dlopen` on Linux when installed), with image sequences plus WAV as the fallback everywhere. The Linux path is evaluated on the [dependency page](../../learning/best-practices/dependencies.md) first. Complements 6.3 (Smacker, the original's films) | `platform`, `app` | [R-MOD-046](../../specification/requirements/modding.md#r-mod-046-video-cutscenes) |

**Exit**: every screen of both campaigns skinned; video and music working; manual
presentation tests recorded.
