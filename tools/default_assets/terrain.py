"""Terrain dressing for the `default` campaign: ground detail textures and scatter models.

Detail textures (`textures/terrain_*.png`, 256 x 256) tile across the ground, one repeat
every `detail_scale` metres (see `terrains:` in rules/environments.yaml). The engine
*multiplies* the ground's vertex colours by them, so they are painted light and nearly
grey: white leaves the terrain colour unchanged, darker texels darken it. They must tile
seamlessly, so every noise and every feature here wraps around the image edges (the noise
is periodic: its lattice repeats every `period` cells).

* `terrain_regolith.png` - moon dust: fine grain, pebbles and small craterlets;
* `terrain_grass.png`    - meadow: soft clumps and short streaks of tufts;
* `terrain_rock.png`     - bare rock for steep ground: cracks across mottled stone.

Scatter models (`models/scatter_*.obj`) stand on their origin, which the engine puts on
the ground; each reaches a little below it so a piece on a slope does not float. They are
untextured (their colour is the MTL `Kd`) and small in triangles, because hundreds are
drawn:

* `scatter_rock`    - a 2-3 m stone;
* `scatter_boulder` - a 6-8 m boulder;
* `scatter_tree`    - a 10 m conifer: a trunk and two stacked cones.
"""
import math
import os
import random

from common import Canvas, Model, PACK, cylinder_between, icosphere, scale_m, validate_model, write_model

TEXTURES = os.path.join(PACK, "textures")
MODELS = os.path.join(PACK, "models")
SIZE = 256


def periodic_noise(seed, period):
    """Smooth value noise on a lattice that repeats every `period` cells, so a texture
    sampled over exactly `period` cells tiles without a seam. f(x, y) -> 0..1."""
    def h(ix, iy):
        ix, iy = ix % period, iy % period
        n = (ix * 374761393 + iy * 668265263 + seed * 2147483647) & 0xFFFFFFFF
        n = ((n ^ (n >> 13)) * 1274126177) & 0xFFFFFFFF
        return ((n ^ (n >> 16)) & 0xFFFF) / 65535.0

    def f(x, y):
        ix, iy = math.floor(x), math.floor(y)
        fx, fy = x - ix, y - iy
        sx, sy = fx * fx * (3 - 2 * fx), fy * fy * (3 - 2 * fy)
        a = h(ix, iy) + (h(ix + 1, iy) - h(ix, iy)) * sx
        b = h(ix, iy + 1) + (h(ix + 1, iy + 1) - h(ix, iy + 1)) * sx
        return a + (b - a) * sy

    return f


def periodic_fbm_grid(seed, base_period, octaves=4):
    """Fractal noise for every pixel of a SIZE x SIZE tile: octave k has a lattice of
    base_period * 2^k cells across the image, so every octave wraps at the edges."""
    layers = [(periodic_noise(seed + k * 101, base_period << k), (base_period << k) / SIZE, 0.5 ** k)
              for k in range(octaves)]
    total = sum(a for _, _, a in layers)
    rows = []
    for y in range(SIZE):
        row = []
        for x in range(SIZE):
            row.append(sum(n(x * s, y * s) * a for n, s, a in layers) / total)
        rows.append(row)
    return rows


def stretch(v, lo=0.25, hi=0.75):
    """Spreads noise (which crowds around 0.5) back over 0..1."""
    return max(0.0, min(1.0, (v - lo) / (hi - lo)))


def base_tile(seed, period, low, high, tint=(1.0, 1.0, 1.0)):
    """A tile shaded between brightness `low` and `high` by fractal noise, tinted."""
    c = Canvas(SIZE, SIZE)
    grid = periodic_fbm_grid(seed, period)
    c.shade(lambda x, y, p: [(low + (high - low) * stretch(grid[y][x])) * t for t in tint])
    return c


def regolith():
    c = base_tile(11, 4, 0.74, 0.98)
    fine = periodic_fbm_grid(12, 32, 2)  # grain
    c.shade(lambda x, y, p: [v * (0.93 + 0.07 * fine[y][x]) for v in p])
    rng = random.Random(13)
    for _ in range(14):  # craterlets: a dark bowl with a light rim
        x, y, r = rng.uniform(0, SIZE), rng.uniform(0, SIZE), rng.uniform(4, 13)
        c.disc(x, y, r * 1.15, (1.0, 1.0, 1.0), 0.35, soft=2.0, wrap=True)
        c.disc(x, y, r, (0.62, 0.62, 0.64), 0.55, soft=2.5, wrap=True)
    for _ in range(160):  # pebbles
        x, y = rng.uniform(0, SIZE), rng.uniform(0, SIZE)
        k = rng.uniform(0.55, 1.0)
        c.disc(x, y, rng.uniform(0.6, 2.0), (k, k, k * 0.98), 0.8, wrap=True)
    return c


def grass():
    c = base_tile(21, 4, 0.72, 0.98, (0.98, 1.0, 0.94))
    rng = random.Random(22)
    for _ in range(1400):  # tufts: short, mostly upright streaks
        x, y = rng.uniform(0, SIZE), rng.uniform(0, SIZE)
        k = rng.uniform(0.62, 1.0)
        col = (k * 0.96, k, k * 0.88)
        lean = rng.uniform(-0.6, 0.6)
        for s in range(int(rng.uniform(3, 7))):
            c.blend(int(x + lean * s) % SIZE, int(y - s) % SIZE, col, 0.55)
    return c


def rock():
    c = base_tile(31, 8, 0.66, 0.97, (1.0, 0.98, 0.95))
    rng = random.Random(32)
    for _ in range(18):  # cracks: random walks of dark pixels
        x, y = rng.uniform(0, SIZE), rng.uniform(0, SIZE)
        a = rng.uniform(0, 2 * math.pi)
        for _ in range(int(rng.uniform(30, 90))):
            c.blend(int(x) % SIZE, int(y) % SIZE, (0.45, 0.43, 0.42), 0.6)
            a += rng.uniform(-0.5, 0.5)
            x, y = x + math.cos(a), y + math.sin(a)
    return c


TERRAIN_TEXTURES = [("terrain_regolith", regolith), ("terrain_grass", grass), ("terrain_rock", rock)]

STONE = {"stone": ((0.62, 0.60, 0.58), None)}
TREE = {"bark": ((0.32, 0.22, 0.14), None), "needles": ((0.16, 0.32, 0.17), None)}


def scatter_models():
    """The scatter models, built and ready to validate and write."""
    out = []
    # Thousands of these are baked into the ground's meshes, so triangles are few: a
    # stone is a roughened icosahedron (20 faces), a boulder one subdivision more.
    r = Model("scatter_rock", STONE, 40)
    # Flattened, and raised so about a fifth is buried.
    r.add(icosphere("stone", 1.0, "stone", 0, rough=0.25, seed=41).transformed(scale_m(1.3, 0.75, 1.0), (0, 0.55, 0)))
    out.append(r)
    b = Model("scatter_boulder", STONE, 100)
    b.add(icosphere("boulder", 3.2, "stone", 1, rough=0.22, seed=42).transformed(scale_m(1.1, 0.8, 1.25), (0, 1.9, 0)))
    out.append(b)
    t = Model("scatter_tree", TREE, 60)
    t.add(cylinder_between("trunk", (0, -0.6, 0), (0, 3.0, 0), 0.35, "bark", 5, radius_b=0.25))
    for k, (y0, h, rad) in enumerate([(2.0, 5.0, 2.4), (5.0, 5.0, 1.7)]):
        t.add(cylinder_between(f"cone{k}", (0, y0, 0), (0, y0 + h, 0), rad, "needles", 6, radius_b=0.0))
    out.append(t)
    return out


def build_all(write=True):
    """Paints the detail textures and builds the scatter models. Returns
    (texture rows (name, w, h), models)."""
    os.makedirs(TEXTURES, exist_ok=True)
    tex = []
    for name, painter in TERRAIN_TEXTURES:
        img = painter()
        if write:
            img.save(os.path.join(TEXTURES, name + ".png"))
        tex.append((name + ".png", img.w, img.h))
    models = scatter_models()
    for m in models:
        validate_model(m)
        if write:
            write_model(m, MODELS, header="tools/default_assets/terrain.py")
    return tex, models


if __name__ == "__main__":
    rows, models = build_all()
    for name, w, h in rows:
        print(f"{name:24s} {w}x{h}")
    for m in models:
        lo, hi = m.bounds()
        print(f"{m.name:24s} {m.triangles():4d} tris  {hi[0]-lo[0]:5.1f} x {hi[1]-lo[1]:5.1f} x {hi[2]-lo[2]:5.1f} m")
