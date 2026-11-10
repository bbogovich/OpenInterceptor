# Make your first mod

A walkthrough that builds a new, original-free mod from the `sample` pack. No programming and
no X-COM data needed.

## 1. Copy the sample pack

```sh
MODS="$HOME/Library/Application Support/OpenInterceptor/mods"   # Linux: ~/.local/share/openinterceptor/mods
mkdir -p "$MODS"
cp -R packs/sample "$MODS/frontier"
```

## 2. Give it its own id

Edit `frontier/pack.ini`:

```ini
[pack]
id = frontier
name = Frontier
version = 0.1
description = My first OpenInterceptor mod
requires_original = no
```

Because the copy defines the same ships as `sample`, it overrides them — fine for
experimenting. For a real mod, rename the ids (`ship:dart` → `ship:hornet`, …) so both coexist.

## 3. Change a ship

In `frontier/rules/sample.ini`, make the Dart faster and give it a third gun:

```ini
[ship:dart]
Name = Dart Mk II
MaxSpeed = 650
Beams = pulse_laser, pulse_laser, pulse_laser
```

## 4. Add a scenario

Create `frontier/scenarios/patrol.ini` (see [Scenarios](scenarios.md)):

```ini
[scenario]
id = frontier-patrol
title = Frontier patrol
player = dart
order = 5

[object.ring]
class = ring_station
position = 0, 0, -1500

[object.r1]
class = raider
ai = enemy
position = 0, 300, -5000
speed = 250
```

## 5. Bring your own model (optional)

Export an OBJ + MTL from Blender (Forward −Z, Up Y, metres) into `frontier/models/`, then point
a ship at it: `Model = models/hornet.obj`. Textures referenced by `map_Kd` are relative to the
`.mtl` file. See [Models and textures](models.md).

## 6. Validate and play

```sh
openinterceptor --check --no-original      # fix anything it reports
openinterceptor --no-original              # "Frontier patrol" is on the launcher
```

Use the **Model viewer** to inspect your models, then fly the scenario.
