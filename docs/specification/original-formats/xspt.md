---
spec_version: 1.0.0
status: current
updated: 2026-11-11
---

# XSPT sprites

Ten animated sprites in `xobjects2.dll`: `blam`, `blamo_`, `boom`, `flak`, `kaplo_`, `laser`,
`pop`, `pow`, `puff`, `wip` — explosions, flak bursts, hit flashes.

| Offset | Type | Field | Confidence |
|---|---|---|---|
| 0x00 | char[12] | name | verified |
| 0x0C | char[4] | tag `SPT\0` | verified |
| 0x20 | u32, u32 | frame width, height | derived |
| 0x28 | u32, u32 | centre (hot spot) x, y | derived |
| 0x30 | f32 | frame duration or scale (0.1, 0.15) | unknown |
| 0x34 | u32 | frame count | derived |
| 0x38 | u32 | 1 | unknown |
| 0x3C | u32[] | per-frame data offsets | derived |
| … | | per frame: a small header (size, origin), then 16-bit RGB555 pixel runs | unknown |

The pixel encoding is a run-length scheme still to be decoded (Stage 2). Until then the engine
draws procedural explosions.

## Revision history

| Version | Date | Change |
|---|---|---|
| 1.0.0 | 2026-11-11 | Adopted into Specification 1.0.0 as a normative format page; requirements link here |
