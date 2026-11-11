---
spec_version: 1.0.0
status: current
updated: 2026-11-11
---

# Formats

How each file type on the CD is stored, as worked out for OpenInterceptor. Every field is marked
**verified**, **derived** or **unknown** (see [the original game](../../reference/original-game/index.md)). All numbers are
little-endian. Offsets are hexadecimal.

| Format | Where | Page | State |
|---|---|---|---|
| Win32 PE resources | `Resources/*.dll` | [PE resource container](pe-resources.md) | verified |
| `XMSH` mesh | `xobjects*.dll` | [XMSH meshes](xmsh.md) | verified (layout), derived (some fields) |
| `XTXR` texture | `xobjects*.dll` | [XTXR textures](xtxr.md) | verified |
| `TARG` / `.tga` image | cockpit, sprite and wrapper DLLs; `3DObjects/` | [TGA](tga.md) | verified |
| `XSPT` sprite animation | `xobjects2.dll` | [XSPT](xspt.md) | partly derived |
| WAV (MS-ADPCM, PCM), AIFF-C | `SoundFX/`, `Loops/`, `CD Audio/` | [Audio](audio.md) | verified (WAV) |
| Smacker video | `Animations/*.smk` | [Smacker](smacker.md) | public format |
| INI | `ProgramFiles/*.ini` | [INI rule files](ini.md) | verified |
| Object and ship table | strings in `Interceptor.exe` | [Object table](object-table.md) | derived |

Format pages describe structure only. No original data is reproduced on this site.

## Revision history

| Version | Date | Change |
|---|---|---|
| 1.0.0 | 2026-11-11 | Adopted into Specification 1.0.0 as a normative format page; requirements link here |
