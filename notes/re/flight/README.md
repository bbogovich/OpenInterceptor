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
| Beams | 0x740680 | 0x94 | Side +0x1, WeaponClass +0x52, Range +0x58, Damage +0x5c, EnergyCost +0x60, HitPoints +0x64, RepairRate +0x68 (never read), RechargeRate +0x6c, RateOfFire +0x70, Penalty +0x74 (float), Speed +0x78 |
| Missile launchers | 0x73c210 | 0x84 | AmmoType +0x58, Rounds +0x5c, HitPoints +0x60, RateOfFire +0x68 |
| Missiles | 0x73eee8 | 0x84 | WeaponClass +0x52, Speed +0x58, Duration +0x5c (s), ImpactDamage +0x60, ProximityDamage +0x64, HitPoints +0x68; `Range` and `Penalty` are never loaded (§5.2) |
| Equipment (armour, shields, engines, …) | 0x73f820 | 0x88 | EnergyCost +0x58, RechargeRate +0x5c, HitPoints +0x60, RepairRate +0x64 (never read), Range +0x68, Special +0x6c |
| `BeamRechargeRate` A–E | 0x73dc20 | 5 × int | |
| `ShieldRechargeRate` A–E | 0x73f7f0 | 5 × int | |
| `SensorsSetting{A,B,C}` LockTime / LockAngle | 0x740ba8 | 3 × (int, float) | |
| `Afterburners` Boost / Cost | 0x740660 / 0x740ba0 | | read, then used **only** by the multiplayer settings sync (`0x435814`); flight never reads them |
| Station HP charts `*HP` HP1–HP4, SD | 0x73f628 | 0x14 per chart, 11 charts in INI order | only **HP1** is read: `0x41f9fe` returns it as the station's maximum HP (XCOMBase, AlienBase, PirateArms/Raider/Base, Science, NovaScience); HP2–HP4, SD and the four Outpost/OPP charts have no references |

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
(6 bytes; masses in §5), beam mounts +0xa0 (6 × {next-fire ms, hit points, beam type}), flight-control health
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
   the ship's systems instead (`0x44ddee`, §4.1). If `rand(100) < damage` (a chance of
   `min(damage, 100)` %), the **pilot** loses 1–3 health (`0x4516ae`, §4.2). *Corrected: this was
   earlier read as further system hits.*
4. If the hull is greater than the damage, it loses the damage. Otherwise the ship is destroyed
   (`0x4a10cf`).

### 4.1 System damage `0x44ddee(rt, damage)` (verified)

This routine takes the 30 % share. It makes up to 5 tries, each with `roll = rand(100) + 1`. A try
that finds the system missing or already at 0 still counts.

| Roll | System | HP field (maximum) | When damage ≥ the system's HP |
|---|---|---|---|
| 1–10, 26–35 | a random fitted beam mount | mount +0xa4 (beam `HitPoints`) | HP 0, the rest is lost |
| 11–25, 36–50 | a random fitted launcher | +0x54 (launcher `HitPoints`) | HP 0, its rounds (+0x50) cleared, the rest is lost |
| 51–60 | shield generator | +0x120 (equipment `HitPoints`) | HP 0, **the rest carries to the next try** |
| 61–70 | flight control | +0x124 (40) | HP set to **1**, the rest is lost |
| 71–80 | targeting sensor (type +0x132) | +0x11c | carries |
| 81–90 | main engine | +0x118 | carries; `0x44d874` (engine off) runs when destroyed, and also on some non-destroying hits (threshold not decoded) |
| 91–95 | hyperdrive | +0x128 (80) | not traced |
| 96–98 | a random special system (+0x134) | +0x100 + 4i | HP 0 |
| 99–100 | none; the damage is discarded | | |

The 50 % and 25 % thresholds inside this routine only choose messages. The effects themselves are
those in §3 (beams, shields) and §5 (engine, control). What a damaged sensor or hyperdrive does is
unknown.

### 4.2 Pilot injury `0x4516ae(rt, n)` (verified)

This applies only to ship objects (rt +0x44 = 1) whose pilot is alive. The pilot's health (pilot
record +0x24, a dword) drops by `n`:

* At 0 the pilot is unconscious: rt flag 0x1000 is set.
* Below 0 the pilot is dead: flag 0x1000 is cleared and 0x2000 set.
* Unless rt flag 0x8 is set, a dead pilot's ship becomes a tumbling wreck: thrust is zeroed, the
  object's angular fields +0x68/+0x6c/+0x70 get `rand() / 100` and +0x74 gets 1.0, and the velocity
  becomes 10 × an object axis.

Callers:

| Caller | n |
|---|---|
| Common path | 1–3, when the `min(damage, 100)` % roll succeeds |
| Permeator, Enveloping, a beam path | rand(3) + 1, always |
| `0x4a1372` (collision, derived) | 5–10 |
| `0x605f58` | 20 |

### 4.3 Repair `0x458534` (verified)

Repair runs from the 1 s systems tick after the recharge (`0x45813b`). Each call repairs **2 points,
or 4 with a ShipDamageControlComputer** (special type 6, test `0x4092b6`), and all of them go to the
first item in this order that needs repair:

1. Engine
2. Hull +0xec
3. Shield generator
4. Flight control, up to 40
5. Hyperdrive, up to 80
6. Sensor
7. The first damaged beam mount, then launcher, then special system

**A destroyed system (HP 0) is never repaired**; for the engine, shield, sensor, mounts and
specials, repair needs HP > 0. With rt flag 0x8 (meaning unknown), a different branch repairs the
hull first. **The INI `RepairRate` keys are never read**: beams +0x68 (0x7406e8) and equipment +0x64
(0x73f884) have no code references.

### 4.4 Special beams and point defence (verified)

**TachyonPulser (8)** (`0x451b1e`) affects ships only. The struck side's shield absorbs 1:1, with no
`m` factor, and is skipped while the target is cloaked. Whatever is left **bypasses armour and hull**
and goes to a second system table, `0x44fe59`:

| Roll | System hit |
|---|---|
| 1–5, 11–15 | beam mount |
| 6–10, 16–20 | none (wasted try) |
| 21–30 | shield generator |
| 31–35 | sensor |
| 36–85 | engine |
| 86–95 | hyperdrive |
| 96–98 | special system |

The table has no launcher or control entries. A Tachyon hit also sets rt flag 0x10000, which slows
repair from every 1 s to every 10 s (`0x45a3cf`). Repair clears the flag once nothing is left to
repair.

**PsiBlaster (7)** (`0x452bd0`) does no damage.

* It needs pilots on both ships, and the target's pilot health must be > 0.
* Chance: `t = attacker.Psionics + 10 − target.Psionics`, minus 50 when the target carries
  ShieldType3 (equipment 5). The attack fails when `rand(100) > t`.
* Effect, with `b = 100 − target.Bravery`: b > 80 takes control (agent mode 2), 41–79 panic
  (mode 1), b < 40 flee (mode 3). At exactly 40 or 80 nothing happens (a quirk of the original).
* Duration: rand[30,000, 120,000) ms, through `0x5a49aa(mode, ms)`.
* It never affects the local player, who only hears a sound.

**Point defence.** There is no automatic PD turret. A beam bolt that strikes a missile object has a
90 % chance of applying the beam's `Damage`, with no 75–100 % roll. A missile with less HP than that
is destroyed (`0x455dfa`, so it detonates); otherwise its HP drops by the damage. Missile HP =
clamp(`HitPoints`, 10, 500), set at launch. Station `*_DEFENSE_MODULE_*` defences were not traced.

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
systems (jump tables 0x407e57 / 0x407e73): ShipDamageControlComputer 100, VectorControlThrusters 450,
DimensionJumpDrive 350, CloakingDevice 550, ECMPodType1 and 2 200 each, TractorBeam 300, engines and
sensors 0. Hull bases: IntType1 / Pirate1 / Corp1 6,700; IntType2 / Pirate2 /
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
(2, 2, 3). The value 225, 100 or 250 at agent +0x144 (`0x5863dd`) is **not a cruise speed**: it is
the obstacle look-ahead distance in metres (see §5.1 below).

Units of the turn request (verified): the requested angle errors come from `0x5f4371`, which returns
**degrees** (`× 180 / π`), while `XCGENRotateObject` takes radians. So the clamp is applied to a
degree error and the result is used as radians: above the per-axis maximum the agent turns at the
maximum, below it the per-frame step is `error° × t` rad, which closes almost all of a small error
in one 60 Hz frame. When the pilot's accuracy roll fails (below), `0x585bf6` runs in its second mode
for that frame: step = `error° × 0.06 × t`, no clamp (a sluggish frame).

### 5.1 AI speed, attack and fire control (`Agents.cpp`; verified unless marked)

**Agent record** (offsets in the agent object): rt index +0x8, rt record pointer +0xc, hull class
+0x10, **speed factor S** +0x14, **throttle** +0x18, target rt index +0x6c, target distance +0x70,
turn maxima +0x27c/+0x280/+0x284, look-ahead +0x144, trigger interval +0x148 (ms), last beam shot
+0x14c, last missile +0x150, last weapon choice +0x154, pilot temperament +0x2bc.

**Thrust.** AI ships never run the player's thrust routine (`0x601d95` is called only from the player
routine `0x602154`). An agent writes its acceleration straight into the xcgen object:
`a = S × throttle` along the nose (`0x585b5f`). xcgen's motion step (`0x100073e0` → `0x10007300`)
applies `dv/dt = a − 2v` to every object, so **terminal speed = S × throttle / 2**. Mass, engine
type, engine damage, power settings, afterburner and the hull's `MaxSpeed` play **no part**: the
MaxSpeed cap (`0x60313c`) sits inside the player routine, and no Agents code reads engine health
+0x118 or control health +0x124.

Throttle setters: `SetThrottle(x)` (`0x585a1b`) clamps x to [−0.25, 1.0]; `SetSpeed(v)`
(`0x585a79`) sets throttle = v / S (formation keeping); `0x585aec` scales it (× 1.25 is used). The
constructor default is 0.5. Seen in manoeuvres: 0.25, 0.5, 0.75, 1.0, and in obstacle avoidance 0.5
when the obstacle is nearer than half the look-ahead, else 0.75.

S per hull class (agent factory `0x5a3152`, jump table by hull class; then `0x58645f` clamps S to
**[100, 500]**, a value below 100 becoming 100). `rand[a,b)` is `0x4050d0`, d the difficulty 0–4:

| Classes | Constructor | S before clamp | S used | Top speed (S/2) |
|---|---|---|---|---|
| IntType1, Pirate1, Corp1 | `0x592205` | rand[100,150) + 25d + 500 | 500 | 250 m/s |
| IntType2, Pirate2, Corp2 | `0x592ba4` | … + 550 | 500 | 250 m/s |
| IntType3, Pirate3, Corp3 | `0x593305` | … + 600 | 500 | 250 m/s |
| XCOM, Outpost, Pirate freighters | `0x593be9` | 100 + rand % 75 | 100–174 | 50–87 m/s |
| AlienType1, 2 | `0x58ca7b` | rand[50,100) + 40d + 300 | 350–500 | 175–250 m/s |
| AlienType3 | `0x58d92c` | rand[50,100) + 30d + 400 | 450–500 | 225–250 m/s |
| AlienType4 | `0x58f153` | rand[100,150) + 600 | 500 | 250 m/s |
| AlienType5 | `0x590790` | uniform float [200, 250) | 200–250 | 100–125 m/s |
| AlienType6 (base agent) | `0x585133` | 175 + rand % 175 | 175–349 | 87–175 m/s |
| AlienFreighter | `0x5912ed` | 100 + rand % 75 | 100–174 | 50–87 m/s |
| Patton | `0x59444a` | 0 | 100 | 50 m/s |
| Drone (`drone01`, `0x5a3edd`) | `0x58c5ed` | 250 | 250 | 125 m/s |

So the difficulty term only matters for AlienType1–3 (every fighter class is at the 500 cap).

**Look-ahead** +0x144 (`0x5863dd`, from pilot record +0xa0 via `0x410a56`, called
"temperament" here; its meaning is unknown): 0 → 225 m, 2 → 100 m, anything else (and no pilot) → 250 m. `0x587065` / `0x58f33b`
cast `XCGENLineOfSight` from the ship along its **velocity** for that distance and steer and slow
when it hits something.

**Pilot record** (from `0x40fb76` on the pilot table at 0x6bb8d8; labels from the pilot screen at
`0x537ef1`): Intelligence +0x2a, Stamina +0x2b, **Accuracy +0x2c**, Flying ability +0x2d,
Aggressiveness +0x2e, Bravery +0x2f, Psionics +0x30, health +0x24. A ship with no pilot record uses
50 wherever a pilot value is needed (25 in weapon choice). Whether alien ships carry pilot records is
not confirmed (derived: they do not, so they use the defaults).

**Attack routine** `0x588891`, used by every fighter class (IntType/Pirate/Corp 1–3, AlienType1–4,
drone); the freighters, AlienType5–6 and Patton do not call it. Per frame against a ship target:

1. Range `dist` = agent +0x70. Beyond 1,750 m the aim point is the target's position (pure
   pursuit), the fire mode is set to 0, and beyond 3,000 m, when the temperament is non-zero or
   d > 2, the agent sprints: `a = 1.25 × 0.85 × S` (terminal ≈ 0.53 S).
2. Within 1,750 m it **leads**: aim = target position + target velocity × `T` + offset, with
   `T = dist / Speed` of the selected beam type (beam table +0x78; power block +0x1c). The accuracy
   `A` (%) is: with no pilot, 50; with d ≥ 2, the pilot's Accuracy; with d < 2, agents whose side byte
   (+0x1c, from object +0xc0 bits 24–31; 0 = X-COM, derived) is 0 get clamp(Accuracy, 50, 100) and others clamp(Accuracy, 25 + 10d, 50 + 10d). Then with 50 % chance
   `T × = A/100`, otherwise `T × = 2 − A/100` (under- or over-lead by `1 − A/100`). If `A < 50`,
   the roll `chance(A)` fails and `dist > 200 m`, a lateral offset of uniform 15–25 m, random sign,
   along the target's x or y axis (50/50) is added.
3. It turns toward the aim point (`0x585bf6`), in the sluggish mode when `chance(A)` fails.
4. Fire mode (power block +0x18): when the yaw error is under 5°, `chance((Intelligence + Accuracy)
   / 2)` sets mode 1 (all mounts of the selected type), otherwise mode 0 (the selected mount).
5. **Beam fire** when all hold: |yaw| and |pitch| error < `C = 2.5 − 0.25 (4 − d)` degrees (1.5° on
   Beginner … 2.5° on Superhuman); `dist < R = 2500 − 125 (4 − d)` m (2,000 … 2,500); a line of
   sight along the nose for `R` hits the target or nothing; and `now − last shot > +0x148`. Then
   `0x4ff900` fires through the player's beam path (`0x4fed62`: charge, `EnergyCost`, `RateOfFire`
   and mount damage all apply). +0x148 is 0 for every class except the drone
   (1,500 + 50 (4 − d) ms) and AlienType1–2 (500 + 60 (4 − d) ms). An rt object without a power
   block fires beam type 6 (PlasmaCannon) through `0x50040e` instead.
6. **Missile fire** is tried first, in the same cone/range block: a missile slot is selected (power
   block +0x1e), the lock timer (power block +0x20) is running, `dist < R / 2`, more than 5,000 ms
   since the last missile, and `chance(5 + 2d)` per frame. The lock timer starts when both errors are
   within ±5° and resets to "none" when either leaves. A NovaBomb (missile 4) fires at once when the
   agent's flag +0x2b8 is set. Otherwise the code fires only while `now < lock start + LockTime`
   (`0x458c12`: LockTime of the fitted SensorTargetingSystem1–3, 0 with no sensor). As read, that
   makes an AI ship without a sensor unable to fire missiles; the comparison may be inverted in the
   original (derived).
7. Under 250 m it switches to state 3 (break off, `0x585828`).

Against a non-ship target (rt types 2 and 10–20), it aims at the target position or a hardpoint
without lead.

**Weapon choice** `0x5853b8`, at most every 3,000 ms (+0x154). With `W = 0.75 × Intelligence +
0.25 × Flying` (25 with no pilot) it re-picks the selected beam type from its mounts:

| Range | Preferred | Alternatives (each later one replaces with 33 %) | Never chosen |
|---|---|---|---|
| ≤ 1,500 m | GatlingLaser, PhaseCannon | LaserCannon, PlasmaCannon, TachyonPulser | Heavy, Tracer, TrackingTracer, Psi |
| > 1,500 m | HeavyLaser, PhaseCannon | Laser, Gatling, Tracer, TrackingTracer, Plasma | Psi, Tachyon |

It takes the preferred mount with `chance(W)`, else an alternative with a second `chance(W)`, else
keeps the current one. If the selected missile slot is empty, it selects the first launcher slot
with rounds left that is not a NovaBomb. Hull classes 8–10 (AlienType4–6) skip the whole routine
and keep their initial selection.

### 5.2 Missiles (verified unless marked)

**Sensor lock.** The loader (`0x5f0093`) stores `LockAngle` as `180 / (LockAngle / 1000)` and
`0x458e37` turns it back into `π / stored`, so **`LockAngle` is in millidegrees** and `LockTime` in ms:

| Sensor (ship +0x132) | Setting | LockTime | Cone half-angle | Range vs ships | Range vs rt types 2–5 |
|---|---|---|---|---|---|
| SensorTargetingSystem1 (12) | A | 2,500 ms | 2.5° | 3,000 m | 5,000 m |
| SensorTargetingSystem2 (13) | B | 2,000 ms | 5° | 4,000 m | 6,000 m |
| SensorTargetingSystem3 (14) | C | 1,500 ms | 6° | 5,000 m | 7,000 m |

`0x458c12` returns the LockTime: 0 with no sensor or while the ship is cloaked. The lock update
`0x458cf8` runs every frame **for the local player only** (`0x45a646`), on power block +0x20
(−1 = none):

* A target counts as on target when it is within range and the angle between the nose and the
  target is within the cone. For rt types 2, 4 and 5, a line of sight along the nose to that range
  must hit the target instead.
* On target, the timer grows by the frame ms. Off target, it shrinks by the frame ms and becomes −1
  below 0, so a lock decays as fast as it builds.
* Once the timer passes LockTime, the cone doubles (`0x459064`).
* A guided shot with timer < LockTime launches **unguided** (`0x4ff972`).
* Cloaked ships cannot fire missiles.
* The AI keeps its own lock timer (§5.1).

**Launchers** (`0x4ff972`). There are 5 mounts at ship +0x50 + 16·i: rounds (word) +0x50, next fire
(ms) +0x58, launcher type +0x5c. A mount fires only when its HP / launcher `HitPoints` > 0.25 and it
has rounds left. The next shot is `1000 / RateOfFire` ms later (**shots per second**), doubled at
≤ 50 % mount health. Mounts of the same type alternate, and the next one is ready after
`500 / RateOfFire` ms. `Rounds` fills +0x50 (derived).

**Units and unread keys.** `Duration` is in seconds (expiry `now + Duration × 1000`, `0x4fe74d`).
PassiveCloaking (8) never expires. An expired missile is removed by `0x457855` without a blast,
so it is a dud (derived). **The missile `Range` and `Penalty` keys are never read** (loader block
`0x5eea3c–0x5eec6a`). Missile `HitPoints` is clamped to [10, 500] (`0x4fe77b`); its final use is
unknown.

**Guidance** (base class `0x5952c3`, per frame `0x596227` → `0x595721`). Agent fields: Speed +0x2c,
mode +0x28 (0 straight, 1 homing, 2 aim point), throttle +0x38 (starts at 1.0), turn limit
+0x3c = 2.5.

* For the first 250 ms it flies straight: accel = nose × Speed.
* Homing aim point: the target's position when d < 100 m (always for Permeator). Otherwise **lead =
  target position + v̂ × 200 × d / Speed**, a fixed 200 m/s rather than the target's real speed.
  Targets slower than 50 m/s get no lead.
* Turn (`0x595520`): each axis `clamp(error°, ±2.5) × t` rad, so **at most 2.5 rad/s**, plus
  roll = 0.3 × yaw (none for Sidestep).
* Throttle, with e = max(|yaw|, |pitch|) error in degrees:
  * Off target (e > 5°, d < 7,500 m and throttle > 0.2): −0.1 per frame when e > 15°, −0.05 when
    e > 10°, −0.025 when e > 5°.
  * On target (e ≤ 2° and throttle ≤ 0.9): +0.05 per frame.
* Commanded acceleration is `max(throttle × Speed, 250)` along the nose.
* Whether xcgen drag applies to missiles is unknown: the no-drag bit is not set at launch. If drag
  applies, the speed settles at half the command. Permeator writes its velocity directly.
* Lock loss: when |yaw| and |pitch| both exceed 170°, the lock is lost, except for Enveloping (6)
  and PassiveCloaking (8).
* Cloaked target: every 2–3 s the missile keeps the lock with chance `50 + 2d` % when the
  launcher's side code (rt +0x2c) is 9, otherwise `50 − 5d` %. This is the only difficulty term in
  missiles.
* Sidestep (7) adds a lateral 2.5 m/s² per degree of error beyond ±10°.
* The target gets rt flag 0x8000, an incoming-missile warning (derived).

**Fuse and blast** (`0x455dfa`). A collision (`0x4574cc`, rt types 10–20) counts as d = 1 for the
struck object. A missile also detonates when it is within 25 m and the error exceeds 45° (passing
the target). Every object at d < 15 m takes `ImpactDamage`, and at 15–200 m `ProximityDamage`. Each
hit is × 75–100 %, with no distance falloff. rt types 3 and 0x20 are skipped for proximity, and no
side filter was seen (derived).

**Permeator (2)** (`0x451d75`) and **Enveloping (6)** (`0x45261b`) against ships:

* Permeator: half the damage runs the struck side's normal chain (shield, skipped while cloaked →
  armour → hull). The other half **skips the shield** and goes armour → hull.
* Enveloping: half the damage runs the **fore** chain and half the **aft** chain, whatever side was
  hit. Its per-frame routine `0x596955` only adds particles.
* Hull step in both: below 75 % hull, 30 % of the hull damage goes to systems (`0x44ddee`), and
  the **pilot always loses 1–3 health** (`0x4516ae`, §4.2). There is no `min(damage, 100)` %
  roll, and no shield factor m.
* Non-ship targets use `0x45481f`.

Missile agent classes (factory `0x5a425d`): base for 0–2, 5 and 7; Doppelganger `0x596320` (per
frame `0x5965b0`); Nova `0x596af0` (`0x596dea`); Enveloping `0x596796`; PassiveCloaking `0x597a37`
(`0x597b9b`); Parallax `0x597386` (`0x5974e4`). The Doppelganger, Nova, PassiveCloaking and
Parallax per-frame routines are not traced.

## 6. Strategy clock (verified)

`GameClock::SetSpeed(n)` = `0x411f74` (n = 0–5). Game time (seconds) advances by
`elapsed_ms / 1000 × rate[n]`, with `rate` at 0x66f6f0 = **5, 60, 300, 1800, 3600, 86400** game-seconds
per real second (5 s, 1 min, 5 min, 30 min, 1 h, 1 day per second). The dial has no pause hole: the
slowest speed is 5× real time, and the "OK (5 SECS)" / "Return to base (5 secs)" buttons select it.
Modal dialogs pause and resume the clock with `0x4120b7` / `0x41207f`.

## 7. Still open

* AI: what the pilot "temperament" field (pilot record +0xa0) means. Whether alien ships carry
  pilot records. Whether the AI missile-lock comparison (§5.1 step 6) is really inverted. The other
  agent states (`0x585828`), formation speed matching (`0x5a5be0`), and the non-fighter agents
  (freighters, AlienType5–6, Patton).
* Missiles: whether xcgen drag applies to missile objects. The Doppelganger, Nova, PassiveCloaking
  and Parallax per-frame routines. The final use of missile `HitPoints` beyond point defence.
* Systems: what damaged sensors and hyperdrives do. The engine-off threshold in `0x44d874`. The
  meaning of rt flag 0x8 in repair and pilot death.
* Station defence modules (`*_DEFENSE_MODULE_*`).
