# Audio

`src/audio.rs`: sound effects driven by simulation events. Plan: Stage 1 (event sounds);
music loops and CD audio are Stage 5.

## Architecture Overview

```mermaid
flowchart LR
    R["Ruleset: sounds, beam sound, missile sound"] --> L["SoundBank::load (async)<br/>Content::load_sound → wav::decode (PCM / MS ADPCM / IMA ADPCM)<br/>→ Pcm::to_wav_bytes → load_sound_from_bytes"]
    EV["world.events"] --> ES["event_sound(world, event, listener, ear)<br/>→ (SoundKey, volume)"]
    ES --> P["SoundBank::play"]
    L --> P
```

`event_sound` is pure and tested: beam and missile sounds use the firing ship's weapon;
hits use `shield_hit` or `hull_hit`; destruction uses `big_explosion` for objects with radius
over 100 m, else `explosion` (always at least 15 % volume); `lock` and `denied` only for the
listener; since Stage 2, `system_damage` when one of the listener's systems is damaged
(`SystemChanged`, not repairs), `tractor` / `cloak` / `ecm` when a switchable system goes on or
off (`Switched`; quieter when switching off, attenuated for other ships) and `hyperspace` when a
ship jumps out (`Departed`); everything else is attenuated by distance (`attenuate`: full within 50 m, half at
about 350 m).

Playback uses macroquad's `audio` feature (quad-snd: CoreAudio on macOS, ALSA on Linux).

## Key Symbols

| Symbol | What it is |
|---|---|
| `SoundKey` | `Named(&'static str)`, `Beam(u16)`, `Missile(u16)`. |
| `NAMED` | The fifteen event names: `explosion`, `big_explosion`, `shield_hit`, `hull_hit`, `lock`, `missile_launch`, `afterburner`, `target`, `denied`, `collision`, and since Stage 2 `system_damage`, `tractor`, `cloak`, `ecm`, `hyperspace`. |
| `SoundBank` | Decoded sounds by key, and `errors` for sounds that failed. |

## API Reference

- `SoundBank::load(&Content).await`, `bank.play(&SoundKey, volume)`,
  `bank.play_events(&World, listener, ear)`.
- `event_sound(&World, &SimEvent, ShipId, Vec3) -> Option<(SoundKey, f32)>`, `attenuate(distance)`.

## Debugging

- Sounds that fail to load are printed at start-up as `sound: …` lines; `--check` loads every
  referenced sound too.
- No sound at all on Linux: the build needs the ALSA development package (`libasound2-dev`).
