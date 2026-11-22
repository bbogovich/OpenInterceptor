# Track A — combat-stat fidelity

A cross-stage track (proposed 2026-11-19, issue **#45** *fidelity*): make battles in the
`interceptor` campaign behave as the original does, by reading **what the original program does**
with its data — damage, beam fire, shields, flight, missiles, AI — and turning each fact into
**pack data** behind engine keys that any campaign can use. The facts come from static analysis of
the player's own `Interceptor.exe` and `xcgen.dll`, read as data only; the repository keeps them as
facts and numbers (`notes/re/flight/README.md`), marked **verified** (read from the machine code)
or **derived** (inferred from how the code uses a value). No code or text from the originals is
reproduced. The default campaign must keep working: it sets none of the new keys and plays as
before.

Each item: unit tests with synthetic data; the [Mod file reference](../../modding/reference.md)
and [Rulesets](../../modding/rulesets.md) pages for every new key; the
[Deviations](../deviations/index.md) revised; manual cases (time a duel against a target value).

## Items

| # | Work | Status |
|---|---|---|
| A0 | **Measurement harness**: a hit-rate option for `--balance` and a `--duel-report` (seeded AI-vs-AI duels per class pair and level). | Not started. `--balance` (app 0.15.0) is the only report; duel times are a manual case (MT-4-62). |
| A1 | **Damage pipeline**: shield → armour → hull, facings, systems. | **Largely done — app 0.21.0** (main `cb79f45`): `damage.spread` (75 – 100 %), per-side armour banks, `system_below` / `system_divert` (30 % of hull damage to systems below 75 % hull), the beam `shield_factor` (Phase Cannon 1.5), the difficulty factors on bank maxima and on the player's hits, `protection_scale`. Open: the system-hit table and per-system thresholds (DEV-064), pilot injury (DEV-065), the repair order (DEV-029), mount damage (DEV-068), the Tachyon and Psi effects. |
| A2 | **Beam fire**: `RateOfFire`, `EnergyCost`, `Speed`, `Range`. | **Largely done — app 0.21.0**: `RateOfFire` is shots per second per mount and `EnergyCost` energy per shot from the one pool (as built since Stage 2, now verified); `Range` is km (DEV-008 confirmed). Open: bolts do not inherit the ship's velocity in the original (DEV-043, to decide); knock-back (DEV-067); mount-health fire rate (DEV-068). |
| A3 | **Shields and energy**: the pool, recharge, balance. | **Largely done — app 0.21.0**: the 1000-point pool at 0 / 7 / 10 / 15 / 22 per second (`power.energy`, `energy_recharge`), start C / C, `power.balance: share`, bank maxima from `Special` × the difficulty factor, the afterburner at 100 per second. Open: accumulator shields (DEV-066), the cloak's drain (DEV-072). |
| A4 | **Stations**: the `*HP` charts. | Facts found: the original reads only `HP1`, as the station's maximum (verified); the engine still sums `HP1` – `HP4` (DEV-056). Station defence modules not traced. |
| A5 | **Missiles**: lock, guidance, fuse, special warheads. | Facts found (lock cones and times, `LockAngle` in millidegrees, guidance turn limit and lead, fuse radii, Permeator and Enveloping splits; verified) — not yet adopted. Missile and system `mass` landed in app 0.21.0 for the drive. DEV-073. |
| A6 | **Flight**: per-class values, `AngularDamp`, `LockSpeed`, `MaxSpeed`, acceleration. | **Largely done — app 0.21.0**: `flight.turning` (the original's damped turn model in its own units; AI at the per-class `ai_turn` table) and `flight.drive` (drag `dv/dt = a − 2v`; the player's thrust × engine power × throttle / mass, held to `MaxSpeed`; AI by the per-class drive factor `ai_speed`); hull, missile and system masses; fitted engines give thrust. DEV-011 and DEV-012 revised. Open: DEV-069 (AI speed requests), DEV-070 (live mass), DEV-071 (start throttle). |
| A7 | **AI lethality**: accuracy, fire cones, missile timing, point defence. | Facts found (lead and accuracy, fire cone and range by level, trigger intervals, weapon choice, AI missile lock; verified) — not yet adopted; DEV-032 and DEV-038 stand. |
| A8 | **Pilot stats in battle** (#28). | Facts found (which pilot values the AI reads; pilot injury) — not started. |
| A9 | **Default campaign re-tune** (#56). | The default campaign sets none of the app 0.21.0 keys and plays as before; a re-tune is not needed yet. |
| A10 | **Auto-resolve** (DEV-057) with the corrected numbers. | Not started. |

## B3 — the clock (settled)

The strategy-screen parity track's question about the clock is settled by the program
(verified): the dial has six speeds — 5 s, 1 min, 5 min, 30 min, 1 h and 1 day of game time per
real second — and **no pause**; the "(5 secs)" buttons select the slowest speed; modal dialogs
pause the clock while open. App 0.21.0 makes the clock data (`clock:` in the strategy layer,
[Strategy layer › `clock`](../../modding/strategy.md#clock)): the `interceptor` campaign has the
dial's six speeds, events drop to 5 s per second and Space runs at 30 min per second; the
default campaign keeps a pause and four speeds. Still to observe in play: whether the clock runs
on when the player leaves the main screen, and when alerts interrupt. The specification's
R-STR-001 (pause and four speeds) describes the default clock; its As-built text is to be
revised in the next specification release.

## Still open in the analysis

From the notes' list of open questions: what the pilot "temperament" field means and whether
alien ships carry pilot records; whether the AI missile-lock comparison is inverted; the other
agent states, formation speed matching and the non-fighter agents; whether drag applies to
missiles; the Doppelganger, Nova, Passive Cloaking and Parallax per-frame routines; what damaged
sensors and hyperdrives do; the engine-off threshold; station defence modules.
