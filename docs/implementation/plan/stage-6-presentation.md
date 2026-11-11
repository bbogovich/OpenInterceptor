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
| 6.8 | `default`: the art-deco skin, chapter title cards and recaps, synthesised music and stings (generator tools for UI art) | `packs/default`, `tools` |

**Exit**: every screen of both campaigns skinned; video and music working; manual
presentation tests recorded.
