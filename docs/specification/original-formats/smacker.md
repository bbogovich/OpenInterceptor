---
spec_version: 1.16.0
status: current
updated: 2026-11-11
---

# Smacker video

`Animations/*.smk` are RAD Game Tools Smacker files (`SMK2`/`SMK4`): the intro, alien
transmissions, endings and credits; the cockpit display's talking heads are Smacker clips
bundled in `Animations/mfdanim.mps` ([MPSRC packs](mpsrc.md)). The Smacker format is publicly
documented. OpenInterceptor decodes it in-crate (`formats::smacker`, app 0.12.0); the original
`SMACKW32.DLL` is never used. The decoder was checked against **all eleven** `.smk` files of the
reference CD — every frame and its audio decode (*verified*).

## Layout

All numbers little-endian.

| Part | Contents | State |
|---|---|---|
| Header (104 bytes) | Signature `SMK2` or `SMK4`; width; height; frame count; frame rate (positive: milliseconds per frame; negative: hundred-thousandths; zero: 10 fps); flags (bit 0: a ring frame follows the last; bits 1 – 2: lines doubled or interlaced); seven audio-track sizes; the size of the Huffman tree data; the four trees' sizes; seven audio-track rate words (rate in the low 24 bits; flags: data present, 16-bit, stereo, compressed); a reserved word | verified |
| Frame sizes | One `u32` per frame (+1 with a ring frame); the low two bits are flags, the rest the size | verified |
| Frame types | One byte per frame: bit 0 a palette chunk follows; bits 1 – 7 an audio chunk for tracks 0 – 6 | verified |
| Trees | The four big Huffman trees — `mmap`, `mclr`, `full`, `type` — packed least-significant bit first | verified |
| Frames | Each: an optional palette chunk, the audio chunks of the tracks flagged, then the video bitstream | verified |

## Huffman trees

A **small tree** has 8-bit leaves: a bit `1` is a node (left subtree, then right), `0` a leaf
followed by its 8-bit value. A **big tree** (16-bit leaves) is preceded by a low-byte and a
high-byte small tree and three 16-bit **escape** values; a leaf's value is a pair (low, high)
decoded through the small trees. A leaf equal to an escape instead stands for one of a
three-slot cache of the most recent values; every decoded value pushes into the cache. The
caches are reset at the start of each frame.

## Video

The frame is cut into 4 × 4 blocks in raster order. A `type` value gives the block kind (two
bits), a run length (six bits, through a table: 1 – 59, then 128, 256, 512, 1024, 2048) and, for
solid blocks, the colour (the high byte):

| Kind | Pixels |
|---|---|
| Mono | `mclr` gives two colours, `mmap` a 16-bit pattern choosing between them per pixel |
| Full | `full` gives the 16 pixels two at a time; in `SMK4`, two further modes give half-resolution blocks (pixels doubled) |
| Skip | Unchanged from the previous frame |
| Solid | One colour for all 16 pixels |

## Palette

A palette chunk (its length in units of four bytes, then commands) rebuilds the 256 entries from
the previous palette: a byte with the top bit set keeps the next `(b & 0x7F) + 1` entries; one
with bit 6 set copies `(b & 0x3F) + 1` entries from a given index of the old palette; otherwise
the byte and the next two are 6-bit red, green and blue, scaled to 8 bits.

## Audio

Up to seven tracks. Each audio chunk in a frame starts with its own length (a `u32`, itself
included). A compressed chunk's body starts with its unpacked size, then a bit for "data", one
for stereo and one for 16-bit, then one small tree per byte stream (one or two per channel), the
first samples as raw values, and deltas from there (DPCM). Uncompressed chunks are raw PCM;
8-bit samples are unsigned. The engine plays track 0.

## Revision history

| Version | Date | Change |
|---|---|---|
| 1.0.0 | 2026-11-11 | Adopted into Specification 1.0.0 as a normative format page; requirements link here |
| 1.16.0 | 2026-11-11 | The format as decoded in-crate (app 0.12.0): header, tables, trees with the escape cache, block kinds, palette and audio; verified on the reference CD's eleven videos; the talking heads' MPSRC pack |
