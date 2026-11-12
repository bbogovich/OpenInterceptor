#!/usr/bin/env python3
"""Synthesises the default campaign's interactive music (Stage 6.4).

Four states, two segments each, every segment a seamless loop of four bars, plus two
stingers - all written to resources/default/music/ as 16 kHz mono WAVs:

* cruise   - a calm saw-pad progression with a soft vibraphone arpeggio
* threat   - the same chords darkened, a pulsing bass and a theremin glide (the pulp
             serial's warning sound)
* battle   - faster, a driving bass ostinato, snare and brassy square stabs
* strategy - slow pads and a glockenspiel figure for the star map
* victory  - a rising major fanfare; death - a falling minor figure into noise

Everything is generated from a short score in this file: no samples, no dependencies
(standard library only), deterministic output.
"""

import math
import os
import random
import struct
import wave

RATE = 16000
OUT = os.path.join(os.path.dirname(__file__), "..", "..", "resources", "default", "music")
TAU = 2 * math.pi


def midi(n):
    return 440.0 * 2 ** ((n - 69) / 12)


def saw(p):
    return 2 * (p % 1.0) - 1


def square(p, duty=0.5):
    return 1.0 if (p % 1.0) < duty else -1.0


def tri(p):
    q = p % 1.0
    return 4 * q - 1 if q < 0.5 else 3 - 4 * q


def render(length, notes):
    """Mixes `notes` - (start s, duration s, frequency, voice, level) - into a buffer of
    `length` seconds. A note running past the end wraps round to the start, so the
    buffer loops seamlessly."""
    n = int(length * RATE)
    buf = [0.0] * n
    for start, dur, freq, voice, level in notes:
        s0 = int(start * RATE)
        count = int(dur * RATE)
        lp = 0.0
        for i in range(count):
            t = i / RATE
            p = freq * t
            if voice == "pad":
                x = (saw(p) + saw(p * 1.004) + 0.5 * saw(p * 0.5)) / 2.5
                env = min(1.0, t / 0.3) * min(1.0, (dur - t) / 0.4)
                lp += (x - lp) * 0.08  # soften the saws
                x = lp
            elif voice == "vibe":
                x = math.sin(TAU * p) * (1 + 0.3 * math.sin(TAU * 5 * t))
                env = math.exp(-t / 0.5) * min(1.0, t / 0.005)
            elif voice == "glock":
                x = math.sin(TAU * p) + 0.3 * math.sin(TAU * p * 2.76)
                env = math.exp(-t / 0.35) * min(1.0, t / 0.002)
            elif voice == "bass":
                x = tri(p) * 0.8 + 0.2 * square(p)
                env = math.exp(-t / 0.25) * min(1.0, t / 0.004)
            elif voice == "theremin":
                # A slow glide up to the note with a wide vibrato.
                f = freq * (0.94 + 0.06 * min(1.0, t / 0.4)) * (1 + 0.012 * math.sin(TAU * 6 * t))
                lp += f / RATE
                x = math.sin(TAU * lp)
                env = min(1.0, t / 0.15) * min(1.0, (dur - t) / 0.2)
            elif voice == "stab":
                x = (square(p, 0.3) + square(p * 1.5, 0.3)) / 2
                env = math.exp(-t / 0.12) * min(1.0, t / 0.003)
            elif voice == "snare":
                x = random.uniform(-1, 1)
                env = math.exp(-t / 0.08)
            elif voice == "noise":
                x = random.uniform(-1, 1)
                env = min(1.0, t / 0.2) * math.exp(-t / 0.8)
            else:
                continue
            buf[(s0 + i) % n] += x * env * level
    return buf


# The progression: D minor, B flat, F, C (i - VI - III - VII), one chord a bar.
CHORDS = [[50, 53, 57], [46, 50, 53], [53, 57, 60], [48, 52, 55]]
DARK = [[50, 53, 56], [46, 49, 53], [49, 53, 56], [48, 51, 55]]   # diminished colours


def bars(bpm):
    return 4 * 60.0 / bpm


def cruise(variant):
    bar = bars(96)
    notes = []
    for b, ch in enumerate(CHORDS):
        for k in ch:
            notes.append((b * bar, bar * 1.02, midi(k), "pad", 0.22))
        pattern = [0, 1, 2, 1] if variant == 0 else [2, 1, 0, 1, 2, 0, 1, 2]
        step = bar / len(pattern)
        for i, c in enumerate(pattern):
            notes.append((b * bar + i * step, step * 2, midi(ch[c] + 24), "vibe", 0.16))
    return render(4 * bar, notes)


def threat(variant):
    bar = bars(100)
    notes = []
    for b, ch in enumerate(DARK):
        for k in ch:
            notes.append((b * bar, bar * 1.02, midi(k), "pad", 0.18))
        for i in range(8):
            notes.append((b * bar + i * bar / 8, bar / 8, midi(ch[0] - 12), "bass", 0.35))
    melody = [74, 73, 70, 69] if variant == 0 else [77, 74, 73, 70]
    for b, m in enumerate(melody):
        notes.append((b * bar + bar * 0.1, bar * 0.85, midi(m), "theremin", 0.22))
    return render(4 * bar, notes)


def battle(variant):
    bar = bars(132)
    notes = []
    for b, ch in enumerate(CHORDS if variant == 0 else DARK):
        root = ch[0] - 12
        for i, off in enumerate([0, 0, 12, 0, 7, 0, 10, 12]):
            notes.append((b * bar + i * bar / 8, bar / 8, midi(root + off), "bass", 0.4))
        for beat in (1, 3):
            notes.append((b * bar + beat * bar / 4, 0.2, 0, "snare", 0.25))
        for hit in (0, 2.5) if variant == 0 else (0, 1.5, 3):
            notes.append((b * bar + hit * bar / 4, 0.3, midi(ch[1] + 12), "stab", 0.2))
    return render(4 * bar, notes)


def strategy(variant):
    bar = bars(72)
    notes = []
    for b, ch in enumerate(CHORDS if variant == 0 else [CHORDS[2], CHORDS[3], CHORDS[0], CHORDS[1]]):
        for k in ch:
            notes.append((b * bar, bar * 1.02, midi(k - 12), "pad", 0.2))
        for i, c in enumerate([2, 0, 1]):
            notes.append((b * bar + i * bar / 3, bar / 2, midi(ch[c] + 24), "glock", 0.12))
    return render(4 * bar, notes)


def victory():
    notes = [(i * 0.18, 0.5 if i < 3 else 1.6, midi(n), "stab", 0.3) for i, n in enumerate([62, 66, 69, 74])]
    notes += [(0.54, 1.6, midi(n), "pad", 0.25) for n in (62, 66, 69)]
    buf = render(2.4, notes)
    return buf[: int(2.3 * RATE)]


def death():
    notes = [(i * 0.25, 0.6, midi(n), "theremin", 0.3) for i, n in enumerate([69, 65, 62, 57])]
    notes.append((0.9, 1.5, 0, "noise", 0.25))
    return render(2.6, notes)


def write(name, buf, peak=0.8, fade=False):
    if fade:  # a stinger ends in silence
        k = int(0.05 * RATE)
        for i in range(k):
            buf[-1 - i] *= i / k
    m = max(1e-9, max(abs(x) for x in buf))
    data = b"".join(struct.pack("<h", int(round(x / m * peak * 32767))) for x in buf)
    with wave.open(os.path.join(OUT, name + ".wav"), "wb") as w:
        w.setnchannels(1)
        w.setsampwidth(2)
        w.setframerate(RATE)
        w.writeframes(data)
    return len(buf) / RATE


def build_all():
    random.seed(1984)
    os.makedirs(OUT, exist_ok=True)
    out = []
    for name, fn in (("cruise", cruise), ("threat", threat), ("battle", battle), ("strategy", strategy)):
        for v in (0, 1):
            out.append((f"{name}{v + 1}", write(f"{name}{v + 1}", fn(v))))
    out.append(("victory", write("victory", victory(), fade=True)))
    out.append(("death", write("death", death(), fade=True)))
    return out


if __name__ == "__main__":
    for name, secs in build_all():
        print(f"{name:10s} {secs:5.2f} s")
