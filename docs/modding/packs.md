# Packs

A pack is a folder. The engine loads built-in packs from `packs/` beside the binary, then user
mods from `mods/` in the user data folder, in the order given by `mods/order.txt` (or
alphabetically).

```
packs/<id>/
  pack.ini          identity and dependencies (required)
  rules/*.ini       ships, weapons, objects, factions, cockpits  (loaded in name order)
  models/           .obj models (+ .mtl)
  textures/         .png / .tga
  scenarios/*.ini   testbed scenarios and, later, missions
  sounds/           .wav (Stage 2)
  scripts/          (Stage 3)
```

## pack.ini

```ini
[pack]
id = my-campaign            ; lowercase, unique
name = My Campaign
version = 1.0.0
depends = sample            ; packs that must load first, comma-separated
requires_original = no      ; yes = needs the Interceptor CD data (--data)
description = A short line shown in the launcher
```

A pack with `requires_original = yes` is skipped, with a message, when no original data is
present. The two built-in packs:

| Pack | requires_original | Holds |
|---|---|---|
| `interceptor` | yes | Rules that import the original `GameSettings.ini`, the object table (class → mesh), cockpits, scenarios. **No assets**: they are read from the player's CD. |
| `sample` | no | A small original-free set: a couple of OBJ ships, a station, a scenario. The template for your own pack. |

## The virtual file system

Packs and the original data are mounted into one namespace. A rule says `model = inttypea`;
the VFS resolves it by searching, newest pack first: `models/inttypea.obj` in each pack, then
the original `XMSH` resource `inttypea`. So a mod **replaces** an original model by shipping a
file with the same name, and **adds** one by using a new name.
