# Looks: model sets, debris and effects

How an object *looks* in every state is kept apart from what it *is*. A ship or object class
names a **model set** (`model_set: comet`); the model set names its model, the texture swaps
for its damage levels and named states, the **debris set** it breaks into and the **effect** of
its explosion. Several classes can share a look, and a remaster can replace a look without
touching any class. Keys from `src/content/looks.rs`; every field with type and default:
[Mod file reference](reference.md#modelsets).

```yaml
modelsets:
  comet:
    intact: models/comet.obj
    damage:                                   # texture swaps as the hull falls
      - {below: 66%, textures: {guard_hull: textures/guard_hull_scorched.png}}
      - {below: 33%, textures: {guard_hull: textures/guard_hull_burnt.png}}
    debris: comet                             # the debris set used when it explodes
    destroyed_effect: explosion_small
debris:
  comet:
    pieces: [models/comet_debris1.obj, models/comet_debris2.obj, models/comet_debris3.obj]
    pick: [2, 3]                              # how many pieces, chosen at random
    speed: [20, 60]                           # m/s, thrown outwards
    spin: [30, 120]                           # degrees per second
    life: 20s
    chunks: {models: [models/chunk1.obj], count: 6, speed: [40, 120]}
effects:
  explosion_small: {size: 2.5, duration: 1.2s, color: [1, 0.6, 0.2], shockwave: false}
ships:
  comet:
    model_set: comet
```

The Stage 1 keys still work: a class with `model:` (and `debris:` as a list of pieces) and no
`model_set` gets a simple model set of its own.

## Model sets

| Part | How it works |
|---|---|
| `intact` | The normal model, with all its LODs (`models/x.obj` or `original:MESH`); `scale` converts model units to metres. |
| `textures` | Swaps always applied: `texture name: replacement`. |
| `damage` | A list of levels, each `{below: <hull fraction>, textures: {…}}`. Every level the hull is below applies, in order, so the worst level wins where two swap the same texture. |
| `states` | Named conditions with their own swaps: `building` (construction, later stages), `destroyed` (see below). |
| `debris` | The debris set thrown when the object is destroyed. |
| `destroyed_effect` | The effect of its explosion (default: the engine's `explosion`). |

**Texture names.** A swap matches a model's texture by **name**: the file name without folders
or extension, compared case-insensitively (`texture_key`). `guard_hull` matches
`../textures/guard_hull.png` in an OBJ's `.mtl`; `inttop` matches the original texture `INTTOP`.
So a modder's damage textures need no change to the model itself.

**Destroyed in place.** A station **module** whose model set has a `destroyed` state does not
vanish when it is destroyed: it stays mounted on its station as a wreck drawn with the
`destroyed` swaps (for example a blackened turret), until the station itself goes
([Stations and modules](stations.md)).

The renderer loads each distinct combination of swaps once and caches it, so damage levels cost
nothing per frame.

## Debris sets

| Key | Meaning |
|---|---|
| `pieces` | Piece models, **in the object's own model space**: placed at the object, they reassemble it. Each piece starts where it belongs and flies off with the object's velocity plus a random outward push. |
| `pick` | `[min, max]` pieces thrown, chosen at random without repeats (default: all of them). |
| `speed`, `spin` | `[min, max]` outward speed (m/s, default 20–60) and tumble (°/s, default 30–120). |
| `life` | How long the pieces last (default 15 s); wreckage darkens as it cools over its last third. |
| `chunks` | Generic rubble: `{models: […], count, speed: [min, max]}` scattered from the centre (default none). |

The default campaign's generator (`tools/default_assets/models.py`) cuts each model into its
pieces with planes, so they fit together exactly; the scorched, burnt and wrecked textures come
from `textures.py`.

## Effects

| Key | Meaning |
|---|---|
| `size` | Fireball size: a multiple of the object's radius (for a missile's `explosion`, metres). |
| `duration` | How long it lasts. |
| `color` | Fireball colour. |
| `shockwave` | `true` adds an expanding ring of light in the wreck's plane. |
| `sound` | A sound event name (from `sounds:`) or asset, played with the effect. |

Effects are used by model sets (`destroyed_effect`; without one, the effect named `explosion` —
a pack may define it, otherwise the engine's: size 2.5, 1.6 s, orange) and by missiles
(`explosion`, sized in metres; without one, a 25 m, 0.8 s fireball).

## The interceptor pack's looks

`packs/interceptor/rules/looks.yaml` restates the original's wreck table: **28 named wreck
sets** of up to five chunk meshes each (sets with fewer chunks repeat the first), plus generic
rubble, the alien body parts that float in alien wrecks, and the shock-wave meshes. The file
holds only **names and numbers** — set names and `original:` mesh references read from
`Interceptor.exe` (verified) — never the meshes; which class uses which set follows the names
(derived). The original's explosions are XSPT sprite animations, not yet decoded, so the
fireballs are the engine's (DEV-034).

## Checking looks

- **Model viewer: D** cycles a class's looks — intact, each damage level, each state, and the
  debris pieces pulled apart from the centre — with the look's name on the info line.
- `--check` loads every swapped texture and every debris piece and chunk.
- The `debris-showcase` scenario (both campaigns) lines up every class at 2 % hull: the damage
  textures show at once, and one shot each shows the explosion, shock wave and debris.
