"""Hull textures of the `default` campaign, painted procedurally.

Each texture is drawn on a `Canvas` (see common.py) from simple layers: a two-tone base
coat, panel seams with rivet rows, chrome trim bands, glowing portholes and a faction
insignia, then a light grime pass of fractal noise. Every hull texture also gets two
**damage variants** with the same layout, so the engine can swap them as a ship is hurt:

* `<name>_scorched.png` - soot blotches and heat-darkened paint;
* `<name>_burnt.png` - heavier blackening plus burn-through holes with glowing rims.

How the models map these images (see models.py) decides the layouts:

* *Rocket layout* (`guard_hull`, `senate_hull`, `cartel_hull`, `raider_hull`): u runs
  along the hull, nose at the left edge; v runs around it. Because the engine flips v,
  image row y holds v = 1 - y/H: rows 0 and H are the top of the hull, row H/2 the belly,
  row H/4 the left (-X) side and row 3H/4 the right (+X) side.
* *Around layout* (`dominion_hull`, `robot_tin`, `guard_station`): u runs around a
  Y-axis lathe (x = 0 faces the nose), v runs up its profile: the bottom row is the
  bottom of the saucer or tower, the top row its top.
* *Planar* (`prospector_hull`, `asteroid_rock`): tiles mapped onto box faces or a rock.

Insignia are original designs for this project: the Guard's winged star, the
Dominion's eclipse sigil, the Senate's sunburst seal, the Cartel's ringed diamond and
the Black Comet raiders' comet.
"""
import math
import os
import random

from common import Canvas, fbm, noise_grid, PACK

OUT = os.path.join(PACK, "textures")


def mix(a, b, t):
    return tuple(x + (y - x) * t for x, y in zip(a, b))


def grime(c, seed, amount=0.12, scale=24.0):
    """Darkens the canvas by up to `amount` with fractal noise (weathering)."""
    n = noise_grid(seed, c.w, c.h, scale)
    c.shade(lambda x, y, p: [v * (1 - amount * n[y][x]) for v in p])


def panel_lines(c, xs, ys, color, rivet_color, rivet_step=7, wrap=True):
    """Dark seams at the given columns and rows, each flanked by a rivet row."""
    for x in xs:
        c.rect(x, 0, x + 1, c.h, color, 0.7)
        for y in range(3, c.h, rivet_step):
            c.disc(x + 3.5, y, 1.1, rivet_color, 0.8, wrap=wrap)
    for y in ys:
        c.rect(0, y, c.w, y + 1, color, 0.7)
        for x in range(3, c.w, rivet_step):
            c.disc(x, y + 3.5, 1.1, rivet_color, 0.8, wrap=wrap)


def chrome_band(c, x0, x1):
    """A vertical chrome strip with a soft highlight - the trim band look."""
    for x in range(int(x0), int(x1)):
        t = (x - x0) / max(1, x1 - x0 - 1)
        k = 0.62 + 0.38 * math.sin(t * math.pi) ** 2
        c.rect(x, 0, x + 1, c.h, (0.86 * k, 0.88 * k, 0.93 * k))


def porthole(c, x, y, r, rim, glass, glow):
    c.glow(x, y, r * 2.2, glow, 0.35, wrap=True)
    c.disc(x, y, r + 2.2, rim, wrap=True)
    c.disc(x, y, r, glass, wrap=True)
    c.disc(x - r * 0.3, y - r * 0.3, r * 0.35, (1, 1, 0.95), 0.7, wrap=True)


def star_points(cx, cy, r_out, r_in, n=5, rot=-90):
    pts = []
    for i in range(n * 2):
        a = math.radians(rot + 180 * i / n)
        r = r_out if i % 2 == 0 else r_in
        pts.append((cx + r * math.cos(a), cy + r * math.sin(a)))
    return pts


def winged_star(c, cx, cy, s, body, star, outline):
    """The Interplanetary Guard insignia: a star in a roundel with swept wings of three
    feathers on each side, pointing along the hull."""
    for side in (-1, 1):
        for k in range(3):
            y0 = cy - s * 0.45 + k * s * 0.32
            length = s * (2.3 - k * 0.45)
            pts = [(cx + side * s * 0.6, y0), (cx + side * (s * 0.6 + length), y0 - s * 0.12),
                   (cx + side * (s * 0.6 + length * 0.86), y0 + s * 0.22), (cx + side * s * 0.6, y0 + s * 0.26)]
            if side < 0:
                pts = pts[::-1]
            c.polygon(pts, outline)
            inner = [(px + (cx - px) * 0.06, py + (cy - py) * 0.12) for px, py in pts]
            c.polygon(inner, body)
    c.disc(cx, cy, s * 0.95, outline)
    c.disc(cx, cy, s * 0.82, body)
    c.polygon(star_points(cx, cy, s * 0.72, s * 0.3), star)


def eclipse_sigil(c, cx, cy, s, glow, dark, ray):
    """The Umbral Dominion insignia: an eclipsed sun - a dark disc slightly off-centre
    over a glowing one, leaving a crescent - with eight short rays."""
    for i in range(8):
        a = math.radians(i * 45 + 22.5)
        p = [(cx + math.cos(a) * s * 1.2, cy + math.sin(a) * s * 1.2),
             (cx + math.cos(a + 0.12) * s * 1.75, cy + math.sin(a + 0.12) * s * 1.75),
             (cx + math.cos(a - 0.12) * s * 1.75, cy + math.sin(a - 0.12) * s * 1.75)]
        c.polygon(p, ray)
    c.glow(cx, cy, s * 1.6, glow, 0.5)
    c.disc(cx, cy, s, glow)
    c.disc(cx + s * 0.22, cy - s * 0.12, s * 0.9, dark)


def sunburst_seal(c, cx, cy, s, gold, navy, white):
    """The Solar Senate seal: a gold sunburst ring around a navy disc with a white star."""
    for i in range(16):
        a = math.radians(i * 22.5)
        p = [(cx + math.cos(a - 0.1) * s * 0.8, cy + math.sin(a - 0.1) * s * 0.8),
             (cx + math.cos(a) * s * 1.35, cy + math.sin(a) * s * 1.35),
             (cx + math.cos(a + 0.1) * s * 0.8, cy + math.sin(a + 0.1) * s * 0.8)]
        c.polygon(p, gold)
    c.disc(cx, cy, s * 0.9, gold)
    c.disc(cx, cy, s * 0.74, navy)
    c.polygon(star_points(cx, cy, s * 0.6, s * 0.25), white)


# ---------------------------------------------------------------------------------------
# Rocket-layout textures
# ---------------------------------------------------------------------------------------

def two_tone(c, top, belly, stripe, split=0.18, stripe_w=3):
    """Top colour over most of the hull, a belly band across the middle rows (the
    underside), and pinstripes along the boundary."""
    H = c.h
    lo, hi = H * (0.5 - split), H * (0.5 + split)
    c.shade(lambda x, y, p: belly if lo <= y < hi else top)
    for y in (lo - stripe_w - 2, hi + 2):
        c.rect(0, y, c.w, y + stripe_w, stripe)


def guard_hull():
    c = Canvas(512, 256)
    red, cream, teal = (0.80, 0.11, 0.08), (0.95, 0.90, 0.74), (0.08, 0.62, 0.60)
    two_tone(c, red, cream, teal)
    panel_lines(c, [64, 160, 256, 352, 448], [128], (0.25, 0.04, 0.03), (0.95, 0.7, 0.6))
    chrome_band(c, 18, 30)
    chrome_band(c, 470, 484)
    for side_row in (0.19, 0.81):  # portholes just above each side line
        for i in range(5):
            porthole(c, 92 + i * 30, 256 * side_row, 7, (0.8, 0.82, 0.88), (1.0, 0.85, 0.4), (1.0, 0.8, 0.3))
    for side_row in (0.25, 0.75):
        y = 256 * side_row
        for k in range(3):  # speed lines trailing the insignia
            c.rect(372, y - 10 + k * 8, 460 - k * 18, y - 7 + k * 8, cream)
        winged_star(c, 300, y, 20, cream, teal, (0.1, 0.1, 0.12))
    grime(c, 101)
    return c


def senate_hull():
    c = Canvas(512, 256)
    white, navy, gold = (0.94, 0.94, 0.96), (0.10, 0.14, 0.40), (0.95, 0.74, 0.25)
    c.shade(lambda x, y, p: white)
    for side in (0.25, 0.75):  # a navy flash along each side with gold edging
        y = 256 * side
        c.rect(0, y - 14, 512, y + 14, navy)
        c.rect(0, y - 17, 512, y - 14, gold)
        c.rect(0, y + 14, 512, y + 17, gold)
    panel_lines(c, [96, 224, 352], [], (0.55, 0.56, 0.6), (0.8, 0.8, 0.85), 9)
    chrome_band(c, 14, 22)
    for side in (0.17, 0.83):
        for i in range(4):
            porthole(c, 70 + i * 28, 256 * side, 6, gold, (0.75, 0.9, 1.0), (0.6, 0.85, 1.0))
    for side in (0.25, 0.75):
        sunburst_seal(c, 300, 256 * side, 19, gold, navy, white)
    grime(c, 202, 0.06)
    return c


def cartel_hull():
    c = Canvas(512, 256)
    mustard, brown = (0.85, 0.64, 0.15), (0.38, 0.23, 0.12)
    two_tone(c, mustard, brown, (0.2, 0.12, 0.06), split=0.22, stripe_w=4)
    rng = random.Random(303)
    for _ in range(14):  # patched cargo plates
        x, y = rng.randrange(0, 480), rng.randrange(0, 230)
        c.rect(x, y, x + rng.randrange(20, 50), y + rng.randrange(14, 30), (0.55, 0.40, 0.14), 0.45)
    panel_lines(c, [48, 128, 208, 288, 368, 448], [64, 192], (0.22, 0.13, 0.05), (0.9, 0.75, 0.4), 8)
    for side in (0.25, 0.75):  # the Cartel's ringed diamond
        cx, cy, s = 260, 256 * side, 22
        c.polygon([(cx, cy - s), (cx + s, cy), (cx, cy + s), (cx - s, cy)], brown)
        c.disc(cx, cy, s * 0.62, mustard)
        c.disc(cx, cy, s * 0.45, brown)
        c.disc(cx, cy, s * 0.2, mustard)
        for i in range(4):
            # Just above the side line: toward row 0 on the left side, row H on the right.
            porthole(c, 90 + i * 34, cy - 15 if side < 0.5 else cy + 15, 6, (0.75, 0.6, 0.3), (1.0, 0.8, 0.45),
                     (1.0, 0.7, 0.3))
    # Rust streaks running back along the hull: noise stretched 10x along u, made by
    # sampling a narrow noise image and interpolating across it.
    g = noise_grid(304, c.w // 10 + 2, c.h, 6, 3, step=2)

    def rust(x, y, p):
        i, t = divmod(x, 10)
        v = g[y][i] + (g[y][i + 1] - g[y][i]) * t / 10
        return mix(p, (0.42, 0.2, 0.08), 0.35 * max(0.0, v - 0.55) * 2.2)

    c.shade(rust)
    grime(c, 305, 0.15)
    return c


def raider_hull():
    """Black Comet raiders: gloss black with hot-rod flames licking back from the nose."""
    c = Canvas(512, 256)
    black = (0.07, 0.06, 0.06)
    c.shade(lambda x, y, p: black)
    nf = fbm(401, 3)
    wobble = [nf(y / 9, 0.5) for y in range(c.h)]  # one noise value per row
    yellow, orange, red = (1.0, 0.9, 0.3), (1.0, 0.5, 0.06), (0.85, 0.1, 0.05)

    def flame(x, y, p):
        # Each row's flame reaches back to a wavy limit: long tongues at the sides of
        # the hull, shorter on top and belly. Colour runs yellow -> orange -> red.
        side = abs(math.cos(2 * math.pi * y / 256))  # 1 at top/belly, 0 at the sides
        reach = 330 - 120 * side + 70 * math.sin(y / 256 * 2 * math.pi * 6) + 40 * (wobble[y] - 0.5)
        start = 70
        if x < start or x > reach:
            return p
        t = (x - start) / max(1, reach - start)
        col = mix(yellow, orange, min(1, t * 1.8)) if t < 0.55 else mix(orange, red, (t - 0.55) / 0.45)
        edge = min(1.0, (reach - x) / 6)
        return mix(p, col, edge)

    c.shade(flame)
    for side in (0.25, 0.75):  # the Black Comet: a white head with a streaming tail
        cx, cy = 420, 256 * side
        c.polygon([(cx, cy - 9), (cx + 70, cy - 2), (cx + 70, cy + 2), (cx, cy + 9)], (0.9, 0.9, 0.9))
        c.disc(cx, cy, 11, (0.95, 0.95, 0.95))
        c.disc(cx, cy, 6, (1.0, 0.45, 0.1))
    chrome_band(c, 490, 500)
    grime(c, 402, 0.08)
    return c


# ---------------------------------------------------------------------------------------
# Around-layout textures
# ---------------------------------------------------------------------------------------

def dominion_hull():
    c = Canvas(512, 256)
    violet, black, green = (0.40, 0.14, 0.60), (0.07, 0.06, 0.09), (0.55, 1.0, 0.25)
    H = 256
    # Rows from the bottom: v = 1 - y/H. Underside below v = 0.58, upper hull above.
    c.shade(lambda x, y, p: violet if (1 - y / H) > 0.58 else black)
    c.rect(0, H * (1 - 0.6), 512, H * (1 - 0.58) + 1, (0.6, 0.6, 0.7))  # rim seam
    for k in range(16):  # radial ribs, black on violet, violet on black
        x = k * 32
        c.rect(x, 0, x + 3, H * 0.4, (0.12, 0.04, 0.18))
        c.rect(x + 16, H * 0.44, x + 18, H, (0.25, 0.08, 0.35))
    y_ring = H * (1 - 0.25)  # the underside's engine ring
    c.rect(0, y_ring - 5, 512, y_ring + 5, green)
    for x in range(0, 512, 4):
        c.glow(x, y_ring, 12, green, 0.15, wrap=True)
    for k in range(32):  # a row of small green ports on the upper hull
        c.disc(k * 16 + 8, H * (1 - 0.66), 2.5, green, wrap=True)
    for cx in (128, 384):
        eclipse_sigil(c, cx, H * 0.2, 17, green, black, (0.62, 0.3, 0.85))
    grime(c, 501, 0.1)
    return c


def robot_tin():
    c = Canvas(256, 256)
    tin = (0.66, 0.66, 0.72)
    n = noise_grid(601, c.w, c.h, 30, 3)
    c.shade(lambda x, y, p: mix(tin, (0.55, 0.5, 0.66), n[y][x]))
    panel_lines(c, [64, 128, 192], [40, 128, 216], (0.25, 0.24, 0.3), (0.85, 0.85, 0.9), 8)
    for cx in (0, 128):  # chest panel facing the nose (u = 0), repeated at the back
        c.rect(cx - 26, 70, cx + 26, 118, (0.15, 0.12, 0.2), wrap=True)
        for i, col in enumerate([(1, 0.2, 0.2), (1, 0.85, 0.2), (0.4, 1, 0.3), (0.3, 0.7, 1)]):
            c.disc(cx - 18 + i * 12, 80, 4, col, wrap=True)
        c.disc(cx, 104, 10, (0.85, 0.85, 0.9), wrap=True)
        c.disc(cx, 104, 8, (0.1, 0.1, 0.12), wrap=True)
        c.rect(cx - 1, 96, cx + 1, 104, (0.55, 1.0, 0.25), wrap=True)
    grime(c, 602, 0.12, 16)
    return c


def guard_station():
    c = Canvas(512, 256)
    cream, red, warm = (0.93, 0.88, 0.72), (0.75, 0.12, 0.09), (1.0, 0.85, 0.45)
    c.shade(lambda x, y, p: cream)
    for x in range(0, 512, 32):  # deco pilasters
        c.rect(x, 0, x + 4, 256, red)
        c.rect(x + 4, 0, x + 5, 256, (0.6, 0.55, 0.45))
    rng = random.Random(701)
    for row in range(0, 256, 16):  # window rows, most lit
        for x in range(8, 512, 32):
            for k in range(3):
                lit = rng.random() < 0.8
                col = warm if lit else (0.2, 0.2, 0.25)
                c.rect(x + k * 7, row + 5, x + k * 7 + 4, row + 11, col)
    for y in (0, 128):
        c.rect(0, y, 512, y + 4, (0.82, 0.85, 0.9))
    grime(c, 702, 0.08)
    return c


# ---------------------------------------------------------------------------------------
# Planar textures
# ---------------------------------------------------------------------------------------

def prospector_hull():
    c = Canvas(256, 256)
    orange, grey = (0.93, 0.45, 0.08), (0.46, 0.48, 0.50)
    c.shade(lambda x, y, p: orange if (x // 128 + y // 128) % 2 == 0 else grey)
    panel_lines(c, [0, 64, 128, 192], [0, 64, 128, 192], (0.2, 0.15, 0.1), (0.85, 0.8, 0.7), 8)
    for y in range(200, 232):  # hazard stripes
        for x in range(256):
            if ((x + y) // 12) % 2 == 0:
                c.blend(x, y, (0.1, 0.1, 0.1))
            else:
                c.blend(x, y, (1.0, 0.8, 0.1))
    c.disc(64, 96, 14, (0.15, 0.15, 0.15))  # a stencilled pick-axe badge
    c.polygon([(52, 90), (76, 90), (66, 98), (62, 98)], (1.0, 0.8, 0.1))
    c.rect(62, 92, 66, 108, (1.0, 0.8, 0.1))
    grime(c, 801, 0.22, 20)
    return c


def asteroid_rock():
    c = Canvas(256, 256)
    n = noise_grid(901, c.w, c.h, 40, 5, step=2)
    c.shade(lambda x, y, p: mix((0.30, 0.27, 0.24), (0.58, 0.53, 0.47), n[y][x]))
    rng = random.Random(902)
    for _ in range(26):  # craters: a dark bowl with a lit rim
        x, y, r = rng.uniform(0, 256), rng.uniform(0, 256), rng.uniform(4, 18)
        c.disc(x, y, r + 1.5, (0.65, 0.6, 0.55), 0.5, wrap=True)
        c.disc(x, y, r, (0.2, 0.18, 0.16), 0.55, wrap=True)
    for _ in range(40):  # cosmium specks
        x, y = rng.uniform(0, 256), rng.uniform(0, 256)
        c.glow(x, y, 5, (0.4, 1.0, 0.8), 0.6, wrap=True)
        c.disc(x, y, 1.2, (0.7, 1.0, 0.9), wrap=True)
    return c


# ---------------------------------------------------------------------------------------
# Damage variants
# ---------------------------------------------------------------------------------------

def damaged(base, level, seed):
    """A copy of `base` with soot blotches (level 1, scorched) or heavier blackening and
    burn-through holes (level 2, burnt), or a wreck (level 3: a station module that has
    been destroyed but stays mounted - charred almost black, paint drained of colour,
    cracked, with ember-lit holes). Blotches are noise-edged discs; positions wrap
    around the texture edges so the seam of a lathed hull does not cut them off."""
    c = base.copy()
    rng = random.Random(seed)
    n = noise_grid(seed + 1, c.w, c.h, 14)
    count = {1: 9, 2: 18, 3: 26}[level]
    blots = [(rng.uniform(0, c.w), rng.uniform(0, c.h), rng.uniform(0.07, 0.16) * c.w) for _ in range(count)]
    soot = (0.05, 0.04, 0.035)
    strength = {1: 0.8, 2: 0.95, 3: 1.0}[level]
    overall = {1: 0.0, 2: 0.28, 3: 0.55}[level]
    grey = 0.7 if level == 3 else 0.0  # how far the paint is drained toward grey

    # Build the soot mask blot by blot, visiting only the pixels near each blot.
    mask = [[0.0] * c.w for _ in range(c.h)]
    for bx, by, r in blots:
        reach = int(r * 1.4) + 1
        for yy in range(int(by) - reach, int(by) + reach + 1):
            y = yy % c.h
            row = mask[y]
            for xx in range(int(bx) - reach, int(bx) + reach + 1):
                d = math.hypot(xx - bx, yy - by) / r
                if d < 1.4:
                    x = xx % c.w
                    m = 1 - d + 0.6 * (n[y][x] - 0.5)
                    if m > row[x]:
                        row[x] = m

    def shade(x, y, p):
        m = max(0.0, min(1.0, mask[y][x] * 1.6)) * strength
        lum = 0.3 * p[0] + 0.55 * p[1] + 0.15 * p[2]
        q = [(v + (lum - v) * grey) * (1 - overall) for v in p]
        return mix(q, soot, m)

    c.shade(shade)
    if level == 3:
        for _ in range(9):  # cracks: dark random walks across the plating
            x, y = rng.uniform(0, c.w), rng.uniform(0, c.h)
            a = rng.uniform(0, 2 * math.pi)
            for _ in range(rng.randrange(25, 60)):
                c.disc(x, y, 1.1, (0.0, 0.0, 0.0), wrap=True)
                a += rng.uniform(-0.6, 0.6)
                x, y = x + math.cos(a) * 2, y + math.sin(a) * 2
    if level >= 2:
        for _ in range(6 if level == 2 else 10):  # burn-through holes with glowing rims
            x, y, r = rng.uniform(0, c.w), rng.uniform(0, c.h), rng.uniform(0.008, 0.02) * c.w
            c.glow(x, y, r * 2.2, (1.0, 0.45, 0.08), 0.5, wrap=True)
            c.disc(x, y, r * 1.25, (0.75, 0.25, 0.04), wrap=True)
            c.disc(x, y, r, (0.0, 0.0, 0.0), wrap=True)
    return c


def module_drum(base, band, trim, flute, seed, badge=None):
    """Around-layout texture for a lathed station module (u around, v up the profile):
    vertical fluting (the art-deco column look), two coloured bands with trim lines and
    rivets, and an optional badge painter called at two places around."""
    c = Canvas(256, 256)
    c.shade(lambda x, y, p: base if (x // 8) % 2 == 0 else mix(base, flute, 0.35))
    for y0, y1 in ((40, 72), (176, 200)):
        c.rect(0, y0, 256, y1, band)
        c.rect(0, y0 - 3, 256, y0, trim)
        c.rect(0, y1, 256, y1 + 3, trim)
        for x in range(4, 256, 10):
            c.disc(x, y0 + 5, 1.2, trim, wrap=True)
            c.disc(x, y1 - 5, 1.2, trim, wrap=True)
    if badge:
        for cx in (64, 192):
            badge(c, cx, 128)
    grime(c, seed, 0.1, 18)
    return c


def guard_turret():
    cream, red, chrome = (0.93, 0.88, 0.72), (0.78, 0.11, 0.08), (0.85, 0.87, 0.92)
    return module_drum(cream, red, chrome, (0.6, 0.5, 0.35), 1101,
                       lambda c, x, y: winged_star(c, x, y, 13, (0.95, 0.9, 0.75), (0.08, 0.62, 0.6), (0.1, 0.1, 0.12)))


def guard_shield_generator():
    chrome, teal, red = (0.80, 0.83, 0.88), (0.08, 0.62, 0.60), (0.78, 0.11, 0.08)
    return module_drum(chrome, teal, red, (0.4, 0.42, 0.5), 1201)


def dominion_turret_pod():
    violet, black, green = (0.40, 0.14, 0.60), (0.07, 0.06, 0.09), (0.55, 1.0, 0.25)
    return module_drum(violet, black, green, (0.12, 0.04, 0.18), 1301,
                       lambda c, x, y: eclipse_sigil(c, x, y, 14, green, black, (0.62, 0.3, 0.85)))


def dominion_shield_pod():
    """The shield pod's glowing core: swirling bands of acid green energy."""
    c = Canvas(256, 256)
    n = noise_grid(1401, 256, 256, 32, 4, step=2)
    dark, mid, hot = (0.05, 0.25, 0.05), (0.35, 0.85, 0.15), (0.85, 1.0, 0.6)

    def energy(x, y, p):
        # Bands that twist with the noise: a sine of height bent by the noise value.
        t = 0.5 + 0.5 * math.sin(y / 256 * 2 * math.pi * 5 + n[y][x] * 9)
        return mix(dark, mid, t) if t < 0.8 else mix(mid, hot, (t - 0.8) / 0.2)

    c.shade(energy)
    return c


def sky_eye_probe():
    chrome, red, teal = (0.84, 0.86, 0.91), (0.78, 0.11, 0.08), (0.08, 0.62, 0.60)
    return module_drum(chrome, red, teal, (0.55, 0.58, 0.66), 1501,
                       lambda c, x, y: winged_star(c, x, y, 12, (0.95, 0.9, 0.75), teal, (0.1, 0.1, 0.12)))


def jammer_buoy():
    violet, black, green = (0.40, 0.14, 0.60), (0.07, 0.06, 0.09), (0.55, 1.0, 0.25)
    return module_drum(violet, black, green, (0.05, 0.02, 0.08), 1601)


def station_skin(base, pilaster, band, window, seed, row_step=16, stripe=None):
    """A Guard facility skin in the around layout (256 square): pilasters, rows of lit
    windows, chrome bands and an optional hazard stripe band - the guard_station look
    in other colours, so each facility reads as its own building."""
    c = Canvas(256, 256)
    c.shade(lambda x, y, p: base)
    for x in range(0, 256, 32):
        c.rect(x, 0, x + 4, 256, pilaster)
        c.rect(x + 4, 0, x + 5, 256, mix(pilaster, (0, 0, 0), 0.4))
    rng = random.Random(seed)
    for row in range(0, 256, row_step):
        for x in range(8, 256, 32):
            for k in range(3):
                col = window if rng.random() < 0.75 else (0.2, 0.2, 0.25)
                c.rect(x + k * 7, row + 5, x + k * 7 + 4, row + 11, col)
    for y in (0, 128):
        c.rect(0, y, 256, y + 4, band)
    if stripe:
        for y in range(224, 240):
            for x in range(256):
                c.blend(x, y, (0.1, 0.1, 0.1) if ((x + y) // 8) % 2 == 0 else stripe)
    grime(c, seed + 1, 0.1, 18)
    return c


def mining_facility():
    """Ore refinery: rust-orange and cream with a hazard stripe."""
    return station_skin((0.86, 0.80, 0.66), (0.72, 0.32, 0.10), (0.82, 0.85, 0.9), (1.0, 0.8, 0.4), 1701,
                        stripe=(1.0, 0.78, 0.1))


def research_station():
    """Observatory: cream and teal with dense, cool-lit laboratory windows."""
    return station_skin((0.94, 0.91, 0.80), (0.08, 0.55, 0.55), (0.85, 0.87, 0.92), (0.75, 0.92, 1.0), 1801, row_step=12)


def manufacturing_center():
    """Orbital factory: red and cream with warm furnace-lit windows and hazard trim."""
    return station_skin((0.90, 0.84, 0.70), (0.72, 0.10, 0.08), (0.80, 0.82, 0.88), (1.0, 0.65, 0.3), 1901,
                        row_step=20, stripe=(1.0, 0.78, 0.1))


def cargo_pod():
    """Planar tile for cargo containers (8 m per tile): ribbed cream plating, a red
    band with a stencilled star, rivets and hazard-striped edges."""
    c = Canvas(256, 256)
    c.shade(lambda x, y, p: (0.92, 0.87, 0.72) if (x // 16) % 2 == 0 else (0.84, 0.79, 0.64))
    c.rect(0, 96, 256, 160, (0.76, 0.11, 0.08))
    c.polygon(star_points(128, 128, 22, 9), (0.95, 0.9, 0.75))
    for y in (0, 248):
        for yy in range(y, y + 8):
            for x in range(256):
                c.blend(x, yy, (0.1, 0.1, 0.1) if ((x + yy) // 8) % 2 == 0 else (1.0, 0.78, 0.1))
    panel_lines(c, [0, 128], [92, 164], (0.3, 0.25, 0.2), (0.8, 0.8, 0.85), 8)
    grime(c, 2001, 0.12, 20)
    return c


def scaffold():
    """Construction girders: diagonal yellow and black hazard bands with dark edges and
    bolt heads (u runs along a girder, v around it)."""
    c = Canvas(256, 256)
    c.shade(lambda x, y, p: (1.0, 0.80, 0.12) if ((x + y) // 32) % 2 == 0 else (0.09, 0.08, 0.07))
    for y in range(0, 256, 64):  # the girder's edges (four sides around)
        c.rect(0, y, 256, y + 4, (0.25, 0.22, 0.18))
    for x in range(8, 256, 32):
        for y in range(32, 256, 64):
            c.disc(x, y, 3, (0.55, 0.55, 0.58), wrap=True)
    grime(c, 2101, 0.2, 16)
    return c


# Station module textures also get a `_wrecked` variant (a destroyed module left in place).
MODULE_TEXTURES = {"guard_turret", "guard_shield_generator", "dominion_turret_pod", "dominion_shield_pod"}

TEXTURES = [
    ("guard_hull", guard_hull), ("senate_hull", senate_hull), ("cartel_hull", cartel_hull),
    ("raider_hull", raider_hull), ("dominion_hull", dominion_hull), ("robot_tin", robot_tin),
    ("guard_station", guard_station), ("prospector_hull", prospector_hull), ("asteroid_rock", asteroid_rock),
    ("guard_turret", guard_turret), ("guard_shield_generator", guard_shield_generator),
    ("dominion_turret_pod", dominion_turret_pod), ("dominion_shield_pod", dominion_shield_pod),
    ("sky_eye_probe", sky_eye_probe), ("jammer_buoy", jammer_buoy),
    ("mining_facility", mining_facility), ("research_station", research_station),
    ("manufacturing_center", manufacturing_center), ("cargo_pod", cargo_pod), ("scaffold", scaffold),
]


def build_all():
    """Paints and writes every texture and its damage variants. Returns the list of
    written file names."""
    os.makedirs(OUT, exist_ok=True)
    written = []
    for i, (name, painter) in enumerate(TEXTURES):
        base = painter()
        variants = [("", base), ("_scorched", damaged(base, 1, 1000 + i * 10)),
                    ("_burnt", damaged(base, 2, 2000 + i * 10))]
        if name in MODULE_TEXTURES:
            variants.append(("_wrecked", damaged(base, 3, 3000 + i * 10)))
        for suffix, img in variants:
            img.save(os.path.join(OUT, f"{name}{suffix}.png"))
            written.append((f"{name}{suffix}.png", img.w, img.h))
    return written


if __name__ == "__main__":
    for name, w, h in build_all():
        print(f"{name:28s} {w}x{h}")
