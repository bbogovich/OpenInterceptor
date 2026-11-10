# Format: interface, text and media

**Status: design; implemented in [Stage 6](../implementation-plan/stage-6-presentation.md)
(screens, skins, media) with the screens' logic in Stage 4.** Files in `interface/*.ini`,
`text/<language>/`, `images/`, `music/`, `video/`.

## Screens

The engine provides the strategy screens as **functional components** — the logic and the
widgets — and a mod lays them out and skins them. The set follows the original's structure,
which is the foundation for every campaign:

| Screen id | Function | Original |
|---|---|---|
| `starmap` | Interactive map: zoom map/system/object, sensor ranges, time controls, object info and squadron reports, launch probe, launch mission, event prompts | CSD |
| `bases` | Base list, 3D base view with slots, add/dismantle modules, build/dismantle bases, status overlays (normal, defences, damage) | Bases |
| `research` | Topics, current work, progress, rate, estimated completion, blocked capacity | Research |
| `hangar` | Equip ship: slots and items, stock designs, assign pilots | Equip Ship |
| `market` | Purchase, manufacture, queue with priorities, deliveries and escorts, selling, recruiting | Equipment & Personnel |
| `pilots` | Roster, stats, training, medical, sack | Assign Pilots |
| `reports` | Periodic reports, finances, activity, holdings | Status Reports |
| `relations` | Factions, standings and stances, history (campaigns with dynamic relations) | — |
| `encyclopaedia` | Categories and entries; unlocked by research and encounters | UFOpedia |
| `messages` | Message log, news ticker, event choices | Communications, Hypernews |
| `briefing`, `debriefing` | Mission text, map, forces; results, salvage, score | Briefing/Debriefing |
| `simulator` | Combat simulator setup | Combat Simulator |
| `chapter` | Title card and recap between story chapters (default campaign) | — |
| `main_menu`, `load_save`, `options`, `difficulty`, `credits`, `ending` | Shell screens | same |

A campaign's `[campaign] Systems` decides which screens are reachable; a mod can add
**custom panels** to a screen (text, gauges and lists bound to campaign queries).

### Layout and skin: `[screen:<id>]`, `[skin:<id>]`

```ini
[skin:deco]                         ; default campaign
Font.title = fonts/poster.png, size: 32       ; bitmap font sheet (or TTF, Stage 5 decision)
Font.body = fonts/typewriter.png
Panel = images/ui/panel_chrome.png, slice: 24        ; nine-slice frame
Button = images/ui/button_bakelite.png, slice: 12, hover: images/ui/button_hover.png
Colour.text = 0.95, 0.9, 0.75
Colour.accent = 0.85, 0.15, 0.1
Sound.click = sounds/ui/click.wav

[screen:starmap]
Skin = skin:deco
Design = 1920, 1080                 ; design space, scaled to the window (Stage 1 cockpit rule)
Background = images/ui/starmap_frame.png
Widget.map = 40, 80, 1400, 940
Widget.time = 1480, 80, 400, 90
Widget.info = 1480, 200, 400, 500
Widget.buttons = 1480, 740, 400, 280, buttons: launch_mission, launch_probe, bases, research
```

The `interceptor` campaign's skin uses the original's wrapper art (`WrappersDLL.dll` images)
by `original:` reference, laid out like the original screens at 640×480; the remastered mod
can supply a widescreen layout.

## Cockpits and HUD

Cockpits stay as in Stage 1 ([cockpits](../modding/cockpits.md)): 2D art at any resolution, its
own design space, or a 3D model; instrument rectangles. Additions:

- `Instrument.comms`: the MFD talking-head panel (the original's 113 Smacker clips; any image
  sequence or video for mods).
- `Instrument.<custom>` bound to a query (`hull`, `plan(eclipse)`) with a gauge style — for
  campaign-specific instruments.
- Per-environment HUD elements (altitude and vertical speed, horizon line) shown only in
  atmospheric battles.

## Text: `text/<language>/*.ini`

```ini
; text/en/ships.ini
[text]
ship.lightning.name = X-1A Lightning II
ship.lightning.desc = …
msg.alien_2 = An intercepted transmission speaks of a fleet massing near {system}.
```

- Every player-visible string is a text key (`@ship.lightning.name` in rules). Missing keys
  show the key itself and are reported by `--check`.
- `{variables}` are filled from the context (`{system}`, `{corporation}`, `{pilot}`,
  `{money}`, `{date}`), matching the original's `[SYSTEMNAME]`-style tokens.
- **Name pools** (`[names:star_names]`, `[names:pilots.male_first]`…) and **news items**
  (`[news:<id>]` with headline, text and the event that publishes it) live here too.
- The `interceptor` mod reads the original text by reference — `Text = original:LangFile1.txt`
  imports its 3,782 keys under `orig.*` (format `KEY "text"` with `~` breaks), plus
  `starcorp1.txt`, `Names1.txt`, `news1.txt`, `short1.txt` — so no original text is copied
  into the mod.

## Encyclopaedia: `[encyclopaedia:<id>]`, `[entry:<id>]`

Categories, entries with text keys, a model or image, stats pulled from the rules (`Stats =
auto`), and an unlock condition (`Unlocked = research(x) or encountered(class)`). The original's
nine categories and about 150 `DATABANK_*` entries map one-to-one.

## Music: `[music:<id>]`

The original uses **interactive music**: three sets, each with cruise, threat and battle states
of five segments, plus death, victory and good/bad-result stingers, with transition tables
(`Loops/*.wtl`). The format:

```ini
[music:interceptor_set_1]
State.cruise = original:Loops/cruise1a.wav, original:Loops/cruise1b.wav, …
State.threat = …
State.battle = …
Stinger.death = …
Stinger.victory = …
Switch = threat when: enemies_detected; battle when: enemies_within(5km); cruise otherwise
Transition = at segment end
```

Strategy-screen music, CD audio tracks (the original's Red Book audio, played from the
player's ripped files) and the default campaign's synthesised loops use the same section.

## Voices and chatter

`[voice:<id>]`: a set of lines for wingman and pilot chatter (the original's 5 male and 5
female voices × 60 lines, computer voice, alien and station lines), each line a text key plus
an audio file. Missions trigger lines by event (`Chatter` keys); pilots are assigned voices.

## Video

`[video:<id>]`: intro, endings, cut-scenes. Smacker (the original; in-crate decoder, Stage 5)
or an image sequence with audio for mods. An ending, briefing or chapter card may name a video.
