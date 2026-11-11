---
spec_version: 1.0.0
status: current
updated: 2026-11-11
---

# Audio

| Files | Format | Confidence |
|---|---|---|
| `SoundFX/**/*.wav` | RIFF WAVE, **Microsoft ADPCM** (format 2), mono, 22,050 Hz | verified |
| `Loops/*.wav` | RIFF WAVE, PCM 16-bit stereo, 22,050 Hz | verified |
| `CD Audio/*.aiff` (extracted by the player) | AIFF-C (compressed; codec to check) | verified container |

MS-ADPCM is decoded in-crate (`formats::wav`): per block, a predictor index and initial delta
per channel, two seed samples, then 4-bit nibbles using the seven standard coefficient pairs
from the `fmt ` chunk. The decoded PCM is handed to macroquad's audio.

`soundfx.ini` maps game events to sound files (Stage 2).

## Revision history

| Version | Date | Change |
|---|---|---|
| 1.0.0 | 2026-11-11 | Adopted into Specification 1.0.0 as a normative format page; requirements link here |
