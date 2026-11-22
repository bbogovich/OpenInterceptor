# Gameplay

## The space simulator

Each battle is one 3D arena. The player flies one ship; the other X-COM ships are AI wingmen.
There are **no in-mission navigation points**: travel between systems happens on the strategy
map, and the player leaves a battle by hyperspace (`H`, with an abortable countdown).
(derived from the bindable action list and the walkthrough)

### Damage model

Three layers: **shields** (fore and aft facings), then **armour**, then **hull**. The cockpit
reports per-system damage and repair (shield generator, manoeuvring, sensors, engines,
hyperdrive, tractor, cloak, VCT, damage control, ECM). (derived from cockpit strings)

**From the program (verified, app 0.21.0 notes).** Read from the player's own `Interceptor.exe`
as data; facts and numbers only.

- **A beam hit** does `Damage − rand(Damage / 4 + 1)`: 75 – 100 % of `Damage`. On Beginner and
  Experienced the player's own hits are × 1.15; on Beginner hits **on** the player are × 0.85.
  A missile hit does `ImpactDamage` (direct) or `ProximityDamage`, each × 75 – 100 %, with no
  difficulty factor.
- **The chain**, on the side that was hit (fore or aft): the **shield bank** of that side stops the
  hit if it holds `damage × m` (it loses that much), otherwise it stops `bank / m` and falls to 0;
  `m` is 1.5 for the Phase Cannon and 1 for everything else. Then the **armour bank of that side**:
  armour is per side, and each bank absorbs what it holds. Then the **hull**: below 75 % of the
  hull's `HitPoints`, 30 % of what reaches it goes to the ship's systems instead; the ship is
  destroyed when the hull cannot take the rest.
- **Bank maxima.** A shield bank holds the shield's `Special` × f and an armour bank the armour's
  `HitPoints` × g, both banks full at the start:

    | Difficulty | f, X-COM | f, others | g, X-COM | g, others |
    |---|---|---|---|---|
    | Beginner | 1.50 | 0.40 | 1.30 | 0.50 |
    | Experienced | 1.25 | 0.52 | 1.15 | 0.60 |
    | Veteran | 1.00 | 0.64 | 1.00 | 0.70 |
    | Genius | 1.00 | 0.76 | 1.00 | 0.80 |
    | Superhuman | 1.00 | 0.88 | 1.00 | 0.90 |

    The Patton class holds three times its shields and four times its armour.

- **System damage.** The 30 % share makes up to five tries at a random system (beam mounts and
  launchers most often, then shield generator, flight control, sensor, engine, hyperdrive, a
  special system; 2 % of tries hit nothing). Beam mounts fire at full rate above 50 % of their
  `HitPoints`, at half rate at 25 – 50 % and not at all below; shield recharge is halved below
  50 % and stops below 25 %; engine thrust is × 0.5 below 50 %, × 0.25 below 25 % and nothing
  below 15 %.
- **The pilot.** A hull hit injures the pilot (1 – 3 health) with a chance of `min(damage, 100)` %;
  Permeator and Enveloping hits always do. A dead pilot's ship tumbles out of control.
- **Repair** runs once a second: 2 points, or 4 with a damage control computer, to the first item
  that needs it in a fixed order (engine, hull, shield generator, flight control, hyperdrive,
  sensor, then beam mounts, launchers and special systems). A destroyed system is never
  repaired. The INI `RepairRate` keys are never read.
- **Special weapons.** The Heavy Laser and Plasma Cannon also knock the target back. The Tachyon
  Pulser's damage, after the shield, bypasses armour and hull and goes to systems (mostly the
  engine). The Psi Blaster does no damage: it can take control of, panic or rout the enemy pilot
  for 30 – 120 s. The fourth shield type (accumulator) turns a hit on a raised bank into a quarter
  of its damage added to the bank. A Permeator sends half its damage past the shield; an
  Enveloping torpedo splits it between the fore and aft chains.
- **Point defence** is not a turret: a beam bolt that strikes a missile applies the beam's
  `Damage` with a 90 % chance; a missile with fewer hit points (10 – 500) is destroyed, otherwise
  it loses that much.

**In OpenInterceptor (Stage 2, implemented; the original's pipeline since app 0.21.0):** the
`interceptor` campaign uses the original's spread, per-side armour, Phase Cannon factor, 30 % of
hull damage to systems below 75 % hull, and the difficulty factors ([Rulesets › How a hit
lands](../../modding/rulesets.md#how-a-hit-lands)). Every fitted system has hit points; systems
work at full, half or no effect, can be destroyed for the battle, and are repaired worst first
(faster with a damage control computer). Engines and manoeuvring both out leaves a ship
*disabled*. Not yet modelled: the per-system thresholds (DEV-064), the pilot (DEV-065),
accumulator shields (DEV-066), knock-back (DEV-067), mount damage (DEV-068) and the original's
repair order (DEV-029). Behaviour: [Simulation › systems](../../implementation/architecture/sim.md).

### Power

One power supply feeds **beam weapons** and **shields**. Each has a recharge setting with five
steps (A–E) cycled with F9 / F10; `;` and `'` move power between them; `S` cycles shield balance
(equal / full forward / full aft). Tractor, cloak and ECM also draw power; beams cannot fire
while the tractor is on. (GameSettings.ini `BeamRechargeRate`, `ShieldRechargeRate`; strings)

**From the program (verified, app 0.21.0 notes).** Each ship has one **energy pool** of 1000
points, full at the start. Once a second it gains `BeamRechargeRate` for the beam setting (0, 7,
10, 15, 22) and the shields gain `ShieldRechargeRate` for the shield setting (0, 1, 2, 3, 5),
shared by the balance: fore first and the overflow aft, half each, or aft first; each bank is
capped at its maximum. Every beam shot costs `EnergyCost`; `RateOfFire` is shots per second per
mount. The afterburner costs 100 a second (the INI's `Afterburners` keys are only sent to
multiplayer peers). A cloaked ship loses 50 a second instead of recharging its shields, and the
cloak drops at 0. There is no step budget: ships start at C / C (50 / 50 %, throttle 50 %), and
the settings and systems cost engine power instead (below).

**In OpenInterceptor (Stage 2, implemented):** all of the above, with the original's keys
([Controls](controls.md)). Since app 0.21.0 the `interceptor` campaign has the original's pool,
start levels and balance routing; the budget is set out of reach, and the cloak's drain is not
modelled (DEV-024, DEV-072). The tractor
beam tows a target and a disabled ship held at the moment of a hyperjump is captured; the cloak
hides the ship from enemy sensors beyond its range and is dropped by firing (DEV-025); ECM and
the cloak make homing missiles lose their target; the hyperdrive counts down and can be aborted
(DEV-026). Wingmen obey all five orders of the comms menu plus a status report.

### Flight model

Per hull, `GameSettings.ini` gives roll, yaw and pitch rates, an angular damping value
("0 tight – 10 loose"), a **LockSpeed** ("speed at which pitch and yaw rates are zero") and a
**MaxSpeed**. Turn rate therefore falls as speed rises: the central handling idea. Speeds are in
metres per second (weapon speeds are commented in m/s). The afterburner's `Boost` and `Cost`
values are read only for multiplayer (verified).

**From the program (verified, app 0.21.0 notes).**

- **Turning (the player).** Per axis the stick asks for `u = Rate × stick × 0.08`, pitch and yaw
  times `1 − speed / LockSpeed` (roll is not slowed). The turn rate approaches `u / k` with time
  constant `1 / k`, `k = 4 × (1 − 0.1 × AngularDamp) × q`, where `q` is the flight control's
  health / 40 (1 undamaged). Angles are radians. A Lightning (Roll 200, Pitch and Yaw 100,
  AngularDamp 2) rolls at 5 rad/s (286°/s) and pitches at 2.5 rad/s (143°/s) at rest, τ 0.31 s.
- **Thrust (the player).** Engine power is `1 − (beam% + shield%) / 300 − draw%` (2/3 at the
  start); the push is `EngineType Special (kN) × 1000 × power × throttle% / mass`, half in
  reverse; the afterburner adds `Special × 50 / mass`. Mass is the hull's base (6,700 kg for
  IntType1 up to 250,000 for the Patton), 250 kg with a shield, each missile carried (85 kg for an
  X-Winder … 11,000 for a Nova Bomb) and the special systems (damage control 100, VCT 450, jump
  drive 350, cloak 550, ECM 200, tractor 300).
- **Motion.** Every object obeys `dv/dt = a − 2v`: it settles at `a / 2` with a 0.5 s time
  constant and coasts to rest. The player's speed is then scaled back to `MaxSpeed`. A Lightning
  as fitted (75 kN, about 8,300 kg) settles at about 150 m/s at 50 % throttle, 300 m/s at full,
  450 m/s with beams and shields at A.
- **AI ships** ignore the hull's turn and speed values. Each class turns at fixed rates from a
  table (about 1.3 – 3.75 rad/s for fighters, 0.3 – 1.0 for freighters, 0.15 – 0.3 for the
  Patton) and drives with a factor S, `a = S × throttle`, so its top speed is S / 2: 250 m/s for
  every fighter class, 175 – 250 m/s for the lighter alien saucers (rising with difficulty),
  50 – 87 m/s for freighters. Mass, engines, power settings and `MaxSpeed` play no part.

**VCT** (Vector Control Thrusters) is fitted equipment that lets a ship slide sideways and
vertically. Alien ships have it, which is why they out-turn missiles.

### Missile lock

Three sensor settings give a lock time and a lock cone: A 2.5 s, B 2.0 s, C 1.5 s, with widening
cones. They correspond (derived) to the three targeting systems: Standard, Emissions and AATS.
Some missiles need a minimum targeting system.

**From the program (verified, app 0.21.0 notes).** `LockAngle` is in millidegrees: the cones are
2.5°, 5° and 6° (half-angles), with ranges of 3, 4 and 5 km against ships. The lock builds by
frame time while the target is in the cone and decays as fast outside it; once locked the cone
doubles; a shot fired before the lock flies unguided. Missile `Duration` is in seconds; `Range`
and `Penalty` are never read. Guided missiles turn at most 2.5 rad/s and lead a target as if it
flew 200 m/s.

**More from the program (verified, app 0.22.0 notes).** The `interceptor` campaign adopts the
lock and the guidance turn and lead in app 0.22.0
([Rulesets › Missile lock](../../modding/rulesets.md#missile-lock)).

- **Lock.** Against stations and other non-ship objects the three sensors reach 5, 6 and 7 km.
  With no sensor, or while cloaked, the lock time is 0, and a cloaked ship cannot fire missiles.
  The lock timer runs for the player's ship only; AI ships keep their own (below).
- **Launchers.** A launcher fires `RateOfFire` rounds per second, at half that rate at or below
  50 % of its hit points and not at all at or below 25 %. Launchers of one type alternate.
- **Guidance.** A missile flies straight for its first 250 ms. Within 100 m it aims at the
  target itself (the Permeator always does); targets slower than 50 m/s get no lead. Each axis
  turns by at most 2.5 rad/s, with roll at 0.3 × yaw (none for the Sidestep). More than 5° off
  target (within 7.5 km) the missile throttles back, by more the further off it is; within 2°
  it throttles up again. The lock is lost when the target is more than 170° off in both yaw and
  pitch, except for the Enveloping and Passive Cloaking torpedoes. A cloaked target keeps a
  missile's lock every 2 – 3 s with a chance of 50 + 2 d % for one side's launchers and
  50 − 5 d % for the others (d the difficulty 0 – 4) — the only difficulty term in missiles. The
  Sidestep adds a sideways push of 2.5 m/s² per degree of error beyond 10°.
- **Fuse and blast.** A missile detonates on a collision, or within 25 m of its target when more
  than 45° off it (passing). Every object within 15 m takes `ImpactDamage` and every object
  within 15 – 200 m `ProximityDamage`, each × 75 – 100 %, with no fall-off with distance. An
  expired missile disappears without a blast.
- **Missile hit points** are `HitPoints` clamped to 10 – 500; beam bolts can shoot missiles down
  (see point defence above).

### AI fire control

**From the program (verified, app 0.22.0 notes).** Every fighter class (the program's interceptor, pirate and
corporate hulls, alien classes 1 – 4 and the drone) runs one attack routine; freighters, alien
classes 5 and 6 and the Patton do not. With d the difficulty 0 (Beginner) – 4 (Superhuman):

- **Aim.** Beyond 1,750 m the pilot flies straight at the target. Within it the pilot leads:
  the aim point is the target's position plus its velocity × the bolt's flight time
  (distance / the beam's `Speed`), and half the time that time is scaled by `A` and half the time
  by `2 − A`, where `A` is the pilot's accuracy: the pilot under- or over-leads by `1 − A`. Below
  50 %, a failed roll against `A` beyond 200 m also aims 15 – 25 m to one side, along the target's
  x or y axis. On a failed roll the ship also turns in a sluggish mode.
- **Accuracy.** A ship without a pilot uses 50. From Veteran up the pilot's Accuracy is used as
  it is; on Beginner and Experienced it is clamped to 50 – 100 for X-COM pilots and to
  25 + 10 d – 50 + 10 d for the others. Whether alien ships carry pilot records is not confirmed
  (derived: they do not, so they use 50).
- **Beam fire** when yaw and pitch are both within `2.5 − 0.25 (4 − d)` degrees (1.5° on
  Beginner … 2.5° on Superhuman), the target is within `2500 − 125 (4 − d)` m (2,000 … 2,500), a
  line of sight along the nose is clear, and the trigger interval has passed. The interval is 0
  for every class except the drone (1.5 s + 50 ms × (4 − d)) and alien classes 1 and 2
  (0.5 s + 60 ms × (4 − d); in `interceptor` the Wraith and Spectre, a derived mapping). The guns' own charge, energy cost and rate of fire still apply.
  Within 5° of yaw, a roll on the mean of Intelligence and Accuracy fires all mounts of the
  selected type together.
- **Missiles** are tried first, in the same cone: within half the beam range, at least 5 s after
  the last one, with a `5 + 2 d` % chance each frame, while the AI's own lock timer (started when
  yaw and pitch are within ±5°, reset when either leaves) has not yet passed the sensor's lock
  time — as read, which may be an inversion in the original.
- **Breaking off** under 250 m.
- **Weapon choice**, at most every 3 s: within 1,500 m the Gatling Laser or Phase Cannon is
  preferred, beyond it the Heavy Laser or Phase Cannon; the chance follows 0.75 × Intelligence +
  0.25 × Flying (25 without a pilot). An empty missile slot is replaced by the first launcher with
  rounds left that is not a Nova Bomb. Alien classes 4 – 6 keep their first choice.

## Ships

### Player hulls

| Hull | Section | Mounts (beam / missile / special) | Character |
|---|---|---|---|
| X-1A Lightning II | `Ship_IntType1` | 2 / 2 / 2 | Fastest turning, fragile |
| S-2 Firestar | `Ship_IntType2` | 4 / 3 / 4 | Middle weight |
| M-3 Super Avenger | `Ship_IntType3` | 4 / 5 / 6 | Heavy; only hull for the Nova Bomb |

### Alien ships

| Ship | Role |
|---|---|
| Wraith | Light saucer fighter |
| Spectre | Multirole saucer |
| Phantasm | Heavy beam saucer; leads assaults |
| Shade | Elite non-saucer fighter: accumulator shields, VCT, cloak, ECM |
| Harbinger | Tug; tows cargo, tries to capture disabled ships |
| Spirit | Freighter; launches drones |
| Drone | Unmanned, fast, unarmoured |
| Probes | Sensor, intel jammer, comm jammer |
| Mines | Explosive, beam, missile |

### Others

Pirates (the Kabron) and corporations fly the same three human hulls in their own colours, plus
freighters. The **Patton-class research cruiser** is an unarmed former warship.

## Stations

X-COM bases are **modular** (hub plus hangar, personnel, medical, training, cargo,
communications, downlinks, sensors, shield generators, ECM, tractor, probe launch, damage
control, and beam/missile defence modules). Also: corporate outposts, Ore Processing Plants
(OPPs), alien bases (a hub with pods; destroy the hub to destroy the base), alien outposts and
mining platforms, pirate bases, the arms dealer, the science station, the Nova science station,
the alien Doomsday weapon and the portal.

## Weapons

**Beams**: Laser Cannon, Heavy Laser, Gatling Laser, Tracer Cannon (continuous), Tracking
Tracer, Phase Cannon (anti-shield), and alien Plasma Cannon, Tachyon Pulser (shields and engines
only, for capture) and Psi Blaster. Beams can be linked, fired fully, or unlinked (`X`).

**Missiles**: X-winder (basic heat seeker), Elerium Torpedo, Permeator (bypasses shields),
Sidestep (own thrusters), Enveloping Fusion Torpedo (damage spread over the surface), Parallax
Torpedo (two decoys), Passive Cloaking Torpedo (dropped; waits cloaked), Doppelganger (a
decoy copy of your ship), Fusion Missile (anti-station) and the Nova Bomb.

**Defensive equipment**: ECM pods, cloaking device, tractor beam (0.25 km), damage control
computer.

## Missions

Defend outpost / OPP / base, defend freighter convoys, defend probe or fleet, attack alien fleet,
attack freighter or tug convoys, attack alien OPP / outpost / base / probe, attack the portal
fleet, patrol, and the two-part final mission. Side missions: rogue freighter, pirate raiders,
defend a pirate base, the hijacked Nova Bomb prototype, the arms dealer, survey a system. After a
battle: debrief and score (`MissionScores`). Disabled ships can be tractored home for salvage.

## Strategy layer

A star map of the Frontier with time compression; bases and modules; research done on Earth and
sent via **downlinks**; manufacturing and purchase; pilots; probes; money from corporate
protection payments and salvage; an alien strategic AI with its own budget split between
research, goods and the Doomsday project (`Think.ini`). The tech tree is the `R_*` sections of
`GameSettings.ini`.

**The clock (verified, app 0.21.0 notes).** The time dial has six speeds — 5 s, 1 min, 5 min,
30 min, 1 h and 1 day of game time per real second — and **no pause**: the slowest is five times
real time, and the "(5 secs)" buttons select it. Modal dialogs pause the clock while they are
open. The `interceptor` campaign's `clock:` reproduces the dial
([Strategy layer › `clock`](../../modding/strategy.md#clock)).

## Multiplayer

Dogfight arenas (deep space, asteroids, black hole, around each station type) and team modes
(`MPSettings.INI`, `NetSettings.ini`). Deferred: see [deviations](../../implementation/deviations/index.md).
