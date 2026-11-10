# XTXR textures

Palettised textures with 16 shade tables and mipmaps. **verified** for every size present.

| Offset | Type | Field | Confidence |
|---|---|---|---|
| 0x00 | char[12] | name | verified |
| 0x0C | char[4] | tag `TXR\0` | verified |
| 0x20 | u16, u16 | width, height (powers of two, 16–256) | verified |
| 0x24 | u16 | `0xCDCD` | — |
| 0x26 | u16 | 0 | — |
| 0x28 | u16[16][256] | 16 palettes of 256 **RGB555** colours | verified |
| 0x2028 | 16 bytes | unknown | unknown |
| 0x2038 | u8[] | 4 mip levels of 8-bit indices: w×h, w/2×h/2, w/4×h/4, w/8×h/8 | verified |

Total size = 8,248 + w·h·85/64 bytes.

- **Palette 0 is full brightness**; palettes 1–15 are progressively darker (shade tables for
  the 1998 software renderer). The engine uses palette 0 and lights with the GPU.
- Rows are stored **bottom-up** (text on textures reads correctly only after flipping V).
- Whether an index is transparent (e.g. index 0 or black) is still **unknown**; no transparent
  surfaces have been confirmed on hulls.
- A few 32×32 textures are 15,416 bytes rather than 9,608: 7 KiB more than expected, perhaps
  animation frames. **unknown**
