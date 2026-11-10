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

### Power

One power supply feeds **beam weapons** and **shields**. Each has a recharge setting with five
steps (A–E) cycled with F9 / F10; `;` and `'` move power between them; `S` cycles shield balance
(equal / full forward / full aft). Tractor, cloak and ECM also draw power; beams cannot fire
while the tractor is on. (GameSettings.ini `BeamRechargeRate`, `ShieldRechargeRate`; strings)

### Flight model

Per hull, `GameSettings.ini` gives roll, yaw and pitch rates, an angular damping value
("0 tight – 10 loose"), a **LockSpeed** ("speed at which pitch and yaw rates are zero") and a
**MaxSpeed**. Turn rate therefore falls as speed rises: the central handling idea. Speeds are in
metres per second (weapon speeds are commented in m/s). The afterburner has `Boost` and `Cost`
values whose units are not yet confirmed (derived: +150 m/s for 50 energy per second).

**VCT** (Vector Control Thrusters) is fitted equipment that lets a ship slide sideways and
vertically. Alien ships have it, which is why they out-turn missiles.

### Missile lock

Three sensor settings give a lock time and a lock cone: A 2.5 s, B 2.0 s, C 1.5 s, with widening
cones. They correspond (derived) to the three targeting systems: Standard, Emissions and AATS.
Some missiles need a minimum targeting system.

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

## Multiplayer

Dogfight arenas (deep space, asteroids, black hole, around each station type) and team modes
(`MPSettings.INI`, `NetSettings.ini`). Deferred: see [deviations](../deviations/index.md).
