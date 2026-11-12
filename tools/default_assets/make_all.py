#!/usr/bin/env python3
"""Generates every procedural asset of the `default` campaign pack.

Run from anywhere:

    python3 tools/default_assets/make_all.py            # build everything
    python3 tools/default_assets/make_all.py --check    # build twice, prove the output is identical

It writes into resources/default/:

* `models/`   - ships, stations, rubble, debris pieces and the Guard 3D cockpit (OBJ +
                MTL), plus `guard_cockpit.instruments.yaml` (models.py);
* `textures/` - hull paint and its damage variants (PNG, textures.py);
* `sounds/`   - synthesised effects (WAV, sounds.py).

Every model passes the validator in common.py before it is written (closed solids wound
outward, no degenerate faces, within its triangle budget); a failure stops the run with
the model and part named. Only the Python standard library is used, and every random
choice has a fixed seed, so the output is byte-identical from run to run - `--check`
proves it by hashing the files of two consecutive builds.
"""
import hashlib
import os
import sys

sys.dont_write_bytecode = True  # keep __pycache__ out of the source tree
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

import models  # noqa: E402  (after the path tweak, so `common` is importable)
import music  # noqa: E402
import sounds  # noqa: E402
import textures  # noqa: E402
from common import PACK, ValidationError  # noqa: E402

GENERATED = ("models", "textures", "sounds", "music")
# Hand-made files of the old sample content that live in the same folders; they are
# neither generated nor touched here.
LEGACY = {"dart.obj", "dart.mtl", "dart_cockpit.obj", "dart_cockpit.mtl", "raider.obj", "raider.mtl", "ring.obj",
          "ring.mtl", "panel.png"}


def build():
    for sub in GENERATED:
        os.makedirs(os.path.join(PACK, sub), exist_ok=True)
    rows, rects = models.build_all()
    tex = textures.build_all()
    snd = sounds.build_all()
    snd += music.build_all()
    return rows, rects, tex, snd


def digest():
    """SHA-256 of every generated file, by path."""
    out = {}
    for sub in GENERATED:
        folder = os.path.join(PACK, sub)
        for name in sorted(os.listdir(folder)):
            if name in LEGACY:
                continue
            with open(os.path.join(folder, name), "rb") as fh:
                out[f"{sub}/{name}"] = hashlib.sha256(fh.read()).hexdigest()
    return out


def main():
    try:
        rows, rects, tex, snd = build()
    except ValidationError as e:
        sys.exit(f"VALIDATION FAILED: {e}")
    print("models (triangles, size x/y/z in metres, debris pieces):")
    for model, pieces in rows:
        lo, hi = model.bounds()
        print(f"  {model.name:20s} {model.triangles():5d}  {hi[0] - lo[0]:6.1f} {hi[1] - lo[1]:6.1f} {hi[2] - lo[2]:6.1f}"
              f"  {pieces}")
    print("cockpit instruments:", rects)
    print(f"textures: {len(tex)}  sounds: {len(snd)}")
    if "--check" in sys.argv:
        first = digest()
        build()
        second = digest()
        changed = [k for k in first if first[k] != second.get(k)]
        if changed:
            sys.exit(f"NOT DETERMINISTIC: {changed[:10]}")
        print(f"determinism check passed: {len(first)} files identical across two builds")


if __name__ == "__main__":
    main()
