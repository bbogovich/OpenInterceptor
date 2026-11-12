# Music

Battles and the strategy screen play **interactive music** (app 0.11.0, plan item 6.4): a
music set has **states** — calm, threatened, fighting, the star map — each a list of
**segments**, and **stingers** played once when a mission is won or lost. The music changes
state only when a segment ends, so it never cuts off mid-phrase. Music is **rules data**: a
campaign without a `music:` section is silent, and a mod can replace or add sets without code.

Requirement: [R-CPN-012](../specification/requirements/campaigns.md#r-cpn-012-video-and-music).
Every key with its type: [Mod file reference › `music`](reference.md#music). How the engine plays
it: [Architecture › Audio › Music](../implementation/architecture/audio.md#music-app-0110).
The player sets the music volume in the launcher ([User guide › Settings](../user-guide/settings.md#settingsyaml)).

## The states

| State | When it plays |
|---|---|
| `cruise` | In a battle with no hostile craft within `threat_range` of the player. Also the fallback for any state a set does not define. |
| `threat` | A hostile craft is within `threat_range` (default 12 km), but none within `battle_range`. |
| `battle` | A hostile craft is within `battle_range` (default 4 km). |
| `strategy` | On the strategy screen; also on the main menu when the set has no `menu` state. |
| `menu` | On the main menu (app 0.14.0), when the first set defines it. |

"Hostile" means a live ship of a faction hostile to the player's; stations and other static
objects do not count. Only these state names are used by the engine; a set may define
others, but nothing will ask for them.

Each state's segments play **in turn** (the first, the second, … then the first again). When a
segment ends the player starts the next segment of whatever state is wanted *now*, so a fight
that starts in the middle of a cruise segment brings in the battle music at the end of that
segment.

## Stingers

| Stinger | When it plays |
|---|---|
| `victory` | A mission's result is decided as a win (with the computer voice, before the debrief). |
| `death` | Any other result (lose, draw, retreat, killed). |
| `good`, `bad` | Accepted and loaded; not played by the engine yet (reserved for good and bad strategy news). |

A stinger plays once, over the segment that is playing.

## Which set plays

- **In a battle** one of the campaign's sets is chosen by the battle's star-field seed (set
  `seed mod number of sets`), so the same battle always has the same music and different
  battles vary.
- **On the strategy screen** the first set plays its `strategy` state (or its `cruise`).
- **On the main menu** the first set plays its `menu` state, else its `strategy` state.
- Every other screen — the launcher, the credits roll, the viewer, the asset browser, videos and cutscenes — is
  silent.

## A worked example: the default campaign

The default campaign's music is synthesised by `tools/default_assets/music.py` (Python standard
library only, deterministic): four states of two seamless four-bar loops each, and two
stingers, written as 16 kHz mono WAVs to `resources/default/music/`.

```yaml
# resources/default/rules/music.yaml
music:
  serial:
    states:
      cruise: [music/cruise1.wav, music/cruise2.wav]
      threat: [music/threat1.wav, music/threat2.wav]
      battle: [music/battle1.wav, music/battle2.wav]
      strategy: [music/strategy1.wav, music/strategy2.wav]
    stingers: {victory: music/victory.wav, death: music/death.wav}
    threat_range: 10km
    battle_range: 3500m
    volume: 55%
```

!!! warning "Generate the default music after cloning"
    The repository's safety net ignores every `*.wav`; the default campaign's other sounds are
    committed as exceptions, but its `music/*.wav` files are not (at main `76d64d5`). After a
    fresh clone run `python3 tools/default_assets/make_all.py`; without it the music files are
    reported at start-up as `sound: …` lines and the campaign is silent.

## The `interceptor` campaign

`packs/interceptor/rules/music.yaml` maps the original's three loop sets by file name from the
player's own data (`Loops/` on the CD; `original:<file>.wav` finds a file by name in any
layout): for each set, five `cruise`, five `threat` and five `battle` segments, and the `death`,
`victory`, `good` and `bad` stingers. The strategy screen plays each set's cruise segments as
its `strategy` state (*derived*: what the original plays on its strategy screens is not mapped).

The original also has **transition tables** (`Loops/*.wtl`) that say how one segment leads to
another; they are not read. The engine always changes at the end of a segment
([DEV-053](../implementation/deviations/index.md)).

## Writing your own

1. Make loops that start and end at the same point in the bar, so a segment can follow itself
   or another state's segment without a click. Any WAV the engine reads works (PCM 8/16-bit,
   MS ADPCM, IMA ADPCM).
2. Put them in your pack (`music/…`) and list them under `states`. A state may be a single file
   instead of a list: `threat: music/threat.wav`.
3. Set `threat_range` and `battle_range` for your battles' scale, and `volume` for the set's
   level against the effects.
4. `--check` validates the section; the sounds themselves are loaded when the game starts, and
   a file that fails prints a `sound: …` line.

Several sets in one campaign are allowed: a battle picks one by its seed. An addon can add a set
or replace one by id like any other definition ([Rulesets](rulesets.md#layering-inherit-delete-and-list-edits)).

## Volume

What you hear is `volume` (the set's) × the player's **music** volume (`settings.yaml`
`audio.music`, 70 % by default). The player's **effects** volume scales every other sound —
weapons, explosions, voices, the computer voice — but not the music or its stingers.

## Not built yet

- Switching rules as data (the design's `switch:` list with conditions); the states are chosen
  by the fixed nearest-hostile rule above.
- The original's `.wtl` transition tables; CD audio tracks; the `good` and `bad` stingers on
  strategy news.
- Music in cutscenes (a cutscene's `sounds` are named event sounds) and over videos (a video's
  own soundtrack plays).

## Errors

| Message | Cause |
|---|---|
| `music.x: a music set needs states (cruise, threat, battle, strategy)` | No `states`. |
| `music.x: states.y: no segments` | A state with an empty list. |
| `sound: <file>: …` (at start-up) | A segment or stinger file that is missing or does not decode. |
