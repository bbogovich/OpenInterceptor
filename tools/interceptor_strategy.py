#!/usr/bin/env python3
"""Writes the interceptor pack's strategy *references* (Stage 4.12).

The X-COM: Interceptor campaign's strategy values (research sizes and prerequisites,
item and module costs, production times, maintenance) stay in the player's own
GameSettings.ini: the pack only names the sections to inherit from. Two things the
original keeps in its executable are recorded in the pack as facts needed to reference
that data:

* the **tech-id table**: the numeric ids that Prereq1..3 and TechRequired use, mapped to
  research sections. The executable holds it as 12-byte entries
  {pointer to the R_ section name, string id, tech id}; this tool finds the run of
  entries whose pointers lead to R_ section names and reads the tech ids.
* the section names themselves (as inheritance sources).

No values are copied: the output holds ids, section names and numbers that are ids.
Run it with the player's data, then review the diff:

    python3 tools/interceptor_strategy.py resources/xcom-cd/Game/ProgramFiles \\
        > packs/interceptor/strategy/references.yaml

Standard library only.
"""

import re
import struct
import sys
from pathlib import Path


def sections(ini_text):
    """[(name, {key: value})] in file order."""
    out = []
    for name, body in re.findall(r"^\[([^\]]+)\]\s*\n(.*?)(?=^\[|\Z)", ini_text, re.M | re.S):
        keys = {}
        for line in body.splitlines():
            if "=" in line and not line.strip().startswith(";"):
                k, v = line.split("=", 1)
                keys[k.strip()] = v.split(";")[0].strip()
        out.append((name.strip(), keys))
    return out


def tech_table(exe_bytes, r_names):
    """[(section name, tech id)] from the executable's table (see the module docs)."""
    b = exe_bytes
    pe = struct.unpack_from("<I", b, 0x3C)[0]
    count = struct.unpack_from("<H", b, pe + 6)[0]
    opt = struct.unpack_from("<H", b, pe + 20)[0]
    base = struct.unpack_from("<I", b, pe + 24 + 28)[0]
    sects = []
    for i in range(count):
        o = pe + 24 + opt + i * 40
        vsz, va, rsz, raw = struct.unpack_from("<IIII", b, o + 8)
        sects.append((va, vsz, raw, rsz))

    def va_of(off):
        for va, vsz, raw, rsz in sects:
            if raw <= off < raw + rsz:
                return base + va + off - raw
        return None

    def off_of(v):
        r = v - base
        for va, vsz, raw, rsz in sects:
            if va <= r < va + max(vsz, rsz):
                return raw + r - va
        return None

    def text_at(v):
        o = off_of(v)
        if o is None or o >= len(b):
            return None
        e = b.find(b"\0", o)
        s = b[o:e]
        if 0 < len(s) < 60 and all(32 <= c < 127 for c in s):
            return s.decode("latin-1")
        return None

    # Every place a pointer to an R_ name is stored; the table is the longest run of
    # such pointers 12 bytes apart.
    refs = set()
    for n in r_names:
        at = b.find(n.encode() + b"\0")
        v = va_of(at) if at >= 0 else None
        if v is None:
            continue
        for m in re.finditer(re.escape(struct.pack("<I", v)), b):
            refs.add(m.start())
    best = (0, None)
    for r in refs:
        if r - 12 in refs:
            continue
        n = 0
        while r + 12 * n in refs:
            n += 1
        best = max(best, (n, r))
    start = best[1]
    if start is None:
        sys.exit("no tech table found")
    # Extend over entries whose pointer leads to any name (placeholders included).
    while text_at(struct.unpack_from("<I", b, start - 12)[0]):
        start -= 12
    table = []
    k = 0
    while True:
        p, _string_id, tech = struct.unpack_from("<III", b, start + 12 * k)
        name = text_at(p)
        if name is None:
            break
        table.append((name, tech))
        k += 1
    return table


def snake(name):
    """Section name to a readable id: R_TRI_TITANIUM_ARMOR -> tri_titanium_armor,
    M_XCOMHangar -> hangar, LaserCannon -> laser_cannon."""
    name = re.sub(r"^(R_|M_XCOM|M_)", "", name)
    name = re.sub(r"(?<=[a-z0-9])(?=[A-Z])", "_", name)
    return re.sub(r"[^a-z0-9]+", "_", name.lower()).strip("_")


ITEM_KINDS = [
    (r"MissileLauncher$|TorpedoLauncher$|BombLauncher$", "launcher"),
    (r"Missile$|Torpedo$|Bomb$", "missile"),
    (r"Mine$", "mine"),
    (r"^ArmorType", "armor"),
    (r"^ShieldType", "shield"),
    (r"^EngineType", "engine"),
    (r"^SensorTargetingSystem", "sensor"),
    (r"Probe$", "probe"),
    (r"^(ECMPodType|CloakingDevice|DimensionJumpDrive|ShipDamageControlComputer|TractorBeam|VectorControlThrusters)", "special"),
    (r"(Laser|Cannon|Pulser|Blaster)$", "beam"),
]

# Items that are the same thing as a battle beam or missile keep its id, so a fitted
# craft flies it (see Campaign::loadout).
SAME_AS_BATTLE = {
    "LaserCannon": "laser_cannon", "HeavyLaser": "heavy_laser", "GatlingLaser": "gatling_laser",
    "TracerCannon": "tracer_cannon", "PhaseCannon": "phase_cannon", "PlasmaCannon": "plasma_cannon",
    "TachyonPulser": "tachyon_pulser", "XWinderMissile": "xwinder", "PermeatorMissile": "permeator",
    "SidestepMissile": "sidestep", "FusionMissile": "fusion", "EleriumTorpedo": "elerium_torpedo",
    "EnvelopingFusionTorpedo": "enveloping_torpedo", "ParallaxTorpedo": "parallax_torpedo",
    "PassiveCloakingTorpedo": "cloaking_torpedo", "DoppelgangerMissile": "doppelganger",
}


def main():
    root = Path(sys.argv[1])
    ini = sections((root / "GameSettings.ini").read_text(encoding="latin-1"))
    names = {n for n, _ in ini}
    r_names = [n for n, _ in ini if n.startswith("R_")]
    table = tech_table((root / "Interceptor.exe").read_bytes(), r_names)
    out = []
    w = out.append
    w("# yaml-language-server: $schema=../../../schemas/rules.schema.json")
    w("# GENERATED by tools/interceptor_strategy.py from the player's own data: section names")
    w("# to inherit from and the executable's tech-id table (facts needed to reference the")
    w("# original's numeric ids). No values: they are read from GameSettings.ini at load.")
    w("")
    w("tech_ids:")
    for name, tech in table:
        if name in names:
            w(f"  {tech}: {snake(name)}")
    w("")
    w("research:")
    for n in r_names:
        w(f"  {snake(n)}: {{inherit: original:GameSettings.ini/{n}, name: {snake(n).replace('_', ' ').title()}}}")
    w("")
    w("items:")
    for n, keys in ini:
        if "TechItem" not in keys:
            continue
        kind = next((k for pat, k in ITEM_KINDS if re.search(pat, n)), None)
        if kind is None:
            continue
        iid = SAME_AS_BATTLE.get(n, snake(n))
        title = re.sub(r"(?<=[a-z])(?=[A-Z])", " ", n)
        w(f"  {iid}: {{inherit: original:GameSettings.ini/{n}, name: {title}, kind: {kind}}}")
    w("")
    w("base_modules:")
    for n, _ in ini:
        if n.startswith("M_XCOM") and n != "M_XCOMCentralHub":
            title = re.sub(r"(?<=[a-z])(?=[A-Z])", " ", n[len("M_XCOM"):]).replace("_", " ")
            w(f"  {snake(n)}: {{inherit: original:GameSettings.ini/{n}, name: {title}}}")
    print("\n".join(out))


if __name__ == "__main__":
    main()
