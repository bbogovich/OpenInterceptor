"""Top-down silhouettes of the `default` campaign's player craft (#147).

The cockpit's ship status display shows the player's craft in the middle of its shield
rings. The original's cockpit art paints that craft; the `default` campaign has no 2D
art, so each flyable class names a `silhouette:` image instead. This module makes those
images from the very models models.py builds, so a silhouette always matches its ship.

How: every face of the model is projected straight down onto the X-Z plane (the engine's
ships point their nose along -Z, so -Z is "up" in the picture) and filled into a grid
four times finer than the image; each image pixel's alpha is then the share of its 4 x 4
sub-pixels that were covered (anti-aliasing by supersampling). The colour is white, so
the engine can tint it by the hull's condition.

Output: `textures/silhouette_<model>.png`, 64 x 64 RGBA, standard library only.
"""
import os
import struct
import zlib

from common import PACK

SIZE = 64  # image pixels
SUB = 4  # sub-pixels per image pixel, each way
PAD = 2  # empty image pixels kept round the craft


def rgba_png(width, height, rows):
    """Encodes 8-bit RGBA rows (bytes of width*4 each) as a PNG: colour type 6 is RGBA;
    otherwise the same layout as common.png_bytes (filter 0 per row, zlib stream)."""
    raw = b"".join(b"\x00" + bytes(r) for r in rows)

    def chunk(t, d):
        return struct.pack(">I", len(d)) + t + d + struct.pack(">I", zlib.crc32(t + d) & 0xFFFFFFFF)

    return (b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0))
            + chunk(b"IDAT", zlib.compress(raw, 9)) + chunk(b"IEND", b""))


def silhouette(model):
    """The model seen from above as RGBA rows: white, alpha = coverage."""
    lo, hi = model.bounds()
    span = max(hi[0] - lo[0], hi[2] - lo[2], 1e-6)
    grid = SIZE * SUB
    scale = (SIZE - 2 * PAD) * SUB / span
    # Centre the craft: offsets so its X-Z box sits in the middle of the grid.
    ox = (grid - (hi[0] - lo[0]) * scale) / 2
    oz = (grid - (hi[2] - lo[2]) * scale) / 2
    cover = bytearray(grid * grid)

    def to_grid(p):
        return ((p[0] - lo[0]) * scale + ox, (p[2] - lo[2]) * scale + oz)

    for part in model.parts:
        for face in part.faces:
            pts = [to_grid(p) for p in face.pts]
            # A convex face is a fan of triangles from its first corner.
            for i in range(1, len(pts) - 1):
                fill_triangle(cover, grid, pts[0], pts[i], pts[i + 1])
    rows = []
    for y in range(SIZE):
        row = bytearray()
        for x in range(SIZE):
            n = sum(cover[(y * SUB + j) * grid + x * SUB + i] for j in range(SUB) for i in range(SUB))
            row += bytes((255, 255, 255, n * 255 // (SUB * SUB)))
        rows.append(row)
    return rows


def fill_triangle(cover, grid, a, b, c):
    """Marks the sub-pixels whose centres lie inside triangle abc (either winding: a
    face seen from below projects mirrored, so both signs of the edge functions count)."""
    x0, x1 = max(int(min(a[0], b[0], c[0])), 0), min(int(max(a[0], b[0], c[0])) + 1, grid)
    y0, y1 = max(int(min(a[1], b[1], c[1])), 0), min(int(max(a[1], b[1], c[1])) + 1, grid)

    def edge(p, q, x, y):
        # Twice the signed area of (p, q, point): which side of p->q the point is on.
        return (q[0] - p[0]) * (y - p[1]) - (q[1] - p[1]) * (x - p[0])

    for y in range(y0, y1):
        py = y + 0.5
        for x in range(x0, x1):
            px = x + 0.5
            e0, e1, e2 = edge(a, b, px, py), edge(b, c, px, py), edge(c, a, px, py)
            if (e0 >= 0 and e1 >= 0 and e2 >= 0) or (e0 <= 0 and e1 <= 0 and e2 <= 0):
                cover[y * grid + x] = 1


def build_all(models):
    """Writes a silhouette for each model given; returns the file names."""
    out = []
    folder = os.path.join(PACK, "textures")
    for model in models:
        name = f"silhouette_{model.name}.png"
        with open(os.path.join(folder, name), "wb") as fh:
            fh.write(rgba_png(SIZE, SIZE, silhouette(model)))
        out.append(name)
    return out
