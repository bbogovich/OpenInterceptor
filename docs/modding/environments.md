# Environments and terrain

Where a battle happens: deep space, in orbit over a planet, in a small "pocket", or in the air
above a planet's surface with ground installations on it. Environments and terrains are
**rules data** (app 0.6.0; ground installations 0.6.1), so a mod adds worlds without code. A
mission or scenario picks one with `environment: <id>`; without it the battle is in open space,
exactly as before.

Every key with its type and default: [Mod file reference › `environments`](reference.md#environments)
and [`terrains`](reference.md#terrains). Requirements: [R-MSN-008](../specification/requirements/missions.md#r-msn-008-battle-environments),
[R-FLT-011](../specification/requirements/flight.md#r-flt-011-atmospheric-flight),
[R-FLT-012](../specification/requirements/flight.md#r-flt-012-terrain),
[R-HUD-012](../specification/requirements/cockpit-hud.md#r-hud-012-atmosphere-instruments).
Hazards (asteroids, mines, black holes, ion storms) are separate and combine with any
environment: [Missions › Hazards](missions.md#hazards).

## A worked example

The `default` campaign's `resources/default/rules/environments.yaml` defines four environments
and two terrains:

```yaml
environments:
  orbit:
    # High orbit over a blue world: open-space physics, a planet in the sky.
    backdrop: {planet: {color: [0.30, 0.52, 0.95], size: 48, direction: [0.25, -0.55, -1]}}
  pocket:
    # A small, enclosed arena: tight, close-quarters fights.
    arena: 12km
    backdrop: {planet: {color: [0.55, 0.25, 0.60], size: 90, direction: [-0.3, 0.2, 1]}}
  moon_surface:
    # A cratered moon: thin air, low gravity, a dark violet sky with stars showing.
    physics: atmosphere
    gravity: 3.7
    air_density: 0.35
    ceiling: 9km
    terrain: craters
    sky: {top: [0.03, 0.02, 0.08], horizon: [0.32, 0.24, 0.34], fog: [0.30, 0.23, 0.32], fog_distance: 14km, stars: 0.6}
  dusk_hills:
    # Rolling hills at dusk: thick air, full gravity, a warm sky. Rockets need speed to fly.
    physics: atmosphere
    gravity: 9.8
    air_density: 1.0
    ceiling: 7km
    terrain: hills
    sky: {top: [0.10, 0.12, 0.30], horizon: [0.95, 0.55, 0.35], fog: [0.80, 0.50, 0.38], fog_distance: 10km, stars: 0.1}

terrains:
  craters:
    generate: {kind: craters, seed: 7}
    size: 30km
    height: [0, 1600]
    colors: {low: [0.30, 0.27, 0.26], high: [0.62, 0.58, 0.55], slope: [0.20, 0.18, 0.18]}
  hills:
    generate: {kind: hills, seed: 3}
    size: 30km
    height: [0, 1400]
    colors: {low: [0.22, 0.33, 0.16], high: [0.55, 0.50, 0.32], slope: [0.36, 0.30, 0.22]}
```

A scenario on the moon:

```yaml
# resources/default/scenarios/surface-moon.yaml (abridged)
id: surface-moon
player: choose
environment: moon_surface
player_position: [0, 2600, 6000]     # 2.6 km up: start well above the highest ground
objects:
  raider1: {class: raider_saucer, ai: enemy, position: [-800, 2400, 1000], speed: 200}
```

## Space environments

With `physics: space` (the default) the campaign's flight model is unchanged. Two keys change
the battle:

- `arena`: the arena radius for battles here, replacing `flight.arena_radius` — a pocket is a
  small arena with its edge close by ("ARENA BOUNDARY").
- `backdrop: {planet: {color, size, direction}}`: a planet hanging in the sky, `size` degrees
  across, in `direction`, lit from the sun's side with a darker limb. It is drawn behind
  everything, like the stars.
- `backdrop: {nebula: {color, second_color, density, seed}}` (app 0.9.1): soft clouds of two
  colours all around the battle, the stars dimmed behind them — the inside of a nebula. `color`
  is the clouds' main colour and `second_color` the colour they shade into (defaults a violet
  and a blue); `density` (0..1, default 0.8) is how thick they are, and also how much they hide
  the stars (by up to 60 %); `seed` (default 1) picks a different cloud pattern. The pattern is
  smooth 3D value noise sampled on a sky sphere, built once per nebula, so it costs nothing per
  frame. The default campaign's `pocket` uses one (it showed a large purple planet until the
  user's playtest asked for a nebula):

  ```yaml
  pocket:
    arena: 12km
    backdrop: {nebula: {color: [0.60, 0.22, 0.62], second_color: [0.20, 0.40, 0.85], density: 0.85, seed: 7}}
  ```

## Atmospheric flight

With `physics: atmosphere` every craft that can move feels, each tick (ρ = `air_density`,
g = `gravity`):

| Force | Rule | Effect |
|---|---|---|
| **Drag** | `a = −k · ρ · |v| · v`, with `k = acceleration / atmosphere_max_speed²` | Full thrust balances drag at `atmosphere_max_speed` in air of density 1. Without thrust a craft slows down. Thinner air (ρ < 1) lets it go faster. |
| **Gravity** | g along −Y, except for `hover: true` craft | A winged craft must hold altitude with lift; a hovering saucer stays up on its drive. |
| **Lift** | `g · ρ · (forward speed / lift)²` along the craft's own up, capped at 2g | At the `lift` speed the wings carry the weight exactly; slower the craft sinks, faster it climbs. Banking tilts the lift, which is how a winged craft turns. |
| **Ground** | the terrain's height under the craft | A touchdown at no more than **25 m/s** into the ground while upright is a **landing**: the craft stops sinking and slides to a halt. Faster, or not upright (up more than about 45° off the ground's normal), is a **crash**: damage of (impact speed)² × 0.5, a collision sound and a bounce that keeps half the speed. |
| **Ceiling** | the environment's `ceiling` | Climbing above it leaves the battle, as a hyperspace jump does ("LEFT THE ATMOSPHERE"). |

Ship keys that tune it (every ship class; [reference](reference.md#ships-and-objects)):

| Key | Default | Meaning |
|---|---|---|
| `atmosphere_max_speed` | 60 % of `max_speed` | Top speed in air of density 1, m/s. |
| `lift` | 120 for `fighter`, 0 for every other kind | The wing speed that carries the weight, m/s; 0 = no wings. |
| `hover` | `false` | Gravity does not pull it down. The default campaign's saucers, dreadnought, robot sentinel, grappler and cargo orb hover; the rockets fly on wings. |
| `atmospheric` | `true` | Can enter an atmosphere. `false` (a craft without heat shields or wings, say) is refused in an `atmosphere` environment by the Simulator (app 0.6.3); the strategy layer will also keep it out of surface missions (Stage 4). |

**Choosing numbers.** A craft's stall speed is its `lift` speed scaled by the air:
level flight needs `forward speed = lift / √ρ`. In `dusk_hills` (ρ = 1) a fighter with
`lift: 120` must fly at 120 m/s or more to hold height; on the moon (ρ = 0.35, g = 3.7) the
same wings need about 203 m/s, but gravity is weak, so sinking is slow. Its top speed in the
moon's thin air is `atmosphere_max_speed / √ρ` ≈ 1.7 × as fast as in thick air.

**Simple physics** (R-FLT-013) keeps its arcade handling in an atmosphere: the forces act, but
the craft's velocity is still set along its nose each tick.

**AI pilots** pull up to a climb at a speed their wings can fly when they are below 150 m, or
when their vertical speed would take them below 100 m within 3 s.

## Terrains

A terrain is a square **heightfield**: a grid of `resolution` × `resolution` heights (default
129, 17 – 257), `size` metres across (default 30 km), centred on the origin. Beyond the square the
edge heights continue. The heights span `height: [low, high]` metres. They come from exactly one
of:

- `generate: {kind, seed}` — `hills`: smooth value noise summed over five octaves (rolling
  hills); `craters`: a gently rolling plain with about sixty bowl-shaped craters with raised
  rims, a few large and many small. The same seed always gives the same ground. Built once,
  while the rules are read.
- `heightmap: textures/terrain/my_world.png` — a greyscale PNG or TGA from your pack: black is
  `low`, white is `high` (the red channel is read). Any size; it is resampled to `resolution`.
  Built once, when the content loads.

The ground height under a craft is interpolated between the four grid points around it
(bilinear), so it has no steps. The terrain is drawn as one mesh, coloured from
`colors: {low, high, slope}`: from `low` to `high` by height, blended towards `slope` where the
ground is steep, with a slight per-point variation so wide flats do not look painted. It is lit
by the sun and fades into the fog with distance.

Higher `resolution` means more detail and a heavier mesh: 129 points per side is about 32 000
triangles; 257 is about 131 000.

### Detail textures (app 0.18.0)

Vertex colours are smooth across a grid square tens of metres wide, so close to the ground the
terrain looks like painted plastic. A **detail texture** fixes that: a small image that tiles
over the ground and **multiplies** the colours above — white leaves a colour unchanged, darker
pixels darken it — so the `colors` still give the large-scale look and the texture adds grain.

```yaml
terrains:
  craters:
    generate: {kind: craters, seed: 7}
    colors: {low: [0.30, 0.27, 0.26], high: [0.62, 0.58, 0.55], slope: [0.20, 0.18, 0.18]}
    detail: textures/terrain_regolith.png    # tiles over the ground
    detail_scale: 40m                        # one repeat covers 40 m (default 50 m)
    slope_detail: textures/terrain_rock.png  # on steep ground instead (needs detail)
```

- Make the texture **seamless** (its left edge continues its right, its top its bottom) and
  mostly light: a mid-grey texture darkens the whole terrain.
- Give it **power-of-two sides** (256 × 256, 512 × 512): the engine then builds smaller,
  averaged copies (*mipmaps*) so the thousands of repeats in the distance blend into one colour
  instead of shimmering. Other sizes still work but shimmer; `--check` warns about them.
- `slope_detail` is used on triangles steep enough to take the slope colour (bare rock on crater
  walls); it needs `detail`.
- Across the mirror-image tiles beyond the field the pattern continues without a seam.

### Scattered objects (app 0.18.0)

`scatter:` places copies of a model over the ground — rocks, boulders, trees. Each entry is its
own kind of piece with its own rules:

```yaml
terrains:
  hills:
    # ...
    scatter:
      # Woods on the lower, gentler ground; boulders anywhere not too steep.
      - {model: models/scatter_tree.obj, count: 6000, scale: [0.7, 1.5], max_slope: 18deg, height: [0, 900], draw_distance: 5km}
      - {model: models/scatter_boulder.obj, count: 1000, scale: [0.5, 1.2], max_slope: 28deg, align: 0.8, draw_distance: 4km}
```

| Key | Default | Meaning |
|---|---|---|
| `model` | *required* | The model (OBJ, or `original:`); its first LOD is used. |
| `count` | 100 | How many, 1 – 20 000. Fewer are placed if few spots suit (30 tries per piece). |
| `scale` | 1 | A size, or `[smallest, largest]` for a random size in between. |
| `max_slope` | 30° | The steepest ground a piece stands on, 0 – 90°. |
| `height` | the whole range | `[lowest, highest]` metres of ground to use (trees below the tree line). |
| `align` | 0 | Lean with the ground: 0 upright (trees) … 1 square to the slope (flat rocks). |
| `seed` | the entry's number | Mixed with the battle's seed; change it to reshuffle one entry. |
| `draw_distance` | 4 km | Not drawn farther away than this. |

Placement is **deterministic**: the same battle has the same pieces in the same places every
time, and every mirror tile repeats them. Pieces get a random turn about the vertical.

**Scatter is decoration only.** Craft fly straight through it — it has no collision and cannot
be shot. Anything a craft should hit or destroy is a [ground installation](#ground-installations):
a real object with a collision shape and hit points.

**Cost.** The pieces are merged, once per battle, into one mesh per cell of a 16 × 16 grid over
the field (about 1.9 km across on a 30 km field), and a cell is drawn only within its entry's
`draw_distance`. That makes drawing cheap but costs memory: the default campaign's scatter (up to
9 500 pieces on `craters`) uses roughly 90 – 100 MB. Keep `count` and the model's triangle count
modest, and use a short `draw_distance` for small pieces. Pieces appear abruptly at the draw
distance (no fade).

The default campaign's textures and models are generated by `tools/default_assets/terrain.py`
(`terrain_regolith`, `terrain_grass`, `terrain_rock`; `scatter_rock`, `scatter_boulder`,
`scatter_tree`) and used by its `craters` and `hills` terrains. The `interceptor` campaign has no
surfaces, so none of this.

The design named these keys differently (`texture: {base, detail}`, `objects:`):
[DEV-061](../implementation/deviations/index.md).

## Sky and fog

`sky:` replaces black space:

- a dome around the camera coloured from `top` (straight up) to `horizon` (at and below the
  horizon), drawn behind everything; the screen is cleared to the horizon colour;
- `stars` (0–1) dims the starfield seen through the sky (0 = no stars, the default);
- distant things fade into `fog` (default: the horizon colour). The fade grows with the square
  of the distance: a quarter at half `fog_distance`, complete at `fog_distance` (default 15 km).
  Make `fog_distance` shorter than half the terrain's `size`, so its edge stays hidden.

## Ground installations

An object class of `kind: ground` is a ground installation: it never moves, and it is **set
down on the terrain wherever it is placed**, at the ground height plus a quarter of its
collision radius — the `y` of its position is replaced. The environment is set before anything
is placed, so this works for scenario objects and mission roles alike. In space a ground object
stays where it is placed.

**Anti-air** is simply a turret on the ground: a `kind: ground` class with beams, a
`turret_arc` and a turret behaviour. The default campaign's `resources/default/rules/ground.yaml`:

```yaml
objects:
  guard_colony:
    name: "@object.guard_colony.name"
    model_set: mining_facility
    kind: ground
    faction: guard
    hit_points: 6000
    shields: 800
    armor: 300
  dominion_flak:                 # an anti-air battery
    name: "@object.dominion_flak.name"
    model_set: dominion_turret_pod
    kind: ground
    faction: dominion
    hit_points: 400
    armor: 120
    beams: [death_ray]
    turret_arc: 180
    behaviour: station_turret
```

## The HUD in an atmosphere

Over a ground the status line adds `ALT 1240  VS -35`: height above the terrain directly below,
and vertical speed (+ up). The forward view shows an **artificial horizon** — a level line straight
ahead in your compass direction, whatever your pitch and roll — and a pitch ladder with labelled
rungs at −20°, −10°, +10° and +20°. In space neither is shown.

## Testbed scenarios

| Scenario (default campaign) | Environment | What to try |
|---|---|---|
| `env-orbit` | `orbit` | The planet backdrop; a dogfight with two scout saucers. |
| `env-pocket` | `pocket` | The 12 km arena edge; two Black Comet hot-rods. |
| `surface-moon` | `moon_surface` | Low gravity, thin air, craters, the violet sky with stars; landing and crashing; regolith and rock detail, scattered rocks and boulders (app 0.18.0). |
| `surface-hills` | `dusk_hills` | Full gravity and thick air: rockets need speed to stay up; the warm dusk sky and fog; the ceiling at 7 km; grass and rock detail, woods below 900 m and boulders (app 0.18.0). |
| `surface-colony` | `moon_surface` | Defend a Guard colony (protected) on the ground from two raider saucers. |
| `surface-strike` | `dusk_hills` | Knock out three Dominion flak batteries in the hills; they track low fliers. |

The first four were soaked with the AI pilot (app 0.6.0). The `interceptor` campaign stays in space, as the
original did.

## Common errors

| Message | Cause |
|---|---|
| `environments.x: an atmosphere needs a terrain` | `physics: atmosphere` without `terrain:`. |
| `environments.x: unknown terrain 'y'` | `terrain:` names no `terrains:` entry. |
| `environments.x: physics: 'y' is not space or atmosphere` | A misspelled `physics`. |
| `terrains.x: needs generate or heightmap` / `give generate or heightmap, not both` | A terrain must have exactly one source. |
| `terrains.x: generate.kind: 'y' is not hills or craters` | An unknown generator. |
| `terrains.x: heightmap <file>: …` | The heightmap image could not be loaded. |
| `terrains.x: slope_detail needs a detail texture too` | `slope_detail` without `detail`. |
| `terrains.x: detail_scale must be a distance more than 0 (e.g. 40m)` | A zero, negative or unreadable `detail_scale`. |
| `terrains.x: scatter[n]: count c is not between 1 and 20000` | Too many (or no) pieces. |
| `terrains.x: scatter[n]: max_slope s is not between 0 and 90 degrees` / `align a is not between 0 and 1` / `height [a, b] has the highest below the lowest` / `scale [a, b] must be positive, smallest first` | A scatter value out of range. |
| `terrains.x: detail <file>: …` / `terrains.x: scatter <model>: …` (`--check`) | A detail texture or scatter model (or one of its textures) could not be loaded. |
| `terrains.x: detail <file> is WxH; power-of-two sides let it fade smoothly into the distance` (`--check` warning) | The texture will shimmer in the distance. |
| `mission x: unknown environment 'y'` (`--check`) | A mission names an environment no pack defines (a scenario fails to start with `unknown environment 'y'`). |
