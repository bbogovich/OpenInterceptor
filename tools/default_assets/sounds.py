"""Synthesised sound effects of the `default` campaign (retro science-fiction).

Every sound is built sample by sample from a handful of classic synthesiser parts:

* **Oscillators** - sine, square, sawtooth and white noise. An oscillator is driven by a
  running *phase* (in cycles) rather than `sin(2 pi f t)`, so the frequency can glide
  smoothly (a pitch sweep) without clicks.
* **Envelopes** - how loudness changes over time: an attack/decay shape or an
  exponential decay.
* **Filters** - a one-pole low-pass (smooths noise into a rumble) and a state-variable
  band-pass whose centre can move (the 'whoosh' of a sweep).
* **Modulation** - vibrato (wobbling pitch: the theremin), tremolo (wobbling volume) and
  ring modulation (multiplying two tones: the metallic ray-gun sound).

All randomness comes from `random.Random(seed)`, so the files are identical every run.
The two looping sounds (`afterburner`, `saucer_hum`) are made seamless: their tones use
whole numbers of cycles over the loop, and noise is cross-faded from the end into the
start. Output: 22050 Hz, mono, 16-bit PCM WAV via the standard `wave` module.
"""
import math
import os
import random
import struct
import wave

from common import PACK

RATE = 22050
OUT = os.path.join(PACK, "sounds")
TAU = 2 * math.pi


def n_samples(seconds):
    return int(round(seconds * RATE))


class Phase:
    """A running oscillator phase. `step(f)` advances by one sample at frequency f and
    returns the phase in cycles (0..1)."""

    def __init__(self, start=0.0):
        self.p = start

    def step(self, f):
        self.p = (self.p + f / RATE) % 1.0
        return self.p


def sine(p):
    return math.sin(TAU * p)


def square(p, duty=0.5):
    return 1.0 if p < duty else -1.0


def saw(p):
    return 2.0 * p - 1.0


def tri(p):
    return 4 * abs(p - 0.5) - 1


class LowPass:
    """One-pole low-pass: each output moves a fraction of the way toward the input. The
    fraction comes from the cutoff frequency, so higher cutoffs follow faster."""

    def __init__(self):
        self.y = 0.0

    def __call__(self, x, cutoff):
        a = 1 - math.exp(-TAU * cutoff / RATE)
        self.y += a * (x - self.y)
        return self.y


class BandPass:
    """Chamberlin state-variable filter, band-pass output. Stable for cutoffs below about
    a sixth of the sample rate; `q` sets how narrow the band is."""

    def __init__(self):
        self.low = self.band = 0.0

    def __call__(self, x, cutoff, q=2.0):
        f = 2 * math.sin(math.pi * min(cutoff, RATE / 6) / RATE)
        self.low += f * self.band
        high = x - self.low - self.band / q
        self.band += f * high
        return self.band


def env_ad(t, attack, total):
    """Linear attack to 1 then linear decay to 0 at `total` seconds."""
    if t < attack:
        return t / attack
    return max(0.0, 1 - (t - attack) / max(1e-6, total - attack))


def env_exp(t, tau, attack=0.002):
    """Fast attack then exponential decay with time constant tau."""
    return min(1.0, t / attack) * math.exp(-t / tau)


def fade_out(buf, seconds):
    n = min(len(buf), n_samples(seconds))
    for i in range(n):
        buf[-1 - i] *= i / n
    return buf


def loop_crossfade(buf, seconds):
    """Makes a buffer loop seamlessly: the extra `seconds` at its end are faded over its
    start, and the result is that much shorter. The last sample then flows into the
    first exactly as the original signal flowed past the cut."""
    f = n_samples(seconds)
    L = len(buf) - f
    out = buf[:L]
    for i in range(f):
        w = i / f
        out[i] = buf[i] * w + buf[L + i] * (1 - w)
    return out


def write(name, buf, peak=0.89):
    """Normalises to `peak` and writes a 16-bit mono WAV. Returns the duration."""
    m = max(1e-9, max(abs(x) for x in buf))
    data = b"".join(struct.pack("<h", int(round(x / m * peak * 32767))) for x in buf)
    with wave.open(os.path.join(OUT, name + ".wav"), "wb") as w:
        w.setnchannels(1)
        w.setsampwidth(2)
        w.setframerate(RATE)
        w.writeframes(data)
    return len(buf) / RATE


# ---------------------------------------------------------------------------------------
# Weapons
# ---------------------------------------------------------------------------------------

def ray_zap():
    """The classic ray gun: a fast falling sweep, ring-modulated for a metallic edge."""
    T = 0.35
    a, b = Phase(), Phase()
    out = []
    for i in range(n_samples(T)):
        t = i / RATE
        f = 300 + 2600 * math.exp(-t * 14)
        s = sine(a.step(f)) * (0.6 + 0.4 * sine(b.step(f * 1.51)))
        out.append(s * env_exp(t, 0.12))
    return out


def heat_ray():
    """A buzzing, crackling beam: a low sawtooth with a wavering pitch, filtered noise
    crackle on top."""
    T = 0.6
    rng = random.Random(2)
    a, v = Phase(), Phase()
    lp = LowPass()
    out = []
    for i in range(n_samples(T)):
        t = i / RATE
        f = 170 + 25 * t + 8 * sine(v.step(23))
        s = lp(saw(a.step(f)) + 0.5 * saw(a.p * 2 % 1), 2200)
        crackle = (rng.random() * 2 - 1) * (1 if rng.random() < 0.08 else 0.15)
        out.append((s * 0.8 + crackle * 0.35) * env_ad(t, 0.02, T))
    return out


def disintegrator():
    """Dissolving chirps: a falling tone ring-modulated by a fast warble, with bursts of
    noise that thin out."""
    T = 0.8
    rng = random.Random(3)
    a, m = Phase(), Phase()
    bp = BandPass()
    out = []
    for i in range(n_samples(T)):
        t = i / RATE
        f = 1400 * math.exp(-t * 2.2) + 200
        tone = sine(a.step(f)) * sine(m.step(37 + 60 * t))
        burst = bp(rng.random() * 2 - 1, 3000 - 2000 * t / T, 4) * (1 if (i // 600) % 2 == 0 else 0.2)
        out.append((tone * 0.7 + burst * 1.5) * env_ad(t, 0.01, T))
    return out


def ion_ray():
    """A pulsed rising buzz: a square wave chopped 30 times a second."""
    T = 0.5
    a, c = Phase(), Phase()
    lp = LowPass()
    out = []
    for i in range(n_samples(T)):
        t = i / RATE
        f = 500 + 500 * t / T
        gate = 1.0 if c.step(30) < 0.6 else 0.25
        s = lp(square(a.step(f), 0.3), 4000) * gate + 0.3 * sine(a.p * 3 % 1)
        out.append(s * env_ad(t, 0.01, T))
    return out


def seeker_launch():
    """A missile leaving the rail: a thump, then a hiss sweeping upward and away."""
    T = 1.0
    rng = random.Random(5)
    bp, lp = BandPass(), LowPass()
    th = Phase()
    out = []
    for i in range(n_samples(T)):
        t = i / RATE
        noise = rng.random() * 2 - 1
        whoosh = bp(noise, 400 + 2600 * (t / T) ** 0.7, 1.5) * env_ad(t, 0.05, T)
        thump = sine(th.step(70 - 30 * t)) * env_exp(t, 0.08)
        hiss = lp(noise, 6000) * 0.25 * env_exp(t, 0.3)
        out.append(whoosh * 1.4 + thump * 0.9 + hiss)
    return out


# ---------------------------------------------------------------------------------------
# Impacts and explosions
# ---------------------------------------------------------------------------------------

def boom(T, seed, depth, bursts):
    """An explosion: low-passed noise whose cutoff falls as it decays, a sinking sub-bass
    thump, and extra bursts at the given times."""
    rng = random.Random(seed)
    lp1, lp2 = LowPass(), LowPass()
    th = Phase()
    out = []
    for i in range(n_samples(T)):
        t = i / RATE
        noise = rng.random() * 2 - 1
        e = env_exp(t, T * 0.28, 0.004)
        for bt in bursts:
            if t > bt:
                e += 0.6 * env_exp(t - bt, T * 0.15, 0.004)
        rumble = lp1(lp2(noise, 200 + 2500 * math.exp(-t * 4 / T)), 1800) * 3
        sub = sine(th.step(depth * (1 - 0.5 * t / T))) * env_exp(t, T * 0.3)
        crackle = noise * 0.15 if rng.random() < 0.02 * math.exp(-t * 2) else 0
        out.append(rumble * e + sub * 0.8 + crackle)
    return fade_out(out, 0.2)


def explosion():
    return boom(1.4, 6, 60, [0.18])


def big_explosion():
    return boom(2.8, 7, 42, [0.25, 0.6, 1.1])


def metal(T, seed, partials, noise_amt, tau):
    """A struck metal object: inharmonic sine partials with their own decays plus a short
    noise transient."""
    rng = random.Random(seed)
    ph = [Phase() for _ in partials]
    lp = LowPass()
    out = []
    for i in range(n_samples(T)):
        t = i / RATE
        s = sum(sine(p.step(f)) * a * math.exp(-t / (tau * d)) for p, (f, a, d) in zip(ph, partials))
        n = lp(rng.random() * 2 - 1, 3500) * noise_amt * env_exp(t, 0.025)
        out.append(s * min(1, t / 0.001) + n)
    return out


def hull_hit():
    return metal(0.35, 8, [(180, 1.0, 1.0), (412, 0.6, 0.7), (733, 0.45, 0.5), (1210, 0.3, 0.3)], 2.0, 0.09)


def collision():
    """A heavy clank followed by a short scrape."""
    a = metal(0.55, 9, [(95, 1.0, 1.0), (233, 0.7, 0.8), (391, 0.5, 0.6), (587, 0.35, 0.4)], 2.5, 0.14)
    rng = random.Random(10)
    bp = BandPass()
    for i in range(len(a)):
        t = i / RATE
        if t > 0.08:
            a[i] += bp(rng.random() * 2 - 1, 1800 + 600 * math.sin(t * 40), 3) * 0.8 * env_ad(t - 0.08, 0.03, 0.45)
    return a


def shield_hit():
    """An energy shield ringing: bell-like partials with a fast vibrato and a sizzle."""
    T = 0.45
    rng = random.Random(11)
    parts = [(620, 1.0), (1013, 0.6), (1475, 0.4), (2140, 0.25)]
    ph = [Phase() for _ in parts]
    v = Phase()
    bp = BandPass()
    out = []
    for i in range(n_samples(T)):
        t = i / RATE
        wob = 1 + 0.03 * sine(v.step(28))
        s = sum(sine(p.step(f * wob)) * a for p, (f, a) in zip(ph, parts))
        sizzle = bp(rng.random() * 2 - 1, 5000, 1.2) * 0.8
        out.append((s * 0.5 + sizzle) * env_exp(t, 0.12))
    return out


# ---------------------------------------------------------------------------------------
# Cockpit and interface
# ---------------------------------------------------------------------------------------

def beeps(notes, dur, gap, wave_fn=square, level=0.5):
    out = []
    for f in notes:
        p = Phase()
        for i in range(n_samples(dur)):
            t = i / RATE
            out.append(wave_fn(p.step(f)) * level * env_ad(t, 0.004, dur))
        out += [0.0] * n_samples(gap)
    return out


def lock():
    """Missile lock: three rising beeps."""
    return beeps([880, 1175, 1568], 0.09, 0.05) + [0.0] * n_samples(0.05)


def target():
    """Target select: a short bright blip."""
    p = Phase()
    return [sine(p.step(1500 + 600 * i / RATE / 0.06)) * env_ad(i / RATE, 0.003, 0.06) for i in range(n_samples(0.06))]


def ui_click():
    """A Bakelite switch: a 30 ms click of filtered noise and a tick."""
    rng = random.Random(12)
    lp = LowPass()
    p = Phase()
    out = []
    for i in range(n_samples(0.035)):
        t = i / RATE
        out.append((lp(rng.random() * 2 - 1, 5000) * 1.5 + sine(p.step(2400)) * 0.5) * env_exp(t, 0.006, 0.0005))
    return out


def denied():
    """Not allowed: a low detuned buzz, twice."""
    out = []
    for _ in range(2):
        a, b = Phase(), Phase()
        lp = LowPass()
        for i in range(n_samples(0.16)):
            t = i / RATE
            out.append(lp(square(a.step(110)) + square(b.step(116.5)), 1500) * env_ad(t, 0.005, 0.16))
        out += [0.0] * n_samples(0.05)
    return out


def system_damage():
    """Sparks (random crackles) over a two-tone alarm."""
    T = 0.9
    rng = random.Random(13)
    a = Phase()
    hp_prev = 0.0
    out = []
    for i in range(n_samples(T)):
        t = i / RATE
        f = 720 if int(t / 0.15) % 2 == 0 else 540
        alarm = square(a.step(f), 0.4) * 0.3 * env_ad(t, 0.01, T)
        spark = 0.0
        if rng.random() < 0.012 * (1.2 - t / T):
            spark = rng.random() * 2 - 1
        x = (rng.random() * 2 - 1) * 0.3 * env_exp(t, 0.15)
        hp = x - hp_prev
        hp_prev = x
        out.append(alarm + spark + hp)
    return out


# ---------------------------------------------------------------------------------------
# Engines, travel and special devices
# ---------------------------------------------------------------------------------------

def afterburner():
    """Rocket roar, loopable: low-passed noise with a rumbling sub-bass. The rumble's
    frequencies are whole multiples of 1/2 Hz, so they repeat exactly every 2 s."""
    L, F = 2.0, 0.25
    rng = random.Random(14)
    lp1, lp2, hp = LowPass(), LowPass(), LowPass()
    a, b = Phase(), Phase()
    buf = []
    for i in range(n_samples(L + F)):
        noise = rng.random() * 2 - 1
        roar = lp2(lp1(noise, 900), 1400)
        crackle = noise - hp(noise, 3000)
        buf.append(roar * 2.5 + crackle * 0.08 + 0.35 * sine(a.step(45)) + 0.2 * sine(b.step(67.5)))
    # Render F seconds more than the loop, then fade that tail over the start.
    return loop_crossfade(buf, F)


def saucer_hum():
    """The flying-saucer drone, loopable: a 60 Hz hum with harmonics, a slowly beating
    'wah' and a high whine with vibrato - every rate a whole number of cycles per 2 s."""
    L = 2.0
    ph = [Phase() for _ in range(6)]
    out = []
    for i in range(n_samples(L)):
        t = i / RATE
        wah = 0.65 + 0.35 * math.sin(TAU * 1.5 * t)
        hum = sine(ph[0].step(60)) + 0.5 * sine(ph[1].step(120)) + 0.3 * tri(ph[2].step(180)) * wah
        whine = 0.12 * sine(ph[3].step(1500 + 30 * math.sin(TAU * 5 * t)))
        beat = 0.25 * sine(ph[4].step(61.5))
        out.append((hum + beat) * wah + whine)
    return out


def hyperspace():
    """A rising whoosh: band-passed noise and a tone gliding up together."""
    T = 1.8
    rng = random.Random(15)
    bp = BandPass()
    a, s = Phase(), Phase()
    out = []
    for i in range(n_samples(T)):
        t = i / RATE
        x = t / T
        noise = bp(rng.random() * 2 - 1, 200 + 3200 * x * x, 2.5) * 1.6
        tone = sine(a.step(90 * 2 ** (x * 4.5))) * 0.4
        shimmer = sine(s.step(2200 + 900 * x)) * 0.1 * x
        out.append((noise + tone + shimmer) * env_ad(t, T * 0.75, T))
    return out


def tractor():
    """A tractor beam: a sine warbling (frequency modulation at 6 Hz) with tremolo."""
    T = 1.5
    a = Phase()
    out = []
    for i in range(n_samples(T)):
        t = i / RATE
        f = 300 + 70 * math.sin(TAU * 6 * t)
        s = sine(a.step(f)) + 0.3 * sine(a.p * 2 % 1)
        out.append(s * (0.75 + 0.25 * math.sin(TAU * 11 * t)) * env_ad(t, 0.1, T))
    return out


def theremin_tone(notes, vib=6.0, depth=0.012, glide=0.08):
    """A theremin line: a sine voice gliding between (frequency, seconds) notes with
    vibrato. The pitch moves exponentially toward each new note (a portamento)."""
    a, v = Phase(), Phase()
    out = []
    f = notes[0][0]
    for target_f, dur in notes:
        for i in range(n_samples(dur)):
            f += (target_f - f) * (1 - math.exp(-1 / (glide * RATE)))
            vf = f * (1 + depth * sine(v.step(vib)))
            p = a.step(vf)
            out.append(sine(p) + 0.12 * sine(p * 2 % 1))
    return out


def cloak():
    """Cloaking device: a theremin sliding down and fading into nothing."""
    T = 1.5
    a, v = Phase(), Phase()
    out = []
    for i in range(n_samples(T)):
        t = i / RATE
        f = 900 * (1 / 3) ** (t / T)
        out.append(sine(a.step(f * (1 + 0.015 * sine(v.step(6.5))))) * (1 - t / T) ** 1.5 * min(1, t / 0.05))
    return out


def theremin_sting():
    """The serial's dramatic sting: a theremin phrase over a swelling organ chord."""
    notes = [(659.3, 0.35), (523.3, 0.35), (415.3, 0.45), (440.0, 1.35)]
    voice = theremin_tone(notes)
    T = len(voice) / RATE
    organ = [Phase() for _ in range(6)]
    # A minor chord with an added sixth, organ-style: each note with octave harmonics.
    chord = [110.0, 220.0, 261.6, 329.6, 440.0, 370.0]
    out = []
    for i in range(len(voice)):
        t = i / RATE
        o = sum(sine(p.step(f)) * (0.5 if k < 2 else 0.3) for k, (p, f) in enumerate(zip(organ, chord)))
        swell = min(1, t / 0.6) * (1 - max(0, t - T + 0.5) / 0.5)
        out.append(voice[i] * 0.8 * min(1, t / 0.03) + o * 0.22 * swell)
    return fade_out(out, 0.5)


LOOPS = {"afterburner", "saucer_hum"}  # seamless loops: their ends must not be faded

def squelch(seconds, seed, level=0.35):
    """A radio squelch: a short burst of band-limited noise, as a channel opens or closes."""
    rng = random.Random(seed)
    out = []
    y = 0.0
    for i in range(n_samples(seconds)):
        x = rng.uniform(-1.0, 1.0)
        y += 0.35 * (x - y)  # one-pole low-pass: hiss rather than white noise
        out.append(y * level * env_ad(i / RATE, 0.002, seconds))
    return out


def radio_ack():
    """Wingman acknowledges an order: squelch, a two-tone chirp, squelch (no words)."""
    return squelch(0.05, 3) + beeps([1320, 1760], 0.07, 0.03, wave_fn=sine, level=0.45) + squelch(0.06, 4)


def radio_status():
    """Wingman status report: squelch and three quick chirps."""
    return squelch(0.05, 5) + beeps([1175, 1175, 1568], 0.05, 0.03, wave_fn=sine, level=0.45) + squelch(0.05, 6)


def radio_down():
    """A wingman lost: squelch, a falling tone, and a long crackle as the channel dies."""
    T = 0.6
    p = Phase()
    tone = [sine(p.step(900 - 600 * (i / RATE) / T)) * 0.45 * env_ad(i / RATE, 0.01, T) for i in range(n_samples(T))]
    return squelch(0.05, 7) + tone + squelch(0.35, 8, level=0.5)


def mission_win():
    """Mission accomplished: a bright rising arpeggio."""
    return beeps([523, 659, 784, 1047], 0.12, 0.02, wave_fn=tri, level=0.5) + [0.0] * n_samples(0.2)


def mission_fail():
    """Mission failed: a slow falling minor figure."""
    return beeps([659, 587, 523, 415], 0.18, 0.03, wave_fn=tri, level=0.45) + [0.0] * n_samples(0.2)


SOUNDS = [
    ("ray_zap", ray_zap), ("heat_ray", heat_ray), ("disintegrator", disintegrator), ("ion_ray", ion_ray),
    ("seeker_launch", seeker_launch), ("explosion", explosion), ("big_explosion", big_explosion),
    ("shield_hit", shield_hit), ("hull_hit", hull_hit), ("lock", lock), ("afterburner", afterburner),
    ("target", target), ("denied", denied), ("collision", collision), ("hyperspace", hyperspace),
    ("tractor", tractor), ("cloak", cloak), ("system_damage", system_damage), ("ui_click", ui_click),
    ("theremin_sting", theremin_sting), ("saucer_hum", saucer_hum),
    ("radio_ack", radio_ack), ("radio_status", radio_status), ("radio_down", radio_down),
    ("mission_win", mission_win), ("mission_fail", mission_fail),
]


def build_all():
    """Synthesises and writes every sound. Returns (name, seconds) pairs."""
    os.makedirs(OUT, exist_ok=True)
    out = []
    for name, fn in SOUNDS:
        buf = fn()
        if name not in LOOPS:
            fade_out(buf, 0.005)  # end exactly at silence: no click when the sound stops
        out.append((name, write(name, buf)))
    return out


if __name__ == "__main__":
    for name, secs in build_all():
        print(f"{name:16s} {secs:5.2f} s")
