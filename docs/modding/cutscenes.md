# Cutscenes

A **cutscene** is a short film the engine plays with its own ships, effects and sounds: craft
that fly keyframed paths, fire and explode on cue, cameras that follow them with cuts between,
and captions (app 0.13.0, plan item MD2). It is a pack file, `cutscenes/<id>.yaml`, written by
hand until the Cutscene Designer (MD6) edits it. It runs on the ordinary battle simulation at
the fixed 60 Hz tick with a fixed seed, so it looks exactly like a battle and plays the same way
every time.

Requirements: [R-MOD-033 – R-MOD-045](../specification/requirements/modding.md#r-mod-033-cutscene-files)
(several partly met; see [Not built yet](#not-built-yet)). Design (the full planned format, of
which this page is the built part): [Mod designer › The Cutscene Designer](../specification/design/designer.md#3-the-cutscene-designer).
Editor schema: `schemas/cutscene.schema.json`. How it plays: [Architecture › Presentation ›
Cutscenes](../implementation/architecture/presentation.md#cutscenes-appcutscene).

For films made in another program, the original's Smacker videos play through the `video(…)`
action and `campaign.intro_video` ([below](#videos)); other video files are plan item MD3.

## Where cutscenes play

| How | Where it is set | Then |
|---|---|---|
| A new campaign's introduction | `campaign: {intro: <id>}` | The strategy screen. |
| A story event | the action `play_cutscene(<id>)` in any story action list ([Story layer](story.md#actions)) | Back to the strategy screen; the clock stays stopped. |
| From the command line | `openinterceptor --cutscene <id>` | The launcher. |

The player skips a cutscene with **Esc** or **Enter**. Music stops while one plays.

## A worked example: the default campaign's intro

A Guard patrol of three Comets meets a Dominion saucer over the Gatehouse station; the leader
fires and the saucer explodes; the camera ends on the station.

```yaml
# resources/default/cutscenes/intro.yaml
id: intro
title: Perils of the Void
length: 20s
environment: {starfield_seed: 31, sun: [-0.4, 0.45, -0.8], environment: orbit}
entities:
  gatehouse: {class: guard_station, keys: {position: [[0s, [0, -150, -3200]]]}}
  lead:
    class: comet
    keys: {position: [[0s, [-260, 40, 900]], [5s, [-80, 60, -400]], [10s, [60, 90, -1500]], [16s, [300, 160, -2600]]]}
    actions: [[9s, fire_beams, 2.5s]]
  # left and right: two more Comets on parallel paths
  saucer:
    class: raider_saucer
    enter: 6s
    keys: {position: [[6s, [1800, 400, -3800]], [9s, [700, 180, -2400]], [11s, [260, 110, -1900]]]}
    destroy_at: 11s
cameras:
  wide: {position: [[0s, [-120, 70, 780]], [6s, [60, 110, -260]]], look_at: lead, fov: 50}
  chase: {attach: lead, distance: [[6s, 70], [10s, 40]], orbit: [[6s, [25, 10]], [10s, [10, 6]]], fov: 60}
  station: {position: [[11s, [-500, 260, -1200]], [20s, [-700, 320, -900]]], look_at: gatehouse, fov: 45}
cuts: [[0s, wide], [6.5s, chase], [11.2s, station]]
captions:
  - [1s, 5s, "The frontier, 2084."]
  - [5.5s, 9.5s, "Regent Vexor's saucers prowl the cosmium belts..."]
  - [14s, 19.5s, "Only the Interplanetary Guard stands in their way."]
```

```yaml
# resources/default/campaign/campaign.yaml
campaign:
  intro: intro
```

Play it on its own with `openinterceptor --no-original --cutscene intro`.

## Coordinates and times

- Positions are **metres** in the battle's world: X right, **Y up**, **−Z forward** (the
  direction a craft with no rotation faces). The origin is arbitrary; keep the scene within a
  few kilometres of it, as a battle would.
- Times are seconds from the start, with a unit or without: `9s`, `2.5s`, `1min`, `12`.
- Keys are `[time, value]` pairs; they are sorted by time when the file loads.

## The file

| Key | Type | Default | Meaning |
|---|---|---|---|
| `id` | id | *required* | The cutscene's id (named by `campaign.intro`, `play_cutscene`, `--cutscene`). A later pack's file with the same id replaces it. |
| `title` | text | the id | Its title (for the designer and listings; not shown during play). |
| `length` | time | `10s` | When it ends. |
| `environment.starfield_seed` | number | 1 | The stars, and the seed of the scene's simulation. |
| `environment.sun` | three numbers | `[-0.4, 0.45, -0.8]` | The direction of the sun. |
| `environment.environment` | id | none (deep space) | A battle [environment](environments.md): its sky, planet, nebula and lighting. |
| `entities` | map | none | The craft and objects, by id (below). |
| `cameras` | map | none | The cameras, by id (below). |
| `cuts` | `[[time, camera], …]` | *required with cameras* | Which camera is in use from each time on. |
| `sounds` | `[[time, sound, volume], …]` | none | Named event sounds (below). |
| `captions` | `[[from, to, text], …]` | none | Caption lines and when they show. |

### Entities

| Key | Type | Default | Meaning |
|---|---|---|---|
| `class` | id | *required* | Any ship or object class of the loaded packs (fighters, saucers, stations, asteroids). An unknown class is a load error. |
| `faction` | id | the class's | Its faction (this decides its radar colour and who its weapons may hit). |
| `enter` | time | its first key's time | When it appears, at its place on the path. |
| `leave` | time | never | When it disappears (quietly, without exploding). |
| `keys.position` | `[[time, [x, y, z]], …]` | *required* (at least one) | Its path. One key holds it still. |
| `actions` | list | none | Timed actions (below). |
| `destroy_at` | time | never | Shorthand for `[time, destroy]`. |
| `harmless` | bool | `false` | Its bolts and missiles fire, hit and show their effects but do **no damage** (app 0.14.1). |

**Paths.** The keys are joined by a **centripetal Catmull-Rom spline**: the path passes through
every key exactly at its time, curves smoothly between them, and — unlike a plain Catmull-Rom
spline — never overshoots or loops at a sharp corner. Before the first key and after the last
the entity holds its position. The speed follows from the keys: far-apart keys close in time
make it fast.

**Facing.** An entity faces along its path and **banks into its turns** (up to 60°, from how
sharply the path bends). An entity with a single key faces straight ahead (−Z).

**How entities move.** An entity is placed on its path every tick: the simulation never steers
it, so the flight model (Realistic or Simple) and the AI do not apply. Its velocity is the
path's, so bolts it fires and its engine glow look right.

### Actions

| Action | Written | Does |
|---|---|---|
| Fire beams | `[time, fire_beams, seconds]` (default `1s`) | Holds its beam trigger for that long: its own beams fire as in a battle. |
| Missile | `[time, missile, <entity>]` | Targets that entity and fires its missile, as the battle does (the class needs a missile). |
| Destroy | `[time, destroy]` | Explodes with the class's explosion, debris and sound. |

Each action calls the battle's own code, so bolts, hits, shield flashes, explosions, debris and
their sounds are the battle's. **Beams and missiles do real damage**: a craft caught in another's
fire may be destroyed before its own `destroy_at`. Mark the shooter `harmless: true` for a show
of fire that hurts nothing (since app 0.14.1: its hits land, with their flashes and sounds, but
do no damage), and use `destroy` for kills.

### Cameras

A camera is **fixed** (keyframed positions) or **attached** to an entity.

| Key | Camera | Default | Meaning |
|---|---|---|---|
| `position` | fixed | *required* | `[[time, [x, y, z]], …]`, joined by the same spline as entity paths. |
| `look_at` | fixed | the origin | An entity id, or a point `[x, y, z]`. The horizon is kept level. |
| `attach` | attached | — | The entity it follows. |
| `distance` | attached | — | `[[time, metres], …]` behind the entity, linear between keys. |
| `orbit` | attached | `[0, 0]` | `[[time, [yaw, pitch]], …]` degrees around the entity, in the entity's frame (yaw 0, pitch 0 is straight behind). |
| `fov` | both | 60 | Vertical field of view in degrees (10 – 120). |

`cuts` switch cameras instantly. Before the first cut the first cut's camera is used; with no
cameras at all, a fixed view from `[0, 50, 300]` looking ahead.

### Sounds and captions

- `sounds: [[time, name, volume]]` plays one of the campaign's **named event sounds** — the
  `sounds:` section's `explosion`, `big_explosion`, `shield_hit`, `hyperspace`,
  `mission_accomplished` and the rest ([Mod file reference › `sounds`](reference.md#sounds)) —
  at that volume (0 – 1, default 1). The entities' own weapons and explosions make their sounds
  anyway.
- `captions: [[from, to, text]]` shows a line at the bottom of the screen, centred in the
  letterbox bar, in the skin's font, from `from` until `to`. Quote a caption that contains a
  colon or a comma.

### What the player sees

Black letterbox bars cover the top and bottom tenth of the screen; captions sit in the bottom
bar. When the cutscene ends (or is skipped) the next screen draws its own sky again.

## Videos

The original's films are Smacker videos, played full screen and letterboxed (app 0.12.0):

| How | Written |
|---|---|
| A story event | the action `video(original:<file>.smk)` ([Story layer](story.md#actions)) |
| A new campaign's introduction | `campaign: {intro_video: original:<file>.smk}` — used when the campaign has no `intro` cutscene |
| An ending | `endings: {<id>: {video: …}}` ([Story layer](story.md#endings-and-campaign_end)) |
| From the command line | `openinterceptor --video <ref>` |

A video reference may also be a pack path (`video/<file>.smk`); only Smacker files play. A pack
path is looked for in every active pack, latest first (app 0.14.1). Enter, Esc or a click skips a
video; its soundtrack follows the effects volume (app 0.14.1).

## Validating

Cutscene files are read with every other pack file when the game or `--check` loads: a mistake
is reported with the file and the key, for example
`default/cutscenes/intro.yaml: cameras.chase: no entity 'leed'`. The editor schema
`schemas/cutscene.schema.json` checks keys and types as you type. A `campaign.intro` that names
no cutscene is a load error (`campaign.intro: no cutscene 'x' (cutscenes/<id>.yaml)`, app
0.14.1). A `play_cutscene` id that does not exist is still reported only when it would play
(`cutscene x: no cutscene 'x'` on standard error), and the game carries on without it.

## Not built yet

From the [design](../specification/design/designer.md#32-what-a-scene-contains), not in app 0.13.0:

- **Rotation keys** (slerp) and **speed keys**; entities only face along their paths.
- **Autopilot entities** flown by the AI (`aggressive`), switching between scripted and
  autopilot, and **formations** of a group following a leader.
- The actions **chase**, **capture** (tractor) and **jump** (hyperspace); `harmless` on a single action (it is per entity).
- **Sound files** at times with fades, positions and music ducking; captions as **text keys**
  (they are plain text, not translated).
- Camera **height and look-offset keys**, keyframed field of view, **blended** cuts.
- Cutscenes as a **mission's intro and outro**; skipping from the tablet console or the pad's
  Start button.
- A `video:` file in place of a cutscene, and video files other than Smacker (MD3).
- Scrubbing and snapshots (the designer's seeking, MD6).

## Errors

| Message | Cause |
|---|---|
| `…/cutscenes/x.yaml: needs an id` | No `id`. |
| `entities.e: needs keys: {position: [[time, [x, y, z]], …]}` | An entity without position keys. |
| `entities.e: needs a class` / `…/cutscenes/x.yaml: unknown class 'y'` | No class, or one the packs do not define. |
| `entities.e: actions[i]: 'y' is not fire_beams, missile or destroy` | An unknown action. |
| `entities.e: key i: expected [time, value]` (also `bad time`, `bad value`) | A malformed key. |
| `cameras.c: no entity 'y'` | `attach` or `look_at` names no entity. |
| `cameras.c: needs position keys (or attach:)` | A fixed camera without positions. |
| `cuts: no camera 'y'` / `cuts: needs at least one [time, camera]` | A cut naming no camera, or cameras without cuts. |
| `sounds: expected [time, sound, volume]`, `captions: expected [from, to, text]` | Malformed lists. |
| `campaign.intro: no cutscene 'x' (cutscenes/<id>.yaml)` | `campaign.intro` names no cutscene (app 0.14.1). |
