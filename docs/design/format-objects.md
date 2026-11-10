# Format: space objects and models

**Status: design.** Extends the Stage 1 `[ship:]` and `[object:]` sections
([rulesets](../modding/rulesets.md)); their existing keys keep their meaning.

Every thing in space — fighter, freighter, station, module, probe, mine, asteroid, debris — is
an **object class** that names a **model set**. Splitting the two lets several classes share one
look (the pirate and the corporate Lightning differ in rules and colours, not geometry), and
lets a remaster replace a look without touching a class.

## Model sets: `[modelset:<id>]`

A model set is everything needed to draw one object in every state it can be in.

```ini
[modelset:lightning]
Intact = original:INTTYPEA              ; the normal model (all its LODs)
Scale = 1
Textures.hull = original:inttop         ; optional: retexture one texture slot by name
; Damage: texture swaps (and optional model swaps) applied as hull integrity falls.
Damage.1 = 0.66, textures: inttop>inttop_scorched      ; (example: the original has no
Damage.2 = 0.33, textures: inttop>inttop_burnt         ;  ship damage textures; mods may add them)
; Destruction: the model is replaced by debris pieces that fly apart.
Debris = debris:lightning
DestroyedEffect = effect:explosion_medium
EngineGlow = attachments                ; or a list of points
```

| Key | Meaning |
|---|---|
| `Intact` | Model reference: `original:<mesh>`, a pack OBJ, or another model set (`modelset:<id>`). Required. |
| `Scale` | Model units → metres. |
| `Textures.<slot>` | Replace the texture a model names `<slot>` for this set only (faction colours, variants). |
| `Damage.<n> = <hull fraction>, textures: a>b c>d, model: <ref>` | Below that hull fraction, swap textures (and optionally the model). Levels are checked from the highest number down. Swaps are cumulative from level 1 upwards. |
| `Debris` | A debris set (below) used when the object is destroyed. |
| `DestroyedEffect` | Effect played on destruction (explosion size, sound, flash, shock wave). |
| `States` | Named construction or condition states (stations; below). |
| `Attach.<slot>` | Models mounted at the model's attachment points (below). |
| `EngineGlow` | `attachments` (rear-facing attachment points, the Stage 1 rule) or explicit points. |
| `Lights` | Blinking/marker lights: `x,y,z,colour,period`. |

### Debris: `[debris:<id>]`

When an object is destroyed its model is removed and these pieces are spawned at the object's
position and orientation, then fly apart and tumble. Debris is also a target type (salvage,
collisions) and fades out after its lifetime.

```ini
[debris:lightning]
Piece.1 = original:X-SHIPA1, offset: 0,0,-2, speed: 20-60, spin: 30-120, life: 20
Piece.2 = original:X-SHIPA2, offset: 1,0,1
Piece.3 = original:X-SHIPA3
Piece.4 = original:X-SHIPA4
Chunks = original:CHUNK1, original:CHUNK2, count: 6, speed: 40-120   ; generic rubble
Salvage = salvage:human_light_hull        ; optional: what tractoring a piece yields
```

| Key | Meaning |
|---|---|
| `Piece.<n> = <model>, offset:, speed:, spin:, life:` | A named piece placed at `offset` (model space; default: its own centre), thrown outwards from the object centre (plus the object's velocity) at a random speed in the range, tumbling at a random rate (degrees/second). Ranges are `min-max`; seeded from the battle seed, so replays are deterministic. |
| `Chunks = <models>, count:, speed:` | Generic pieces scattered randomly. |
| `Salvage` | Salvage definition if pieces can be tractored. |

The original program has exactly this, hard-coded: a **wreck table of 28 named sets of up to
five chunk meshes** (sets with fewer chunks repeat the first), for example `UFOARandom` →
`a-shipa2…6`, `UFOShade` → `ashadc1…5`, `XCOMARandom` → `x-shipa1…4`, `XCOMBaseRandom` →
`xbasech1,2,3,5,6`, `XCOMPatton` → `xkpat1,2,3,5,6`, plus generic rubble `chunk1…6`,
`chunkf1…2`, `trichunk`, alien corpse parts floating in alien wrecks, and the `shokwave` shock
wave mesh. The `interceptor` mod writes the table as 28 `[debris:]` sections
([analysis](original-analysis.md#space-objects-and-models)). The "Random" in the set names is
honoured with `Pick = 3-5` (choose that many pieces at random from the list).

### Construction and condition states (stations)

A station looks different depending on what has been built — and in the original each X-COM
module has three textures: **normal, under construction and destroyed** (`x-hanger`,
`hang-const`, `xhngr-brn`), alien modules two (normal and `-b`). States express this:

```ini
[modelset:xcom_hangar]
Intact = original:XMMOD02              ; one generic module mesh, retextured per type
Textures.xmmod = original:x-hanger
State.building = textures: xmmod>hang-const
State.destroyed = textures: xmmod>xhngr-brn
```

The engine sets `building` while a module's build time runs, and `destroyed` (a module wreck
stays attached, unlike a destroyed ship) when its hit points reach zero; damaged-but-working is
shown with the HUD's base-view overlay, as in the original.

The hub itself has **slots** for modules (each with a size requirement: the original refuses
"module [size 2] to position 7 [size requirement 1]") and may show or hide overlay faces by
condition:

```ini
[modelset:xcom_base]
Intact = original:XBASE7
; Attachment slots (the mesh's manipulators) and what each can hold
Slot.dock_1 = attachment: 0, size: 2
Slot.dock_2 = attachment: 1, size: 2
Slot.turret_1 = attachment: 33, size: 0
; Overlay faces of the hub that depend on state (the XMSH overlay flag, low bits = group)
OverlayGroup.21 = visible-when: module(landing_pad)
OverlayGroup.29 = visible-when: not module(landing_pad)
```

- **Slots** are named attachment points (the original's manipulators; for OBJ models, empties
  named `slot_<name>` — see below) with a size class (0 small, 1 medium, 2 large, 3 special,
  the original's `SizeClass`). A **module** class says its size and which model it shows.
- **Overlay groups** use the overlay faces decoded in Stage 1 (flag `0x2000_0000`) whose low
  bits may number a group (unconfirmed; [XMSH](../original-game/formats/xmsh.md)). A model set
  can show or hide each group on a condition. If the low bits turn out not to be groups, the
  same key accepts a face list (`faces: 4, 5, 6`), authored with the viewer's face selection.
- `visible-when` takes a [condition](format-behaviour.md#conditions).

### Model references in OBJ

For modders' own models: an OBJ object or group named `slot_<name>` is not drawn; its centre
and orientation (first face's normal = forward) become attachment point `<name>`. Groups named
`overlay_<group>` are overlay groups. Groups named `lod<n>_...` belong to LOD *n* (with
`Lod.<n>.Distance` in the model set). One OBJ can therefore carry LODs, slots and overlays.

## Object classes

Stage 1 keys stay. New keys:

| Key | Meaning |
|---|---|
| `ModelSet` | The model set (replaces `Model`; `Model = <ref>` remains as shorthand for a model set with only `Intact`). |
| `Class` | Strategic class used by the campaign: `interceptor`, `freighter`, `tug`, `probe`, `mine`, `outpost`, `opp`, `base`, `module`… (free list, defined in the campaign ruleset). |
| `Mounts` | Mount points by type: `beam: 4, missile: 3, special: 4` (the original's three mount kinds). |
| `Loadout` | Default equipment: `beam: laser*2, missile: xwinder*4, special: ecm`. |
| `Systems` | Damageable systems and their hit points: `engines, sensors, shields, hyperdrive, …`. |
| `Behaviour` | Behaviour id ([behaviours](format-behaviour.md)); per difficulty with `Behaviour.<difficulty>`. |
| `Power` | Power plant output and the beam/shield split rules. |
| `Cost`, `BuildTime`, `Upkeep` | Strategy values (see [strategy](format-strategy.md#ships-and-equipment)). |
| `Capturable`, `Salvage` | Whether a disabled object can be tractored, and what it yields. |
| `Cargo` | What a freighter carries (resources and amounts) for the economy. |
| `Crew` | Pilot slots: `pilot: 1` for fighters; capital ships may have more. |
| `Launches` | Craft this object can launch (freighter drones, base defence craft, probe launchers). |

### Modules: `[module:<id>]`

A station module is an object class too (it can be shot and destroyed) plus strategy effects.

```ini
[module:hangar]
Name = @module.hangar
ModelSet = modelset:xcom_hangar
Fits = slot: dock_*            ; which slots it may occupy
Cost = 250000
BuildTime = 10 days
Upkeep = 5000 / month
Effects = hangar_capacity +4
Requires = research(base_modules)
```

`Effects` are named campaign variables changed while the module exists (see
[strategy](format-strategy.md#bases-and-modules)).

## Effects: `[effect:<id>]`

Explosions, shock waves, shield flashes, engine trails, beam impacts: sprite or particle
definitions using images (the original XSPT sprites, or pack PNG sequences), size, duration,
light and sound. Stage 2 implements the set the original uses; mods add their own.
