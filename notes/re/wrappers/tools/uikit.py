#!/usr/bin/env python3
"""Decode the strategy-UI tables of the player's own Interceptor.exe (stdlib only).

Reads the original binaries as data; nothing is executed. Prints FACTS (ids, numbers,
short key names) as YAML-ish text on stdout. Never write the output into the repository
next to original art; it contains no art, but keep the habit of running it into a scratch
folder.

Subcommands (paths are arguments, nothing is hard-coded):

  sprites EXE [--sprites-dll DLL]   the 20-byte sprite table (index, resource, hot spot,
                                    flags, group), located by content signature, so it
                                    works for the CD build (VA 0x69bc18) and patch 1.2
                                    (VA 0x69b598) alike. With --sprites-dll each row also
                                    gets the TGA size of its resource.
  sounds EXE                        the interface sound table (40-byte rows: index, key).
  fonts EXE SPRITES_DLL             the nine sheet fonts: resource id, cell pitch, glyph
                                    box, per-glyph advance widths (derived from pixels).
  spr N EXE                         one sprite-table row and its state images.

Record layouts (verified from the loader 0x56e729 / 0x56ecc7, CD addresses):

  sprite row (20 bytes): int32 index (== position), int32 resource id (0 = group end),
    int32 hot_x, int32 hot_y, u8 keep_format, u8 surface, u16 pad.
    hot -1 -> 0 (top-left), -2 -> -(size/2) (centre), other values are pixel offsets
    added to the draw position. keep_format=0 converts the image to blit mode 3
    (0x62af2c); 1 keeps the source format (only the CSD star-map icon group 434-563
    and 842). surface=1 loads the image as a whole DirectDraw surface (0x62fb32)
    instead of a sprite (only 429 and 430).
  sound row (40 bytes): int32 index, char key[32], int32 voices.
"""
import struct
import sys


# ---------------------------------------------------------------- PE helpers
class PE:
    """Minimal PE32 reader: sections, VA <-> file offset, resource walk."""

    def __init__(self, path):
        d = self.d = open(path, "rb").read()
        pe = struct.unpack_from("<I", d, 0x3C)[0]
        nsec = struct.unpack_from("<H", d, pe + 6)[0]
        opt = struct.unpack_from("<H", d, pe + 20)[0]
        o = pe + 24
        self.base = struct.unpack_from("<I", d, o + 28)[0]
        self.dirs = [struct.unpack_from("<II", d, o + 96 + 8 * i) for i in range(16)]
        self.secs = []
        so = o + opt
        for i in range(nsec):
            name = d[so + 40 * i:so + 40 * i + 8].rstrip(b"\0").decode("latin1")
            vs, va, rs, ro = struct.unpack_from("<IIII", d, so + 40 * i + 8)
            self.secs.append((name, va, vs, ro, rs))

    def off2va(self, off):
        for _n, va, _vs, ro, rs in self.secs:
            if ro <= off < ro + rs:
                return self.base + va + off - ro
        return None

    def va2off(self, v):
        r = v - self.base
        for _n, va, vs, ro, rs in self.secs:
            if va <= r < va + max(vs, rs) and r - va < rs:
                return ro + r - va
        return None

    def section(self, name):
        for s in self.secs:
            if s[0] == name:
                return s
        return None

    def resources(self):
        """Yield (type, id, file_offset, size) for every resource leaf."""
        rva = self.dirs[2][0]
        if not rva:
            return
        base = self.va2off(self.base + rva)

        def name(e):
            if e & 0x80000000:
                o = base + (e & 0x7FFFFFFF)
                n = struct.unpack_from("<H", self.d, o)[0]
                return self.d[o + 2:o + 2 + 2 * n].decode("utf-16le")
            return e

        def walk(o, path):
            nn, ni = struct.unpack_from("<HH", self.d, o + 12)
            for i in range(nn + ni):
                e, off = struct.unpack_from("<II", self.d, o + 16 + 8 * i)
                if off & 0x80000000:
                    yield from walk(base + (off & 0x7FFFFFFF), path + [name(e)])
                else:
                    drva, sz = struct.unpack_from("<II", self.d, base + off)
                    yield path + [name(e)], self.va2off(self.base + drva), sz

        for path, o, sz in walk(base, []):
            yield path[0], path[1], o, sz


# ---------------------------------------------------------------- TGA (pixels for metrics only)
def tga_header(d, o):
    idl, _cmt, it = d[o], d[o + 1], d[o + 2]
    w, h = struct.unpack_from("<HH", d, o + 12)
    return {"type": it, "w": w, "h": h, "bpp": d[o + 16], "desc": d[o + 17], "idlen": idl}


def tga_pixels(d, o):
    """Decode a type 2 / 10 TGA into rows of int pixel values, top row first."""
    h = tga_header(d, o)
    w, ht, bpp = h["w"], h["h"], h["bpp"]
    if h["type"] not in (2, 10) or bpp not in (16, 24, 32):
        return None
    n = bpp // 8
    p = o + 18 + h["idlen"]
    total = w * ht
    px = []
    if h["type"] == 2:
        for i in range(total):
            px.append(int.from_bytes(d[p + i * n:p + i * n + n], "little"))
    else:
        while len(px) < total:
            c = d[p]
            p += 1
            k = (c & 0x7F) + 1
            if c & 0x80:
                v = int.from_bytes(d[p:p + n], "little")
                p += n
                px.extend([v] * k)
            else:
                for _ in range(k):
                    px.append(int.from_bytes(d[p:p + n], "little"))
                    p += n
    rows = [px[r * w:(r + 1) * w] for r in range(ht)]
    if not (h["desc"] & 0x20):  # bottom-up origin -> flip so row 0 is the top
        rows.reverse()
    return rows


# ---------------------------------------------------------------- sprite table
HOT_OK = set(range(-64, 65))


def find_sprite_table(pe, min_run=64):
    """Locate the table by content: a run of 20-byte rows whose first field counts 0,1,2..

    Returns (file_offset, row_count). The run stops at the first row whose index is not
    its position (1527 rows in both known builds).
    """
    d = pe.d
    for name in (".data", ".rdata"):
        s = pe.section(name)
        if not s:
            continue
        _n, _va, _vs, ro, rs = s
        o = ro
        end = ro + rs - 20 * min_run
        pat = b"\0\0\0\0"
        while True:
            o = d.find(pat, o, end)
            if o < 0:
                break
            if (o - ro) % 4 == 0:
                ok = True
                for i in range(min_run):
                    idx, res, hx, hy = struct.unpack_from("<iiii", d, o + 20 * i)
                    if idx != i or hx not in HOT_OK or hy not in HOT_OK or not (0 <= res < 40000):
                        ok = False
                        break
                if ok:
                    n = 0
                    while struct.unpack_from("<i", d, o + 20 * n)[0] == n:
                        n += 1
                    return o, n
            o += 1
    raise SystemExit("sprite table not found")


def sprite_rows(pe):
    o, n = find_sprite_table(pe)
    rows = []
    group = 0
    for i in range(n):
        idx, res, hx, hy = struct.unpack_from("<iiii", pe.d, o + 20 * i)
        keep, surf = pe.d[o + 20 * i + 16], pe.d[o + 20 * i + 17]
        rows.append({"index": idx, "res": res, "hot": (hx, hy), "keep_format": keep,
                     "surface": surf, "group": group})
        if res == 0:  # the loader's group ranges end on these rows (inclusive)
            group += 1
    return pe.off2va(o), rows


def sprite_sizes(dll_path):
    dll = PE(dll_path)
    out = {}
    for t, rid, o, _sz in dll.resources():
        if t == "TARG" and isinstance(rid, int):
            h = tga_header(dll.d, o)
            out[rid] = (h["w"], h["h"])
    return out


# ---------------------------------------------------------------- sound table
def find_ui_sound_table(pe):
    """The interface sound rows: 40-byte {index, key[32], voices}; key 0 is 'Click'."""
    d = pe.d
    o = d.find(b"\0\0\0\0Click\0")
    while o >= 0:
        if struct.unpack_from("<i", d, o + 40)[0] == 1:
            break
        o = d.find(b"\0\0\0\0Click\0", o + 1)
    if o < 0:
        raise SystemExit("sound table not found")
    rows = []
    i = 0
    while struct.unpack_from("<i", d, o + 40 * i)[0] == i:
        key = d[o + 40 * i + 4:o + 40 * i + 36].split(b"\0")[0].decode("latin1")
        voices = struct.unpack_from("<i", d, o + 40 * i + 36)[0]
        rows.append((i, key, voices))
        i += 1
    return pe.off2va(o), rows


# ---------------------------------------------------------------- fonts
def find_font_ids(pe):
    """The font init pushes nine resource ids, each followed by `mov/lea ecx, font`; call."""
    d = pe.d
    _n, va, vs, ro, rs = pe.section(".text")
    best = []
    o = ro
    while o < ro + rs - 16:
        if d[o] == 0x68:
            ids = []
            p = o
            while d[p] == 0x68:
                v = struct.unpack_from("<I", d, p + 1)[0]
                q = p + 5
                if d[q] == 0xB9:
                    q += 5
                elif d[q:q + 2] == b"\x8d\x0d":
                    q += 6
                else:
                    break
                if d[q] != 0xE8:
                    break
                ids.append((v, struct.unpack_from("<I", d, p + 6 if d[p + 5] == 0xB9 else p + 7)[0]))
                p = q + 5
            if len(ids) > len(best) and all(20000 < v < 30000 for v, _ in ids):
                best = ids
            o = p if ids else o + 1
        else:
            o += 1
    return best


def font_metrics(rows):
    """Cell pitch, glyph box and per-glyph widths, as the loader 0x5fd750 derives them.

    Pixel (0,0) is the grid-line colour; the bottom-right pixel is the transparent key.
    The pitch is the first row y >= 1 whose column-1 pixel is the grid colour; cells are
    square; glyph g (1..154) sits at ((g-1) % cols, (g-1) // cols) with a 1 px border.
    Width = the rightmost non-key column + 1 inside the cell (derived: the trim done by
    the glyph copy 0x61a991 is assumed to be a right/bottom trim).
    """
    h = len(rows)
    w = len(rows[0])
    grid = rows[0][0]
    key = rows[h - 1][w - 1]
    pitch = None
    for i in range(h - 1):
        if rows[i + 1][1] == grid:
            pitch = i + 1
            break
    if pitch is None:
        return None
    size = pitch - 2
    cols = w // pitch
    widths = {}
    heights = {}
    for g in range(1, 155):
        cx = ((g - 1) % cols) * pitch + 1
        cy = ((g - 1) // cols) * pitch + 1
        if cy + size > h:
            break
        right = 0
        bottom = 0
        for y in range(size):
            row = rows[cy + y]
            for x in range(size):
                v = row[cx + x]
                if v != key and v != grid:
                    right = max(right, x + 1)
                    bottom = max(bottom, y + 1)
        widths[g] = right
        heights[g] = bottom
    return {"pitch": pitch, "box": size, "cols": cols, "widths": widths, "heights": heights,
            "grid_colour": grid, "key_colour": key, "ink": ink_colours(rows, pitch, cols, size, grid, key)}


def to_rgb(v, bpp):
    """A TGA pixel as (r, g, b): 16 bpp is A1R5G5B5, 24/32 bpp is B, G, R(, A)."""
    if bpp == 16:
        return (((v >> 10) & 31) * 255 // 31, ((v >> 5) & 31) * 255 // 31, (v & 31) * 255 // 31)
    return ((v >> 16) & 255, (v >> 8) & 255, v & 255)


def ink_colours(rows, pitch, cols, size, grid, key, top=3):
    """The most frequent glyph pixel values (raw), i.e. the colours baked into the sheet."""
    import collections
    c = collections.Counter()
    h = len(rows)
    for g in range(1, 155):
        cx = ((g - 1) % cols) * pitch + 1
        cy = ((g - 1) // cols) * pitch + 1
        if cy + size > h:
            break
        for y in range(size):
            for x in range(size):
                v = rows[cy + y][cx + x]
                if v != key and v != grid:
                    c[v] += 1
    return c.most_common(top)


def char_map(pe):
    """The 256-entry char -> glyph map (0 = not drawn): found by its content."""
    d = pe.d
    # 'A' (65) -> glyph 1 and 'a' (97) -> 27: search for the run 1,2,3,...26 at 65..90.
    pat = b"".join(struct.pack("<i", i) for i in range(1, 27))
    o = d.find(pat)
    while o >= 0:
        base = o - 65 * 4
        if struct.unpack_from("<i", d, base + 97 * 4)[0] == 27:
            return pe.off2va(base), struct.unpack_from("<256i", d, base)
        o = d.find(pat, o + 1)
    return None, None


# ---------------------------------------------------------------- main
def main(argv):
    if len(argv) < 3:
        print(__doc__)
        return 2
    cmd = argv[1]
    if cmd == "sprites":
        pe = PE(argv[2])
        sizes = {}
        if "--sprites-dll" in argv:
            sizes = sprite_sizes(argv[argv.index("--sprites-dll") + 1])
        va, rows = sprite_rows(pe)
        print("# sprite table at VA 0x%x, %d rows" % (va, len(rows)))
        print("# index group res hot_x hot_y keep_format surface [w h]")
        for r in rows:
            wh = sizes.get(r["res"])
            print("%d %d %d %d %d %d %d%s" % (r["index"], r["group"], r["res"], r["hot"][0], r["hot"][1],
                                             r["keep_format"], r["surface"],
                                             (" %d %d" % wh) if wh else ""))
    elif cmd == "sounds":
        pe = PE(argv[2])
        va, rows = find_ui_sound_table(pe)
        print("# interface sound table at VA 0x%x" % va)
        for i, k, v in rows:
            print("%d %s %d" % (i, k or "-", v))
    elif cmd == "fonts":
        pe = PE(argv[2])
        dll = PE(argv[3])
        res = {rid: o for t, rid, o, _ in dll.resources() if t == "TARG"}
        ids = find_font_ids(pe)
        cmva, cmap = char_map(pe)
        print("# char map at VA 0x%x" % (cmva or 0))
        for slot, (rid, obj) in enumerate(ids):
            o = res.get(rid)
            hd = tga_header(dll.d, o)
            rows = tga_pixels(dll.d, o)
            m = font_metrics(rows) if rows else None
            print("- font: %d   # slot %d, object VA 0x%x, sheet %dx%d %dbpp" % (rid, slot, obj, hd["w"], hd["h"], hd["bpp"]))
            if m:
                print("  pitch: %d\n  box: %d\n  cols: %d" % (m["pitch"], m["box"], m["cols"]))
                print("  line_height: %d" % max(m["heights"].values()))
                print("  max_width: %d" % max(m["widths"].values()))
                print("  ink_rgb: \"%s\"   # most frequent glyph pixel colours (count)" % ", ".join(
                    "#%02x%02x%02x(%d)" % (to_rgb(v, hd["bpp"]) + (n,)) for v, n in m["ink"]))
                adv = {}
                for c in range(256):
                    g = cmap[c] if cmap else 0
                    if g:
                        adv[c] = m["widths"].get(g, 0)
                adv[32] = m["widths"].get(31, 0)  # space advance = width of glyph 31 ('e')
                print("  advance: {%s}" % ", ".join("%d: %d" % (c, adv[c]) for c in sorted(adv)))
    elif cmd == "spr":
        n = int(argv[2])
        pe = PE(argv[3])
        _va, rows = sprite_rows(pe)
        r = rows[n]
        print(r)
    else:
        print(__doc__)
        return 2
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
