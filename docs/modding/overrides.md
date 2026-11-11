# Overrides and extraction

Any original asset can be replaced **one file at a time**: put a file at the asset's
*override path* inside an active pack, and the engine uses it instead of the original version.
Everything you do not replace still comes from the original data (your installed copy or CD). The highest-priority pack wins (the
last one in load order), so several mods can stack.

| Original asset | Override path in your pack | Format |
|---|---|---|
| mesh `INTTYPEA` (any `original:NAME` model) | `models/inttypea.obj` (+ its `.mtl`) | Wavefront OBJ, metres, +Y up, nose along −Z |
| texture `CUBETOP` (named by an original model) | `textures/cubetop.png` or `.tga` | any size |
| image `CockpitA.dll#12000` (cockpits, HUD art, menus, sprites) | `images/cockpita/12000.png` or `.tga` | any size; alpha = see-through |
| sound `Xwinder.wav` | `sounds/xwinder.wav` | WAV (PCM or ADPCM) |

Names are lower-case; the image folder is the DLL's name without `.dll`. Replacement images
are used as they are — the original cockpits' cyan colour key is *not* applied, so mark
see-through areas with PNG alpha.

## Extracting the originals

You do not need to know the names: the engine converts all of your original data for you, writing every
asset **at its override path**:

```sh
openinterceptor --extract ~/Desktop/interceptor-extracted
# extract: 297 models, 799 textures, 3244 images, 792 sounds, 27 reference files; 0 errors
```

```text
interceptor-extracted/
  README.txt, pack.yaml     (id interceptor-extracted; personal use only)
  models/<name>.obj, .mtl   most detailed LOD; textures referenced as ../textures/<tex>.png
  textures/<name>.png
  images/<dll>/<id>.png     cockpit cyan already turned into transparency
  sounds/<name>.wav         16-bit PCM (sound effects and music loops)
  original-data/            the original INI and text files, for reference (read-only data)
```

The extracted files are converted copies of copyrighted content from **your** copy of the game: use them to
make your own art, keep them to yourself, and never put them in a mod you share. A shared mod
should contain only files you made.

To find which file you want, open the launcher's **Asset browser**: it lists every texture,
image and sound under the same names, and marks the ones a pack already overrides.

## Walkthrough: repaint the cockpit in HD

The original Lightning II cockpit is 640×480. Here is how to replace it with a modern
repaint, without touching any rule.

1. **Extract** (once): `openinterceptor --extract ~/interceptor-extracted`.
2. **Find the image**: the Lightning II uses cockpit `a`, whose rules say
   `image: original:CockpitA.dll#12000`. Its extracted copy is
   `~/interceptor-extracted/images/cockpita/12000.png`.
3. **Paint** a new version at any resolution with the **same 4:3 proportions** — for example
   2560×1920. Keep the windows transparent (alpha 0) and keep the instruments' screens where
   they were, scaled up: the instrument rectangles are given in the 640×480 design space, so a
   4× repaint lines up automatically.
4. **Make a mod**:

    ```text
    ~/Library/Application Support/OpenInterceptor/mods/hd-cockpit/
      pack.yaml
      images/cockpita/12000.png      ← your repaint
    ```

    ```yaml
    id: hd-cockpit
    name: HD cockpit
    kind: addon
    targets: [interceptor]
    requires_original: true
    format: 2
    ```

5. **Check and fly**: `openinterceptor --check` (no errors), then the Asset browser shows
   `CockpitA.dll #12000 [overridden by hd-cockpit]`, and any flight with the Lightning II uses
   the new art.

This was verified with a 2560×1920 test repaint: it replaced the original in game and every
instrument stayed in place. To compare with the original, add `hd-cockpit` to
`packs: {disabled: [...]}` in `settings.yaml`.

**A widescreen redesign** (16:9 art rather than 4:3) needs new instrument positions too: add a
`cockpits: {a: …}` entry to your mod with `design: [1920, 1080]` and new `instruments`
rectangles in that space — only the keys you change; the rest is inherited by layering. For a
3D cockpit the in-game layout editor (F12) writes such an addon for you. See
[Cockpits](cockpits.md).

## Other common overrides

- **New ship model**: export from Blender as OBJ (metres, +Y up, forward −Z) to
  `models/<original mesh name>.obj` — e.g. `models/inttypea.obj` — and every Lightning II uses
  it. For a ship of your own, add a ship under `ships:` instead, with `model: models/...`.
- **Re-skin**: `textures/<texture name>.png` changes that texture on every model using it.
- **Sounds**: `sounds/xwinder.wav` replaces the X-Winder launch sound.

## Troubleshooting

- *Nothing changed*: the file name or folder does not match — compare with the extracted path
  or the Asset browser label (all lower-case). Is the pack disabled in `settings.yaml`, or does its `targets` miss the campaign being played?
- *Black or cyan edges*: the image needs real alpha; the cyan key is only applied to the
  original files.
- *Model faces the wrong way*: the engine's forward is −Z; rotate the model 180° before export.
