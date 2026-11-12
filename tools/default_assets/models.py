"""The ships, stations, rubble and cockpit of the `default` campaign.

Every model is assembled from the closed primitives in `common.py`:

* Rocket hulls, nose cones, exhaust bells, saucer discs, domes and tori are *lathed*
  profiles - a list of (axial position, radius) points swept around an axis. A profile
  point may switch material, which is how chrome trim bands are made.
* Fins, pylons, claws and deco plates are *prisms*: a flat outline given a thickness
  that can taper from root to tip.
* Barrels, struts, antennae and arm segments are `cylinder_between` two points.

The look is the 1950s film-serial / pulp-cover rocket: needle and cigar hulls with swept
fins and chrome bells for the Interplanetary Guard, domed saucers with rim lights and a
tin-can robot for the Umbral Dominion. Each design is original to this project.

Orientation: metres, +Y up, nose along -Z. Lathes are built around +Z, so a rocket's
profile runs from the nose (negative z) to the tail; saucers and towers are lathed the
same way and then turned upright with `Z_TO_Y` (which maps lathe +Z to world +Y and the
lathe's "top" seam, angle 0, to the nose at -Z).

Every model returned by `build_all()` is validated (closed, outward, non-degenerate,
within its triangle budget) before it is written; then it is cut into debris pieces.
"""
import math
import os
import random
import zlib

from common import (Model, Part, Face, lathe, torus, sphere, cylinder_between, prism, box, frame, icosphere, frame_from_z,
                    Z_TO_Y, IDENTITY, rot_x, scale_m, mat_apply, add, sub, mul, dot, cross, norm, length,
                    validate_model, write_model, make_debris, PACK)

# ---------------------------------------------------------------------------------------
# Material tables. Kd is the diffuse colour; it multiplies the texture, so a textured
# material uses white. Untextured materials are flat colours (chrome, glass, glows).
# ---------------------------------------------------------------------------------------

CHROME = (0.86, 0.88, 0.92)
GLOW_GREEN = (0.55, 1.0, 0.25)

GUARD = {
    "hull": ((1, 1, 1), "guard_hull.png"),
    "red": ((0.82, 0.12, 0.09), None),
    "cream": ((0.96, 0.91, 0.76), None),
    "chrome": (CHROME, None),
    "teal": ((0.10, 0.70, 0.66), None),
    "glass": ((0.55, 0.88, 1.0), None),
    "dark": ((0.10, 0.10, 0.12), None),
    "exhaust": ((1.0, 0.62, 0.18), None),
}
DOMINION = {
    "hull": ((1, 1, 1), "dominion_hull.png"),
    "violet": ((0.42, 0.16, 0.62), None),
    "black": ((0.07, 0.06, 0.09), None),
    "chrome": ((0.62, 0.60, 0.70), None),
    "glow": (GLOW_GREEN, None),
    "dome": ((0.62, 1.0, 0.55), None),
    "dark": ((0.05, 0.05, 0.06), None),
}
ROBOT = dict(DOMINION, tin=((1, 1, 1), "robot_tin.png"), eye=((1.0, 0.25, 0.85), None))
SENATE = {
    "hull": ((1, 1, 1), "senate_hull.png"),
    "white": ((0.95, 0.95, 0.97), None),
    "gold": ((0.95, 0.74, 0.25), None),
    "navy": ((0.10, 0.14, 0.38), None),
    "glass": ((0.55, 0.85, 1.0), None),
    "dark": ((0.10, 0.10, 0.12), None),
    "exhaust": ((0.70, 0.85, 1.0), None),
}
CARTEL = {
    "hull": ((1, 1, 1), "cartel_hull.png"),
    "mustard": ((0.86, 0.66, 0.16), None),
    "brown": ((0.36, 0.22, 0.12), None),
    "brass": ((0.80, 0.62, 0.32), None),
    "glass": ((0.95, 0.80, 0.45), None),
    "dark": ((0.10, 0.08, 0.06), None),
    "exhaust": ((1.0, 0.55, 0.2), None),
}
PROSPECTOR = {
    "hull": ((1, 1, 1), "prospector_hull.png"),
    "orange": ((0.95, 0.45, 0.08), None),
    "grey": ((0.48, 0.50, 0.52), None),
    "steel": ((0.70, 0.72, 0.74), None),
    "glass": ((0.95, 0.85, 0.40), None),
    "dark": ((0.10, 0.10, 0.10), None),
    "exhaust": ((1.0, 0.6, 0.2), None),
}
RAIDER = {
    "hull": ((1, 1, 1), "raider_hull.png"),
    "black": ((0.08, 0.07, 0.07), None),
    "flame": ((1.0, 0.42, 0.06), None),
    "chrome": (CHROME, None),
    "glass": ((1.0, 0.55, 0.25), None),
    "dark": ((0.06, 0.05, 0.05), None),
    "exhaust": ((1.0, 0.75, 0.25), None),
}
STATION = dict(GUARD, hull=((1, 1, 1), "guard_station.png"), lights=((1.0, 0.92, 0.55), None))
ROCK = {
    "rock": ((1, 1, 1), "asteroid_rock.png"),
    "crystal": ((0.35, 1.0, 0.75), None),
    "metal": ((0.45, 0.45, 0.48), None),
}

# Attachment points are never drawn; the colour only makes them easy to spot in a viewer.
SLOT = {"slot": ((1.0, 0.0, 1.0), None)}
GUARD_TURRET = dict(GUARD, hull=((1, 1, 1), "guard_turret.png"))
GUARD_SHIELD = dict(GUARD, hull=((1, 1, 1), "guard_shield_generator.png"))
DOMINION_TURRET = dict(DOMINION, hull=((1, 1, 1), "dominion_turret_pod.png"))
DOMINION_SHIELD = dict(DOMINION, hull=((1, 1, 1), "dominion_shield_pod.png"))

# Triangle budgets: the validator fails a model that grows past these.
FIGHTER, CAPITAL = 1500, 6000


# ---------------------------------------------------------------------------------------
# Rocket parts
# ---------------------------------------------------------------------------------------

def ogive(z_tip, z_base, r_base, n, mat, power=0.72):
    """Profile points of a rounded nose cone from its tip (r = 0) to its base. The radius
    follows sin(t * 90 deg) ** power: a power below 1 gives the bullet-like curve of a
    pulp-cover rocket rather than a straight cone."""
    out = []
    for i in range(n + 1):
        t = i / n
        out.append((z_tip + (z_base - z_tip) * t, r_base * math.sin(t * math.pi / 2) ** power, mat))
    return out


def bell(name, z0, length_, r_throat, r_exit, segments=16, out="chrome", inner="dark", glow="exhaust"):
    """An exhaust bell opening toward +Z (the tail), as a closed solid: the outer flare,
    a lip, the inner wall running back into the bell, and a glowing disc at the bottom
    (the 'fire'). The inner wall is traversed back toward -Z, and the lathe's normal
    rule turns its normals toward the axis - out of the solid, into the bell's mouth."""
    L = length_
    prof = [(z0, r_throat, out), (z0 + 0.45 * L, r_throat * 1.12, out), (z0 + L, r_exit, out),
            (z0 + L, r_exit * 0.86, inner), (z0 + 0.4 * L, r_throat * 0.72, glow), (z0 + 0.4 * L, 0.0, glow)]
    return lathe(name, prof, out, segments)


def fin(name, angle_deg, outline, mat, th_root=0.18, th_tip=0.06, root_s=None, centre=(0, 0, 0), axis="z"):
    """A tail fin standing out from a hull's axis at `angle_deg` (0 = straight up,
    measured toward +X like the lathe). `outline` is the fin's shape in (s, t) = (distance
    from the axis, position along the axis); corners at s <= root_s get the root
    thickness and the others the tip thickness, so fins taper to a crisp edge."""
    a = math.radians(angle_deg)
    if axis == "z":
        S, T = (math.sin(a), math.cos(a), 0.0), (0.0, 0.0, 1.0)
    else:  # an upright (Y-axis) body: s is horizontal, t runs up
        S, T = (math.sin(a), 0.0, -math.cos(a)), (0.0, 1.0, 0.0)
    rs = root_s if root_s is not None else min(p[0] for p in outline) + 1e-6
    th = [th_root if p[0] <= rs else th_tip for p in outline]
    return prism(name, outline, th, mat, origin=centre, s_axis=S, t_axis=T)


def dome(name, z0, r, h, mat, segments=16, n=5, uv=None):
    """A quarter-ellipse dome rising along +Z from a flat base of radius r."""
    prof = [(z0 + h * math.sin(i / n * math.pi / 2), r * math.cos(i / n * math.pi / 2)) for i in range(n + 1)]
    prof[-1] = (prof[-1][0], 0.0)
    return lathe(name, prof, mat, segments, uv=uv, uv_mode="around")


def ellipsoid(name, centre, radii, mat, segments=12, rings=6):
    """A sphere scaled to the given (x, y, z) radii: canopies, pods, heads."""
    return sphere(name, 1.0, mat, segments, rings).transformed(scale_m(*radii), centre)


def gun(name, a, b, r, mat="chrome", tip_mat="teal", tip_r=None, segments=6):
    """A ray-gun barrel from `a` to the muzzle `b`, with two cooling rings and an emitter
    ball at the muzzle - the finned look of a pulp-magazine blaster."""
    parts = [cylinder_between(name, a, b, r, mat, segments)]
    for f in (0.35, 0.6):
        c = add(a, mul(sub(b, a), f))
        d = mul(norm(sub(b, a)), r * 0.6)
        parts.append(cylinder_between(name + "_ring", sub(c, d), add(c, d), r * 1.7, mat, segments))
    parts.append(sphere(name + "_tip", tip_r or r * 1.6, tip_mat, 8, 4).transformed(IDENTITY, b))
    return parts


def accordion(name, a, b, r, mat, bumps=3, segments=6):
    """A corrugated tube from a to b (robot arms, flexible couplings): the profile
    alternates between two radii."""
    L = length(sub(b, a))
    prof = []
    for i in range(bumps * 2 + 1):
        prof.append((L * i / (bumps * 2), r if i % 2 == 0 else r * 1.3))
    return lathe(name, prof, mat, segments, crease=30).transformed(frame_from_z(sub(b, a)), a)


# ---------------------------------------------------------------------------------------
# Interplanetary Guard rockets
# ---------------------------------------------------------------------------------------

def teardrop(z_nose, z_tail, r_max, nose_mat="chrome", band_at=0.42, tail_r=None):
    """Profile of a retro-rocket body: a rounded chrome nose swelling quickly to a fat
    belly (widest about 40 % back), then tapering to the tail - the bulbous "bullet with
    fins" of the 1930s serials. A chrome band rings the widest point."""
    length = z_tail - z_nose
    tail_r = r_max * 0.55 if tail_r is None else tail_r
    z_base = z_nose + length * 0.16
    prof = ogive(z_nose, z_base, r_max * 0.62, 4, nose_mat)
    z_w = z_nose + length * band_at
    pts = []
    for k in range(1, 5):  # nose base to the belly: a smooth swell
        f = k / 4
        z = z_base + (z_w - z_base) * f
        pts.append((z, r_max * (0.62 + 0.38 * math.sin(f * math.pi / 2)), "hull"))
    pts += [(z_w + 0.08, r_max * 1.04, "chrome"), (z_w + 0.4, r_max * 1.04, "chrome"), (z_w + 0.48, r_max, "hull")]
    for k in range(1, 5):  # belly to the tail: an even taper
        f = k / 4
        z = z_w + 0.48 + (z_tail - z_w - 0.48) * f
        r = r_max + (tail_r - r_max) * (f ** 1.6)
        pts.append((z, r, "chrome" if k == 4 else "hull"))
    return prof + pts


def comet():
    """Light fighter, ~11 m: a bulbous teardrop hull with a chrome nose and band, bubble
    canopy, three swept tail fins and two wing-stub ray guns."""
    m = Model("comet", GUARD, FIGHTER)
    R = 1.5
    m.add(lathe("hull", teardrop(-5.6, 4.5, R), "hull", 18, uv=(0, 1, 0, 1)))
    m.add(bell("bell", 4.4, 1.2, 0.6, 0.95))
    m.add(ellipsoid("canopy", (0, R * 0.78, -2.0), (0.55, 0.5, 1.15), "glass"))
    outline = [(R * 0.7, 0.8), (R * 0.7, 4.2), (R + 1.9, 5.9), (R + 1.9, 5.1)]
    for a in (0, 120, 240):
        m.add(fin(f"fin{a}", a, outline, "red", th_root=0.26, th_tip=0.08))
    m.add(sphere("fin_tip", 0.2, "teal", 8, 4).transformed(IDENTITY, (0, R + 1.95, 5.5)))
    stub = prism("stub", [(R * 0.85, -2.4), (R * 0.85, -0.4), (R + 0.75, -0.8), (R + 0.75, -2.1)], [0.3, 0.3, 0.12, 0.12],
                 "red", origin=(0, -0.3, 0), s_axis=(1, 0, 0), t_axis=(0, 0, 1))
    m.add_mirrored_x(stub)
    for p in gun("gun", (R + 0.75, -0.3, -0.6), (R + 0.75, -0.3, -3.4), 0.1):
        m.add_mirrored_x(p)
    return m


def meteor():
    """Medium fighter, ~14 m: a bulbous teardrop hull flanked by twin booster pods on
    pylons that are broad and thick at the body and taper towards the boosters."""
    m = Model("meteor", GUARD, FIGHTER)
    R = 1.8
    m.add(lathe("hull", teardrop(-7.0, 5.3, R), "hull", 16, uv=(0, 1, 0, 1)))
    m.add(bell("bell", 5.2, 1.4, 0.7, 1.1))
    m.add(ellipsoid("canopy", (0, R * 0.8, -3.0), (0.62, 0.56, 1.3), "glass"))
    m.add(fin("fin_top", 0, [(R * 0.75, 1.8), (R * 0.75, 5.0), (R + 1.8, 6.6), (R + 1.8, 5.8)], "red",
              th_root=0.3, th_tip=0.1))
    # One booster, built at the origin then moved out to the right; mirrored for the left.
    bprof = ogive(-2.2, -0.9, 0.5, 3, "teal") + [(-0.9, 0.54, "hull"), (3.6, 0.54, "hull"), (4.0, 0.42, "chrome")]
    pod = [lathe("booster", bprof, "hull", 12, uv=(0.1, 0.9, 0, 1)),
           bell("booster_bell", 3.9, 0.8, 0.34, 0.52, 12),
           fin("booster_fin", 90, [(0.36, 1.8), (0.36, 3.8), (1.4, 4.6), (1.4, 4.1)], "red"),
           fin("booster_fin_lo", 180, [(0.36, 1.8), (0.36, 3.8), (1.2, 4.5), (1.2, 4.0)], "red")]
    bx = R + 1.35
    off = (bx, -0.45, 0.6)
    for p in pod:
        m.add_mirrored_x(p.transformed(IDENTITY, off))
    # Pylon: a long, thick root along the hull narrowing to a short, thin joint at the pod.
    root, tip = R * 0.8, bx - 0.45
    pylon = prism("pylon", [(root, -2.2), (root, 3.0), (tip, 2.0), (tip, 0.2)], [0.62, 0.62, 0.2, 0.2], "chrome",
                  origin=(0, -0.4, 0), s_axis=(1, 0, 0), t_axis=(0, 0, 1))
    m.add_mirrored_x(pylon)
    for p in gun("gun", (0.0, -R * 0.92, -2.0), (0.0, -R * 0.92, -4.8), 0.12):
        m.add(p)
    return m


def thunderbolt():
    """Heavy fighter, ~18 m: a big bulbous teardrop with four swept fins in an X, each
    ending in a gun pod, a twin-seat canopy and a chin turret."""
    m = Model("thunderbolt", GUARD, FIGHTER)
    R = 2.4
    m.add(lathe("hull", teardrop(-9.0, 6.0, R), "hull", 16, uv=(0, 1, 0, 1)))
    m.add(bell("bell", 5.9, 2.0, 1.05, 1.5, 16))
    m.add(ellipsoid("canopy", (0, R * 0.8, -3.8), (0.8, 0.72, 2.0), "glass"))
    outline = [(R * 0.7, 0.6), (R * 0.7, 5.6), (R + 2.6, 8.6), (R + 2.6, 7.0)]
    for a in (45, 135, 225, 315):
        m.add(fin(f"fin{a}", a, outline, "red", th_root=0.4, th_tip=0.14))
        r = math.radians(a)
        c = ((R + 2.6) * math.sin(r), (R + 2.6) * math.cos(r), 0)
        pod = lathe("finpod", ogive(-2.3, -1.5, 0.3, 2, "teal") + [(-1.5, 0.32, "chrome"), (2.0, 0.32, "chrome"),
                                                                     (2.3, 0.2, "teal")], "chrome", 8)
        m.add(pod.transformed(IDENTITY, (c[0], c[1], 7.4)))
    m.add(sphere("turret", 0.7, "chrome", 10, 5).transformed(IDENTITY, (0, -R * 0.88, -4.3)))
    for x in (-0.28, 0.28):
        for p in gun("chin", (x, -R * 0.95, -4.4), (x, -R * 0.95, -7.0), 0.11):
            m.add(p)
    return m


def victory():
    """Flagship, ~120 m: an art-deco battleship-rocket - a long hull with chrome bands,
    a stepped bridge tower, deco speed-line fins, a collar ring, deck turrets, four
    giant tail fins and a cluster of five exhaust bells."""
    m = Model("victory", GUARD, CAPITAL)
    prof = ogive(-60, -44, 7.2, 6, "chrome") + [(-44, 7.6, "hull")]
    z = -44
    for i in range(6):  # body bays separated by stepped chrome bands
        z1 = z + 13
        prof += [(z1 - 1.2, 9.0 - abs(i - 2.5) * 0.35, "chrome"), (z1 - 1.0, 9.6 - abs(i - 2.5) * 0.35, "chrome"),
                 (z1, 9.6 - abs(i - 2.5) * 0.35, "hull"), (z1 + 0.2, 9.0 - abs(i - 2.5) * 0.35, "hull")]
        z = z1
    prof += [(42, 8.0, "hull"), (48, 6.4, "chrome"), (50, 5.6, "chrome")]
    m.add(lathe("hull", prof, "hull", 24, uv=(0, 5, 0, 1)))
    # Five bells: a big centre one and four around it.
    m.add(bell("bell", 49, 9, 3.6, 5.2, 20))
    for a in (45, 135, 225, 315):
        r = math.radians(a)
        m.add(bell("bell_s", 46, 7, 1.6, 2.4, 12).transformed(IDENTITY, (5.2 * math.sin(r), 5.2 * math.cos(r), 0)))
    # Stepped bridge tower (art-deco setbacks) on the dorsal spine.
    y = 6.0
    for i, (w, h, z0, z1) in enumerate([(11, 6, -10, 22), (8.5, 5, -6, 17), (6, 4, -2, 12), (3.6, 3.5, 1, 8)]):
        m.add(box(f"bridge{i}", -w / 2, w / 2, y, y + h, z0, z1, "cream" if i % 2 == 0 else "red"))
        m.add(box(f"bridge_trim{i}", -w / 2 - 0.3, w / 2 + 0.3, y + h - 0.4, y + h, z0 - 0.3, z1 + 0.3, "chrome"))
        y += h
    m.add(box("bridge_glass", -4.0, 4.0, 13.4, 14.8, -6.6, -5.8, "glass"))
    m.add(cylinder_between("mast", (0, y, 4.5), (0, y + 14, 4.5), 0.5, "chrome", 6, radius_b=0.0))
    m.add(sphere("mast_ball", 0.9, "teal", 8, 4).transformed(IDENTITY, (0, y + 5, 4.5)))
    # Three deco speed-line fins along the spine ahead of the bridge.
    for x in (-2.5, 0, 2.5):
        m.add(prism("speedline", [(0, -38), (0, -8), (3.4 - abs(x) * 0.4, -10), (1.0, -36)], 0.5, "chrome",
                    origin=(x, 7.0, 0), s_axis=(0, 1, 0), t_axis=(0, 0, 1)))
    # Collar ring on four struts.
    m.add(torus("collar", 15.5, 1.2, "chrome", 32, 6).transformed(IDENTITY, (0, 0, 22)))
    for a in (0, 90, 180, 270):
        r = math.radians(a)
        d = (math.sin(r), math.cos(r), 0)
        m.add(cylinder_between("collar_strut", add(mul(d, 7.5), (0, 0, 22)), add(mul(d, 15), (0, 0, 22)), 0.7, "red", 6))
    # Deck turrets: a domed base with twin barrels, top and bottom.
    for i, (zt, up) in enumerate([(-30, 1), (-18, 1), (-30, -1), (-14, -1), (30, 1)]):
        base_y = up * 7.6
        t = dome("turret", 0, 3.6, 2.8, "red", 12, 3)
        t = t.transformed(rot_x(-90) if up > 0 else rot_x(90), (0, base_y, zt))
        m.add(t)
        for x in (-1.1, 1.1):
            for p in gun("turret_gun", (x, base_y + up * 1.7, zt), (x, base_y + up * 1.7, zt - 11), 0.45):
                m.add(p)
    # Four giant tail fins in an X with teal tip pods.
    outline = [(7.0, 14), (7.0, 44), (24, 58), (24, 47)]
    for a in (45, 135, 225, 315):
        m.add(fin(f"fin{a}", a, outline, "red", th_root=1.4, th_tip=0.6))
        r = math.radians(a)
        m.add(sphere("fintip", 1.6, "teal", 10, 5).transformed(scale_m(1, 1, 3.2), (24 * math.sin(r), 24 * math.cos(r), 52)))
    return m


# ---------------------------------------------------------------------------------------
# Umbral Dominion saucers and robots
# ---------------------------------------------------------------------------------------

def saucer_profile(R, h_bot, h_top, r_top):
    """A classic saucer cross-section from the bottom centre up to the dome ring: a
    shallow underside with an engine ring, a chrome rim at the widest point, then a
    convex upper hull. Returned in lathe (axial, radius) terms - the axis becomes +Y."""
    return [(-h_bot, 0.0, "hull"), (-h_bot * 0.97, R * 0.18, "glow"), (-h_bot * 0.9, R * 0.3, "hull"),
            (-h_bot * 0.62, R * 0.55, "hull"), (-h_bot * 0.3, R * 0.82, "hull"), (-h_bot * 0.08, R * 0.96, "chrome"),
            (0.0, R, "chrome"), (h_top * 0.12, R * 0.97, "hull"), (h_top * 0.4, R * 0.82, "hull"),
            (h_top * 0.75, R * 0.6, "hull"), (h_top, r_top, "hull")]


def upright(part, y=0.0, x=0.0, z=0.0):
    """Stands a Z-axis lathe up along +Y and moves it."""
    return part.transformed(Z_TO_Y, (x, y, z))


def rim_lights(m, R, y, count, size, segs=6):
    for k in range(count):
        a = 2 * math.pi * (k + 0.5) / count
        m.add(sphere("rim_light", size, "glow", segs, 3).transformed(IDENTITY, (R * math.sin(a), y, -R * math.cos(a))))


def scout_saucer():
    """Dominion scout, ~12 m across: a domed disc with rim lights and an antenna."""
    m = Model("scout_saucer", DOMINION, FIGHTER)
    m.add(upright(lathe("disc", saucer_profile(6.0, 1.1, 1.0, 2.5), "hull", 24, uv=(0, 1, 0, 1),
                        uv_mode="around", u_repeat=2)))
    m.add(upright(dome("dome", 0.95, 2.55, 1.9, "dome", 20)))
    rim_lights(m, 6.02, 0.0, 12, 0.22)
    m.add(cylinder_between("antenna", (0, 2.6, 0), (0, 4.2, 0), 0.06, "chrome", 6))
    m.add(sphere("antenna_ball", 0.18, "glow", 8, 4).transformed(IDENTITY, (0, 4.25, 0)))
    for a in (60, 180, 300):  # three short spikes on the upper hull
        r = math.radians(a)
        b = (3.6 * math.sin(r), 0.75, -3.6 * math.cos(r))
        m.add(cylinder_between("spike", b, add(b, (1.6 * math.sin(r), 1.1, -1.6 * math.cos(r))), 0.12, "black", 6,
                               radius_b=0.0))
    return m


def raider_saucer():
    """Dominion raider, ~16 m: a heavier saucer with a ray-gun turret on the front of
    the upper hull, under the dome."""
    m = Model("raider_saucer", DOMINION, FIGHTER)
    m.add(upright(lathe("disc", saucer_profile(8.0, 1.5, 1.4, 3.2), "hull", 24, uv=(0, 1, 0, 1),
                        uv_mode="around", u_repeat=3)))
    m.add(upright(dome("dome", 1.3, 3.25, 2.3, "dome", 20)))
    m.add(upright(torus("dome_ring", 3.3, 0.18, "chrome", 24, 4), y=1.35))
    rim_lights(m, 8.03, 0.0, 12, 0.26)
    # The turret: a black ball half sunk into the hull at the front, with a ray gun.
    m.add(sphere("turret", 0.9, "black", 10, 5).transformed(IDENTITY, (0, 0.95, -4.6)))
    for p in gun("ray", (0, 1.1, -5.2), (0, 1.25, -9.0), 0.16, "chrome", "glow", 0.34):
        m.add(p)
    for a in (120, 240):
        r = math.radians(a)
        b = (4.6 * math.sin(r), 0.95, -4.6 * math.cos(r))
        m.add(cylinder_between("spike", b, add(b, (2.4 * math.sin(r), 1.6, -2.4 * math.cos(r))), 0.16, "black", 6,
                               radius_b=0.0))
    m.add(cylinder_between("antenna", (0, 3.4, 0), (0, 5.4, 0), 0.07, "chrome", 6))
    m.add(sphere("antenna_ball", 0.22, "glow", 8, 4).transformed(IDENTITY, (0, 5.45, 0)))
    return m


def dreadnought():
    """Dominion capital ship, ~70 m: three stacked discs, a crown dome, rim spikes,
    turrets, and the green 'eclipse lens' emitter underneath."""
    m = Model("dreadnought", DOMINION, CAPITAL)
    m.add(upright(lathe("tier1", saucer_profile(35, 5, 4, 22), "hull", 40, uv=(0, 1, 0, 1), uv_mode="around",
                        u_repeat=6)))
    m.add(upright(lathe("tier2", [(0, 23, "black"), (1.5, 23.5, "chrome"), (2.5, 22.5, "hull"), (6, 15, "hull"),
                                  (6.5, 14, "hull")], "hull", 32, uv=(0, 1, 0.55, 1), uv_mode="around", u_repeat=4),
                  y=3.6))
    m.add(upright(lathe("tier3", [(0, 14.5, "black"), (1.0, 15, "chrome"), (2.0, 14, "violet"), (5, 9, "violet"),
                                  (5.5, 8.6, "violet")], "violet", 24), y=9.9))
    m.add(upright(dome("crown", 15.2, 8.7, 7.5, "dome", 24)))
    m.add(cylinder_between("mast", (0, 22.5, 0), (0, 34, 0), 0.6, "black", 6, radius_b=0.0))
    # The underside lens: a hanging cone with a glowing tip.
    m.add(upright(lathe("lens_mount", [(-13, 0.0, "glow"), (-12, 2.5, "black"), (-9, 3.5, "black"),
                                       (-4.5, 6, "black")], "black", 20)))
    rim_lights(m, 35.05, 0.0, 24, 0.9, 6)
    for k in range(12):  # rim spikes
        a = 2 * math.pi * k / 12
        d = (math.sin(a), 0, -math.cos(a))
        m.add(cylinder_between("rim_spike", mul(d, 33.5), mul(d, 42.0), 1.1, "black", 6, radius_b=0.0))
    for k in range(4):  # turrets on tier 2
        a = 2 * math.pi * (k + 0.5) / 4
        d = (math.sin(a), 0, -math.cos(a))
        c = add(mul(d, 19), (0, 7.2, 0))
        m.add(sphere("turret", 2.2, "black", 10, 5).transformed(IDENTITY, c))
        for p in gun("turret_gun", add(c, (0, 1.0, 0)), add(add(c, mul(d, 7)), (0, 2.4, 0)), 0.4, "chrome", "glow", 0.8):
            m.add(p)
    return m


def robot_sentinel():
    """Dominion robot drone, ~6 m tall: a riveted tin-can body, a head with a glass dome
    and glowing eyes, accordion arms ending in claws, and a twin-tank jet pack."""
    m = Model("robot_sentinel", ROBOT, FIGHTER)
    body = [(-1.4, 0.0, "dark"), (-1.4, 0.45, "chrome"), (-0.9, 0.9, "chrome"), (-0.8, 1.1, "tin"),
            (1.2, 1.1, "chrome"), (1.35, 1.2, "chrome"), (1.6, 1.0, "tin"), (1.75, 0.55, "tin")]
    m.add(upright(lathe("body", body, "tin", 14, uv=(0, 1, 0, 1), uv_mode="around", u_repeat=2)))
    m.add(upright(lathe("thruster", [(-2.2, 0.0, "glow"), (-2.2, 0.38, "chrome"), (-1.4, 0.6, "chrome")], "chrome", 12)))
    m.add(upright(cylinder_between("neck", (0, 0, 1.7), (0, 0, 2.1), 0.32, "chrome", 10).transformed(IDENTITY)))
    head = [(2.05, 0.0, "tin"), (2.05, 0.72, "tin"), (2.75, 0.72, "chrome"), (2.85, 0.76, "chrome"), (2.95, 0.72, "tin")]
    m.add(upright(lathe("head", head, "tin", 12, uv=(0, 1, 0.2, 0.8), uv_mode="around")))
    m.add(upright(dome("brain_dome", 2.92, 0.66, 0.75, "dome", 12, 4)))
    m.add(cylinder_between("antenna", (0.25, 3.5, 0), (0.55, 4.6, 0), 0.04, "chrome", 5))
    m.add(sphere("antenna_bulb", 0.13, "eye", 6, 3).transformed(IDENTITY, (0.56, 4.66, 0)))
    for x in (-0.28, 0.28):
        m.add(sphere("eye", 0.15, "eye", 6, 3).transformed(IDENTITY, (x, 2.5, -0.66)))
    m.add(box("grille", -0.32, 0.32, 2.12, 2.3, -0.75, -0.6, "dark"))
    # Arms: shoulder ball, accordion upper arm, elbow ball, accordion forearm, claw.
    shoulder, elbow, wrist = (1.25, 1.0, 0.0), (1.7, 0.1, -0.9), (1.45, 0.0, -2.1)
    claw = [(0.0, 0.0), (0.35, 0.12), (0.65, 0.05), (0.9, -0.22), (0.78, -0.28), (0.55, -0.06), (0.25, -0.1)]
    arm = [sphere("shoulder", 0.36, "chrome", 8, 4).transformed(IDENTITY, shoulder),
           accordion("upper_arm", shoulder, elbow, 0.2, "chrome", 2),
           sphere("elbow", 0.27, "chrome", 6, 4).transformed(IDENTITY, elbow),
           accordion("forearm", elbow, wrist, 0.17, "chrome", 2),
           sphere("wrist", 0.24, "black", 6, 4).transformed(IDENTITY, wrist)]
    for up in (1, -1):  # two claw fingers, one bending up, one down
        outline = [(s, t * up) for s, t in claw]
        arm.append(prism("claw", outline, 0.12, "black", origin=add(wrist, (0, 0.08 * up, -0.1)),
                         s_axis=(0, 0, -1), t_axis=(0, 1, 0)))
    for p in arm:
        m.add_mirrored_x(p)
    # Jet pack: two tanks on the back with downward bells.
    for x in (-0.55, 0.55):
        tank = [(-0.7, 0.0, "violet"), (-0.6, 0.3, "violet"), (-0.45, 0.42, "violet"), (1.3, 0.42, "violet"),
                (1.6, 0.3, "violet"), (1.75, 0.0, "violet")]
        m.add(upright(lathe("tank", tank, "violet", 8), x=x, z=1.35))
        m.add(bell("pack_bell", 0, 0.6, 0.25, 0.4, 8, "chrome", "dark", "glow").transformed(
            Z_TO_Y, (0, 0, 0)).transformed(scale_m(1, -1, 1), (x, -0.55, 1.35)))
    m.add(box("pack_frame", -0.95, 0.95, 0.2, 1.1, 1.0, 1.25, "black"))
    # Built at a comfortable working size; scale the whole drone to ~6 m tall.
    m.parts = [p.transformed(scale_m(0.86, 0.86, 0.86)) for p in m.parts]
    return m


def grappler():
    """Dominion tug, ~25 m: a flat saucer with a forward cab dome and three jointed
    grappling arms with hooked claws hanging below."""
    m = Model("grappler", DOMINION, FIGHTER)
    m.add(upright(lathe("disc", saucer_profile(10.0, 1.6, 1.5, 4.0), "hull", 20, uv=(0, 1, 0, 1),
                        uv_mode="around", u_repeat=3)))
    m.add(upright(lathe("hub", [(1.4, 4.2, "black"), (2.6, 3.6, "violet"), (2.8, 3.2, "violet")], "violet", 16)))
    m.add(upright(dome("dome", 2.75, 3.2, 1.6, "dome", 16)))
    rim_lights(m, 10.03, 0.0, 10, 0.3)
    m.add(upright(lathe("winch", [(-2.5, 0.0, "glow"), (-2.5, 1.2, "black"), (-1.2, 2.0, "black")], "black", 12)))
    claw = [(0.0, 0.25), (1.6, 0.6), (2.6, -0.2), (2.8, -1.4), (2.4, -1.5), (2.1, -0.6), (1.3, -0.1), (0.0, -0.25)]
    for k in range(3):
        a = 2 * math.pi * k / 3
        d = (math.sin(a), 0, -math.cos(a))
        root = add(mul(d, 5.0), (0, -1.0, 0))
        knee = add(mul(d, 9.5), (0, -4.0, 0))
        hand = add(mul(d, 9.0), (0, -8.0, 0))
        m.add(sphere("joint", 0.9, "chrome", 6, 3).transformed(IDENTITY, root))
        m.add(cylinder_between("arm", root, knee, 0.5, "black", 6))
        m.add(sphere("joint", 0.75, "chrome", 6, 3).transformed(IDENTITY, knee))
        m.add(cylinder_between("arm", knee, hand, 0.42, "black", 6))
        m.add(sphere("joint", 0.6, "chrome", 6, 3).transformed(IDENTITY, hand))
        side = norm(cross(d, (0, 1, 0)))
        for sgn in (1, -1):
            # A hooked finger in the vertical plane through the arm, curling inward.
            s_axis = norm(add(mul(d, sgn * 0.8), (0, -0.6, 0)))
            t_axis = norm(cross(side, s_axis)) if sgn > 0 else norm(cross(s_axis, side))
            m.add(prism("claw", claw, 0.35, "chrome", origin=hand, s_axis=s_axis, t_axis=t_axis))
    return m


def cargo_orb():
    """Dominion freighter, ~40 m: a banded sphere with an engine cluster behind and a
    small cab saucer in front."""
    m = Model("cargo_orb", DOMINION, FIGHTER + 1000)
    m.add(upright(sphere("orb", 15.0, "hull", 24, 12, uv=(0, 1, 0, 1), u_repeat=4)))
    for y, r in ((0.0, 15.15), (7.5, 13.05), (-7.5, 13.05)):
        m.add(upright(torus("band", r, 0.7, "chrome", 28, 4), y=y))
    for x, y in ((0, 0), (5, 3), (-5, 3), (0, -5)):
        m.add(bell("engine", 0, 6, 2.0, 3.0, 12, "chrome", "dark", "glow").transformed(IDENTITY, (x, y, 13.5)))
    cab = [(-1.3, 0.0, "black"), (-1.0, 4.8, "black"), (0.0, 5.8, "chrome"), (0.6, 4.8, "violet"), (1.3, 2.6, "violet")]
    m.add(lathe("cab", cab, "violet", 16).transformed(Z_TO_Y, (0, 3.0, -18.0)))
    m.add(dome("cab_dome", 1.25, 2.7, 2.1, "dome", 14).transformed(Z_TO_Y, (0, 3.0, -18.0)))
    m.add(cylinder_between("cab_neck", (0, 2.4, -16.0), (0, 2.4, -13.0), 1.8, "black", 8))
    for a in (30, 150, 270):
        r = math.radians(a)
        d = (math.sin(r), math.cos(r), 0)
        m.add(cylinder_between("spike", mul(d, 14.0), mul(d, 20.0), 0.6, "black", 6, radius_b=0.0))
    return m


# ---------------------------------------------------------------------------------------
# Third parties
# ---------------------------------------------------------------------------------------

def senate_patrol():
    """Solar Senate patrol, ~14 m: a sleek white needle with a long chrome nose spike,
    gold bands, swept delta fins and gold canards."""
    m = Model("senate_patrol", SENATE, FIGHTER)
    prof = [(-7.0, 0.0, "gold"), (-6.2, 0.06, "gold"), (-5.6, 0.1, "white")] + ogive(-5.6, -3.4, 0.62, 4, "white")[1:] + [
        (-3.4, 0.64, "hull"), (-1.0, 0.82, "hull"), (-0.3, 0.84, "gold"), (0.0, 0.88, "gold"), (0.3, 0.84, "hull"),
        (3.5, 0.78, "hull"), (4.8, 0.6, "gold"), (5.2, 0.52, "gold")]
    m.add(lathe("hull", prof, "hull", 16, uv=(0, 1, 0, 1)))
    m.add(bell("bell", 5.1, 1.1, 0.42, 0.62, 16, "gold", "dark", "exhaust"))
    m.add(ellipsoid("canopy", (0, 0.52, -2.2), (0.36, 0.38, 1.1), "glass"))
    for a in (0, 110, 250):
        m.add(fin(f"fin{a}", a, [(0.5, 0.0), (0.5, 4.6), (2.6, 6.2), (2.6, 5.6)], "navy" if a else "white"))
    canard = [(0.4, -3.6), (0.4, -2.6), (1.4, -2.4), (1.4, -2.8)]
    m.add(fin("canard_r", 90, canard, "gold", 0.1, 0.04))
    m.add(fin("canard_l", 270, canard, "gold", 0.1, 0.04))
    return m


def cartel_freighter():
    """Nebula Cartel freighter, ~50 m: a bulbous pear-shaped hull, strapped cargo pods,
    stubby fins and one big bell."""
    m = Model("cartel_freighter", CARTEL, FIGHTER + 1000)
    prof = ogive(-25, -14, 8.5, 5, "hull") + [(-8, 10.5, "hull"), (-1, 11.0, "brass"), (0, 11.3, "brass"),
                                              (1.0, 11.0, "hull"), (8, 9.5, "hull"), (14, 6.5, "hull"),
                                              (17, 5.0, "brass"), (18, 4.6, "brass")]
    m.add(lathe("hull", prof, "hull", 20, uv=(0, 1, 0, 1)))
    m.add(bell("bell", 17.5, 7, 3.6, 5.6, 18, "brass", "dark", "exhaust"))
    m.add(box("bridge", -2.2, 2.2, 7.8, 11.4, -16, -9, "brown"))
    m.add(box("bridge_glass", -1.9, 1.9, 10.0, 10.9, -16.3, -15.9, "glass"))
    for a in (60, 180, 300):
        m.add(fin(f"fin{a}", a, [(5.5, 7), (5.5, 16), (11.5, 20), (11.5, 16.5)], "brown", 0.9, 0.5))
    pod_prof = ogive(-9, -6, 2.4, 3, "mustard") + [(-6, 2.5, "mustard"), (6, 2.5, "mustard")] + \
        [(z, r, "mustard") for z, r, _ in ogive(9, 6, 2.4, 3, "mustard")[::-1][1:]]
    for a in (90, 270, 135, 225):
        r = math.radians(a)
        rad = 11.6 if a in (90, 270) else 10.6
        c = (rad * math.sin(r), rad * math.cos(r), -1.0)
        m.add(lathe("pod", pod_prof, "mustard", 12).transformed(IDENTITY, c))
        for zz in (-4.0, 3.0):
            m.add(torus("strap", 2.55, 0.25, "brown", 12, 4).transformed(IDENTITY, add(c, (0, 0, zz))))
    return m


def prospector_hauler():
    """Belt Prospectors ore hauler, ~30 m: a boxy hull with a cab, an ore hopper, four
    corner thrusters and a big drill nose."""
    m = Model("prospector_hauler", PROSPECTOR, FIGHTER)
    m.add(box("hull", -5, 5, -3.5, 3.5, -7, 11, "hull", uv_scale=1 / 8))
    m.add(box("cab", -3.4, 3.4, 3.5, 6.0, -6.5, -1.5, "hull", uv_scale=1 / 8))
    m.add(box("cab_glass", -3.0, 3.0, 4.3, 5.5, -6.8, -6.4, "glass"))
    hopper = [(-4.6, 0), (4.6, 0), (3.6, 3.8), (-3.6, 3.8)]
    m.add(prism("hopper", hopper, 9.0, "grey", origin=(0, 3.5, 5.5), s_axis=(1, 0, 0), t_axis=(0, 1, 0)))
    m.add(box("ore", -3.4, 3.4, 6.9, 7.6, 1.4, 9.6, "dark"))
    for x0, x1 in ((-5.4, -5.0), (5.0, 5.4)):  # hazard rails along the sides
        m.add(box("rail", x0, x1, -1.0, 1.0, -6.5, 10.5, "orange"))
    # Drill: a stepped cone of grinding rings ending in a point.
    drill = [(-7.2, 3.2, "grey"), (-8.4, 3.0, "steel")]
    z, r = -8.4, 3.0
    for i in range(5):
        drill += [(z - 0.9, r * 0.82, "steel"), (z - 0.9, r * 0.62, "steel")]
        z, r = z - 0.9, r * 0.62
    drill += [(z - 1.6, 0.0, "steel")]
    m.add(lathe("drill", drill[::-1], "steel", 12, smooth=False))
    m.add(box("drill_mount", -3.8, 3.8, -3.0, 3.0, -7.6, -6.9, "grey"))
    for x in (-3.6, 3.6):
        for y in (-2.2, 2.2):
            m.add(cylinder_between("pod", (x, y, 9.0), (x, y, 12.5), 1.1, "grey", 10))
            m.add(bell("thruster", 12.4, 2.4, 0.8, 1.25, 10, "steel", "dark", "exhaust").transformed(IDENTITY, (x, y, 0)))
    for x in (-4.0, 4.0):
        m.add(cylinder_between("lamp", (x, 2.9, -7.0), (x, 2.9, -7.6), 0.45, "glass", 8))
    return m


def hotrod():
    """Black Comet raider, ~10 m: a stubby hot-rod rocket with flame paint, chrome
    side pipes, tall swept tail fins and an oversized exhaust."""
    m = Model("hotrod", RAIDER, FIGHTER)
    prof = ogive(-5.0, -3.6, 0.6, 4, "chrome") + [(-3.6, 0.66, "hull"), (-1.5, 0.95, "hull"), (1.5, 1.0, "hull"),
                                                (2.9, 0.92, "chrome"), (3.2, 0.85, "chrome")]
    m.add(lathe("hull", prof, "hull", 16, uv=(0, 1, 0, 1)))
    m.add(bell("bell", 3.0, 2.0, 0.8, 1.55, 18, "chrome", "dark", "exhaust"))
    m.add(ellipsoid("canopy", (0, 0.72, -0.8), (0.5, 0.48, 1.0), "glass"))
    m.add(fin("fin_top", 0, [(0.7, 0.4), (0.7, 2.6), (2.4, 4.4), (2.4, 3.6)], "flame"))
    for a in (125, 235):
        m.add(fin(f"fin{a}", a, [(0.7, 0.8), (0.7, 2.7), (2.0, 4.6), (2.0, 3.9)], "black"))
    for k, y in enumerate((-0.15, -0.45)):  # side exhaust pipes
        pipe = cylinder_between("pipe", (0.75, y, -2.4 + k * 0.4), (1.15, y - 0.1, 2.6), 0.12, "chrome", 6)
        tip = bell("pipe_tip", 0, 0.5, 0.12, 0.2, 6, "chrome", "dark", "exhaust").transformed(
            IDENTITY, (1.15, y - 0.1, 2.55))
        m.add_mirrored_x(pipe)
        m.add_mirrored_x(tip)
    for p in gun("gun", (0, -0.85, -1.4), (0, -0.85, -4.2), 0.1, "chrome", "flame"):
        m.add(p)
    return m


# ---------------------------------------------------------------------------------------
# Stations and rocks
# ---------------------------------------------------------------------------------------

def horizontal(deg):
    """A horizontal unit direction, 0 degrees = the nose (-Z), 90 = +X."""
    r = math.radians(deg)
    return (math.sin(r), 0.0, -math.cos(r))


def slot(name, at, normal, size=1.0):
    """An attachment point: a group `slot_<name>` holding one small square face centred
    on the mount point and wound counter-clockwise seen from `normal`, so its face normal
    is the direction the mounted module's -Z (its front) will point. The engine does
    not draw it, and it is not a solid, so it is marked open (`closed=False`): the
    validator still checks it for degeneracy but not for closure or volume, and debris
    cutting leaves it out."""
    h = size / 2
    quad = Face("slot", [(-h, -h, 0.0), (h, -h, 0.0), (h, h, 0.0), (-h, h, 0.0)])
    return Part(name, [quad], closed=False).transformed(frame_from_z(normal), at)


def guard_station():
    """Guard station, ~300 m tall: an art-deco spire with setback tiers, buttress fins,
    a habitat ring on spokes and four docking arms."""
    m = Model("guard_station", dict(STATION, **SLOT), CAPITAL)
    prof = [(-150, 0.0, "chrome"), (-120, 6, "chrome"), (-95, 16, "hull"), (-70, 26, "hull"), (-55, 30, "chrome"),
            (-52, 32, "chrome"), (-49, 30, "hull"), (-10, 30, "hull"), (-8, 31.5, "chrome"), (-5, 31.5, "chrome"),
            (-5, 25, "hull"), (30, 25, "hull"), (30, 27, "chrome"), (32, 27, "chrome"), (32, 19, "hull"),
            (60, 19, "hull"), (60, 20.5, "chrome"), (62, 20.5, "chrome"), (62, 13, "hull"), (85, 13, "hull"),
            (85, 14, "chrome"), (87, 14, "chrome"), (87, 7, "hull"), (100, 7, "teal"), (104, 5, "chrome"),
            (150, 0.0, "chrome")]
    # Parts are tagged (see T) for the construction stages; tags do not change the mesh.
    m.add(T(upright(lathe("tower", prof, "hull", 24, uv=(0, 1, 0, 1), uv_mode="around", u_repeat=4)),
            "column", (0, -150, 0), (0, 150, 0), 32))
    for a in (45, 135, 225, 315):  # deco buttress fins
        m.add(fin(f"buttress{a}", a, [(12, -60), (12, 80), (24, 72), (38, -10), (38, -50)], "red", 3.0, 3.0,
                  root_s=13, axis="y"))
    m.add(T(upright(torus("ring", 115, 7, "hull", 40, 8, uv=(0, 6, 0, 1)), y=-25), "ring", (0, -25, 0), 115, 14))
    for a in (0, 90, 180, 270):
        r = math.radians(a)
        d = (math.sin(r), 0, -math.cos(r))
        p0, p1 = add(mul(d, 28), (0, -25, 0)), add(mul(d, 110), (0, -25, 0))
        m.add(T(cylinder_between("spoke", p0, p1, 3.0, "chrome", 8), "column", p0, p1, 3.0))
    for a in (45, 135, 225, 315):
        r = math.radians(a)
        d = (math.sin(r), 0, -math.cos(r))
        root = add(mul(d, 22), (0, 45, 0))
        tip = add(mul(d, 75), (0, 45, 0))
        m.add(T(cylinder_between("arm", root, tip, 3.5, "cream", 8), "column", root, tip, 3.5))
        c0, c1 = add(tip, (0, -6, 0)), add(tip, (0, 6, 0))
        m.add(T(cylinder_between("cradle", c0, c1, 7, "red", 12), "column", c0, c1, 7))
        m.add(T(cylinder_between("cradle_light", add(tip, (0, 6, 0)), add(tip, (0, 8, 0)), 4, "lights", 12), "detail"))
    m.add(T(sphere("beacon", 4.0, "lights", 10, 5).transformed(IDENTITY, (0, 118, 0)), "detail"))
    # Module mounts. Turrets 1-4 on the outer rim of the habitat ring (between the
    # spokes' outer ends, at torus vertices so they sit exactly on the surface), turrets
    # 5-6 on the outer faces of two docking cradles, shields on the upper spire tier.
    for k, a in enumerate((0, 90, 180, 270)):
        d = horizontal(a)
        m.add(slot(f"slot_turret{k + 1}", add(mul(d, 122), (0, -25, 0)), d))
    for k, a in enumerate((45, 225)):
        d = horizontal(a)
        m.add(slot(f"slot_turret{k + 5}", add(mul(d, 82), (0, 45, 0)), d))
    for k, a in enumerate((90, 270)):
        d = horizontal(a)
        m.add(slot(f"slot_shield{k + 1}", add(mul(d, 13), (0, 72, 0)), d))
    return m


def dominion_outpost():
    """Dominion outpost, ~200 m: a squat black hub bristling with spikes, pods on struts
    and a glowing ring."""
    m = Model("dominion_outpost", dict(DOMINION, **SLOT), CAPITAL)
    m.add(upright(sphere("hub", 32, "hull", 28, 12, uv=(0, 1, 0, 1), u_repeat=4).transformed(scale_m(1, 1, 0.7))))
    m.add(upright(torus("glow_ring", 38, 2.5, "glow", 40, 6)))
    m.add(upright(torus("ring_frame", 38, 3.2, "black", 40, 4), y=-3.5))
    for k in range(8):
        a = 2 * math.pi * k / 8
        d = (math.sin(a), 0, -math.cos(a))
        m.add(cylinder_between("spike", mul(d, 28), mul(d, 100 if k % 2 == 0 else 75), 4.0, "black", 6, radius_b=0.0))
    m.add(cylinder_between("spike", (0, -18, 0), (0, -80, 0), 5.0, "black", 6, radius_b=0.0))
    for k in range(6):
        a = 2 * math.pi * (k + 0.5) / 6
        up = 1 if k % 2 == 0 else -1
        d = norm((math.sin(a), 0.55 * up, -math.cos(a)))
        root, tip = mul(d, 26), mul(d, 62)
        m.add(cylinder_between("strut", root, tip, 2.0, "chrome", 6))
        m.add(sphere("pod", 9.0, "violet", 14, 7).transformed(IDENTITY, tip))
        m.add(torus("pod_band", 9.1, 0.8, "glow", 16, 4).transformed(IDENTITY, tip))
    m.add(upright(dome("crown", 21.5, 10, 9, "dome", 20)))
    # Module mounts: six pods on the hub's surface between the spikes and struts,
    # alternately 25 degrees above and below the equator, facing along the surface
    # normal; one shield mount on the crown's apex (the top is left clear for it).
    a_r, b_r = 32.0, 32.0 * 0.7  # the hub's equatorial and polar radii
    for k in range(6):
        el = math.radians(25 if k % 2 == 0 else -25)
        h = horizontal(60 * k)
        d = (h[0] * math.cos(el), math.sin(el), h[2] * math.cos(el))
        # The ray along d meets the ellipsoid x^2/a^2 + y^2/b^2 + z^2/a^2 = 1 at t:
        t = 1 / math.sqrt((d[0] ** 2 + d[2] ** 2) / a_r ** 2 + d[1] ** 2 / b_r ** 2)
        p = mul(d, t)
        normal = norm((p[0] / a_r ** 2, p[1] / b_r ** 2, p[2] / a_r ** 2))  # the gradient
        m.add(slot(f"slot_pod{k + 1}", p, normal))
    m.add(slot("slot_shield1", (0, 30.5, 0), (0, 1, 0)))
    return m


def cosmium_asteroid():
    """A ~120 m asteroid with clusters of glowing cosmium crystals. The rock is a rough
    icosphere; each crystal is a six-sided lathe with a pointed tip, rooted inside it."""
    m = Model("cosmium_asteroid", ROCK, 2500)
    rock = icosphere("rock", 44, "rock", 2, rough=0.16, seed=7, uv=True).transformed(scale_m(1.1, 0.78, 0.92))
    m.add(rock)
    rng = random.Random(11)
    for c in range(6):
        centre = norm((rng.uniform(-1, 1), rng.uniform(-0.7, 1), rng.uniform(-1, 1)))
        for k in range(4):
            d = norm(add(centre, (rng.uniform(-0.3, 0.3), rng.uniform(-0.3, 0.3), rng.uniform(-0.3, 0.3))))
            L = rng.uniform(10, 22)
            r = L * rng.uniform(0.16, 0.22)
            base = (d[0] * 24, d[1] * 18, d[2] * 22)  # well inside the rock
            tip = add(base, mul(d, 26 + L))
            crystal = lathe("crystal", [(0, r), (26 + L * 0.75, r * 1.1), (26 + L, 0.0)], "crystal", 6, smooth=False)
            m.add(crystal.transformed(frame_from_z(sub(tip, base)), base))
    return m


def chunks():
    """Generic rubble, 1-3 m: a rock, a torn hull plate and a jagged lump of metal."""
    out = []
    a = Model("chunk1", ROCK, 400)
    a.add(icosphere("rock", 0.75, "rock", 1, rough=0.3, seed=21, uv=True).transformed(scale_m(1.3, 0.8, 1.0)))
    out.append(a)
    b = Model("chunk2", ROCK, 400)
    rng = random.Random(22)
    outline = []
    for k in range(11):
        ang = 2 * math.pi * k / 11
        rad = (1.3 if k % 2 == 0 else 0.8) * rng.uniform(0.85, 1.15)
        outline.append((rad * math.cos(ang) * 1.2, rad * math.sin(ang) * 0.8))
    b.add(prism("plate", outline, 0.14, "metal", s_axis=(1, 0, 0), t_axis=norm((0, 0.3, 1))))
    out.append(b)
    c = Model("chunk3", ROCK, 400)
    c.add(icosphere("lump", 0.6, "metal", 0, rough=0.45, seed=23).transformed(scale_m(1.6, 0.9, 1.1)))
    out.append(c)
    return out


# ---------------------------------------------------------------------------------------
# Station modules. Each stands on its base at the origin and extends along -Z: the
# engine turns the module so its -Z follows the slot's normal, out of the hull.
# Lathe profiles here run from the base (z = 0) toward -Z; `lathe` notices the reversed
# direction and turns the surface the right way out.
# ---------------------------------------------------------------------------------------

def guard_turret():
    """Guard turret, ~6 m: a stepped round base, a fluted drum with deco fins and twin
    ray guns pointing straight out."""
    m = Model("guard_turret", GUARD_TURRET, FIGHTER)
    prof = [(0.0, 2.8, "chrome"), (-0.35, 2.8, "chrome"), (-0.4, 2.5, "red"), (-0.9, 2.5, "red"), (-0.95, 2.2, "chrome"),
            (-1.2, 2.2, "hull"), (-2.6, 2.0, "hull"), (-3.0, 1.6, "chrome"), (-3.3, 1.0, "chrome"), (-3.4, 0.0, "chrome")]
    m.add(lathe("base", prof, "hull", 20, uv=(0, 1, 0, 1), uv_mode="around", u_repeat=2))
    for a in (45, 135, 225, 315):
        m.add(fin(f"fin{a}", a, [(1.9, -1.2), (1.9, -2.8), (2.9, -2.4), (2.9, -1.4)], "red", 0.2, 0.12))
    for x in (-0.75, 0.75):
        m.add(cylinder_between("mantlet", (x, 0, -2.6), (x, 0, -3.6), 0.45, "red", 10))
        for p in gun("gun", (x, 0, -3.6), (x, 0, -6.0), 0.2, "chrome", "teal", 0.32, 8):
            m.add(p)
    return m


def guard_shield_generator():
    """Guard shield generator, ~8 m: a chrome dome on a drum, crowned by a mast wound
    with a tapering coil and a glowing teal ball."""
    m = Model("guard_shield_generator", GUARD_SHIELD, FIGHTER)
    drum = [(0.0, 3.2, "chrome"), (-0.4, 3.2, "red"), (-1.2, 3.0, "hull"), (-1.3, 3.0, "hull")]
    m.add(lathe("drum", drum, "hull", 20, uv=(0, 1, 0, 1), uv_mode="around", u_repeat=2))
    m.add(dome("dome", 1.25, 2.9, 2.6, "chrome", 20, 5).transformed(scale_m(1, 1, -1)))
    m.add(cylinder_between("mast", (0, 0, -3.6), (0, 0, -7.2), 0.25, "chrome", 8))
    for k in range(5):  # the coil: rings shrinking toward the tip
        z = -4.0 - k * 0.65
        m.add(torus("coil", 1.2 - k * 0.18, 0.12, "red" if k % 2 == 0 else "chrome", 16, 4).transformed(IDENTITY, (0, 0, z)))
    m.add(sphere("ball", 0.55, "teal", 10, 5).transformed(IDENTITY, (0, 0, -7.5)))
    return m


def dominion_turret_pod():
    """Dominion turret pod, ~7 m: a violet orb on a black collar with a death-ray
    emitter pointing straight out."""
    m = Model("dominion_turret_pod", DOMINION_TURRET, FIGHTER)
    m.add(lathe("collar", [(0.0, 2.2, "black"), (-0.6, 2.0, "black"), (-1.0, 1.4, "black"), (-1.0, 0.0, "black")],
                "black", 16))
    m.add(sphere("orb", 2.6, "hull", 18, 9, uv=(0, 1, 0, 1), u_repeat=2).transformed(IDENTITY, (0, 0, -3.2)))
    m.add(torus("band", 2.62, 0.18, "glow", 24, 4).transformed(IDENTITY, (0, 0, -3.2)))
    m.add(cylinder_between("emitter_base", (0, 0, -5.4), (0, 0, -6.0), 0.75, "black", 10, radius_b=0.45))
    for p in gun("ray", (0, 0, -6.0), (0, 0, -7.0), 0.25, "chrome", "glow", 0.42, 8):
        m.add(p)
    for a in (0, 120, 240):  # three short spikes around the orb
        d = (math.sin(math.radians(a)), math.cos(math.radians(a)), 0)
        b = add(mul(d, 2.3), (0, 0, -3.6))
        m.add(cylinder_between("spike", b, add(add(mul(d, 3.6), (0, 0, -4.6)), (0, 0, 0)), 0.2, "black", 6, radius_b=0.0))
    return m


def dominion_shield_pod():
    """Dominion shield pod, ~8 m: an acid-green glowing sphere held in a cage of bars
    between two rings, with a spike on top."""
    m = Model("dominion_shield_pod", DOMINION_SHIELD, FIGHTER)
    m.add(lathe("collar", [(0.0, 2.4, "black"), (-0.5, 2.2, "black"), (-1.0, 1.6, "black"), (-1.0, 0.0, "black")],
                "black", 16))
    m.add(sphere("core", 2.7, "hull", 18, 9, uv=(0, 1, 0, 1), u_repeat=2).transformed(IDENTITY, (0, 0, -4.4)))
    for z in (-1.4, -7.4):
        m.add(torus("cage_ring", 3.0, 0.2, "violet", 20, 4).transformed(IDENTITY, (0, 0, z)))
    for k in range(6):
        a = math.radians(60 * k)
        d = (math.sin(a), math.cos(a), 0)
        m.add(cylinder_between("cage_bar", add(mul(d, 3.0), (0, 0, -1.4)), add(mul(d, 3.0), (0, 0, -7.4)), 0.16, "chrome", 6))
    m.add(lathe("cap", [(-7.2, 1.2, "violet"), (-7.6, 0.9, "violet"), (-7.6, 0.0, "violet")], "violet", 12))
    m.add(cylinder_between("spike", (0, 0, -7.5), (0, 0, -8.6), 0.35, "black", 6, radius_b=0.0))
    return m


# ---------------------------------------------------------------------------------------
# Probes
# ---------------------------------------------------------------------------------------

def sky_eye_probe():
    """Guard sensor probe, ~6 m: a polished sphere trailing four long swept antennas,
    with a small dish and feed horn on the nose."""
    m = Model("sky_eye_probe", dict(GUARD, hull=((1, 1, 1), "sky_eye_probe.png")), FIGHTER)
    m.add(sphere("body", 0.9, "hull", 16, 8, uv=(0, 1, 0, 1), u_repeat=2))
    for sx in (-1, 1):
        for sy in (-1, 1):
            d = norm((sx * 0.35, sy * 0.35, 1.0))
            m.add(cylinder_between("antenna", mul(d, 0.8), mul(d, 4.6), 0.035, "chrome", 5, radius_b=0.015))
            m.add(sphere("antenna_root", 0.09, "red", 6, 3).transformed(IDENTITY, mul(d, 0.88)))
    m.add(cylinder_between("dish_stem", (0, 0, -0.8), (0, 0, -1.35), 0.08, "chrome", 6))
    m.add(lathe("dish", [(-1.3, 0.0, "cream"), (-1.6, 0.75, "chrome"), (-1.68, 0.72, "cream"), (-1.42, 0.0, "cream")],
                "cream", 16, crease=30))
    m.add(cylinder_between("feed", (0, 0, -1.4), (0, 0, -2.1), 0.05, "chrome", 5))
    m.add(sphere("feed_tip", 0.08, "teal", 6, 3).transformed(IDENTITY, (0, 0, -2.12)))
    return m


def jammer_buoy():
    """Dominion jamming buoy, ~5 m: a spiked violet and black orb split around a
    glowing acid-green core, inside a striped ring on three struts."""
    m = Model("jammer_buoy", dict(DOMINION, hull=((1, 1, 1), "jammer_buoy.png")), FIGHTER)
    m.add(sphere("core", 1.3, "glow", 14, 7))
    upper = dome("shell", 0.25, 1.5, 1.35, "hull", 18, 4, uv=(0, 1, 0.5, 1))
    m.add(upper.transformed(Z_TO_Y))
    m.add(upper.transformed(Z_TO_Y).transformed(scale_m(1, -1, 1)))  # the mirrored lower half
    ring = torus("ring", 2.6, 0.16, "hull", 32, 5, uv=(0, 8, 0, 0.2))
    m.add(upright(ring))
    for a in (0, 120, 240):
        d = horizontal(a)
        m.add(cylinder_between("strut", mul(d, 1.2), mul(d, 2.5), 0.07, "chrome", 5))
    for d in ((0, 1, 0), (0, -1, 0), norm((1, 0.9, 1)), norm((-1, 0.9, -1)), norm((1, -0.9, -1)), norm((-1, -0.9, 1))):
        m.add(cylinder_between("spike", mul(d, 1.2), mul(d, 2.5), 0.16, "black", 6, radius_b=0.0))
    return m


# ---------------------------------------------------------------------------------------
# Logistics: Guard facilities, the supply clipper and its cargo pod
# ---------------------------------------------------------------------------------------
#
# Facility parts carry a `tag` describing their rough shape, which the construction
# stages below use to put scaffolding where the part will be:
#   ("column", a, b, r) - something round of radius r from point a to point b;
#   ("ring", centre, R, h) - a horizontal ring of radius R and height h;
#   ("box", lo, hi)     - an axis-aligned block;
#   ("base",)           - present from the start (the anchoring asteroid);
#   ("detail",)         - fitted last (domes, dishes, antennas, lamps).
# Untagged parts are small hull pieces: built, but never scaffolded.

def T(part, *tag):
    """Tags a part (see above) and returns it, so a builder can write m.add(T(...))."""
    part.tag = tag
    return part


def facility_mats(texture):
    return dict(STATION, hull=((1, 1, 1), texture), rock=((1, 1, 1), "asteroid_rock.png"),
                ore=((0.22, 0.2, 0.18), None), glass=((0.75, 0.92, 1.0), None), **SLOT)


def ring_slot(m, name, R, y, deg):
    d = horizontal(deg)
    m.add(slot(name, add(mul(d, R), (0, y, 0)), d))


def mining_facility():
    """Guard ore refinery, ~200 m: a refinery tower anchored in an asteroid, three
    drilling derricks, three ore hoppers on legs feeding conveyor booms, and a docking
    cradle on a long arm."""
    m = Model("mining_facility", facility_mats("mining_facility.png"), CAPITAL)
    rock = icosphere("rock", 46, "rock", 2, rough=0.15, seed=31, uv=True).transformed(scale_m(1.2, 0.7, 1.0), (0, -10, 0))
    m.add(T(rock, "base"))
    prof = [(10, 16, "hull"), (20, 16, "chrome"), (22, 18, "chrome"), (24, 18, "hull"), (24, 15, "hull"), (70, 15, "chrome"),
            (70, 17, "chrome"), (72, 17, "hull"), (72, 13, "hull"), (100, 13, "chrome"), (100, 15, "chrome"),
            (102, 15, "hull"), (102, 11, "hull"), (112, 11, "chrome"), (114, 9, "chrome")]
    m.add(T(upright(lathe("tower", prof, "hull", 24, uv=(0, 1, 0, 1), uv_mode="around", u_repeat=3)),
            "column", (0, 10, 0), (0, 114, 0), 18))
    for k, a in enumerate((0, 120, 240)):
        c = mul(horizontal(a), 48)
        hop = [(28, 0.0, "chrome"), (30, 3, "red"), (46, 12, "hull"), (60, 12, "chrome"), (61, 12.8, "chrome"),
               (62, 12.8, "ore"), (62, 0.0, "ore")]
        m.add(T(upright(lathe("hopper", hop, "hull", 16, uv=(0, 1, 0, 1), uv_mode="around"), x=c[0], z=c[2]),
                "column", (c[0], 28, c[2]), (c[0], 62, c[2]), 13))
        for b in (0, 120, 240):
            leg = add(c, mul(horizontal(b + 60), 9))
            m.add(cylinder_between("leg", add(leg, (0, 44, 0)), add(leg, (0, -4, 0)), 1.2, "chrome", 6))
        top = add(c, (0, 62, 0))
        end = add(mul(horizontal(a), 15), (0, 88, 0))
        m.add(T(cylinder_between("conveyor", top, end, 2.4, "red", 4), "column", top, end, 3))
        m.add(T(sphere("hopper_lamp", 1.6, "lights", 8, 4).transformed(IDENTITY, add(top, (0, 2, 0))), "detail"))
    for a in (60, 180, 300):  # drilling derricks: square tapered towers over drill pipes
        c = mul(horizontal(a), 30)
        der = lathe("derrick", [(12, 7, "red"), (70, 1.6, "chrome"), (75, 1.6, "chrome")], "red", 4, phase=45,
                    smooth=False)
        m.add(T(upright(der, x=c[0], z=c[2]), "column", (c[0], 12, c[2]), (c[0], 75, c[2]), 7))
        m.add(cylinder_between("drill_pipe", add(c, (0, 14, 0)), add(c, (0, -12, 0)), 1.0, "chrome", 6))
        m.add(T(sphere("derrick_lamp", 1.2, "lights", 8, 4).transformed(IDENTITY, add(c, (0, 77, 0))), "detail"))
    arm_a, arm_b = (14, 106, 0), (92, 106, 0)
    m.add(T(cylinder_between("dock_arm", arm_a, arm_b, 3.5, "cream", 8), "column", arm_a, arm_b, 3.5))
    m.add(T(box("cradle", 84, 102, 96, 100, -12, 12, "red"), "box", (84, 96, -12), (102, 100, 12)))
    for z0 in (-12, 9):
        m.add(T(box("cradle_side", 84, 102, 100, 110, z0, z0 + 3, "red"), "box", (84, 100, z0), (102, 110, z0 + 3)))
    m.add(T(cylinder_between("mast", (92, 110, 0), (92, 128, 0), 0.6, "chrome", 6), "detail"))
    m.add(T(sphere("beacon", 1.8, "lights", 8, 4).transformed(IDENTITY, (92, 129, 0)), "detail"))
    for k, a in enumerate((45, 135, 225, 315)):
        ring_slot(m, f"slot_turret{k + 1}", 13, 86, a)
    m.add(slot("slot_shield1", (0, 114, 0), (0, 1, 0)))
    return m


def research_station():
    """Guard observatory, ~180 m: a hub with a telescope dome and its tube, two
    laboratory rings on spokes, and three radio dishes on the outer ring."""
    m = Model("research_station", facility_mats("research_station.png"), CAPITAL)
    prof = [(-60, 6, "chrome"), (-40, 10, "chrome"), (-22, 22, "hull"), (-20, 26, "chrome"), (-17, 26, "hull"),
            (-17, 24, "hull"), (17, 24, "chrome"), (17, 26, "chrome"), (20, 26, "hull"), (22, 20, "hull"), (24, 18, "hull")]
    m.add(T(upright(lathe("hub", prof, "hull", 24, uv=(0, 1, 0, 1), uv_mode="around", u_repeat=3)),
            "column", (0, -60, 0), (0, 24, 0), 26))
    m.add(T(upright(torus("lab_ring", 75, 6, "hull", 36, 8, uv=(0, 8, 0, 1))), "ring", (0, 0, 0), 75, 12))
    for a in (30, 90, 150, 210, 270, 330):
        p0, p1 = mul(horizontal(a), 24), mul(horizontal(a), 69.5)
        m.add(T(cylinder_between("spoke", p0, p1, 2.5, "chrome", 8), "column", p0, p1, 2.5))
    m.add(T(upright(torus("lab_ring2", 45, 4, "hull", 28, 6, uv=(0, 5, 0, 1)), y=-35), "ring", (0, -35, 0), 45, 8))
    for a in (0, 90, 180, 270):
        p0, p1 = add(mul(horizontal(a), 17), (0, -35, 0)), add(mul(horizontal(a), 41.5), (0, -35, 0))
        m.add(T(cylinder_between("spoke2", p0, p1, 1.8, "chrome", 6), "column", p0, p1, 1.8))
    # Observatory: a drum, a dome and the telescope tube poking out of its slit.
    m.add(T(upright(lathe("dome_drum", [(23, 17, "cream"), (32, 17, "chrome"), (33, 16, "chrome")], "cream", 24)), "detail"))
    m.add(T(upright(dome("dome", 32.5, 16, 14, "chrome", 24, 5)), "detail"))
    m.add(T(cylinder_between("telescope", (0, 40, 0), (11, 68, -13), 3.6, "cream", 12), "detail"))
    m.add(T(cylinder_between("telescope_ring", (8.2, 61, -9.7), (9.0, 63, -10.6), 4.2, "red", 12), "detail"))
    for a in (0, 120, 240):  # radio dishes on masts, tilted outward
        base = add(mul(horizontal(a), 75), (0, 5, 0))
        top = add(base, (0, 16, 0))
        m.add(T(cylinder_between("dish_mast", base, top, 0.9, "chrome", 6), "detail"))
        aim = norm(add(mul(horizontal(a), 0.6), (0, 1, 0)))
        dish = lathe("dish", [(0.0, 0.0, "cream"), (2.6, 9.0, "chrome"), (3.2, 8.6, "cream"), (0.6, 0.0, "cream")],
                     "cream", 16, crease=30)
        m.add(T(dish.transformed(frame_from_z(aim), top), "detail"))
        m.add(T(cylinder_between("feed", top, add(top, mul(aim, 7)), 0.3, "chrome", 5), "detail"))
    for k, a in enumerate((60, 180, 300)):
        ring_slot(m, f"slot_turret{k + 1}", 81, 0, a)
    m.add(slot("slot_shield1", (0, -60, 0), (0, -1, 0)))
    return m


def manufacturing_center():
    """Guard orbital factory, ~250 m long: a spine with two smelter drums, side cargo
    bays, and two assembly gantries around a rocket hull under construction."""
    m = Model("manufacturing_center", facility_mats("manufacturing_center.png"), CAPITAL)
    spine = [(-125, 10, "chrome"), (-121, 12, "hull"), (-100, 12, "chrome"), (-99, 13.5, "chrome"), (-97, 13.5, "hull"),
             (-96, 12, "hull"), (96, 12, "chrome"), (97, 13.5, "chrome"), (99, 13.5, "hull"), (100, 12, "hull"),
             (121, 12, "chrome"), (125, 10, "chrome")]
    m.add(T(lathe("spine", spine, "hull", 16, uv=(0, 1, 0, 1), uv_mode="around", u_repeat=2),
            "column", (0, 0, -125), (0, 0, 125), 13.5))
    drum1 = [(-84, 18, "chrome"), (-80, 30, "hull"), (-62, 30, "chrome"), (-61, 31.5, "chrome"), (-59, 31.5, "hull"),
             (-58, 30, "hull"), (-40, 30, "chrome"), (-36, 18, "chrome")]
    m.add(T(lathe("smelter1", drum1, "hull", 24, uv=(0, 1, 0, 1), uv_mode="around", u_repeat=3),
            "column", (0, 0, -84), (0, 0, -36), 31.5))
    drum2 = [(-24, 16, "chrome"), (-20, 26, "hull"), (16, 26, "chrome"), (20, 16, "chrome")]
    m.add(T(lathe("smelter2", drum2, "hull", 24, uv=(0, 1, 0, 1), uv_mode="around", u_repeat=3),
            "column", (0, 0, -24), (0, 0, 20), 26))
    m.add(T(torus("furnace_glow", 26.2, 1.2, "exhaust", 32, 4).transformed(IDENTITY, (0, 0, 10)), "detail"))
    for sx in (-1, 1):  # cargo bays either side
        lo, hi = (min(sx * 14, sx * 44), -14, 26), (max(sx * 14, sx * 44), 14, 56)
        m.add(T(box("cargo_bay", lo[0], hi[0], lo[1], hi[1], lo[2], hi[2], "hull", uv_scale=1 / 16), "box", lo, hi))
        m.add(box("bay_door", lo[0] + 2, hi[0] - 2, 14, 15, 29, 53, "red"))
    for z in (66, 110):  # assembly gantries: two legs and a beam each
        for x0 in (-46, 40):
            m.add(T(box("gantry_leg", x0, x0 + 6, -16, 46, z, z + 5, "red"), "box", (x0, -16, z), (x0 + 6, 46, z + 5)))
        m.add(T(box("gantry_beam", -46, 46, 40, 46, z, z + 5, "cream"), "box", (-46, 40, z), (46, 46, z + 5)))
        m.add(T(box("gantry_hook", -2, 2, 30, 40, z + 1, z + 4, "chrome"), "detail"))
    rocket = ogive(64, 76, 8, 4, "chrome") + [(76, 8.5, "red"), (104, 8.5, "red"), (108, 7, "chrome")]
    m.add(T(lathe("rocket_in_build", rocket, "red", 16).transformed(IDENTITY, (0, 26, 0)), "detail"))
    for z in (-110, 70):
        m.add(T(cylinder_between("mast", (0, 13, z), (0, 34, z), 0.8, "chrome", 6), "detail"))
        m.add(T(sphere("beacon", 1.6, "lights", 8, 4).transformed(IDENTITY, (0, 35, z)), "detail"))
    m.add(slot("slot_turret1", (30, 0, -70), (1, 0, 0)))
    m.add(slot("slot_turret2", (-30, 0, -70), (-1, 0, 0)))
    m.add(slot("slot_turret3", (0, 26, -8), (0, 1, 0)))
    m.add(slot("slot_turret4", (0, -26, -8), (0, -1, 0)))
    m.add(slot("slot_shield1", (0, 0, -125), (0, 0, -1)))
    m.add(slot("slot_shield2", (0, 0, 125), (0, 0, 1)))
    return m


POD_MATS = dict(GUARD, pod=((1, 1, 1), "cargo_pod.png"))


def pod_parts(centre=(0, 0, 0)):
    """A 6 x 6 x 14 m cargo container with chrome ribs, red end frames, lifting lugs and
    a docking latch on top, centred on `centre`."""
    cx, cy, cz = centre
    parts = [box("container", cx - 3, cx + 3, cy - 3, cy + 3, cz - 7, cz + 7, "pod", uv_scale=1 / 8)]
    for z in (-3.5, 3.5):
        parts.append(box("rib", cx - 3.2, cx + 3.2, cy - 3.2, cy + 3.2, cz + z - 0.3, cz + z + 0.3, "chrome"))
    for z in (-7.2, 6.6):
        parts.append(box("end_frame", cx - 3.3, cx + 3.3, cy - 3.3, cy + 3.3, cz + z, cz + z + 0.6, "red"))
    parts.append(cylinder_between("latch", (cx, cy + 3, cz), (cx, cy + 3.8, cz), 0.6, "chrome", 8))
    return parts


def supply_pod():
    """Detachable Guard cargo container, ~15 m."""
    m = Model("supply_pod", POD_MATS, FIGHTER)
    for p in pod_parts():
        m.add(p)
    return m


def supply_ship():
    """Guard cargo clipper, ~85 m: a rocket-nosed cab with portholes, a spine carrying
    twelve cargo pods, and a chunky four-bell engine block."""
    m = Model("supply_ship", POD_MATS, 3000)
    cab = ogive(-42, -30, 7.0, 5, "chrome") + [(-30, 7.3, "hull"), (-21, 8.0, "hull"), (-19, 8.6, "chrome"),
                                               (-17, 8.6, "hull"), (-15, 7.6, "hull"), (-14, 7.0, "chrome")]
    m.add(lathe("cab", cab, "hull", 16, uv=(0, 1, 0, 1)))
    m.add(ellipsoid("canopy", (0, 6.2, -26), (3.0, 2.4, 5.0), "glass"))
    m.add(box("spine", -3, 3, -3, 3, -16, 33, "red"))
    for z in (-8, 7.5, 23):
        for sx in (-1, 1):
            for sy in (-1, 1):
                for p in pod_parts((sx * 6.4, sy * 3.4, z)):
                    m.add(p)
    m.add(box("engine_block", -11, 11, -9, 9, 31, 40, "cream"))
    m.add(box("engine_trim", -11.4, 11.4, -9.4, 9.4, 33, 34.5, "chrome"))
    for sx in (-5.5, 5.5):
        for sy in (-4.5, 4.5):
            m.add(bell("bell", 39, 8, 2.6, 4.0, 12).transformed(IDENTITY, (sx, sy, 0)))
    m.add(fin("fin_top", 0, [(8.5, 24), (8.5, 39), (16, 46), (16, 41)], "red", 0.8, 0.4))
    m.add(fin("fin_bottom", 180, [(8.5, 24), (8.5, 39), (14, 45), (14, 41)], "red", 0.8, 0.4))
    return m


# ---------------------------------------------------------------------------------------
# Construction stages
# ---------------------------------------------------------------------------------------

def beam(name, a, b, w, mat="scaffold"):
    """A triangular girder from a to b, about `w` thick, its texture running
    along it (one hazard repeat per 8 widths)."""
    L = length(sub(b, a))
    reps = max(1, round(L / (8 * w)))
    # Three sides: a triangular girder is 12 triangles against a square one's 16, and
    # scaffolding is mostly girders.
    part = lathe(name, [(0.0, w * 1.4), (L, w * 1.4)], mat, 3, uv=(0, reps, 0, 1), smooth=False)
    return part.transformed(frame_from_z(sub(b, a)), a)


def scaffold(tag, full=True):
    """Scaffolding around a tagged part: returns (girder parts, node points). A column
    gets posts, rings of girders and diagonal braces; a ring gets a truss of two
    polygons with posts; a box gets its twelve edges. `full=False` gives the sparse
    remnant left over once a part is finished (posts and a top ring only)."""
    parts, nodes = [], []
    kind = tag[0]
    if kind == "column":
        a, b, r = tag[1], tag[2], tag[3] * 1.12
        L = length(sub(b, a))
        M = frame_from_z(sub(b, a))
        posts = 6 if r > 12 else 4
        rings = max(2, min(4, round(L / max(2 * r, 12)) + 1))
        w = max(0.25, min(1.0, r * 0.035))

        def P(k, i):
            ang = 2 * math.pi * k / posts + math.pi / posts
            return add(a, mat_apply(M, (r * math.cos(ang), r * math.sin(ang), L * i / (rings - 1))))

        for k in range(posts) if full else range(0, posts, 2):
            parts.append(beam("post", P(k, 0), P(k, rings - 1), w))
        for i in range(rings) if full else (rings - 1,):
            for k in range(posts):
                parts.append(beam("girder", P(k, i), P(k + 1, i), w))
                nodes.append(P(k, i))
        if full and posts == 6:  # braces only on the big columns, to save triangles
            for i in range(rings - 1):
                for k in range(0, posts, 2):
                    parts.append(beam("brace", P(k, i), P(k + 1, i + 1), w * 0.8))
    elif kind == "ring":
        c, R, h = tag[1], tag[2], tag[3]
        segs, w = 12, 0.8

        def P(k, s):
            d = horizontal(360 * k / segs + 15)
            return add(add(c, mul(d, R)), (0, s * (h * 0.65), 0))

        for k in range(segs):
            for s in (-1, 1) if full else (1,):
                parts.append(beam("girder", P(k, s), P(k + 1, s), w))
                nodes.append(P(k, s))
            if full or k % 3 == 0:
                parts.append(beam("post", P(k, -1), P(k, 1), w))
            if full and k % 2 == 0:
                parts.append(beam("brace", P(k, -1), P(k + 1, 1), w * 0.8))
    elif kind == "box":
        lo, hi = tag[1], tag[2]
        pad = 1.0
        lo, hi = sub(lo, (pad, pad, pad)), add(hi, (pad, pad, pad))
        w = max(0.25, min(0.8, min(sub(hi, lo)) * 0.06))
        corners = [(x, y, z) for x in (lo[0], hi[0]) for y in (lo[1], hi[1]) for z in (lo[2], hi[2])]
        for i in range(8):
            for j in range(i + 1, 8):
                diff = sum(1 for q in range(3) if corners[i][q] != corners[j][q])
                if diff == 1 and (full or corners[i][1] == hi[1] or corners[j][1] == hi[1]):
                    parts.append(beam("girder", corners[i], corners[j], w))
        nodes += corners
    return parts, nodes


def lamps(nodes, rng, every, size):
    """Construction lights on some scaffold nodes, and a red blinker on the highest."""
    out = [sphere("work_light", size, "lights", 6, 3).transformed(IDENTITY, p) for i, p in enumerate(nodes) if i % every == 0]
    if nodes:
        top = max(nodes, key=lambda p: (p[1], p[0], p[2]))
        out.append(sphere("blinker", size * 1.4, "blinker", 6, 3).transformed(IDENTITY, add(top, (0, size * 2, 0))))
    return out


def build_stage(builder, stage):
    """`<name>_build1..3`: the facility under construction, in the finished model's own
    space so the stages swap in place. Stage 3 (~90%) is the whole hull without its
    details (domes, dishes, antennas, lamps) and a few scaffold remnants; stage 2 (~60%)
    keeps the core and a seeded selection of hull parts, scaffolding the rest; stage 1
    (~25%) is bare scaffolding over every part's outline with a few hull plates."""
    model = builder()
    name = f"{model.name}_build{stage}"
    rng = random.Random(zlib.crc32(name.encode()))  # a seed from the name: stable across runs
    mats = dict(model.materials, scaffold=((1, 1, 1), "scaffold.png"), lights=((1.0, 0.92, 0.55), None),
                blinker=((1.0, 0.15, 0.1), None), cream=GUARD["cream"])
    out = Model(name, mats, CAPITAL)
    parts = [p for p in model.parts if not p.name.startswith("slot_")]
    tag = lambda p: getattr(p, "tag", ("plain",))
    base = [p for p in parts if tag(p)[0] == "base"]
    hull = [p for p in parts if tag(p)[0] not in ("base", "detail")]
    shaped = [p for p in hull if tag(p)[0] in ("column", "ring", "box")]

    def extent(p):
        lo, hi = p.bounds()
        return length(sub(hi, lo))

    core = max(shaped, key=extent)
    out.parts = list(base)
    nodes = []
    if stage == 3:
        out.parts += hull
        for p in rng.sample(sorted(shaped, key=lambda q: q.name)[1:], 2):
            g, nd = scaffold(tag(p), full=False)
            out.parts += g
            nodes += nd
        every = 6
    elif stage == 2:
        others = [p for p in hull if p is not core]
        keep = set(id(p) for p in rng.sample(others, max(0, round(0.6 * len(hull)) - 1)))
        for p in hull:
            if p is core or id(p) in keep:
                out.parts.append(p)
            elif tag(p)[0] in ("column", "ring", "box"):
                g, nd = scaffold(tag(p))
                out.parts += g
                nodes += nd
        every = 4
    else:
        for p in shaped:
            g, nd = scaffold(tag(p))
            out.parts += g
            nodes += nd
        # A few hull plates already hung on the core's scaffold.
        kind = tag(core)
        if kind[0] == "column":
            a, b, r = kind[1], kind[2], kind[3]
            M = frame_from_z(sub(b, a))
            L = length(sub(b, a))
            for k in range(5):
                ang = 2 * math.pi * rng.random()
                t = rng.uniform(0.15, 0.85)
                radial = mat_apply(M, (math.cos(ang), math.sin(ang), 0))
                centre = add(a, mat_apply(M, (r * math.cos(ang), r * math.sin(ang), L * t)))
                axis = norm(sub(b, a))
                tangent = cross(axis, radial)
                from common import oriented_box
                out.parts.append(oriented_box("plate", centre, (tangent, axis, radial),
                                              (r * 0.35, min(L * 0.08, r * 0.6), max(0.3, r * 0.03)), "cream"))
        every = 3
    size = max(0.6, min(2.5, extent(core) * 0.006))
    out.parts += lamps(nodes, rng, every, size)
    return out


# ---------------------------------------------------------------------------------------
# The Guard 3D cockpit
# ---------------------------------------------------------------------------------------

# The engine's cockpit camera: eye at the origin looking down -Z, vertical field of view
# 60 degrees, aspect 16:9; instrument rectangles are in a 1280 x 720 design space.
DESIGN_W, DESIGN_H = 1280, 720
TAN_V = math.tan(math.radians(30))
TAN_H = TAN_V * 16 / 9


def project(p):
    """3D point (in front of the eye, z < 0) -> design-space pixel."""
    x, y, z = p
    return ((x / -z / TAN_H + 1) / 2 * DESIGN_W, (1 - y / -z / TAN_V) / 2 * DESIGN_H)


def unproject_dir(px, py):
    """Design-space pixel -> direction of the ray from the eye through it."""
    return ((2 * px / DESIGN_W - 1) * TAN_H, (1 - 2 * py / DESIGN_H) * TAN_V, -1.0)


COCKPIT = {
    "ivory": ((0.93, 0.88, 0.74), None),    # the dash face: cream enamel
    "lacquer": ((0.74, 0.10, 0.07), None),  # rocket-red lacquer panels, fascia and consoles
    "chrome": ((0.90, 0.92, 0.96), None),   # polished bezels and trim
    "shine": ((1.0, 1.0, 1.0), None),       # the bright highlight band on each bezel
    "screen": ((0.02, 0.05, 0.04), None),   # dark glass behind the 2D instruments
    "bakelite": ((0.28, 0.13, 0.06), None),  # brown knob bodies
    "brass": ((0.88, 0.68, 0.30), None),
    "teal": ((0.10, 0.66, 0.62), None),
    "lamp": ((1.0, 0.75, 0.3), None),
}


def guard_cockpit():
    """The Guard's 3D cockpit, in metres around the pilot's eye at the origin: a 1950s
    rocket-ship dashboard in ivory enamel and red lacquer with polished chrome.

    The dashboard's face is a plane tilted back 20 degrees about 0.8 m ahead of the eye.
    Every instrument is first laid out in design-space pixels (where the HUD will draw
    it), then *lifted* onto that plane by casting the ray from the eye through each
    outline pixel. The bezel therefore frames exactly the rectangle the engine draws in,
    and the rectangles written to the .ini are measured back from the 3D geometry by
    projecting each opening (so the two can never disagree).

    The design space is 16:9, but on a squarer window the engine widens the vertical
    field of view (to about 76 degrees on 4:3), so the red lower fascia carries the dash
    face down to about 52 degrees below the horizon and side consoles fill the lower
    corners - parts a 16:9 screen never shows.
    Returns (model, {instrument: (x, y, w, h)})."""
    m = Model("guard_cockpit", COCKPIT, CAPITAL)
    tilt = math.radians(20)
    n = (0.0, math.sin(tilt), math.cos(tilt))     # the dash face's normal, toward the pilot
    P0 = (0.0, -0.33, -0.80)                       # a point on the dash face
    U = (1.0, 0.0, 0.0)                            # right, along the face
    V = cross(n, U)                                # up the face; (U, V, n) is right-handed

    def lift(px, py, offset=0.0):
        """Where the ray through pixel (px, py) meets the dash face (moved `offset`
        toward the pilot), as plane coordinates (u, v)."""
        d = unproject_dir(px, py)
        q = add(P0, mul(n, offset))
        t = dot(n, q) / dot(n, d)
        p = mul(d, t)
        return (dot(sub(p, q), U), dot(sub(p, q), V))

    def plane_point(uv, off=0.0):
        return add(add(P0, mul(n, off)), add(mul(U, uv[0]), mul(V, uv[1])))

    def plate(name, outline_px, mat, off, th):
        """A flat plate on the dash face, from `off` to `off + th` toward the pilot."""
        m.add(prism(name, [lift(px, py) for px, py in outline_px], th, mat,
                    origin=add(P0, mul(n, off + th / 2)), s_axis=U, t_axis=V))

    def circle_px(cx, cy, r, k=28):
        return [(cx + r * math.cos(2 * math.pi * i / k), cy - r * math.sin(2 * math.pi * i / k)) for i in range(k)]

    def rrect_px(x0, y0, x1, y1, rad, k=4):
        """A rounded rectangle outline (counter-clockwise on screen, i.e. with y up)."""
        pts = []
        # Screen y grows downward; angles below are measured with y up (hence -sin).
        corners = [(x1 - rad, y0 + rad, 0), (x0 + rad, y0 + rad, 90), (x0 + rad, y1 - rad, 180), (x1 - rad, y1 - rad, 270)]
        for cx, cy, a0 in corners:
            for i in range(k + 1):
                a = math.radians(a0 + 90 * i / k)
                pts.append((cx + rad * math.cos(a), cy - rad * math.sin(a)))
        return pts

    # Dashboard body: an ivory upper face (stepped deco top edge with a raised crest
    # for the badge) over a red lower fascia, both extruded 0.3 m away from the pilot.
    top = [(-80, 505), (260, 505), (300, 462), (540, 462), (575, 410), (705, 410), (740, 462), (980, 462),
           (1020, 505), (1360, 505)]
    join, bottom = 722, 1150
    dash_th = 0.3
    back = sub(P0, mul(n, dash_th / 2))
    for name, outline_px, mat in (("dash", [(-80, join), (1360, join)] + top[::-1], "ivory"),
                                  ("fascia", [(-80, bottom), (1360, bottom), (1360, join), (-80, join)], "lacquer")):
        m.add(prism(name, [lift(px, py) for px, py in outline_px], dash_th, mat, origin=back, s_axis=U, t_axis=V))
    plate("fascia_trim", [(-80, join + 5), (1360, join + 5), (1360, join - 5), (-80, join - 5)], "chrome", 0.0, 0.008)
    for k, py in enumerate((792, 816, 840)):  # chrome speed lines across the fascia
        x0, x1 = 120 + k * 60, 1160 - k * 60
        plate("fascia_line", [(x0, py + 4), (x1, py + 4), (x1, py - 4), (x0, py - 4)], "chrome", 0.0, 0.006)

    # Chrome trim tube along the top edge of the dash.
    tpts = [plane_point(lift(px, py), 0.012) for px, py in top]
    for a, b in zip(tpts, tpts[1:]):
        m.add(cylinder_between("trim", a, b, 0.012, "chrome", 6))
    for p in tpts[1:-1]:
        m.add(sphere("trim_joint", 0.0125, "chrome", 6, 3).transformed(IDENTITY, p))

    # Red lacquer panels behind the two instrument clusters (under the bezels).
    plate("lacquer_l", rrect_px(272, 497, 512, 712, 14), "lacquer", 0.0, 0.002)
    plate("lacquer_r", rrect_px(768, 497, 1008, 712, 14), "lacquer", 0.0, 0.002)

    rects, anchors = {}, {}
    m.anchors = anchors  # {instrument: (centre, normal, (width, height))}, metres

    def ring(name, inner_px, outer_px, depth, mat):
        # Back and front outlines are both lifted from the same pixels, so the inner
        # wall runs along the line of sight and the opening projects exactly onto
        # the pixel outline from the eye.
        inner = [lift(px, py) for px, py in inner_px]
        outer = [lift(px, py) for px, py in outer_px]
        inner_f = [lift(px, py, depth) for px, py in inner_px]
        outer_f = [lift(px, py, depth) for px, py in outer_px]
        m.add(frame(name, inner, outer, depth, mat, P0, U, V, inner_f, outer_f))
        return inner, inner_f

    def bezel(name, outline, rim, depth=0.02):
        """A polished chrome bezel `rim` pixels wide with a raised highlight band, the
        dark glass of the opening, and the measured instrument rectangle."""
        inner, inner_f = ring(name, outline(0), outline(rim), depth, "chrome")
        ring(name + "_shine", outline(rim * 0.38), outline(rim * 0.62), depth + 0.004, "shine")
        m.add(prism(name + "_glass", inner[::-1], 0.004, "screen", origin=add(P0, mul(n, 0.004)), s_axis=U, t_axis=V))
        # What the pilot sees through is where the front and back edges of the opening
        # overlap, so take the intersection of the two projected extents, shrunk to
        # whole pixels inside it.
        ext = []
        for off, loop in ((depth, inner_f), (0.0, inner)):
            pts = [project(plane_point(uv, off)) for uv in loop]
            ext.append((min(p[0] for p in pts), min(p[1] for p in pts), max(p[0] for p in pts), max(p[1] for p in pts)))
        x0, y0 = math.ceil(max(e[0] for e in ext)), math.ceil(max(e[1] for e in ext))
        x1, y1 = math.floor(min(e[2] for e in ext)), math.floor(min(e[3] for e in ext))
        rects[name] = (x0, y0, x1 - x0, y1 - y0)
        # The same opening as a rectangle on the model's surface, for engines that draw
        # instruments in 3D: a plane just in front of the dark glass (whose front is at
        # 6 mm), behind the bezel's front. The panel's axes are U (world X, horizontal)
        # and V (world +Y projected onto the tilted panel), so the plane-coordinate
        # extent of the opening gives its width and height directly.
        off = 0.007
        loop = [lift(px, py, off) for px, py in outline(0)]
        us, vs = [p[0] for p in loop], [p[1] for p in loop]
        centre = plane_point(((min(us) + max(us)) / 2, (min(vs) + max(vs)) / 2), off)
        anchors[name] = (centre, n, (max(us) - min(us), max(vs) - min(vs)))

    def round_instrument(name, cx, cy, r, rim, k=16):
        # k is a multiple of 4, so the outline touches its extreme left/right/top/bottom.
        bezel(name, lambda g: circle_px(cx, cy, r + g, k), rim)

    def rect_instrument(name, x0, y0, x1, y1, rad, rim):
        bezel(name, lambda g: rrect_px(x0 - g, y0 - g, x1 + g, y1 + g, rad + g), rim)

    # The openings are fixed (the rules reference these rectangles); only rims changed.
    round_instrument("radar_front", 178, 618, 78, 20, 24)
    round_instrument("radar_rear", 1102, 618, 78, 20, 24)
    rect_instrument("mfd", 548, 514, 732, 700, 20, 10)
    round_instrument("gauge1", 334, 562, 40, 12)
    round_instrument("gauge2", 444, 562, 40, 12)
    round_instrument("gauge3", 836, 562, 40, 12)
    round_instrument("gauge4", 946, 562, 40, 12)
    rect_instrument("power", 292, 628, 488, 692, 6, 10)
    rect_instrument("systems", 792, 632, 988, 664, 6, 10)
    rect_instrument("message", 404, 474, 876, 500, 8, 6)

    # Winged-star badge on the crest above the message strip and MFD.
    cx, cy = 640, 437
    for side in (-1, 1):
        for k in range(3):  # three swept chrome feathers per wing
            y0 = cy - 9 + k * 7
            reach = 62 - k * 12
            feather = [(cx + side * 16, y0), (cx + side * (16 + reach), y0 - 3), (cx + side * (16 + reach * 0.8), y0 + 5),
                       (cx + side * 16, y0 + 5)]
            plate("badge_wing", feather, "chrome", 0.0, 0.006)
    plate("badge_disc", circle_px(cx, cy, 17, 20), "chrome", 0.0, 0.008)
    plate("badge_field", circle_px(cx, cy, 13, 20), "lacquer", 0.008, 0.002)
    star = []
    for i in range(10):
        a = math.radians(90 + 36 * i)
        r = 11 if i % 2 == 0 else 4.5
        star.append((cx + r * math.cos(a), cy - r * math.sin(a)))
    plate("badge_star", star, "ivory", 0.010, 0.002)

    # Chrome deco speed lines flanking the message strip.
    for k, py in enumerate((478, 488, 498)):
        for x0, x1 in ((318 + k * 18, 392), (888, 962 - k * 18)):
            plate("speedline", [(x0, py + 3), (x1, py + 3), (x1, py - 3), (x0, py - 3)], "chrome", 0.0, 0.006)

    # Bakelite knobs with brass caps, and two warning lamps flanking the MFD.
    def knob(px, py, r=0.022):
        c = plane_point(lift(px, py), 0.0)
        m.add(cylinder_between("knob", c, add(c, mul(n, 0.035)), r, "bakelite", 8))
        m.add(cylinder_between("knob_cap", add(c, mul(n, 0.035)), add(c, mul(n, 0.043)), r * 0.75, "brass", 8))

    knob(258, 700)
    knob(1022, 700)
    for px in (340, 500, 780, 940):
        knob(px, 756, 0.022)
    for px in (520, 760):
        c = plane_point(lift(px, 690), 0.0)
        m.add(sphere("lamp", 0.011, "lamp", 8, 4).transformed(IDENTITY, c))

    # Side consoles beside the pilot's knees, in front of the dash plane so they fill
    # the lower corners of a tall view without covering any instrument.
    for sx in (-1, 1):
        x_in, x_out = sx * 0.55, sx * 0.97
        lo, hi = min(x_in, x_out), max(x_in, x_out)
        m.add(box("console", lo, hi, -0.80, -0.47, -0.70, -0.05, "lacquer"))
        m.add(box("console_top", lo + 0.02, hi - 0.02, -0.47, -0.455, -0.68, -0.07, "ivory"))
        m.add(box("console_trim", lo - 0.005, hi + 0.005, -0.475, -0.465, -0.705, -0.045, "chrome"))
        for z in (-0.5, -0.25):
            c = (sx * 0.76, -0.455, z)
            m.add(cylinder_between("console_knob", c, add(c, (0, 0.03, 0)), 0.02, "bakelite", 8))
            m.add(cylinder_between("console_cap", add(c, (0, 0.03, 0)), add(c, (0, 0.037, 0)), 0.015, "brass", 8))

    # Canopy frame: slim chrome ribs from the dash corners up past the top corners of
    # the view, and an overhead hoop, each with a red lacquer strip on its inner edge.
    def rib(a, b, inward):
        m.add(cylinder_between("rib", a, b, 0.009, "chrome", 6))
        m.add(cylinder_between("rib_inner", add(a, inward), add(b, inward), 0.005, "lacquer", 6))

    for sx in (-1, 1):
        base = plane_point(lift(640 + sx * 655, 505), 0.0)
        mid = (sx * 0.9, 0.2, -1.0)
        top_p = (sx * 0.45, 0.62, -0.9)
        rib(base, mid, (-sx * 0.008, 0.0, 0.004))
        rib(mid, top_p, (-sx * 0.006, -0.006, 0.004))
        m.add(sphere("rib_joint", 0.0095, "chrome", 6, 4).transformed(IDENTITY, mid))
    rib((-0.45, 0.62, -0.9), (0.45, 0.62, -0.9), (0.0, -0.008, 0.004))
    return m, rects


def write_instruments(rects, anchors):
    """Writes guard_cockpit.instruments.yaml: each instrument as a 2D rectangle in the
    1280x720 design space and as an anchor rectangle on the cockpit model's surface.
    Written by hand (no YAML library) in a fixed order, so the file is deterministic.
    Replaces the older guard_cockpit.instruments.ini, which is removed."""
    folder = os.path.join(PACK, "models")
    old = os.path.join(folder, "guard_cockpit.instruments.ini")
    if os.path.exists(old):
        os.remove(old)
    order = ["message", "radar_front", "radar_rear", "mfd", "gauge1", "gauge2", "gauge3", "gauge4", "power", "systems"]

    def num(x):
        return f"{round(x, 4) + 0.0:g}"  # 4 decimals, no trailing zeros, never "-0"

    def vec(v):
        return "[" + ", ".join(num(c) for c in v) + "]"

    with open(os.path.join(folder, "guard_cockpit.instruments.yaml"), "w", newline="\n") as fh:
        fh.write("# Instrument placement for guard_cockpit.obj, generated by tools/default_assets/models.py.\n")
        fh.write("instruments:          # 2D rectangles in the 1280x720 design space (as before)\n")
        for k in order:
            fh.write(f"  {k}: [{', '.join(str(v) for v in rects[k])}]\n")
        fh.write("anchors:              # the same instruments as rectangles ON the model's surface\n")
        for k in order:
            at, normal, size = anchors[k]
            fh.write(f"  {k}: {{at: {vec(at)}, normal: {vec(normal)}, size: {vec(size)}}}\n")


# ---------------------------------------------------------------------------------------
# Building everything
# ---------------------------------------------------------------------------------------

# Model builder, number of debris pieces, debris seed.
SHIPS = [
    (comet, 4), (meteor, 4), (thunderbolt, 4), (victory, 5),
    (scout_saucer, 4), (raider_saucer, 4), (dreadnought, 5), (robot_sentinel, 3), (grappler, 4), (cargo_orb, 4),
    (senate_patrol, 4), (cartel_freighter, 5), (prospector_hauler, 4), (hotrod, 3),
    (guard_station, 5), (dominion_outpost, 5), (cosmium_asteroid, 5),
    # Station modules (appended last so the debris seeds of the models above stay put).
    (guard_turret, 3), (guard_shield_generator, 3), (dominion_turret_pod, 3), (dominion_shield_pod, 3),
    (sky_eye_probe, 3), (jammer_buoy, 3),
    (mining_facility, 3), (research_station, 3), (manufacturing_center, 3), (supply_ship, 3), (supply_pod, 2),
] + [
    # Construction stages (functools.partial-like closures; the default arguments bind
    # the loop values at definition time).
    (lambda b=b, s=s: build_stage(b, s), 2)
    for b in (mining_facility, research_station, manufacturing_center, guard_station) for s in (1, 2, 3)
]


def build_all(write=True):
    """Builds, validates and writes every model and its debris. Returns a list of
    summary rows (name, triangles, size, materials, debris count)."""
    report = []
    folder = os.path.join(PACK, "models")
    for i, (builder, pieces) in enumerate(SHIPS):
        model = builder()
        validate_model(model)
        debris = make_debris(model, pieces, seed=1000 + i)
        for d in debris:
            validate_model(d)
        if write:
            write_model(model, folder)
            for d in debris:
                write_model(d, folder)
        report.append((model, len(debris)))
    for model in chunks():
        validate_model(model)
        if write:
            write_model(model, folder)
        report.append((model, 0))
    cockpit, rects = guard_cockpit()
    validate_model(cockpit)
    if write:
        write_model(cockpit, folder)
        write_instruments(rects, cockpit.anchors)
    report.append((cockpit, 0))
    return report, rects


if __name__ == "__main__":
    rows, rects = build_all()
    for model, n in rows:
        lo, hi = model.bounds()
        print(f"{model.name:20s} {model.triangles():5d} tris  {hi[0]-lo[0]:6.1f} x {hi[1]-lo[1]:6.1f} x {hi[2]-lo[2]:6.1f} m"
              f"  debris {n}")
