# Format: space objects and models

**Status: design.** Extends the `ships:` and `objects:` sections of the rules files
([rulesets](../modding/rulesets.md)); their existing keys keep their meaning. Mod files are
YAML (mod format 2); the examples below use the same form for the planned sections.

Every thing in space — fighter, freighter, station, module, probe, mine, asteroid, debris — is
an **object class** that names a **model set**. Splitting the two lets several classes share one
look (the pirate and the corporate Lightning differ in rules and colours, not geometry), and
lets a remaster replace a look without touching a class.

## Model sets: `modelsets: {<id>: …}` { #model-sets-modelsetid }

A model set is everything needed to draw one object in every state it can be in.

```yaml
modelsets:
  lightning:
    intact: original:INTTYPEA            # the normal model (all its LODs)
    scale: 1
    textures: {hull: original:inttop}    # optional: retexture one texture slot by name
    # Damage: texture swaps (and optional model swaps) applied as hull integrity falls.
    # (Example: the original has no ship damage textures; mods may add them.)
    damage:
      - {below: 66%, textures: {inttop: inttop_scorched}}
      - {below: 33%, textures: {inttop: inttop_burnt}}
    # Destruction: the model is replaced by debris pieces that fly apart.
    debris: lightning
    destroyed_effect: explosion_medium
    engine_glow: attachments             # or a list of points
```

| Key | Meaning |
|---|---|
| `intact` | Model reference: `original:<mesh>`, a pack OBJ, or another model set (`modelsets.<id>`). Required. |
| `scale` | Model units → metres. |
| `textures: {<slot>: <texture>}` | Replace the texture a model names `<slot>` for this set only (faction colours, variants). |
| `damage: [{below:, textures: {a: b}, model:}]` | Below that hull fraction, swap textures (and optionally the model). Levels are checked from the lowest `below` up; swaps are cumulative from the first level onwards. |
| `debris` | A debris set (below) used when the object is destroyed. |
| `destroyed_effect` | Effect played on destruction (explosion size, sound, flash, shock wave). |
| `states` | Named construction or condition states (stations; below). |
| `attach: {<slot>: <model>}` | Models mounted at the model's attachment points (below). |
| `engine_glow` | `attachments` (rear-facing attachment points, the Stage 1 rule) or explicit points. |
| `lights` | Blinking/marker lights: `{at: [x, y, z], color: [r, g, b], period: 1s}`. |

### Debris: `debris: {<id>: …}` { #debris-debrisid }

When an object is destroyed its model is removed and these pieces are spawned at the object's
position and orientation, then fly apart and tumble. Debris is also a target type (salvage,
collisions) and fades out after its lifetime.

```yaml
debris:
  lightning:
    pieces:
      - {model: original:X-SHIPA1, offset: [0, 0, -2], speed: [20, 60], spin: [30, 120], life: 20s}
      - {model: original:X-SHIPA2, offset: [1, 0, 1]}
      - {model: original:X-SHIPA3}
      - {model: original:X-SHIPA4}
    chunks: {models: [original:CHUNK1, original:CHUNK2], count: 6, speed: [40, 120]}   # generic rubble
    salvage: human_light_hull       # optional: what tractoring a piece yields
```

| Key | Meaning |
|---|---|
| `pieces: [{model:, offset:, speed:, spin:, life:}]` | A named piece placed at `offset` (model space; default: its own centre), thrown outwards from the object centre (plus the object's velocity) at a random speed in the range, tumbling at a random rate (degrees/second). Ranges are `[min, max]`; seeded from the battle seed, so replays are deterministic. |
| `chunks: {models:, count:, speed:}` | Generic pieces scattered randomly. |
| `salvage` | Salvage definition if pieces can be tractored. |

The original program has exactly this, hard-coded: a **wreck table of 28 named sets of up to
five chunk meshes** (sets with fewer chunks repeat the first), for example `UFOARandom` →
`a-shipa2…6`, `UFOShade` → `ashadc1…5`, `XCOMARandom` → `x-shipa1…4`, `XCOMBaseRandom` →
`xbasech1,2,3,5,6`, `XCOMPatton` → `xkpat1,2,3,5,6`, plus generic rubble `chunk1…6`,
`chunkf1…2`, `trichunk`, alien corpse parts floating in alien wrecks, and the `shokwave` shock
wave mesh. The `interceptor` mod writes the table as 28 `debris:` entries
([analysis](original-analysis.md#space-objects-and-models)). The "Random" in the set names is
honoured with `pick: [3, 5]` (choose that many pieces at random from the list).

### Construction and condition states (stations)

A station looks different depending on what has been built — and in the original each X-COM
module has three textures: **normal, under construction and destroyed** (`x-hanger`,
`hang-const`, `xhngr-brn`), alien modules two (normal and `-b`). States express this:

```yaml
modelsets:
  xcom_hangar:
    intact: original:XMMOD02             # one generic module mesh, retextured per type
    textures: {xmmod: original:x-hanger}
    states:
      building: {textures: {xmmod: hang-const}}
      destroyed: {textures: {xmmod: xhngr-brn}}
```

**Mesh-swapping stages (planned with [logistics](logistics.md#construction-stages-as-model-set-states)).**
A state may also replace the **model** (`states: {build_25: {model: models/rig_build25.obj}}`),
so a facility can show distinct construction stages — foundation, frame, cladding — chosen
from build progress; the engine is gaining mesh-swapping states in Stage 2.

The engine sets `building` while a module's build time runs, and `destroyed` (a module wreck
stays attached, unlike a destroyed ship) when its hit points reach zero; damaged-but-working is
shown with the HUD's base-view overlay, as in the original.

The hub itself has **slots** for modules (each with a size requirement: the original refuses
"module [size 2] to position 7 [size requirement 1]") and may show or hide overlay faces by
condition:

```yaml
modelsets:
  xcom_base:
    intact: original:XBASE7
    # Attachment slots (the mesh's manipulators) and what each can hold
    slots:
      dock_1: {attachment: 0, size: 2}
      dock_2: {attachment: 1, size: 2}
      turret_1: {attachment: 33, size: 0}
    # Overlay faces of the hub that depend on state (the XMSH overlay flag, low bits = group)
    overlay_groups:
      21: {visible_when: "module(landing_pad)"}
      29: {visible_when: "not module(landing_pad)"}
```

- **Slots** are named attachment points (the original's manipulators; for OBJ models, empties
  named `slot_<name>` — see below) with a size class (0 small, 1 medium, 2 large, 3 special,
  the original's `SizeClass`). A **module** class says its size and which model it shows.
- **Overlay groups** use the overlay faces decoded in Stage 1 (flag `0x2000_0000`) whose low
  bits may number a group (unconfirmed; [XMSH](../original-game/formats/xmsh.md)). A model set
  can show or hide each group on a condition. If the low bits turn out not to be groups, the
  same key accepts a face list (`faces: [4, 5, 6]`), authored with the viewer's face selection.
- `visible_when` takes a [condition](format-behaviour.md#conditions).

### Model references in OBJ

For modders' own models: an OBJ object or group named `slot_<name>` is not drawn; its centre
and orientation (first face's normal = forward) become attachment point `<name>`. Groups named
`overlay_<group>` are overlay groups. Groups named `lod<n>_...` belong to LOD *n* (with
`lod_distances: [..]` in the model set). One OBJ can therefore carry LODs, slots and overlays.

## Object classes

The existing keys stay. Planned keys (those already implemented are marked):

| Key | Meaning |
|---|---|
| `modelset` | The model set (replaces `model`; `model: <ref>` remains as shorthand for a model set with only `intact`). |
| `class` | Strategic class used by the campaign: `interceptor`, `freighter`, `tug`, `probe`, `mine`, `outpost`, `opp`, `base`, `module`… (free list, defined in the campaign ruleset). |
| `mounts` | Mount points by type: `{beam: 4, missile: 3, special: 4}` (the original's three mount kinds). |
| `loadout` | Default equipment: `{beam: [laser, laser], missile: {xwinder: 4}, special: [ecm]}`. |
| `systems` | Damageable systems (**implemented**: `systems: [...]` replaces the default list, `systems+: [ecm]` adds; hit points come from the `systems:` section). |
| `behaviour` | Behaviour id ([behaviours](format-behaviour.md)), **implemented**; per difficulty with `variants: {<level>: {behaviour: …}}` (implemented). |
| `power` | Power plant output and the beam/shield split rules (the global `power:` section is implemented). |
| `cost`, `build_time`, `upkeep` | Strategy values (see [strategy](format-strategy.md#ships-and-equipment)). |
| `capturable`, `salvage` | Whether a disabled object can be tractored (**`capturable` implemented**), and what it yields. |
| `cargo` | What a freighter carries (resources and amounts) for the economy. |
| `crew` | Pilot slots: `{pilot: 1}` for fighters; capital ships may have more. |
| `launches` | Craft this object can launch (freighter drones, base defence craft, probe launchers). |

### Modules: `modules: {<id>: …}` { #modules-moduleid }

A station module is an object class too (it can be shot and destroyed) plus strategy effects.

```yaml
modules:
  hangar:
    name: "@module.hangar"
    modelset: xcom_hangar
    fits: {slot: "dock_*"}         # which slots it may occupy
    cost: 250000
    build_time: 10day
    upkeep: {amount: 5000, per: month}
    effects: {hangar_capacity: +4}
    requires: "research(base_modules)"
```

`effects` are named campaign variables changed while the module exists (see
[strategy](format-strategy.md#bases-and-modules)).

## Effects: `effects: {<id>: …}` { #effects-effectid }

Explosions, shock waves, shield flashes, engine trails, beam impacts: sprite or particle
definitions using images (the original XSPT sprites, or pack PNG sequences), size, duration,
light and sound. Stage 2 implements the set the original uses; mods add their own.
