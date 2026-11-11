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
| `atmospheric` | `true` | Can enter an atmosphere; read but not yet acted on. |

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

Not built yet: detail textures (`texture: {base, detail}`) and scattered objects such as rocks
(`objects:`) — the design is on [Format: missions, environments, events](../specification/design/format-missions.md#battle-environments-environmentid).

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
| `surface-moon` | `moon_surface` | Low gravity, thin air, craters, the violet sky with stars; landing and crashing. |
| `surface-hills` | `dusk_hills` | Full gravity and thick air: rockets need speed to stay up; the warm dusk sky and fog; the ceiling at 7 km. |
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
| `mission x: unknown environment 'y'` (`--check`) | A mission names an environment no pack defines (a scenario fails to start with `unknown environment 'y'`). |
