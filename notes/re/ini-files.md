# The original's INI files: an inventory (2026-11-19)

This report lists every `.ini` file on the X-COM: Interceptor CD copy (`resources/xcom-cd/`; `resources/xcom/`
is empty on this machine). For each file it says what the file holds, how it's structured, where it applies
in OpenInterceptor, and what the engine already reads.

**Facts only.** It gives file, section and key names, value ranges, and the unit notes the authors left in
comments, paraphrased. It doesn't reproduce any file. Confidence marks: **verified** = read from the file;
**derived** = inferred from names, comments or context; **unknown** = still open.

Method: a stdlib-Python scan that groups sections by key signature. The scratch script is kept outside the
repo. The "engine use" column comes from a grep of `packs/`, `resources/default/` and `src/`.

## Summary

| File | Size | Sections | What it is | Relevant to | Engine reads it? |
|---|---|---|---|---|---|
| `Game/ProgramFiles/GameSettings.ini` | 60 KB, 4188 lines | 259 | **The game rules**: economy, weapons, missiles, mines, research, equipment, station modules, hulls, rank, recharge, sensor and lock tables, salvage, mission scores, station hit-point charts | flight, combat, strategy | **Yes**: 187 of 259 sections by `inherit: original:…`; 72 unread (see §1.12) |
| `Game/ProgramFiles/Think.ini` | 6 KB | 34 | **Alien strategic AI**: starting aggression, financial split, Doomsday/portal defences, base-module build priorities | strategy (rival AI, #39) | No |
| `Game/ProgramFiles/soundfx.ini` | 10 KB | 1 | **Sound cue table**: 165 named cues → WAV paths | audio, interface, flight, comms | **Yes**: `audio.cue_table: original:soundfx.ini/InterceptorSounds` |
| `Game/ProgramFiles/MPSettings.INI` | 6 KB | 18 | Multiplayer **arena presets** (dogfight types, base/outpost/OPP placement, hazards) | testbed scenarios (reference only) | No |
| `Game/ProgramFiles/NetSettings.ini` | 8 KB | 28 | Multiplayer **networking**: ports, heartbeat timing, player defaults, chat macros, dead-reckoning thresholds, plus a second copy of the game-type presets | none (multiplayer is out of scope) | No |
| `Game/SETUP.INI` | 75 B | 1 | Installer start-up options | none | No |
| `Game/ProgramFiles/EReg/eregreg2.ini` | 170 B | 1 | Electronic-registration form template (empty demographic fields) | none | No; never read |
| `Game/IE4_0Setup/English/ie4setup.ini` | 686 B | 3 | Internet Explorer 4 redistributable installer | none | No |
| `Game/DirectX/Drivers/usa/{5631,8811,8814}_{dig,hwc,swc}.ini` (9 files) | 33–103 KB | 2–6 | S3 video-capture driver parameter tables, part of the DirectX 5 redistributable | none | No |

Only the first five files concern the game. GameSettings.ini matters most by far; Think.ini is the most
valuable file that isn't read yet.

---

## 1. GameSettings.ini — the rules

The format is plain Windows INI: `;` comments, also after values, and some values left empty. The file is dated
14 May 1998 in its header comment. The file states two scaling rules (**verified**):

- `Penalty` values are stored ×1000 (1000 = 1.0).
- Equipment `ProdTime` is stored ÷1000 (real production = `ProdTime × 1000`); module `Cost` is in
  thousands; module `Maintenance` is **not** scaled; hull `MaintCost` is in thousands.

Each subsection below gives the section group, its keys, the content, and where it applies.

### 1.1 Difficulty economics — `Beginner`, `Experienced`, `Veteran`, `Genius`, `Superhuman` (5)

Keys: `StartingTreasury`, `ExchangeRate`. Starting money ranges from 5,000,000 (Beginner) to 3,000,000
(Superhuman); the exchange rate from 250 to 200 (meaning unknown, probably the sell-price conversion).
**Applies to:** campaign start. **Engine:** `StartingTreasury` is read through
`start.money_from: original:GameSettings.ini/StartingTreasury`, which picks the section for the level
being played. `ExchangeRate` is **unused**.

### 1.2 Beam weapons — 9 sections

Sections: `LaserCannon`, `HeavyLaser`, `GatlingLaser`, `PhaseCannon`, `PlasmaCannon`, `PsiBlaster`,
`TachyonPulser`, `TracerCannon`, `TrackingTracerCannon`.

Keys: `Side, WeaponClass, Cost, ProdTime, Size, Range, Damage, EnergyCost, HitPoints, RepairRate,
RateOfFire, Speed, RechargeRate, Materials, SellPrice, TechRequired, TechRequired2, TechItem`. Gatling
and Heavy laser also have `Penalty`.

Values (e.g. LaserCannon: Range 4, Damage 30, EnergyCost 10, HitPoints 20, RateOfFire 2, Speed 2000). The
file's own comment says `Speed` is in m/s.

**Applies to:** flight combat and the strategy layer (purchase, manufacture, research gates).

**Engine:** seven are flyable beams (`rules/weapons.yaml`). `Range` is read as km (DEV-008). `Damage`,
`RateOfFire` (shots/s), `EnergyCost` and `Speed` are used.

- **Not used:** `RechargeRate`, `HitPoints` (weapon component damage), `RepairRate`, `WeaponClass`, `Penalty`.
- **Gap:** `PsiBlaster` and `TrackingTracerCannon` exist only as strategy items
  (`strategy/references.yaml`), not as flyable beams. Their sound cues `Beam7` and `Beam9` are unused.

### 1.3 Missile launchers — 10 sections (`*Launcher`)

Keys: `AmmoType, Rounds, RateOfFire, HitPoints, RepairRate` plus economy keys. `RateOfFire` is −1 for the
single-shot Doppelganger and Nova launchers.

**Applies to:** loadouts and hardpoints (Equip Ships). **Engine:** inherited by
`strategy/references.yaml` and the missile rules. Missile `rounds` per craft is partly derived (DEV-050).

### 1.4 Missiles — 10 sections

Sections: `XWinderMissile, PermeatorMissile, SidestepMissile, FusionMissile, EleriumTorpedo,
EnvelopingFusionTorpedo, ParallaxTorpedo, PassiveCloakingTorpedo, DoppelgangerMissile, NovaBomb`.

Keys: `Speed, Range, Duration, ProximityDamage, ImpactDamage, HitPoints, RepairRate, Penalty, WeaponClass`
plus economy keys. Doppelganger has no `Range`, `Penalty` or `RepairRate`.

- The file marks NovaBomb's `Duration` (150) as ignored.
- Missile `Penalty` values are −84, −125, −167 and −1000, i.e. −0.084…−1.0 after scaling. Probably a
  handling or lock penalty (**unknown**).

**Engine:** speed, duration and both damages are used. Turn rate and lock time are engine values (DEV-009).
`Range`, `Penalty` and `HitPoints` are unused.

### 1.5 Mines — `BeamMine`, `ExplosiveMine`, `MissileMine` (3)

Keys: `Damage, HitPoints, Range` plus economy keys. Damage and Range are 1/0 placeholders on some
(**derived**: mine behaviour is in code). **Engine:** inherited as strategy items.

### 1.6 Research — `R_*` (71)

Keys: `Level, FileSize, Prereq1..3, HardwareYield1..2, ModuleYield1..2, TechYield1..8`.

- `Prereq*` are ids from a **technology list in the file's comments**: 0–100, plus 127 = "special",
  used for discovery-gated topics such as the saucer examinations.
- The id list also names topics that have **no `R_` section**: 74–100, alien freighter, battlecruiser,
  outpost, mining platform and alien module types. That suggests the exe knows research or encyclopaedia
  entries beyond the INI.
- `FileSize` is the download size (research cost).
- All yield keys are 0 in this file. The exe hard-codes research yields (vault: data-vs-hard-coded
  inventory).

**Applies to:** research tree, encyclopaedia. **Engine:** all 71 inherited. Research pace is still an open
question (#32).

### 1.7 Ship equipment — 23 sections

| Group | Sections | Extra keys | Notes |
|---|---|---|---|
| Armour | `ArmorType1..3` | `Special` | protection = `HitPoints` (250/375/425), derived (DEV-056) |
| Shields | `ShieldType1..4` | `Special` | protection = `Special` (300/400/500/350); `ShieldType4` is `ALIEN_EXCLUSIVE` with no cost |
| Engines | `EngineType1..3` | `Special` | **file comment: `Special` is thrust in kN** (75 / 175 / 300; commented-out older values 100/200). This is the data source for acceleration, which DEV-011 calls an engine value |
| Thrusters | `VectorControlThrusters` | `Special` 30 (kN) | VCT |
| ECM | `ECMPodType1..2` | `Special`, `Range` 250… | |
| Sensors | `SensorTargetingSystem1..3` | — | tie in with `SensorsSettingA..C` (§1.10), **derived** |
| Specials | `CloakingDevice, DimensionJumpDrive, ShipDamageControlComputer, TractorBeam` | — | |
| Probes | `SensorProbe, IntelJammerProbe, HunterProbe` | — | strategy-map probes |

All of them share `EnergyCost, RechargeRate, HitPoints, RepairRate, Range, Penalty` plus the economy keys.
Most specials carry placeholder 1s.

**Engine:** inherited by `strategy/references.yaml` and `rules/systems.yaml`. `HitPoints` is used for system
condition (DEV-029). Engine thrust `Special` is **not** used for acceleration.

### 1.8 Station modules — `M_XCOM*` (35), `M_ALIEN*` (33)

Keys: `Side, Cost` (thousands), `ProdTime, SizeClass` (0 small, 1 medium, 2 large, 3 special, from the file's
comment), `Maintenance, HitPoints, TechRequired, TechRequired2`. Missile-defence modules add `Rounds`.

Types: central hub, auxiliary hub (alien), hangar, cargo, personnel, medical, training, communications,
downlink 1–3, sensor 1–3, ECM 1–2, shield generator 1–3, tractor, probe launch, advanced damage control,
beam defence ×8 weapon types, missile defence ×6 missile types.

**Applies to:** base building (strategy) and station combat (module hit points).

**Engine:** all `M_XCOM*` are used except `M_XCOMCentralHub`. Only 2 of the 33 `M_ALIEN*` are used
(`BeamDefense_PlasmaPulse`, `ShieldGeneratorType2`). The rest are needed for alien base and outpost
modules (DEV-035, plan A4).

### 1.9 Hulls — `Ship_*` (20)

| Group | Sections | Keys |
|---|---|---|
| Player interceptors | `Ship_IntType1..3` | economy, `MaxBeamWeapons, MaxMissileWeapons, SpecialSystemMounts, HitPoints, CargoCapacity, Restrictions`, **flight model** `RollRate, YawRate, PitchRate, AngularDamp, LockSpeed, MaxSpeed` |
| Everything else | `Ship_XCOMFreighter, Ship_OutpostFreighter, Ship_AlienType1..6, Ship_AlienDrone, Ship_AlienFreighter, Ship_PirateType1..3, Ship_PirateFreighter, Ship_CorporateType1..3, Ship_PattonClass` | mounts, `HitPoints`, cargo; **no flight model** |

From the comments (**verified**):

- The flight-model keys are "human ships only".
- `AngularDamp` runs from 0 (tight) to 10 (loose).
- `LockSpeed` is the speed at which pitch and yaw rates fall to zero.
- `MaxSpeed` is the hull's top speed.

The non-player hulls having no flight keys **confirms DEV-012**: their flight values are in the exe.

**Engine:** inherited for the X-COM, alien, drone, freighter and Patton classes. **Unused:**
`Ship_OutpostFreighter`, `Ship_PirateType1..3`, `Ship_CorporateType1..3`. The pack's pirate and corporate
classes copy the X-COM hulls instead of reading these sections (check against plan A6).

### 1.10 Real-time tables — 15 sections

| Section(s) | Keys | Content (verified values) | Engine |
|---|---|---|---|
| `RankAdvancement<Level>` ×5 | `Rookie, Ensign, Lieutenant, LtCommander, Commander` | kills (or score) for each rank, e.g. Beginner 0/1/8/20/35, Superhuman 0/4/17/33/45 | **unused** (pilots #28) |
| `BeamRechargeRate` | `A..E` | 0, 7, 10, 15, 22 per beam power setting (commented-out older values exist) | quoted in `systems.yaml` comments only; the engine's own A–E table is used |
| `ShieldRechargeRate` | `A..E` | 0, 1, 2, 3, 5 | as above |
| `SensorA/B/CDetectionProb` | `A..H` | detection % per range band; the comments give the bands from about 1.5 km down to under 0.1 km. Sensor A: 0…90 %, sensor C: 25…100 % | **unused**. A radar / detection model per sensor grade |
| `Afterburners` | `Boost, Cost` | 150, 50 | used (`rules/flight.yaml`) |
| `SensorsSettingA/B/C` | `LockTime, LockAngle` | **comments: LockTime in ms, LockAngle in 1/1000 rad**. A: 2500 ms / 2500; C: 1500 ms / 6000 | **unused**. DEV-013 says the units are undecoded, but the file states them. Read as given, 2500 mrad ≈ 143° is implausibly wide for a cone, so the value is probably a full or half angle in other units. Still **unknown**: confirm in RE (plan A5) |

### 1.11 Salvage, scores, station hit points

| Section(s) | Keys | Content | Engine |
|---|---|---|---|
| `*Goodies` ×13 (Beam, Cloak, DimJump, Ecm, Engine, Freighter, Missile, Sensor, Shield, DamageComp, VCT, AlienMessage, AlienCorpse) | the five difficulty names | **percent** chance (the comment says all goodies are %) of recovering that item class after a battle, falling with difficulty (e.g. Beam 30→7, Freighter 45→15, AlienCorpse 35 at every level) | **unused**. The campaign uses `salvage_per_kill: 20000` (DERIVED). This is the original's salvage table |
| `MissionScores` | `S0..S255` | 256 keys, 99 non-zero, values −5000…+4500 | **unused**. Which index is which event is in code (coverage open item 1) |
| `*HP` ×11 (XCOMBase, XCOMOutpost, XCOMOPP, AlienBase, AlienOutpost, AlienOPP, PirateArms, PirateRaider, PirateBase, Science, NovaScience) | `HP1..HP4, SD` | hit-point chart per station type, e.g. XCOMBase 6000/2500/3000/0, SD 500; ScienceHP 3000/0/0/0, SD 1200 | used, with HP1–HP4 summed (DEV-056). `PirateRaiderHP` is unused. Meaning of the four HPs and `SD` is **unknown** (coverage open item 6, plan A4) |

### 1.12 Sections no pack reads (72)

- **Strategy:** `*Goodies` ×13, `MissionScores`, `RankAdvancement*` ×5.
- **Combat:** `BeamRechargeRate`, `ShieldRechargeRate`, `SensorA/B/CDetectionProb`, `SensorsSettingA/B/C`.
- **Hulls:** `Ship_OutpostFreighter`, `Ship_PirateType1..3`, `Ship_CorporateType1..3`.
- **Stations and modules:** `PirateRaiderHP`, `M_XCOMCentralHub`, 31 `M_ALIEN*`.
- **Difficulty:** the difficulty sections are read only for `StartingTreasury`.

---

## 2. Think.ini — alien strategic AI (dated 29 Oct 1997)

The header comment says the strategic AI loads these settings at start and its agents use them for each part
of the alien frontier.

| Section(s) | Keys | Content | Applies to |
|---|---|---|---|
| `GeneralAI` | `StartingAggression` | 0.7 | rival aggression; the pack uses its own `aggression` (start 1.0, DERIVED, `strategy/story.yaml`) |
| `FinancialAI` | `ResearchDefault, GoodsDefault, DoomsdayDefault` (0.3 / 0.4 / 0.3: budget split), `NumDaysInFinancialPeriod` 15, `MinimumStartingTreasury` 1,000,000, `ResearchAggressionEval` 0.5, `WklyRschSpendingPerDLink` 20000 | how the alien treasury is split between research, goods and the Doomsday project, and how often | rival economy (#39, #46). The vault records that the exe doesn't read `ResearchAggressionEval` |
| `DoomsdayAI` | `MaxPortalDefensesType1..6`, `MaxPortalDefenses`, `…Variance`, and the same six for `MaxDoomsdayDefenses*` | **five comma-separated values per key, one per difficulty** (Beginner→Superhuman): the maximum alien ships of each type (Type1..6 = the six alien classes, **derived**) guarding the portal and Doomsday sites, the total cap (8→16) and ± variance | endgame mission composition (#43) |
| 31 module sections (`HANGAR_MODULE`, `CARGO_MODULE`, `PERSONNEL_MODULE`, `DOWNLINK_MODULE_TYPE_1..3`, `SENSOR_MODULE_TYPE_2..3`, `SHIELD_GENERATOR_MODULE_TYPE_2..3`, `ECM_MODULE_TYPE_1..2`, `BEAM_DEFENSE_MODULE_*` ×7, `MISSILE_DEFENSE_MODULE_*` ×6, `TRACTOR`, `TRAINING`, `PROBE_LAUNCH`, `COMMUNICATIONS`, `CLOAKING`, `ADVANCED_DAMAGE_CONTROL`) | `PriorityA/B, DCountA/B, ReservesA/B` | three comma-separated values (probably early/mid/late game or tech tier, **unknown**) of build priority, desired count and money reserve for two variants A/B (probably base vs outpost, **unknown**). `*` in Reserves = same as before (derived) | rival base growth (#39, #46) |

**Engine: not read at all.** This is the data for the full strategist (#39), the Doomsday defences (#43)
and rival logistics (#46). Next step: an `original:Think.ini/<section>` inherit into the strategy format.
The comma lists need the same reader as `per_level:`.

---

## 3. soundfx.ini — sound cue table

One section, `[InterceptorSounds]`, with 165 entries of the form `CueName = "SoundFX\<folder>\<file>.wav"`.

| Folder | Cues | Examples |
|---|---|---|
| `SpaceFlight` | 64 | `Beam1..9`, `Missile1..6`, hull and shield hits, explosions, engine rumbles (X-COM 1–3, alien 1–3), fly-bys, station ambients, tractor, cloak, afterburner |
| `Gadgets` | 50 | interface: bleeps, clicks, message box open/button, rotary switch, difficulty dial up/down, screen panels (`T4*`, `T5*`, `S9Shutter`), `snd*` UI effects |
| `Chatter` | 42 | `Comp_*` computer voice: alerts, research complete, base under attack, freighter and Nova countdowns, mission accomplished/failed; `News_Announce1..3` |
| `Wrappers` | 9 | main-screen panel and loop sounds |

The comments give the beam cue order: Beam1 laser, 2 heavy laser, 3 gatling, 4 phase, 5 tracer, 6 plasma,
7 psi blaster, 8 tachyon, 9 tracking tracer. That matches `rules/weapons.yaml`.

**Applies to:** flight audio, the computer voice, and the **strategy-screen interface sounds** (plan B1.2:
the `Gadgets`/`Wrappers` cues are what the original screens play). **Engine:** read as
`audio.cue_table`, 29 cues referenced by the packs so far.

---

## 4. MPSettings.INI and NetSettings.ini — multiplayer

**MPSettings.INI** (18 sections) holds the multiplayer game types. Each has 1–3 description lines, a
`Release` flag and `Flags`, and arena contents:

- `AsteroidCnt`;
- `BlackHoleCnt` and a position;
- `BaseCnt` + `BaseType1` + a position;
- `OutpostCnt`/`OutpostType1`;
- `OPPCnt`/`OPPType1`.

The types are deep-space, asteroid-field, black-hole, X-COM/alien/pirate/arms-dealer base, X-COM/alien
outpost, X-COM/alien OPP dogfights, freighter escort, defend OPP, tournaments and a test mission.

**Applies to:** nothing in the campaign. It's useful as a **reference for testbed arena scenarios**: object
placement coordinates and which station types exist in a dogfight arena.

**NetSettings.ini** (28 sections) holds:

- `PortInfo` (broadcast and secure ports);
- `GameLocator` timing;
- `HeartBeatInfo-IPX/TCP/MODEM`;
- `MPPlayerInfo` (lobby defaults: call sign, name, limits on kills, lives and time, teams, repair/rearm);
- `RTChatStrings` (5 chat macros);
- `DRSetting` (28 dead-reckoning thresholds for network prediction: position, velocity, rotation matrix,
  thrust);
- a second copy of the game types keyed by text id (`S_MPGT_CATAGORY0_0..11`) plus `Capture The Flag`.

**Applies to:** none. Multiplayer is out of scope. Not read.

---

## 5. Non-game INI files

- **`SETUP.INI`**: installer start-up (`AppName`, `FreeDiskSpace`, `EnableLangDlg`).
- **`EReg/eregreg2.ini`**: an empty electronic-registration demographics template.
- **`IE4_0Setup/English/ie4setup.ini`**: the bundled Internet Explorer 4 installer.
- **`DirectX/Drivers/usa/*.ini`**: S3 (copyright 1996) video-capture driver parameter tables
  (`DIGITIZER`, `HWCODEC`, `SWCODEC` and their `FIFO_` variants, keyed by display mode), part of the
  DirectX 5 redistributable.

None of these is relevant to the game or to OpenInterceptor. The original-data checks can ignore them.

---

## 6. What this means for the plan

New facts that change open items (to fold into `plan/2026-11-plan-stats-and-strategy-parity.md` and the
Deviations page):

1. **Engine thrust is data** (`EngineType*.Special`, kN; VCT 30 kN). DEV-011 treats acceleration as an
   engine value. With a hull mass (still unknown) acceleration could be derived. At least the *ratio*
   between hulls fitted with engine 1/2/3 is data (plan A6).
2. **Lock settings have stated units** (`SensorsSetting*`: ms and mrad). DEV-013 can cite them; the angle
   value still needs RE to interpret (plan A5).
3. **Radar detection is data** (`Sensor*DetectionProb` by range band) and the engine doesn't model it yet.
4. **Salvage is data** (`*Goodies`, % by difficulty). It replaces `salvage_per_kill: 20000` (DERIVED).
5. **Rank thresholds are data** (`RankAdvancement*`), for pilots (#28 / #103).
6. **The alien AI is data** (`Think.ini`), for #39, #43 and #46. It needs a comma-list-per-difficulty reader.
7. **Gaps in coverage:** Psi Blaster and Tracking Tracer aren't flyable; 31 `M_ALIEN*` modules, the pirate
   and corporate hulls, and `PirateRaiderHP` are unread.
8. **Still unknown:** the station chart's `HP1..HP4`/`SD`, the meanings of `RechargeRate` and `Penalty`,
   the `MissionScores` indices, `ExchangeRate`, and the Think.ini module triplets and A/B. These are for
   exe RE.

The docs page `specification/original-formats/ini.md` currently gives only a one-line summary for each
file. Its GameSettings and "Others" sections should be expanded from §1–§4 when the docs are next updated.
