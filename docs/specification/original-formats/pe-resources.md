# PE resource container

The asset DLLs are ordinary 32-bit Windows PE files whose `.rsrc` section holds the data. Their
code is a stub; the engine never runs it. **verified**

## Reading

1. `e_lfanew` at `0x3C` → PE header; number of sections at `+6`; optional-header size at `+20`.
2. Data directory 2 (resources) gives the `.rsrc` RVA; the section table maps RVAs to file offsets.
3. The resource directory is a three-level tree: **type → name → language**. Each directory has
   16 bytes of header, then `named + id` entries of 8 bytes: a name (high bit set → offset of a
   UTF-16 length-prefixed string) and an offset (high bit set → subdirectory, else data entry).
4. A data entry gives the RVA and size of the resource bytes.

Names are stored with surrounding double quotes in these DLLs (`"INTTYPEA"`); the engine strips
them and compares case-insensitively. Language is always 1033 (US English).

## Resource types used

| Type | Holds | DLLs | Count (approx.) |
|---|---|---|---|
| `XMSH` | 3D meshes | xobjects1, xobjects2 | 147 + 150 |
| `XTXR` | Textures | xobjects1, xobjects2 | 461 + 338 |
| `XSPT` | Sprite animations (explosions, flak, laser) | xobjects2 | 10 |
| `TARG` | TGA images, numeric IDs | Cockpit*, Sprites, Wrappers | 170–1,733 per DLL |

Each `XMSH`, `XTXR` and `XSPT` resource begins with a 12-byte lowercase name (NUL-padded with
`0xCD`, the MSVC debug fill) and a 4-byte tag `MSH\0`, `TXR\0` or `SPT\0`, then a u32 `0` and
padding to `0x20`.

## In the engine

`formats::pe` parses the tree once per DLL and keeps `(type, name) → byte range`. The packs VFS
exposes each resource as a virtual file, e.g. `xmsh/inttypea`, so the rest of the engine never
knows the data came from a DLL. See [Formats layer](../../implementation/architecture/formats.md).
