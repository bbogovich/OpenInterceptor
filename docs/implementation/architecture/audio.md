# Audio

`src/audio.rs`: sound effects driven by simulation events, and radio voice clips. Plan: Stage 1
(event sounds); radio voices Stage 3.7 (app 0.6.2); music loops and CD audio are Stage 5.

## Architecture Overview

```mermaid
flowchart LR
    R["Ruleset: sounds, beam sound, missile sound,<br/>voice_clips"] --> L["SoundBank::load (async)<br/>Content::load_sound → wav::decode (PCM / MS ADPCM / IMA ADPCM)<br/>→ Pcm::to_wav_bytes → load_sound_from_bytes"]
    EV["world.events"] --> ES["event_sound(world, event, listener, ear)<br/>→ (SoundKey, volume)"]
    ES --> P["SoundBank::play"]
    L --> P
    FS["app::flight: wing order, wingman lost,<br/>MissionRun::voices, debrief"] -->|"SoundKey::Voice(i),<br/>Named(mission_accomplished / failed)"| P
```

`event_sound` is pure and tested: beam and missile sounds use the firing ship's weapon;
hits use `shield_hit` or `hull_hit`; destruction uses `big_explosion` for objects with radius
over 100 m, else `explosion` (always at least 15 % volume); `lock` and `denied` only for the
listener; since Stage 2, `system_damage` when one of the listener's systems is damaged
(`SystemChanged`, not repairs), `tractor` / `cloak` / `ecm` when a switchable system goes on or
off (`Switched`; quieter when switching off, attenuated for other ships) and `hyperspace` when a
ship jumps out (`Departed`), `point_defence` for each point-defence shot (`PointDefence`, at 40 %
of the distance volume); a destruction whose model set's `destroyed_effect`, or a missile blast
whose `explosion` effect, has a `sound` plays that sound instead of the usual event; everything else is attenuated by distance (`attenuate`: full within 50 m, half at
about 350 m).

**Cue table (Stage 2, item 2.12).** Sound references are resolved when the rules load
(`rules::build_at`): a reference written `cue:<Name>` is looked up in the table named by
`audio.cue_table` — `original:<file>/<section>` of an original INI file, the original game's
`soundfx.ini` — whose entry's file name becomes `original:<file>.wav`. By the time `SoundBank`
loads, every reference is an ordinary asset, so cues need no run-time code. Unknown cues and a
missing table are load errors.

**Radio voices (Stage 3.7, app 0.6.2).** The rules collect every voice clip in one list,
`Ruleset::voice_clips` (`(reference, pack)` pairs): first each voice set's clips
(`voices: {<set>: {<line>: clip or [clips]}}`, kept as `VoiceSet { id, lines }` with the clip
indices), then each mission chatter line's `voice:`. `SoundBank::load` loads clip `i` as
`SoundKey::Voice(i)`, beside the named sounds. The audio layer does not decide *when* a voice
speaks: the flight screen (`app::flight`) does —

- a wing order plays `acknowledge` and a status request `status`, from the first living wingman
  of the player whose ship has a voice set with that line (`flight::wingman_voice`; the clip is
  picked from the line's list by the battle time);
- when one of the player's wingmen is destroyed, *another* wingman says `down`;
- chatter clips are queued by the mission (`MissionRun::voices`) as their lines are said, and
  the screen plays and clears them each frame;
- when the debrief appears, the named sound `mission_accomplished` (a win) or `mission_failed`
  (any other result) plays — the computer voice.

A ship's voice is resolved to an index when the ship spawns (`Ship::voice`); an unknown set
named by a ship is a rules error. Voice lines without a clip, and ships without a voice, are
simply silent.

Playback uses macroquad's `audio` feature (quad-snd: CoreAudio on macOS, ALSA on Linux).

## Key Symbols

| Symbol | What it is |
|---|---|
| `SoundKey` | `Named(&'static str)`, `Beam(u16)`, `Missile(u16)`, `Effect(u16)`, and since app 0.6.2 `Voice(u32)` (an index into `Ruleset::voice_clips`). |
| `NAMED` | The eighteen named sounds: since app 0.6.2 `mission_accomplished` and `mission_failed` (the computer voice at the debrief), and the event names `explosion`, `big_explosion`, `shield_hit`, `hull_hit`, `lock`, `missile_launch`, `afterburner`, `target`, `denied`, `collision`, and since Stage 2 `system_damage`, `tractor`, `cloak`, `ecm`, `hyperspace`, `point_defence`. |
| `SoundKey::Effect(i)` | The sound of effect `i` (an event name from `sounds:` or an asset in the effect's pack), loaded with the bank. |
| `rules::VoiceSet`, `VoiceSet::clips(line)` | A voice set: `id` and `lines` (line name → clip indices); `clips` is empty for a line the voice lacks. |
| `Ruleset::voices`, `Ruleset::voice_clips` | The voice sets, and every voice clip (sets first, then chatter) as `(reference, pack)`. |
| `flight::wingman_voice(&World, player, except, line, pick)` | A clip of `line` from the first living wingman of `player` (not `except`) with a voice that has it (pure, tested). |
| `SoundBank` | Decoded sounds by key, and `errors` for sounds that failed. |

## API Reference

- `SoundBank::load(&Content).await`, `bank.play(&SoundKey, volume)`,
  `bank.play_events(&World, listener, ear)`.
- `event_sound(&World, &SimEvent, ShipId, Vec3) -> Option<(SoundKey, f32)>`, `attenuate(distance)`.

## Debugging

- Sounds that fail to load are printed at start-up as `sound: …` lines; `--check` loads every
  referenced sound too.
- A wingman that says nothing: its ship class needs `voice: <set>`, the set needs the line
  (`acknowledge`, `status`, `down`), and the clip must load (a failed clip prints a `sound: …`
  line). Only the player's own wingmen (AI `wingman` led by the player) speak.
- No sound at all on Linux: the build needs the ALSA development package (`libasound2-dev`).
- A `cue:` sound that is silent: `--check` names unknown cues (`sounds.x: unknown sound cue 'Y'`);
  if the cue resolves, the asset browser shows whether the original file (or its override)
  loads.
