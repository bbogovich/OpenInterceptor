"""Shared building blocks for the `default` campaign's asset generators.

Everything the generators make comes from the code in this folder: no image, sound or
model is read from anywhere else, and only the Python standard library is used. Every
random choice is drawn from a `random.Random(seed)` with a fixed seed, so running the
generators twice writes byte-identical files.

This module holds:

* **Small vector maths** on plain tuples (`add`, `sub`, `cross`, ...).
* **The mesh model**: a `Face` is one convex polygon with optional per-corner texture
  coordinates (uv) and normals; a `Part` is a list of faces that together form one
  *closed solid*; a `Model` is a set of parts plus a material table, and is what gets
  written as an OBJ + MTL pair.
* **Primitives** that build closed parts: `lathe` (surfaces of revolution: rocket
  hulls, nose cones, saucer discs, domes, tori), `prism` (a flat polygon given
  thickness: fins, plates, boxes), `cylinder_between` (struts, barrels, antennae),
  `frame` (a bezel around an opening) and `icosphere` (rocks).
* **Plane cutting** (`cut_part`, `make_debris`): splits a model into closed pieces for
  the debris families.
* **The validator** (`validate_model`), which every generated model must pass.
* **File writers**: OBJ/MTL, PNG (via `zlib`) and a tiny texture `Canvas`.

Conventions (the engine's modding rules): metres, +Y up, the nose along -Z. Faces are
one-sided; a face is visible from the side where its corners run counter-clockwise, so
every face of a closed part is wound counter-clockwise *seen from outside*. Equivalently
the face normal (b - a) x (c - a) points out of the solid, and the solid's signed volume
is positive. The validator checks exactly that.
"""
import math
import os
import random
import struct
import zlib

# Folder layout: this file lives in tools/default_assets/, the pack in resources/default/.
ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
PACK = os.path.join(ROOT, "resources", "default")


# ---------------------------------------------------------------------------------------
# Vector maths on tuples. Tuples are immutable, so a vertex can be shared freely.
# ---------------------------------------------------------------------------------------

def add(a, b):
    return (a[0] + b[0], a[1] + b[1], a[2] + b[2])


def sub(a, b):
    return (a[0] - b[0], a[1] - b[1], a[2] - b[2])


def mul(a, s):
    return (a[0] * s, a[1] * s, a[2] * s)


def dot(a, b):
    return a[0] * b[0] + a[1] * b[1] + a[2] * b[2]


def cross(a, b):
    return (a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0])


def length(a):
    return math.sqrt(dot(a, a))


def norm(a):
    n = length(a)
    return (a[0] / n, a[1] / n, a[2] / n) if n > 1e-12 else (0.0, 0.0, 0.0)


def lerp(a, b, t):
    return tuple(x + (y - x) * t for x, y in zip(a, b))


# 3x3 matrices are tuples of three row tuples; `apply(M, v)` is M * v.

def mat_apply(m, v):
    return (dot(m[0], v), dot(m[1], v), dot(m[2], v))


def mat_mul(a, b):
    cols = list(zip(*b))
    return tuple(tuple(dot(row, col) for col in cols) for row in a)


def mat_det(m):
    return dot(m[0], cross(m[1], m[2]))


def mat_inv_t(m):
    """Inverse transpose: the matrix that transforms *normals* when `m` transforms
    points. For a pure rotation it equals `m`; for a scale it divides instead."""
    d = mat_det(m)
    # The rows of the inverse transpose are the cofactors divided by the determinant.
    r0, r1, r2 = m
    return (mul(cross(r1, r2), 1 / d), mul(cross(r2, r0), 1 / d), mul(cross(r0, r1), 1 / d))


IDENTITY = ((1, 0, 0), (0, 1, 0), (0, 0, 1))


def rot_x(deg):
    c, s = math.cos(math.radians(deg)), math.sin(math.radians(deg))
    return ((1, 0, 0), (0, c, -s), (0, s, c))


def rot_y(deg):
    c, s = math.cos(math.radians(deg)), math.sin(math.radians(deg))
    return ((c, 0, s), (0, 1, 0), (-s, 0, c))


def rot_z(deg):
    c, s = math.cos(math.radians(deg)), math.sin(math.radians(deg))
    return ((c, -s, 0), (s, c, 0), (0, 0, 1))


def scale_m(sx, sy, sz):
    return ((sx, 0, 0), (0, sy, 0), (0, 0, sz))


# Lathes are built around +Z; this turns +Z into +Y (for saucers, towers, robots).
Z_TO_Y = rot_x(-90)


def frame_from_z(direction):
    """A rotation taking +Z onto `direction` (columns X', Y', Z' form a right-handed
    frame, so the determinant is +1 and winding is preserved)."""
    z = norm(direction)
    up = (0, 1, 0) if abs(z[1]) < 0.9 else (1, 0, 0)
    x = norm(cross(up, z))
    y = cross(z, x)
    # Columns x, y, z -> rows of the matrix are the transposed triples.
    return ((x[0], y[0], z[0]), (x[1], y[1], z[1]), (x[2], y[2], z[2]))


# ---------------------------------------------------------------------------------------
# The mesh model
# ---------------------------------------------------------------------------------------

class Face:
    """One convex polygon. `pts` are positions; `uvs` and `nrm` are per-corner lists
    (or None: the material is untextured / the face is flat-shaded, and the engine then
    uses the face normal - which is how fins get their hard edges)."""

    __slots__ = ("mat", "pts", "uvs", "nrm")

    def __init__(self, mat, pts, uvs=None, nrm=None):
        self.mat, self.pts, self.uvs, self.nrm = mat, list(pts), uvs and list(uvs), nrm and list(nrm)

    def reversed(self):
        r = lambda x: x[::-1] if x else x
        return Face(self.mat, self.pts[::-1], r(self.uvs), r(self.nrm))


class Part:
    """A closed solid made of faces. `closed=False` exempts a part from the closed-solid
    checks (nothing in the default pack needs that, but a modder's generator might)."""

    def __init__(self, name, faces=None, closed=True):
        self.name, self.faces, self.closed = name, list(faces or []), closed

    def transformed(self, m=IDENTITY, t=(0, 0, 0)):
        """A copy moved by the linear map `m` then translated by `t`. Normals use the
        inverse transpose; a mirroring map (negative determinant) would turn the solid
        inside out, so the faces are reversed to keep them wound outward."""
        nm = mat_inv_t(m)
        flip = mat_det(m) < 0
        out = []
        for f in self.faces:
            g = Face(f.mat, [add(mat_apply(m, p), t) for p in f.pts], f.uvs,
                     [norm(mat_apply(nm, n)) for n in f.nrm] if f.nrm else None)
            out.append(g.reversed() if flip else g)
        return Part(self.name, out, self.closed)

    def recolor(self, mat):
        """The same part with every face given material `mat`."""
        return Part(self.name, [Face(mat, f.pts, f.uvs, f.nrm) for f in self.faces], self.closed)

    def bounds(self):
        pts = [p for f in self.faces for p in f.pts]
        return tuple(min(p[i] for p in pts) for i in range(3)), tuple(max(p[i] for p in pts) for i in range(3))


class Model:
    """A named set of parts and the material table they use.

    `materials` maps a material name to `(Kd colour, texture file name or None)`.
    `budget` is the triangle limit the validator enforces."""

    def __init__(self, name, materials, budget=1500):
        self.name, self.materials, self.budget, self.parts = name, dict(materials), budget, []

    def add(self, part, m=IDENTITY, t=(0, 0, 0)):
        self.parts.append(part.transformed(m, t) if (m is not IDENTITY or t != (0, 0, 0)) else part)
        return self

    def add_mirrored_x(self, part):
        """Adds `part` and its mirror image across the X = 0 plane (left/right pairs)."""
        self.parts.append(part)
        self.parts.append(part.transformed(scale_m(-1, 1, 1)))
        return self

    def triangles(self):
        return sum(len(f.pts) - 2 for p in self.parts for f in p.faces)

    def bounds(self):
        bs = [p.bounds() for p in self.parts]
        return (tuple(min(b[0][i] for b in bs) for i in range(3)), tuple(max(b[1][i] for b in bs) for i in range(3)))


# ---------------------------------------------------------------------------------------
# Primitives
# ---------------------------------------------------------------------------------------

def lathe(name, profile, mat, segments=16, uv=None, uv_mode="along", u_repeat=1.0, phase=0.0,
          crease=40.0, smooth=True, closed_loop=False, cap_mat=None):
    """A surface of revolution around the +Z axis - the workhorse for rocket hulls,
    nose cones, exhaust bells, saucer discs, domes and tori.

    `profile` is a list of `(z, r)` or `(z, r, material)` points; the material applies to
    the band from that point to the next, so a chrome trim band is just a profile step
    with its own material. A first or last point with `r == 0` closes the end in a point
    (a nose tip); otherwise the end gets a flat cap. `closed_loop=True` treats the profile
    as a closed curve (a torus cross-section) and makes no caps.

    Each profile point is swept around in `segments` steps. A point at angle `a` sits at
    `(r sin a, r cos a, z)`, so `a = 0` is the top (+Y) of the hull.

    Smooth shading: every vertex gets a normal computed from the profile. Where two
    neighbouring profile segments meet at more than `crease` degrees (the lip of an
    exhaust bell, a trim step) each side keeps its own normal, which gives a hard edge.

    Texture coordinates (`uv` = (u0, u1, v0, v1) or None):
      * `uv_mode="along"` (rockets): u runs along the profile's length from u0 (first
        point) to u1, v runs around from v0 (top) through the belly and back.
      * `uv_mode="around"` (saucers, towers): u runs around (`u_repeat` times), v runs
        along the profile from v0 (first point) to v1.
    """
    pts = [(p[0], p[1], p[2] if len(p) > 2 else mat) for p in profile]
    n_p = len(pts)
    n_seg = n_p if closed_loop else n_p - 1
    # Profile-plane normal of each segment: the tangent (dz, dr) turned to the outside,
    # which for a curve traversed nose-to-tail is (-dr, dz) in (axial, radial) terms.
    seg_n = []
    for j in range(n_seg):
        z0, r0, _ = pts[j]
        z1, r1, _ = pts[(j + 1) % n_p]
        dz, dr = z1 - z0, r1 - r0
        L = math.hypot(dz, dr) or 1.0
        seg_n.append((-dr / L, dz / L))
    # Per-segment, per-end vertex normals (2D), honouring creases.
    cos_c = math.cos(math.radians(crease))

    def end_normal(j, at_start):
        """The 2D normal at one end of segment j (the start or end point)."""
        own = seg_n[j]
        if not smooth:
            return own
        k = j - 1 if at_start else j + 1
        if not closed_loop and (k < 0 or k >= n_seg):
            return own
        other = seg_n[k % n_seg]
        if own[0] * other[0] + own[1] * other[1] < cos_c:
            return own
        a = (own[0] + other[0], own[1] + other[1])
        L = math.hypot(*a) or 1.0
        return (a[0] / L, a[1] / L)

    # Arc length along the profile drives the "along" texture coordinate.
    arc = [0.0]
    for j in range(n_seg):
        z0, r0, _ = pts[j]
        z1, r1, _ = pts[(j + 1) % n_p]
        arc.append(arc[-1] + math.hypot(z1 - z0, r1 - r0))
    total = arc[-1] or 1.0

    def angle(k):
        return 2 * math.pi * k / segments + math.radians(phase)

    def pos(j, k):
        z, r, _ = pts[j % n_p]
        a = angle(k)
        return (r * math.sin(a), r * math.cos(a), z)

    def nrm3(n2, a):
        nz, nr = n2
        return (nr * math.sin(a), nr * math.cos(a), nz)

    def tex(j_arc, k):
        if uv is None:
            return None
        u0, u1, v0, v1 = uv
        s = arc[j_arc] / total
        f = k / segments
        if uv_mode == "along":
            return (u0 + (u1 - u0) * s, v0 + (v1 - v0) * f)
        return (u0 + (u1 - u0) * f * u_repeat, v0 + (v1 - v0) * s)

    faces = []
    for j in range(n_seg):
        m = pts[j][2]
        a_apex = not closed_loop and pts[j][1] == 0
        b_apex = not closed_loop and pts[(j + 1) % n_p][1] == 0
        if a_apex and b_apex:
            continue
        na, nb = end_normal(j, True), end_normal(j, False)
        for k in range(segments):
            mid = (angle(k) + angle(k + 1)) / 2
            # Quad corners in outward counter-clockwise order (see module docs).
            corners = [(j, k, na, j), (j + 1, k, nb, j + 1), (j + 1, k + 1, nb, j + 1), (j, k + 1, na, j)]
            if a_apex:
                corners = [corners[1], corners[2], (j, k, na, j)]
                apex_mid = True
            elif b_apex:
                corners = [corners[0], (j + 1, k, nb, j + 1), corners[3]]
                apex_mid = True
            else:
                apex_mid = False
            p, t, nn = [], [], []
            for (jj, kk, n2, ja) in corners:
                is_apex = pts[jj % n_p][1] == 0 and not closed_loop
                a = mid if (apex_mid and is_apex) else angle(kk)
                p.append(pos(jj, kk) if not is_apex else (0.0, 0.0, pts[jj % n_p][0]))
                if uv is not None:
                    t.append(tex(ja, kk) if not is_apex else tex(ja, k + 0.5))
                nn.append(norm(nrm3(n2, a)))
            faces.append(Face(m, p, t if uv is not None else None, nn))
    if not closed_loop:
        for end in (0, n_p - 1):
            z, r, m = pts[end]
            if r == 0:
                continue
            m = cap_mat or (pts[0][2] if end == 0 else pts[n_p - 2][2])
            c = (0.0, 0.0, z)
            nz = (0.0, 0.0, -1.0 if end == 0 else 1.0)
            ja = 0 if end == 0 else n_seg
            for k in range(segments):
                a, b = pos(end, k), pos(end, k + 1)
                ta, tb, tc = tex(ja, k), tex(ja, k + 1), tex(ja, k + 0.5)
                if uv is not None and uv_mode == "around":
                    tc = (tc[0], uv[2] if end == 0 else uv[3])
                if end == 0:
                    tri, tt = [c, a, b], [tc, ta, tb]
                else:
                    tri, tt = [c, b, a], [tc, tb, ta]
                faces.append(Face(m, tri, tt if uv is not None else None, [nz] * 3))
    part = Part(name, faces)
    if signed_volume(part) < 0:
        # The profile was given tail-to-nose (z decreasing), which sweeps the surface
        # inside out: turn every face and normal around.
        part = Part(name, [Face(f.mat, f.pts[::-1], f.uvs and f.uvs[::-1], f.nrm and [mul(n, -1) for n in f.nrm[::-1]])
                           for f in faces])
    return part


def torus(name, major, minor, mat, segments=24, sides=8, uv=None, phase=0.0):
    """A ring around the Z axis: a lathe of a closed circular cross-section."""
    prof = [(minor * math.sin(2 * math.pi * i / sides), major + minor * math.cos(2 * math.pi * i / sides))
            for i in range(sides)]
    return lathe(name, prof, mat, segments, uv=uv, uv_mode="around", closed_loop=True, phase=phase, crease=90)


def sphere(name, radius, mat, segments=12, rings=8, uv=None, uv_mode="around", u_repeat=1.0):
    """A UV sphere around the origin (poles on the Z axis), smooth-shaded."""
    prof = [(-radius * math.cos(math.pi * i / rings), radius * math.sin(math.pi * i / rings)) for i in range(rings + 1)]
    prof[0] = (prof[0][0], 0.0)
    prof[-1] = (prof[-1][0], 0.0)
    return lathe(name, prof, mat, segments, uv=uv, uv_mode=uv_mode, u_repeat=u_repeat, crease=90)


def cylinder_between(name, a, b, radius, mat, segments=6, radius_b=None, smooth=True):
    """A capped cylinder (or frustum, with `radius_b`) from point `a` to point `b`:
    struts, gun barrels, antennae and arm segments. `radius_b = 0` makes a spike."""
    d = sub(b, a)
    L = length(d)
    rb = radius if radius_b is None else radius_b
    part = lathe(name, [(0.0, radius), (L, rb)], mat, segments, smooth=smooth)
    return part.transformed(frame_from_z(d), a)


def triangulate(poly2d):
    """Ear-clipping triangulation of a simple counter-clockwise polygon. Returns index
    triples. An *ear* is a convex corner whose triangle contains no other corner; cutting
    ears off one at a time always succeeds for a simple polygon. Corners lying on a
    straight line (zero turn) are never used as ear tips, which keeps every triangle
    non-degenerate."""
    idx = list(range(len(poly2d)))
    tris = []

    def cr(o, a, b):
        return (a[0] - o[0]) * (b[1] - o[1]) - (a[1] - o[1]) * (b[0] - o[0])

    def inside(p, a, b, c):
        return cr(a, b, p) >= -1e-12 and cr(b, c, p) >= -1e-12 and cr(c, a, p) >= -1e-12

    guard = 0
    while len(idx) > 3:
        guard += 1
        if guard > 10000:
            raise ValueError("triangulation failed (self-intersecting outline?)")
        n = len(idx)
        for i in range(n):
            i0, i1, i2 = idx[(i - 1) % n], idx[i], idx[(i + 1) % n]
            a, b, c = poly2d[i0], poly2d[i1], poly2d[i2]
            if cr(a, b, c) <= 1e-12:
                continue
            if any(inside(poly2d[j], a, b, c) for j in idx if j not in (i0, i1, i2) and poly2d[j] not in (a, b, c)):
                continue
            tris.append((i0, i1, i2))
            del idx[i]
            break
        else:
            raise ValueError("triangulation found no ear (polygon is not simple or not CCW)")
    tris.append(tuple(idx))
    return tris


def poly_area2d(poly):
    return 0.5 * sum(poly[i][0] * poly[(i + 1) % len(poly)][1] - poly[(i + 1) % len(poly)][0] * poly[i][1]
                     for i in range(len(poly)))


def prism(name, poly2d, thickness, mat, origin=(0, 0, 0), s_axis=(1, 0, 0), t_axis=(0, 1, 0), uv=None,
          side_mat=None):
    """A flat outline given thickness: fins, plates, wedges, boxes.

    `poly2d` is a counter-clockwise outline in the (s, t) plane spanned by `s_axis` and
    `t_axis` at `origin`; it is extruded symmetrically along n = s x t. `thickness` is a
    number or one value per outline corner, so a fin can be thick at the root and thin
    at its tip (every face is still planar: caps are triangulated, and each side quad
    lies in the plane of its edge and n). Flat-shaded: no normals are written, so the
    engine uses face normals and the edges stay crisp.

    `uv` = (scale_u, scale_v, offset_u, offset_v) maps (s, t) onto the texture for the
    caps; sides get a strip mapping."""
    if poly_area2d(poly2d) < 0:
        poly2d = poly2d[::-1]
        if isinstance(thickness, (list, tuple)):
            thickness = list(thickness)[::-1]
    S, T = norm(s_axis), norm(t_axis)
    N = norm(cross(S, T))
    th = thickness if isinstance(thickness, (list, tuple)) else [thickness] * len(poly2d)

    def P(i, sign):
        s, t = poly2d[i]
        return add(origin, add(add(mul(S, s), mul(T, t)), mul(N, sign * th[i] / 2)))

    def UV(i):
        if uv is None:
            return None
        su, sv, ou, ov = uv
        return (ou + poly2d[i][0] * su, ov + poly2d[i][1] * sv)

    faces = []
    tris = triangulate(poly2d)
    for a, b, c in tris:
        faces.append(Face(mat, [P(a, 1), P(b, 1), P(c, 1)], uv and [UV(a), UV(b), UV(c)]))
        faces.append(Face(mat, [P(c, -1), P(b, -1), P(a, -1)], uv and [UV(c), UV(b), UV(a)]))
    n = len(poly2d)
    for i in range(n):
        j = (i + 1) % n
        q = [P(i, -1), P(j, -1), P(j, 1), P(i, 1)]
        faces.append(Face(side_mat or mat, q, uv and [UV(i), UV(j), UV(j), UV(i)]))
    return Part(name, faces)


def box(name, x0, x1, y0, y1, z0, z1, mat, uv_scale=None):
    """An axis-aligned box. With `uv_scale` (texture repeats per metre) each face is
    planar-mapped, which suits hulls built from boxes (the prospector hauler)."""
    c = [(x, y, z) for x in (x0, x1) for y in (y0, y1) for z in (z0, z1)]
    quads = [(0, 1, 3, 2), (4, 6, 7, 5), (0, 4, 5, 1), (2, 3, 7, 6), (0, 2, 6, 4), (1, 5, 7, 3)]
    faces = []
    for q in quads:
        p = [c[i] for i in q]
        t = None
        if uv_scale:
            n = cross(sub(p[1], p[0]), sub(p[2], p[0]))
            ax = max(range(3), key=lambda i: abs(n[i]))
            a, b = [i for i in range(3) if i != ax]
            t = [(v[a] * uv_scale, v[b] * uv_scale) for v in p]
        faces.append(Face(mat, p, t))
    return Part(name, faces)


def oriented_box(name, center, axes, half, mat):
    """A box with edges along the three given (right-handed, orthogonal) axes."""
    X, Y, Z = (norm(a) for a in axes)
    hx, hy, hz = half
    m = ((X[0] * hx, Y[0] * hy, Z[0] * hz), (X[1] * hx, Y[1] * hy, Z[1] * hz), (X[2] * hx, Y[2] * hy, Z[2] * hz))
    return box(name, -1, 1, -1, 1, -1, 1, mat).transformed(m, center)


def frame(name, inner, outer, depth, mat, origin, s_axis, t_axis, inner_front=None, outer_front=None):
    """A bezel: the solid ring between an `inner` and an `outer` outline (same number of
    points, both counter-clockwise in the (s, t) plane), `depth` thick along n = s x t,
    with its back at n = 0 and its front face at n = +depth. The front outlines may
    differ from the back ones (`inner_front`, `outer_front`), e.g. to make the opening a
    tunnel that lines up with the viewer's line of sight."""
    inner_front = inner_front or inner
    outer_front = outer_front or outer
    S, T = norm(s_axis), norm(t_axis)
    N = norm(cross(S, T))

    def P(p, d):
        return add(origin, add(add(mul(S, p[0]), mul(T, p[1])), mul(N, d)))

    faces = []
    n = len(inner)
    for i in range(n):
        j = (i + 1) % n
        fi, fj = P(inner_front[i], depth), P(inner_front[j], depth)
        gi, gj = P(outer_front[i], depth), P(outer_front[j], depth)
        bi, bj, ci, cj = P(inner[i], 0), P(inner[j], 0), P(outer[i], 0), P(outer[j], 0)
        faces.append(Face(mat, [fi, gi, gj, fj]))      # front ring, facing +N
        faces.append(Face(mat, [bi, bj, cj, ci]))      # back ring, facing -N
        faces.append(Face(mat, [ci, cj, gj, gi]))      # outer wall
        faces.append(Face(mat, [bi, fi, fj, bj]))      # inner wall (faces the opening)
    part = Part(name, faces)
    if signed_volume(part) < 0:  # a clockwise outline was passed: turn it outward
        part = Part(name, [f.reversed() for f in faces])
    return part


def icosphere(name, radius, mat, subdivisions=1, rough=0.0, seed=1, uv=False):
    """A geodesic sphere (an icosahedron with each triangle split into four,
    `subdivisions` times), pushed in and out by `rough` (a fraction of the radius) for
    rocks. Flat-shaded. Displacement is radial, so the solid stays closed and outward."""
    t = (1 + math.sqrt(5)) / 2
    verts = [norm(v) for v in [(-1, t, 0), (1, t, 0), (-1, -t, 0), (1, -t, 0), (0, -1, t), (0, 1, t),
                               (0, -1, -t), (0, 1, -t), (t, 0, -1), (t, 0, 1), (-t, 0, -1), (-t, 0, 1)]]
    tris = [(0, 11, 5), (0, 5, 1), (0, 1, 7), (0, 7, 10), (0, 10, 11), (1, 5, 9), (5, 11, 4), (11, 10, 2),
            (10, 7, 6), (7, 1, 8), (3, 9, 4), (3, 4, 2), (3, 2, 6), (3, 6, 8), (3, 8, 9), (4, 9, 5), (2, 4, 11),
            (6, 2, 10), (8, 6, 7), (9, 8, 1)]
    for _ in range(subdivisions):
        cache, new = {}, []

        def mid(a, b):
            key = (min(a, b), max(a, b))
            if key not in cache:
                verts.append(norm(lerp(verts[a], verts[b], 0.5)))
                cache[key] = len(verts) - 1
            return cache[key]

        for a, b, c in tris:
            ab, bc, ca = mid(a, b), mid(b, c), mid(c, a)
            new += [(a, ab, ca), (b, bc, ab), (c, ca, bc), (ab, bc, ca)]
        tris = new
    rng = random.Random(seed)
    radii = [radius * (1 + rough * (2 * rng.random() - 1)) for _ in verts]
    pos = [mul(v, r) for v, r in zip(verts, radii)]
    faces = []
    for a, b, c in tris:
        tt = None
        if uv:
            tt = [((math.atan2(verts[i][0], verts[i][2]) / (2 * math.pi)) % 1.0, verts[i][1] * 0.5 + 0.5) for i in (a, b, c)]
            us = [x[0] for x in tt]
            if max(us) - min(us) > 0.5:  # straddles the seam: unwrap so it does not smear
                tt = [((u + 1) if u < 0.5 else u, v) for u, v in tt]
        faces.append(Face(mat, [pos[a], pos[b], pos[c]], tt))
    part = Part(name, faces)
    if signed_volume(part) < 0:
        part = Part(name, [f.reversed() for f in part.faces])
    return part


# ---------------------------------------------------------------------------------------
# Measuring and validating
# ---------------------------------------------------------------------------------------

def fmt(x):
    """Formats a coordinate exactly as the OBJ writer does; `+ 0.0` turns -0.0 into 0.0
    so a point never gets two spellings."""
    return f"{round(x, 4) + 0.0:.4f}"


def key(p):
    return (fmt(p[0]), fmt(p[1]), fmt(p[2]))


def face_normal(f):
    """Newell's method: an area-weighted normal that works for any planar polygon."""
    n = [0.0, 0.0, 0.0]
    pts = f.pts
    for i in range(len(pts)):
        a, b = pts[i], pts[(i + 1) % len(pts)]
        n[0] += (a[1] - b[1]) * (a[2] + b[2])
        n[1] += (a[2] - b[2]) * (a[0] + b[0])
        n[2] += (a[0] - b[0]) * (a[1] + b[1])
    return tuple(n)


def signed_volume(part):
    """Sum over the triangles of a·(b×c)/6 - the divergence theorem. Positive when the
    faces point outward."""
    v = 0.0
    for f in part.faces:
        a = f.pts[0]
        for i in range(1, len(f.pts) - 1):
            v += dot(a, cross(f.pts[i], f.pts[i + 1])) / 6
    return v


class ValidationError(Exception):
    pass


def validate_part(part, where):
    """Raises ValidationError unless `part` is a well-formed outward closed solid.

    1. *Degenerate faces*: every fan triangle must have area > 1e-7 m^2 (after rounding
       to the precision the file stores).
    2. *Closed and consistently wound*: each directed edge a->b appears exactly once and
       its reverse b->a exactly once. A hole leaves an edge without a partner; a face
       wound the other way repeats a directed edge.
    3. *Outward*: the signed volume is positive.
    4. *Normals*: every written vertex normal points to the same side as its face.
    """
    errors = []
    edges = {}
    for fi, f in enumerate(part.faces):
        ks = [key(p) for p in f.pts]
        q = [tuple(float(c) for c in k) for k in ks]
        if len(set(ks)) != len(ks):
            errors.append(f"face {fi} repeats a corner")
            continue
        for i in range(1, len(q) - 1):
            if length(cross(sub(q[i], q[0]), sub(q[i + 1], q[0]))) / 2 <= 1e-7:
                errors.append(f"face {fi} ({f.mat}) is degenerate")
                break
        if f.nrm:
            fn = face_normal(f)
            if any(dot(n, fn) <= 0 for n in f.nrm):
                errors.append(f"face {fi} ({f.mat}) has a vertex normal pointing inward")
        if part.closed:
            for i in range(len(ks)):
                e = (ks[i], ks[(i + 1) % len(ks)])
                edges[e] = edges.get(e, 0) + 1
    if part.closed:
        bad = [e for e, c in edges.items() if c != 1 or edges.get((e[1], e[0]), 0) != 1]
        if bad:
            errors.append(f"{len(bad)} edges are open or inconsistently wound, e.g. {bad[0]}")
        vol = signed_volume(part)
        if vol <= 0:
            errors.append(f"signed volume {vol:.6f} is not positive (faces point inward)")
    if errors:
        raise ValidationError(f"{where} / part '{part.name}': " + "; ".join(errors[:5]))


def validate_model(model):
    for part in model.parts:
        validate_part(part, model.name)
    tris = model.triangles()
    if tris > model.budget:
        raise ValidationError(f"{model.name}: {tris} triangles exceeds the budget of {model.budget}")
    used = {f.mat for p in model.parts for f in p.faces}
    missing = used - set(model.materials)
    if missing:
        raise ValidationError(f"{model.name}: materials {sorted(missing)} are not defined")


# ---------------------------------------------------------------------------------------
# Plane cutting (debris)
# ---------------------------------------------------------------------------------------

SCAR = "scar"  # material name of the cut faces of a debris piece


def _clip_face(f, dist, plane_n, plane_d, keyed_cut):
    """Clips one convex face to the half-space dist <= 0 (Sutherland-Hodgman). Returns
    the clipped face (or None) and the cut segment, as the pair of points where the
    outline leaves and re-enters the half-space."""
    n = len(f.pts)
    out_p, out_t, out_n = [], [], []
    exit_pt = enter_pt = None
    for i in range(n):
        j = (i + 1) % n
        di, dj = dist[i], dist[j]
        if di <= 0:
            out_p.append(f.pts[i])
            out_t.append(f.uvs[i] if f.uvs else None)
            out_n.append(f.nrm[i] if f.nrm else None)
        if (di <= 0) != (dj <= 0):
            # The crossing point must be bit-identical for the two faces that share this
            # edge (they walk it in opposite directions), so compute it from the edge's
            # endpoints in a fixed order.
            a, b, da, db = (i, j, di, dj) if key(f.pts[i]) < key(f.pts[j]) else (j, i, dj, di)
            t = da / (da - db)
            x = keyed_cut(f.pts[a], f.pts[b], t)
            xt = lerp(f.uvs[a], f.uvs[b], t) if f.uvs else None
            xn = norm(lerp(f.nrm[a], f.nrm[b], t)) if f.nrm else None
            out_p.append(x)
            out_t.append(xt)
            out_n.append(xn)
            if di <= 0:
                exit_pt = x
            else:
                enter_pt = x
    if len(out_p) < 3:
        return None, None
    g = Face(f.mat, out_p, out_t if f.uvs else None, out_n if f.nrm else None)
    seg = (enter_pt, exit_pt) if exit_pt is not None and enter_pt is not None else None
    return g, seg


def _half(part, n, d):
    """The piece of `part` on the side n·p <= d, closed with a flat cap. Returns None if
    nothing of the part is on that side."""
    cache = {}

    def cut(a, b, t):
        k = (key(a), key(b))
        if k not in cache:
            cache[k] = lerp(a, b, t)
        return cache[k]

    faces, segs = [], []
    for f in part.faces:
        dist = [dot(n, p) - d for p in f.pts]
        if all(x <= 0 for x in dist):
            faces.append(f)
            continue
        if all(x > 0 for x in dist):
            continue
        g, seg = _clip_face(f, dist, n, d, cut)
        if g:
            faces.append(g)
        if seg:
            segs.append(seg)
    if not faces:
        return None
    if segs:
        # The cap's outline: each clipped face left an edge exit->enter on the plane; the
        # cap needs the reverse edge enter->exit, so chaining those gives outlines wound
        # counter-clockwise seen from outside (from +n).
        nxt = {}
        pt = {}
        for a, b in segs:
            nxt[key(a)] = key(b)
            pt[key(a)], pt[key(b)] = a, b
        U = norm(cross(n, (1, 0, 0) if abs(n[0]) < 0.9 else (0, 1, 0)))
        V = cross(n, U)  # (U, V, n) right-handed, so CCW in (U, V) is CCW seen from +n
        seen = set()
        for start in sorted(nxt):
            if start in seen:
                continue
            loop, k = [], start
            while k not in seen:
                seen.add(k)
                loop.append(pt[k])
                if k not in nxt:
                    raise ValidationError(f"{part.name}: cut outline is not closed")
                k = nxt[k]
            if k != start:
                raise ValidationError(f"{part.name}: cut outline is not a simple loop")
            p2 = [(dot(p, U), dot(p, V)) for p in loop]
            if poly_area2d(p2) <= 0:
                raise ValidationError(f"{part.name}: cut made a cap with a hole; choose another plane")
            for a, b, c in triangulate(p2):
                faces.append(Face(SCAR, [loop[a], loop[b], loop[c]]))
    return Part(part.name, faces, part.closed)


def cut_part(part, n, d):
    """Splits a closed part along the plane n·p = d into (back, front) closed pieces;
    either may be None."""
    back = _half(part, n, d)
    front = _half(part, mul(n, -1), -d)
    return back, front


def _safe_offset(parts, n, d, extent):
    """Nudges the plane offset so no vertex lies within 5 mm of it (a vertex on the plane
    would make a zero-area sliver). Deterministic: tries offsets in a fixed order."""
    keys = sorted({dot(n, p) for part in parts for f in part.faces for p in f.pts})
    gap = max(0.005, extent * 1e-4)
    for step in range(200):
        cand = d + (step // 2 + 1) * gap * 0.37 * (1 if step % 2 else -1) if step else d
        if all(abs(k - cand) > gap for k in keys):
            return cand
    raise ValidationError("could not find a clean cutting plane")


def make_debris(model, pieces, seed):
    """Cuts `model` into `pieces` closed pieces by repeatedly splitting the largest piece
    with a plane across its longest dimension (tilted a little at random). Pieces stay in
    the model's own coordinates, so placed at the origin they reassemble the ship."""
    rng = random.Random(seed)
    # Attachment points (slot_* groups) mark the intact model only; they are left out.
    groups = [[p for p in model.parts if not p.name.startswith("slot_")]]
    while len(groups) < pieces:
        groups.sort(key=lambda g: -_extent(g))
        g = groups.pop(0)
        lo, hi = _bounds(g)
        size = sub(hi, lo)
        axis = max(range(3), key=lambda i: size[i])
        for attempt in range(40):
            nv = [rng.uniform(-0.35, 0.35) for _ in range(3)]
            nv[axis] = 1.0
            n = norm(tuple(nv))
            centre = mul(add(lo, hi), 0.5)
            d = dot(n, centre) + size[axis] * rng.uniform(-0.18, 0.18)
            d = _safe_offset(g, n, d, max(size))
            back, front = [], []
            try:
                for part in g:
                    b, f = cut_part(part, n, d)
                    if b:
                        back.append(b)
                    if f:
                        front.append(f)
            except ValidationError:
                continue
            if back and front and min(_extent(back), _extent(front)) > 0.15 * _extent(g):
                groups += [back, front]
                break
        else:
            raise ValidationError(f"{model.name}: could not cut into {pieces} pieces")
    groups.sort(key=lambda g: _centre(g)[::-1])  # by z first, so the nose (-Z) piece is debris1
    out = []
    for i, g in enumerate(groups):
        m = Model(f"{model.name}_debris{i + 1}", dict(model.materials, **{SCAR: ((0.16, 0.13, 0.12), None)}),
                  model.budget + 400)
        m.parts = g
        out.append(m)
    return out


def _bounds(parts):
    bs = [p.bounds() for p in parts]
    return tuple(min(b[0][i] for b in bs) for i in range(3)), tuple(max(b[1][i] for b in bs) for i in range(3))


def _extent(parts):
    lo, hi = _bounds(parts)
    return length(sub(hi, lo))


def _centre(parts):
    lo, hi = _bounds(parts)
    return mul(add(lo, hi), 0.5)


# ---------------------------------------------------------------------------------------
# Writers
# ---------------------------------------------------------------------------------------

def write_model(model, folder=None, header="tools/default_assets/models.py"):
    """Writes `<name>.obj` and `<name>.mtl`. Positions, uvs and normals are de-duplicated
    by their printed text (so equal values share one index); materials appear in the MTL
    in sorted order. Texture paths are written relative to the OBJ (`../textures/x.png`),
    which the engine resolves within the pack."""
    folder = folder or os.path.join(PACK, "models")
    used = sorted({f.mat for p in model.parts for f in p.faces})
    with open(os.path.join(folder, model.name + ".mtl"), "w", newline="\n") as fh:
        fh.write(f"# {model.name}: generated by {header}\n")
        for m in used:
            rgb, tex = model.materials[m]
            fh.write(f"newmtl {m}\nKd {rgb[0]:.3f} {rgb[1]:.3f} {rgb[2]:.3f}\n")
            if tex:
                fh.write(f"map_Kd ../textures/{tex}\n")
    vs, vts, vns = {}, {}, {}
    lines_v, lines_t, lines_n, body = [], [], [], []

    def index(table, lines, prefix, text):
        if text not in table:
            table[text] = len(table) + 1  # OBJ indices start at 1
            lines.append(f"{prefix} {text}\n")
        return table[text]

    current = None
    for part in model.parts:
        body.append(f"g {part.name}\n")
        current = None
        for f in part.faces:
            if f.mat != current:
                body.append(f"usemtl {f.mat}\n")
                current = f.mat
            corners = []
            for i, p in enumerate(f.pts):
                vi = index(vs, lines_v, "v", " ".join(key(p)))
                ti = index(vts, lines_t, "vt", f"{round(f.uvs[i][0], 4) + 0.0:.4f} {round(f.uvs[i][1], 4) + 0.0:.4f}") if f.uvs else None
                ni = index(vns, lines_n, "vn", " ".join(f"{round(c, 3) + 0.0:.3f}" for c in f.nrm[i])) if f.nrm else None
                corners.append(f"{vi}" + (f"/{ti}" if ti else ("/" if ni else "")) + (f"/{ni}" if ni else ""))
            body.append("f " + " ".join(corners) + "\n")
    with open(os.path.join(folder, model.name + ".obj"), "w", newline="\n") as fh:
        fh.write(f"# {model.name}: generated by {header}\n# {model.triangles()} triangles\n")
        fh.write(f"mtllib {model.name}.mtl\n")
        fh.writelines(lines_v)
        fh.writelines(lines_t)
        fh.writelines(lines_n)
        fh.writelines(body)


def png_bytes(width, height, rgb_rows):
    """Encodes 8-bit RGB rows (each a bytes object of width*3) as a PNG file. A PNG is a
    signature then chunks (length, type, data, CRC); the pixels go in IDAT, each row
    prefixed with filter type 0 ("none") and the whole stream zlib-compressed."""
    raw = b"".join(b"\x00" + bytes(r) for r in rgb_rows)

    def chunk(t, d):
        return struct.pack(">I", len(d)) + t + d + struct.pack(">I", zlib.crc32(t + d) & 0xFFFFFFFF)

    return (b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", width, height, 8, 2, 0, 0, 0))
            + chunk(b"IDAT", zlib.compress(raw, 9)) + chunk(b"IEND", b""))


def png_read_rgb(data):
    """Decodes the PNGs this module writes (8-bit RGB, filter 0) - used by the preview
    renderer and tests, not a general PNG reader."""
    pos, w, h, idat = 8, 0, 0, b""
    while pos < len(data):
        n = struct.unpack(">I", data[pos:pos + 4])[0]
        t = data[pos + 4:pos + 8]
        d = data[pos + 8:pos + 8 + n]
        if t == b"IHDR":
            w, h = struct.unpack(">II", d[:8])
        elif t == b"IDAT":
            idat += d
        pos += 12 + n
    raw = zlib.decompress(idat)
    rows = [raw[y * (w * 3 + 1) + 1:(y + 1) * (w * 3 + 1)] for y in range(h)]
    return w, h, rows


class Canvas:
    """A floating-point RGB image (values 0..1) with a few painting helpers. Pixel
    (0, 0) is the top-left, as in the PNG file.

    A note on texture coordinates: OBJ's v axis points *up* the image, and the engine
    flips it when loading, so a vertex with uv (u, v) samples pixel
    (u * width, (1 - v) * height)."""

    def __init__(self, w, h, color=(0, 0, 0)):
        self.w, self.h = w, h
        self.px = [[list(color) for _ in range(w)] for _ in range(h)]

    def copy(self):
        c = Canvas(self.w, self.h)
        c.px = [[list(p) for p in row] for row in self.px]
        return c

    def blend(self, x, y, color, a=1.0):
        if 0 <= x < self.w and 0 <= y < self.h and a > 0:
            p = self.px[y][x]
            for i in range(3):
                p[i] += (color[i] - p[i]) * min(a, 1.0)

    def shade(self, fn):
        """Replaces every pixel with fn(x, y, current_rgb)."""
        for y in range(self.h):
            row = self.px[y]
            for x in range(self.w):
                row[x] = list(fn(x, y, row[x]))

    def rect(self, x0, y0, x1, y1, color, a=1.0, wrap=False):
        for y in range(int(math.floor(y0)), int(math.ceil(y1))):
            for x in range(int(math.floor(x0)), int(math.ceil(x1))):
                if wrap:
                    self.blend(x % self.w, y % self.h, color, a)
                else:
                    self.blend(x, y, color, a)

    def disc(self, cx, cy, r, color, a=1.0, soft=1.0, wrap=False):
        """An anti-aliased filled circle (edge softened over `soft` pixels)."""
        for y in range(int(cy - r - 2), int(cy + r + 3)):
            for x in range(int(cx - r - 2), int(cx + r + 3)):
                d = math.hypot(x + 0.5 - cx, y + 0.5 - cy)
                cov = max(0.0, min(1.0, (r - d) / soft + 0.5))
                if cov > 0:
                    if wrap:
                        self.blend(x % self.w, y % self.h, color, a * cov)
                    else:
                        self.blend(x, y, color, a * cov)

    def glow(self, cx, cy, r, color, a=1.0, wrap=False):
        """A soft radial glow fading to nothing at radius r."""
        for y in range(int(cy - r - 1), int(cy + r + 2)):
            for x in range(int(cx - r - 1), int(cx + r + 2)):
                d = math.hypot(x + 0.5 - cx, y + 0.5 - cy) / r
                if d < 1:
                    xx, yy = (x % self.w, y % self.h) if wrap else (x, y)
                    self.blend(xx, yy, color, a * (1 - d) ** 2)

    def polygon(self, pts, color, a=1.0, ss=3):
        """Fills a polygon (any shape) by testing pixel sample points with the even-odd
        rule, `ss` x `ss` samples per pixel for smooth edges."""
        xs, ys = [p[0] for p in pts], [p[1] for p in pts]
        n = len(pts)
        for y in range(max(0, int(min(ys))), min(self.h, int(max(ys)) + 1)):
            for x in range(max(0, int(min(xs))), min(self.w, int(max(xs)) + 1)):
                hits = 0
                for sy in range(ss):
                    for sx in range(ss):
                        px, py = x + (sx + 0.5) / ss, y + (sy + 0.5) / ss
                        inside = False
                        for i in range(n):
                            (x1, y1), (x2, y2) = pts[i], pts[(i + 1) % n]
                            if (y1 > py) != (y2 > py) and px < x1 + (py - y1) * (x2 - x1) / (y2 - y1):
                                inside = not inside
                        hits += inside
                if hits:
                    self.blend(x, y, color, a * hits / (ss * ss))

    def save(self, path):
        rows = []
        for row in self.px:
            rows.append(bytes(max(0, min(255, int(c * 255 + 0.5))) for p in row for c in p))
        with open(path, "wb") as fh:
            fh.write(png_bytes(self.w, self.h, rows))


def value_noise(seed):
    """Smooth 2D value noise: random values on an integer lattice, blended with a
    smoothstep. Returns a function f(x, y) -> 0..1. `seed` makes it repeatable."""
    def h(ix, iy):
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


def fbm(seed, octaves=4):
    """Fractal noise: several octaves of value noise, each twice as fine and half as
    strong. Gives the cloudy look of scorch marks and rock."""
    ns = [value_noise(seed + i * 101) for i in range(octaves)]

    def f(x, y):
        s, amp, tot = 0.0, 1.0, 0.0
        for i, n in enumerate(ns):
            s += n(x * (1 << i), y * (1 << i)) * amp
            tot += amp
            amp *= 0.5
        return s / tot

    return f


def noise_grid(seed, w, h, scale, octaves=4, step=4):
    """Fractal noise sampled for every pixel of a w x h image (`scale` pixels per noise
    cell). Evaluating noise is the slow part of texture painting in pure Python, so it
    is computed on a grid `step` pixels apart and bilinearly interpolated between - the
    result is indistinguishable for smooth weathering and soot. Returns rows of floats."""
    f = fbm(seed, octaves)
    gw, gh = w // step + 2, h // step + 2
    g = [[f(gx * step / scale, gy * step / scale) for gx in range(gw)] for gy in range(gh)]
    out = []
    for y in range(h):
        gy, ty = divmod(y, step)
        ty /= step
        r0, r1 = g[gy], g[gy + 1]
        row = []
        for x in range(w):
            gx, tx = divmod(x, step)
            tx /= step
            a = r0[gx] + (r0[gx + 1] - r0[gx]) * tx
            b = r1[gx] + (r1[gx + 1] - r1[gx]) * tx
            row.append(a + (b - a) * ty)
        out.append(row)
    return out
