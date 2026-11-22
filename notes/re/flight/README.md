# Reverse-engineering notes: flight, combat and the strategy clock

Facts read from the player's own `Interceptor.exe` and `xcgen.dll` (CD release, 2026-11-21): formulas,
constants, table layouts and addresses. Neither code nor text from the originals is reproduced here.
**Verified** means read from the machine code. **Derived** means inferred from how the code uses a
value. Addresses are for the CD build (see `../wrappers/re-notes.md` for the PE layout; patch 1.2 is a
full rebuild with different addresses).

How it was read: Hopper Disassembler 5 headless (`hopper -e Interceptor.exe -l PE --intel-32 -a -Y
script.py`; the script walks `Document.getCurrentDocument().getSegmentByName('.text')` and writes
`proc.decompile()` for all 20,286 procedures to a scratch file; three procedures crash the decompiler),
plus `../wrappers/tools/d.py` for x87 code, which Hopper leaves as inline `asm`. Outputs stay in scratch.

## 1. GameSettings.ini → memory (verified)

The loader `0x5ee000..0x5f0b00` reads each key with `GetPrivateProfileInt`-style `0x44cd50(section,
key, default, file)` and stores it in a static table. Index order is fixed by the caller
(`0x4349c9`), not by the INI's section order.

| Table | Base (record 0) | Stride | Key fields (offset in record) |
|---|---|---|---|
| Hulls `Ship_*` | 0x73b830 | 0x70 | name +0x00 (0x18), Side +0x19, ProdTime +0x1c, Cost +0x20 (×1000), MaintCost +0x3c (×1000), HitPoints +0x48, CargoCapacity +0x4c, RollRate +0x54, YawRate +0x58, PitchRate +0x5c, AngularDamp +0x60, LockSpeed +0x64, MaxSpeed +0x68, Restrictions +0x6c |
| Beams | 0x740680 | 0x94 | Side +0x1, WeaponClass +0x52, Range +0x58, Damage +0x5c, EnergyCost +0x60, HitPoints +0x64, RepairRate +0x68, RechargeRate +0x6c, RateOfFire +0x70, Penalty +0x74 (float), Speed +0x78 |
| Missile launchers | 0x73c210 | 0x84 | AmmoType +0x58, Rounds +0x5c, HitPoints +0x60, RateOfFire +0x68 |
| Missiles | 0x73eee8 | 0x84 | WeaponClass +0x52, Speed +0x58, Duration +0x5c, ImpactDamage +0x60, ProximityDamage +0x64, HitPoints +0x68 |
| Equipment (armour, shields, engines, …) | 0x73f820 | 0x88 | EnergyCost +0x58, RechargeRate +0x5c, HitPoints +0x60, RepairRate +0x64, Range +0x68, Special +0x6c |
| `BeamRechargeRate` A–E | 0x73dc20 | 5 × int | |
| `ShieldRechargeRate` A–E | 0x73f7f0 | 5 × int | |
| `SensorsSetting{A,B,C}` LockTime / LockAngle | 0x740ba8 | 3 × (int, float) | |
| `Afterburners` Boost / Cost | 0x740660 / 0x740ba0 | | read, then used **only** by the multiplayer settings sync (`0x435814`); flight never reads them |
| Station HP charts `*HP` HP1–HP4, SD | 0x73f628 | 5 × int each | |

Indexes. Beams: 0 LaserCannon, 1 HeavyLaser, 2 GatlingLaser, 3 TracerCannon, 4 TrackingTracerCannon,
5 PhaseCannon, 6 PlasmaCannon, 7 PsiBlaster, 8 TachyonPulser. Missiles: 0 XWinder, 1 Fusion,
2 Permeator, 3 Doppelganger, 4 NovaBomb, 5 EleriumTorpedo, 6 EnvelopingFusionTorpedo, 7 Sidestep,
8 PassiveCloakingTorpedo, 9 ParallaxTorpedo. Equipment: 0–2 ArmorType1–3, 3–5 ShieldType1–3,
6 ShipDamageControlComputer, 7–9 EngineType1–3, 10 VectorControlThrusters, 11 DimensionJumpDrive,
12–14 SensorTargetingSystem1–3, 15 CloakingDevice, 16–17 ECMPodType1–2, 18 TractorBeam, 19–25 probes,
**26 ShieldType4 = accumulator shields** (`R_ACCUMULATOR_SHIELDS`). Hulls: 0–2 IntType1–3,
3 XCOMFreighter, 4 OutpostFreighter, 5–10 AlienType1–6, 11 AlienFreighter, 12–14 PirateType1–3,
15 PirateFreighter, 16–18 CorporateType1–3, **19 PattonClass**.

Difficulty is the global `0x740b9c`, 0 = Beginner … 4 = Superhuman (it indexes the five-entry
Goodies tables).

## 2. Ship record in flight (verified offsets, derived names)

`0x40ae10(class, owner)` creates a ship instance. Hull +0xec (from the hull's HitPoints, no difficulty
factor), shields fore +0xf0 / aft +0xf4, armour fore +0xf8 / aft +0xfc, mass +0xe8, hull class +0x12e,
fitted engine +0x12f, armour +0x130, shield +0x131, owner +0x133 (0 = X-COM), special systems +0x134
(6 bytes), beam mounts +0xa0 (6 × {next-fire ms, hit points, beam type}), flight-control health
+0x124 (starts at 40), engine health +0x118.

A per-ship **power block** (`0x4d44f9(ship)`): beam setting +0xc, shield setting +0xe (0–100 in steps
of 25 = A–E), special-system draw +0x10, **charge** +0x12 (0–1000), throttle +0x14 (−100…+100 %),
shield balance +0x16 (0 fore first, 1 split, 2 aft first). Start values: beams 50, shields 50,
draw 0, charge 1000, throttle 50, balance 1.

## 3. Energy, beam fire and shields

Once per second (`0x45a3cf` → `0x45813b` for each ship, verified):

* charge += `BeamRechargeRate[beam_setting / 25]` (0, 7, 10, 15, 22 per s), capped at 1000.
* **Cloaked** (flag bit 0 at ship +0x40): charge −= 50 per second instead of shield recharge. At
  0 the cloak drops, draw −= 50, and the object's motion fields reset.
* Otherwise shields gain `ShieldRechargeRate[shield_setting / 25]` (0, 1, 2, 3, 5 per s), halved
  when the shield generator is below 50 % of its HitPoints and zero below 25 %. The gain is shared
  by the balance setting (`0x457f1a`): fore first then the overflow aft, half each, or aft first.
  Each bank is capped at the maximum below.

Beam fire (`0x4fed62`, verified). Mode 2 fires every mount, mode 1 every mount of the selected type,
and mode 0 the selected mount. A mount fires when its timer has expired and charge > `EnergyCost`.
Each shot costs `EnergyCost` and sets the next shot `1000 / RateOfFire` ms later, so **RateOfFire is
shots per second per mount**. A mount above 50 % of its `HitPoints` fires at the full rate, one at
25–50 % at half rate, and one at 25 % or below cannot fire.

Bolts (`0x4fd32d`). Velocity is `Speed × aim direction`, **without the firing ship's velocity**.
Lifetime is `Range × 1,000,000 / Speed` ms, so **Range is in km** (DEV-008 confirmed).

Afterburner (flag bit 4, `0x601d95`). While on, it costs 100 charge per second (frame ms × 100 / 1000)
and adds `Thrust × 50 / mass` to the acceleration. It switches off when the charge cannot pay.

Maximum shield per bank (`0x40c333`) = `Special × f`. Maximum armour per bank (`0x40c4ca`) =
armour `HitPoints × g`. **Both banks start full; armour is per side.**

| Difficulty | f, X-COM | f, others | g, X-COM | g, others |
|---|---|---|---|---|
| Beginner | 1.50 | 0.40 | 1.30 | 0.50 |
| Experienced | 1.25 | 0.52 | 1.15 | 0.60 |
| Veteran | 1.00 | 0.64 | 1.00 | 0.70 |
| Genius | 1.00 | 0.76 | 1.00 | 0.80 |
| Superhuman | 1.00 | 0.88 | 1.00 | 0.90 |

Others: `f = max(0.1, 1 − (5 − d) × 0.12)`, `g = max(0.1, 1 − (5 − d) × 0.1)`. X-COM:
`1 + (2 − d) × 0.25` and `1 + (2 − d) × 0.15` when d < 2. The Patton class has shields × 3 and
armour × 4.

## 4. Damage (verified)

A beam hit (`0x45352c`) does `Damage − rand() % (Damage / 4 + 1)`, which is 75–100 % of `Damage`.
On Beginner and Experienced the player's own hits are × 1.15. On Beginner, hits **on** the player are
× 0.85. Then:

* PhaseCannon (5): × 1.5 against shields (multiplier `m` below).
* HeavyLaser (1) and PlasmaCannon (6) also knock the target (`0x45127b`).
* PsiBlaster (7): no damage; its own effect (`0x452bd0`).
* TachyonPulser (8): its own routine (`0x451b1e`).
* Accumulator shields (26): while the struck bank is up, an ordinary hit is not damage; a quarter
  of it is added to the shields.

A missile hit (`0x4539ec`) does `ImpactDamage` (direct) or `ProximityDamage`, each × 75–100 %.
Permeator (2) uses `0x451d75` and Enveloping (6) uses `0x45261b`; all others use the common path.
No difficulty factor applies.

Common path `0x454d25(target, attacker, damage, side, m)`, with side = fore or aft of the target:

1. **Shield bank** of that side (skipped entirely while the target is cloaked). If
   `shield ≥ damage × m`, the shield loses `damage × m` and nothing else happens. Otherwise
   `damage −= shield / m` and the bank goes to 0.
2. **Armour bank** of that side. If `armour ≥ damage` it absorbs everything. Otherwise
   `damage −= armour` and the bank goes to 0.
3. **Hull.** If the hull is below 75 % of the hull's HitPoints, 30 % of the remaining damage goes to
   the ship's systems instead (`0x44ddee`). If `rand(100) < damage` (a chance of `min(damage, 100)` %),
   a further 1–3 system hits follow (`0x4516ae`).
4. If the hull is greater than the damage, it loses the damage. Otherwise the ship is destroyed
   (`0x4a10cf`).

## 5. Flight model

### Player (`0x602154`, every frame; verified)

Per axis, with frame time `t` in seconds and stick input `s ∈ [−1, 1]` (keys give ±1):

* `x = min(speed / LockSpeed, 1)`.
* `u_roll = RollRate × s × 0.08`.
* `u_pitch = PitchRate × s × 0.08 × (1 − x)`, and the same for yaw with YawRate. **Roll is not slowed
  by speed.**
* `k = (1 − 0.1 × AngularDamp) × 4 × q`, where `q = max(control health, 1) / 40`; undamaged, `q = 1`.
* Exact first-order step (`0x6020b2` / `0x6020fc`): `ω ← ω·e^(−kt) + (u/k)(1 − e^(−kt))`. The
  rotation angle is the integral of that, so **the steady rate is `u/k` rad/s** and the time constant
  is `1/k`.
* When `q < 1` (damaged controls), a random wobble proportional to `(1 − q)` and to `x` is added.
* Angles are in **radians**: `XCGENRotateObject` → `xcgen 0x10008dd0` integrates the orientation
  quaternion directly by them.

Worked example, Lightning (Roll 200, Pitch/Yaw 100, AngularDamp 2, LockSpeed 1000): `k = 3.2`,
τ = 0.31 s. Roll is 5.0 rad/s (286°/s). Pitch and yaw are 2.5 rad/s (143°/s) at rest, falling
linearly to 0 at 1000 m/s.

Thrust (`0x601d95`). Engine power `p = (100 − ((beam + shield) / 3 + draw)) / 100`, which is 0.67 at
the start. `a = EngineType.Special × 1000 (N) × p × throttle% / mass`. Reverse throttle is halved.
An engine below 50 % health gives × 0.5, below 25 % × 0.25, and below 15 % nothing. A tractor-beam
tow adds the towed ship's mass, or 100,000 kg for anything else. The thrust is applied along the nose.

Linear motion (`xcgen 0x10007300`). **`dv/dt = a − 2v`**, stepped exactly with `e^(−2t)`. So the
terminal speed is `a / 2` and the speed time constant is 0.5 s. Then, for the player, velocity is
**scaled back to MaxSpeed** whenever it exceeds it (`0x60313c`), afterburner or not.

Mass (`0x407880`, kg) = hull base + 250 with a shield + Σ rounds × per-missile mass + special
systems (a further jump table). Hull bases: IntType1 / Pirate1 / Corp1 6,700; IntType2 / Pirate2 /
Corp2 12,700; IntType3 / Pirate3 / Corp3 22,700; AlienType1 5,500; AlienType2 and 4 11,000;
AlienType3 19,000; AlienType5 and AlienFreighter 110,000; AlienType6, all freighters 157,000;
Patton 250,000. Per missile: XWinder 85, Fusion 940, Permeator 370, Doppelganger 1,145, Nova 11,000,
Elerium 70, Enveloping 670, Sidestep 230, PassiveCloaking 440, Parallax 210.

Worked example, Lightning (75 kN, ≈ 8,300 kg as fitted): at the start settings and 50 % throttle,
`a ≈ 300 m/s²` and terminal speed ≈ 150 m/s. At full throttle it is ≈ 300 m/s. With beams and
shields at A it is ≈ 450 m/s. The afterburner adds ≈ 450 m/s² (+225 m/s terminal), and MaxSpeed
600 caps the total.

### AI (`Agents.cpp`; verified)

AI ships do **not** use the hull's RollRate etc. Each agent turns at a constant clamped rate,
`clamp(request, ±max) × t`, with no damping (`0x585bf6`). Its per-axis maximum is
`table[class] × (2.0, 2.0, 3.0)` rad/s (`0x58645f`; table at 0x67cee0, 3 floats per class):

| Class | × 2.0 | × 2.0 | × 3.0 |
|---|---|---|---|
| IntType1, Corp1 | 1.05 | 1.00 | 0.85 |
| IntType2, Pirate2, Corp2 | 0.85 | 0.70 | 0.50 |
| IntType3, Pirate3, Corp3 | 0.75 | 0.65 | 0.50 |
| XCOMFreighter | 0.50 | 0.50 | 0.10 |
| AlienType1 | 1.15 | 1.05 | 1.25 |
| AlienType2 | 0.85 | 0.85 | 0.25 |
| AlienType3 | 0.65 | 0.65 | 0.50 |
| AlienFreighter | 0.35 | 0.35 | 0.10 |
| Patton | 0.15 | 0.15 | 0.05 |
| Others (OutpostFreighter, AlienType4–6, Pirate1, PirateFreighter) | 1.00 | 1.00 | 1.00 |

Which axis is which is derived; the third (× 3) is probably roll. Agents with no class get
(2, 2, 3). A per-category cruise value of 225, 100 or 250 is set at agent +0x144 (`0x5863dd`); its
use has not been traced yet.

## 6. Strategy clock (verified)

`GameClock::SetSpeed(n)` = `0x411f74` (n = 0–5). Game time (seconds) advances by
`elapsed_ms / 1000 × rate[n]`, with `rate` at 0x66f6f0 = **5, 60, 300, 1800, 3600, 86400** game-seconds
per real second (5 s, 1 min, 5 min, 30 min, 1 h, 1 day per second). The dial has no pause hole: the
slowest speed is 5× real time, and the "OK (5 SECS)" / "Return to base (5 secs)" buttons select it.
Modal dialogs pause and resume the clock with `0x4120b7` / `0x41207f`.

## 7. Still open

Missile guidance, lock time and cone use (`SensorsSetting*`), `Penalty`; what Tachyon, Psi,
Permeator and Enveloping do in their own routines; system-damage distribution (`0x44ddee`,
`0x4516ae`); repair (`0x458534`); AI speed and throttle control; point defence; station HP1–HP4/SD;
the special-systems mass table.
