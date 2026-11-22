# Reverse-engineering notes: the strategy layer's game logic

Facts read from the player's own `Interceptor.exe` (CD release: 3,008,512 bytes, built 28 May 1998) and
`GameSettings.ini` / `Think.ini` (2026-11-22). Formulas, constants, table layouts and addresses only; no
code, text or file content from the originals is reproduced. **Verified** = read from the machine code.
**Derived** = inferred from how the code uses a value (names of fields are always derived). **Unknown** =
not traced. Addresses are for the CD build; patch 1.2 is a full rebuild (see `../online-sources.md` §4).

How it was read: the same Hopper 5 headless whole-binary decompile as `../flight/README.md` (scratch only),
plus `objdump -d` (Apple LLVM, reads the PE as `coff-i386`) for x87 constants and switch tables, plus the
stdlib helpers in `../wrappers/tools/` (`pe.py`) to read tables and string cross-references. The screen
callbacks are in `../wrappers/`; the clock in `../flight/README.md` §6.

Units used throughout: game time is an integer count of **seconds** (`0x412181`); the strategy map is in
**map units, 6,000 per light year** (verified, `0x502c14` divides a base's scan range by 6000.0 to print
light years); money is plain credits.

## 1. The strategy loop (`gamemain.cpp`, `0x552f90`; verified)

Runs every frame while the strategy screens are up. With the game clock running it calls, by real time:

| Every (real ms) | Calls | What (derived) |
|---|---|---|
| frame | queued-event dialogs (types 0xd, 2, 6, 0xc, 0xb via `0x46bdc4`) | pilot out of the medical bay, a ship damaged, people killed, base built, module built. Each is a message box whose second button is "OK (5 secs)"; picking it calls `SetSpeed(0)` (`0x411f74`), the slowest speed |
| 500 | `0x4223a1(dt)`, `0x428e92(dt)`, `0x40f55c(dt)` with `dt` = game seconds since the last call | bases (construction, repair, modules, research), outposts (rebuild queue, repair), pilots (training, healing) |
| 1000 | `0x4b83cc`, `0x406382`, `0x4152c1`; then the month and week checks below | misc. 1 Hz state |
| 5000 | `0x55381b` | alien-message interception (§4.5) |

**Month change** (verified): when the month index `0x41276c(0)` = `12 × (year − 2067) + month` changes,
it runs `0x40befb` (a debug ship census only), **`0x42978e` (outpost evaluation, §2.3)**, **`0x42c1ef`
(monthly accounts, §2.2)**, then the monthly report (`0x4bc8cd`). **Week change**: when
`time / 86400 / 7` changes, the weekly report (`0x4c36b9`). Both are calendar based, not "30 days after
start".

## 2. Economy

### 2.1 Tables (verified)

| What | Address | Layout |
|---|---|---|
| `StartingTreasury` by difficulty | 0x73f770 | 5 × int (defaults in code 5,000,000 … ) |
| `ExchangeRate` by difficulty | 0x740be8 | 5 × int (INI 250/250/225/225/200) |
| Modules `M_*` | 0x73dc38 | stride 0x44, indexed by the **module type** below. ProdTime +0x1c (days, §3.2), Cost +0x20 (× 1000), Side +0x19, SizeClass +0x38, **Maintenance +0x3c (raw; × 1000 when charged)**, HitPoints +0x40 |
| Hulls `Ship_*` | 0x73b830 | as `../flight/README.md` §1; MaintCost +0x3c is stored × 1000 |
| `MissionScores` S0–S255 | 0x71be38 | 256 × int (`0x5f0492`) |

**Module type enum** (verified from the loader `0x5ef513` call order): 0 CentralHub, 1 Hangar, 2 Personnel,
3 Cargo, 4 ProbeLaunch, 5–7 DownlinkType1–3, 8–10 ShieldGeneratorType1–3, 11 Cloaking, 12–14
SensorType1–3, 15–16 ECMType1–2, 17 Training, 18 AdvancedDamageControl, 19–24 MissileDefense (XWinder,
Permeator, EleriumTorp, EnvelopingTorp, SideStep, Parallax), 25–31 BeamDefense (Laser, HeavyLaser,
GatlingLaser, TrackingTracer, Phase, PlasmaPulse, TachyonPulser), 32 Communications, 33 Medical,
34 Tractor; alien modules 35–66 (35 AdvancedDamageControl, 36 Cargo, 37 CentralHub, 38 Cloaking,
39 Communications, 40–42 Downlinks, 43–44 ECM, 45 Hangar, 46–51 MissileDefense, 52–58 BeamDefense,
59 Personnel, 60 ProbeLaunch, 61–62 Sensor2–3, 63–64 Shield2–3, 65 Tractor, 66 Training).

### 2.2 Monthly accounts `0x42c1ef` (verified)

For each of the 8 factions (X-COM is the player byte `0x66f6d4`; the aliens are 9, see §4):

* **Income** = Σ over the faction's outposts that are online (hull HP > 0, `0x427f1c`) **and older than
  0.95 × 28 days** (`0x42560e`: age / 2,419,200 s compared with 0.95) of `0x428b76(outpost)`:
  `round(ExchangeRate[difficulty] × Σ yield(OPP) × funding)`, over the outpost's up to 20 OPPs whose HP > 0.
* **Expenses**, per base: pilot salaries (`0x422e62`, §5.2) + module maintenance (`0x42304e`: Σ
  `Maintenance × 1000` over the base's modules, built or not) + ship maintenance (`0x4230d3`: Σ hull
  `MaintCost` (already × 1000) over ships based there).
* Treasury += income − expenses (`0x406ab4`); income and expenses are also logged as transactions
  (`0x413cb6`, `0x413eab`) for the reports.

**OPP yield** (verified, `0x4264d4`): a word table at 0x65f860 indexed by the orbital body: rows by the
body's class (+0x4c, 1–3), columns by its richness (+0x44, 0–4):

| Class | 0 | 1 | 2 | 3 | 4 |
|---|---|---|---|---|---|
| 1 | 200 | 400 | 600 | 800 | 800 |
| 2 | 300 | 600 | 900 | 1200 | 1200 |
| 3 | 400 | 700 | 1000 | 1300 | 1600 |

A second lookup (0x65f868 by class only, `0x426761`) is used for another OPP kind (derived: asteroid OPPs).
Example: one outpost with three class-2/richness-2 OPPs on Veteran pays 225 × 2,700 × 0.75 = 455,625 a month.

### 2.3 Outpost evaluation and funding `0x42978e` (verified)

Each faction record keeps a **per-system monthly score**: X-COM +0x190 and aliens +0x2e4 (85 systems),
plus a pool for events outside any outpost system (+0x438 X-COM, +0x43c aliens). Scores are added by
`0x4270e9(is_xcom, outpost, amount)`: to that outpost's system, or to the pool when there is none.

Monthly, with `X` = X-COM total and `A` = alien total of all systems plus pools, each system gets
`e = X[i] + X / 10 − (A[i] + A / 5)`. The first threshold `t ≥ e` in the table at 0x65f820 picks a
funding step `s`; the change is a uniform `1..|s|` percentage points with the sign of `s`:

| e | ≤ −1000 | −999…−501 | −500…−251 | −250…−1 | 0 | 1…250 | 251…500 | > 500 |
|---|---|---|---|---|---|---|---|---|
| s | −21 | −11 | −6 | −2 | 0 | +2 | +6 | +11 |

Every outpost (+0x6e0, a float fraction) in that system moves by the change and is clamped to
**[0, 0.95]**. **A new outpost starts at 0.75** (`0x425f1d`; the guide's "× 0.75"). An X-COM outpost
whose funding falls **below 0.03 withdraws** (`0x428051`): an event 8, money −1,500 and −`MissionScores`
entry at 0x71beb8, X-COM score −1000 and alien score +1000 for that system, and the outpost is reset as a
free slot.

**New outposts** (derived from the same routine): when X-COM out-scored the aliens over the month
(`X > A`), it has fewer than 16 outposts and (below Veteran) fewer than 5 per base, it gains new outposts:
Beginner/Experienced 1–4, Veteran 0–2, above 1, capped at 3 per month and 16 in all; none while
`0x6b0854` is set. Each goes to the candidate system in range with the best X-COM score (that system is
then marked −5000/+5000 so it is not picked twice). Then all monthly scores reset to 0.

### 2.4 Score sources (verified unless marked)

* **Research complete** (`0x406584`): +250, +500 for `Level` 2, +750 for `Level` 3, to X-COM's pool.
* **Missions** (debriefing `0x53f0ed`): Σ over 134 event counters at 0x71c240 of `count × MissionScores[k]`.
  The counters fall into groups 0–35 and 91–99, 36–90, 100–133 (derived: kills, losses, objectives). Which
  counter is which event is **unknown** (the writers are spread over `0x4da000–0x4e6000`).
* Outpost withdrawal ±1000 (above).

### 2.5 Manufacturing `0x413443` (verified; field names derived)

The queue (`gManufactureQueue`, entries of 0x28 bytes) shares **100 % of capacity**; each job has a
percentage share (+0x10) taken in queue order until the 100 is used. **Capacity is technicians**:
Σ over the faction's online outposts of `0x427468` = **10 + 10 per working OPP** (`0x426fb4`), minus 10
for each OPP slot that is not working, floored at 0. Per tick a job gains
`share / 100 × technicians × elapsed_seconds / 60` (technician-minutes). A unit is done when progress
exceeds **`ProdTime × 60`** (modules: the module ProdTime; ships: hull ProdTime +0x1c; equipment: its
ProdTime), so ProdTime is in technician-hours. Progress then carries to the next unit of the job.

### 2.6 Prices, market and deliveries (derived; partly unknown)

* Purchase price is `Cost × 1000`; `SellPrice` is raw credits (INI scaling rules, consistent with the
  loaders). `ExchangeRate` is used **only** by outpost income (one reference, `0x428c4a`), not by prices.
* Each outpost owns an outpost freighter (`0x42c445`: hull OutpostFreighter for X-COM-side outposts,
  AlienFreighter for alien ones) idle in a flight plan; the rebuild queue (§3.4) replaces a lost one.
  Delivery reports (`0x4baf90`, `0x4bb654`) check cargo (CSU), personnel and hangar space at the receiving
  base: cargo without room is returned, pilots without quarters are sent back, ships without a bay are
  returned **with a refund**. Delivery travel time and per-seller stock were **not traced**.

## 3. Bases

### 3.1 New base `0x4431ad` (CSD, verified)

* **Price** (`0x42b55d`) = **0.75 ×** (CentralHub + best ShieldGenerator + best Downlink + Hangar +
  Personnel + Cargo + Communications + best Sensor + ProbeLaunch + **2 ×** best BeamDefense) module Costs.
  "Best" by research: shield 10 if TRANS_DIMENSIONAL_SHIELDS (67), 9 if ELECTROPLASMIC (27), else 8;
  downlink 7 if MICROBIT_COMPRESSION (40), 6 if HYPER_DIMENSIONAL (39), else 5; sensor 14 if AATS (1),
  13 if EMISSIONS (29), else 12; beam defence by PHASE (47), TRACKING_TRACER (63), GATLING (34),
  PLASMA (48), HEAVY_LASERS (37) …, else Laser (25). The new base gets that template (derived).
* **Construction time 29 days** (0x263b80 s, `0x4207a6`).
* **Placement**: the click must be within **130 screen pixels** of an existing X-COM base at the current
  zoom, and within **30,000 map units (5 ly) of the map centre** (30000, 30000). At most 20 bases.
* The base's name is asked for (max 25 characters).

### 3.2 Base tick `0x4223a1` (500 ms; verified)

Base records: 20 × 0x26c bytes at the base table (`0x6c49c0` holds the manager). Per base:

* **Construction** (+0x264, seconds) counts down; at 0 the base-built event fires (`0x4206ff`).
* **Hull repair** (+0x34 up to the station chart's HP1, `0x41f9fe`): `max(5, elapsed_min × 2)` per tick,
  **doubled with AdvancedDamageControl** (type 18).
* **Modules** (up to 33 slots): each under construction counts its time down (`0x41c226`); completion
  raises event 0xb. Then a switch on the module type (jump table 0x422878 / index 0x4228bc):
  * **Sensors set the base's scan range** (+0x30): SensorType1 **2,000**, Type2 **4,000**, Type3
    **6,000** map units (⅓, ⅔, 1 ly); alien Sensor2/3 4,000/6,000. A base with no working sensor has 0.
  * **Downlinks do research** (§4.1).
  * Hangar, Personnel, Cargo, Training, Medical and their alien twins call the module's own handler.

### 3.3 Module running costs and limits

Maintenance per month = `Maintenance × 1000` (verified, `0x41c10e`), whether or not the module is
finished. Module build time = ProdTime **days** (derived from the INI values and the engine's reading;
the countdown is in seconds). Per-base module limits and dismantling rules were **not traced** (the
Bases screen callback `0x47f1d9` holds them).

### 3.4 Outposts (`0x428e92`, verified)

A 50-entry rebuild queue (time, kind, outpost, OPP): kind 1 brings an outpost back on line with full
HP (X-COM chart or alien chart by owner), 2 an OPP, 3 an outpost freighter. Outpost and OPP HP regenerate
by `max(5, …)` per tick. When rebuilds are scheduled, and their delays, is **unknown**.

## 4. Research

### 4.1 Rate (verified)

Each base researches **one topic** (+0x250 + 4·base, 20 bases). Every downlink module there adds, per tick,
`rate × elapsed_seconds / 60`, where `rate` = **30, 35, 40 per hour** for DownlinkType1–3 (table 0x65f7ec,
stored per minute as value / 60.0, `0x41c7d3`), and 0 while the module is **jammed** (+0x1c). The fraction
below 1 is carried in a global (0x670f20). Progress is **per topic** (faction +0x2a0 + 4·topic), so two
bases may work on the same topic. A topic completes when progress ≥ **`FileSize × 1000`** (`0x4063cb`).
Nothing found resets a topic's progress when a base switches away (derived).

**Pace.** One Type1 downlink gives 30 per hour: the cheapest topic (FileSize 8) takes 11 days, a typical
one (FileSize 30) 42 days, and all 71 topics (Σ FileSize 2,432) about 9.3 years. **A 180-day run with one
downlink finishes about 4 topics in the original as well.** The original's pace comes from more downlinks
and bases (each new base arrives with the best downlink, §3.1).

### 4.2 Topic table and prerequisites (verified)

Records at 0x73c720, stride 0x2a, indexed by **tech id** = the numbering in `GameSettings.ini`'s comment
(0 ACCUMULATOR_SHIELDS … 73 VECTOR_CONTROL_THRUSTERS; the loader `0x5eef02` is called with that index;
21/32/41 are the three MINEFIELD topics, which have no INI section; ALIEN_DRONE is loaded at 4 and again
at 78). Fields: name +0, Level +0x19, FileSize +0x1a, Prereq1–3 +0x1b–0x1d, the yields after them.

**The 12-byte table at 0x6776ec is not the id map.** Its third field (1, 2, 3, 78, 4, 5, …) is a
UFOpedia/databank entry number; position in it = tech id + 1 from entry 5 on (it lists ALIEN_DRONE twice).
`packs/interceptor/strategy/references.yaml` was generated from that third field, so its `tech_ids` are
shifted by one for every id ≥ 4 and wrong for 0–3. Checked by `Level`: with the verified numbering 4
prerequisite links point to a higher Level, with the pack's 10.

Researchable (`0x415ba8`, verified): not known, and either
* ordinary: every Prereq is −1 or known (`0x40666d`); **Nova Bomb (43)** additionally needs SYNTHETIC_ELERIUM,
  FUSION_MISSILE, AATS, ENVELOPING and ALIEN_DRIVE_COMPONENT known; or
* **special** (a Prereq of 127, or a topic in the switch at 0x415aac): the topic's *available* flag
  (faction +0x1d0 + topic) is set **and** its artefact item (table 0x66f768) is in some base's stores;
  if the artefact is gone, the flag is cleared (`0x40689e`). 26 topics are special (alien craft, crew,
  probes, mines, components, some weapons).

On completion (`0x406584`): known flag +0x150, score (§2.4), event 5, every base on that topic goes idle.

### 4.3 Alien messages `0x55381b` (every 5 s; verified)

MESSAGE_1–5 (topics 11–15) become available, with the transmission dialog `0x4be021(n)`, when their
INI prerequisites hold and the month value `month_index + 1` reaches **3, 5, 7, 9, 12** (with Jan 2067 as
month 1, derived: **Feb, Apr, Jun, Aug, Nov 2067**). The prerequisites (verified numbering) chain them:
2 needs 1 + Structural Component + Type 1 Saucer exam; 3 needs 2 + Type 2 exam; 4 needs 3 + Type 3 exam +
Weapon Component; 5 needs 4 + Drive Component. Videos `alientrans1–5.smk`.

## 5. Pilots

### 5.1 Training and healing `0x40f55c` (500 ms; verified, names derived)

* **Training**: capacity 5 pilots per Training module (type 17) at the base; pilots over capacity drop out.
  Each pilot in training (+0x3c) accrues time (+0x40); **every 5 days** (432,000 s) Accuracy (+0x2c),
  Flying (+0x2d) and Psionics (+0x30) each gain `trunc(Intelligence / 100 × stat × 0.1 / 6 + 1)`, capped
  at 100 (so 1–2 points per 5 days).
* **Healing**: every game day health (+0x24) gains **1, or 2 in a medical bay** (+0x34), capped at the
  pilot's maximum (+0x31). Medical capacity is per Medical module (type 33; 5 each, derived from the
  same loop shape). Leaving the bay raises event 0xd.
* Injury in combat: `../flight/README.md` §4.2.

### 5.2 Salary `0x4106f1` / `0x410297` (verified)

Monthly salary = `(24,000 + v) / 12`, where `v = 24,000 + rank bonus + Σ stat-bracket bonuses`, so a
plain pilot costs 4,000 a month. Rank bonus by rank (+0x29, 0x65f058): −1000, −500, 0, 750, 1500.
Bracket tables (word pairs {threshold, bonus}): Intelligence 0x65f078, Stamina 0x65f090, Accuracy 0x65f0d0
(−300 … +800), Flying 0x65f100 (−150 … +700), Aggressiveness 0x65f130 (peaks at 66–70), Bravery
0x65f160, Psionics 0x65f190 (0 … +1900), a float field +0x1c (0x65f1d0, 0 … +400; derived: kills) and
+0x31 (0x65f1f8). Hiring, the recruit pool and rank promotion (`RankAdvancement*`) were **not traced**.

## 6. Alien and rival activity (`Agents.cpp`, the strategic AI)

### 6.1 Alien treasury `0x57649c` (verified; meanings derived)

* Every **25 days** (2,160,000 s): income = Σ over alien outposts (faction 9) of the §2.2 formula. With at
  most one outpost left, a fallback of **125,000 × (difficulty + 1)** instead.
* Every **6.25 days** (540,000 s): the treasury is split by the Think.ini `FinancialAI` ratios (research /
  goods / Doomsday, adjusted by aggression `0x5a5240`) into a goods fund (+0x10) and a Doomsday fund
  (+0x14).
* Every **20 hours** (72,000 s): research money = min(budget, `WklyRschSpendingPerDLink` (20,000) ×
  alien downlinks / 7, or / 14 with none); the excess /12 goes to goods. **Alien tech level** rises 1→2
  and 2→3 when research money reaches (table 0x67c540, by difficulty Beginner→Superhuman):

  | | Beg | Exp | Vet | Gen | Sup |
  |---|---|---|---|---|---|
  | → level 2 | 1,000,000 | 800,000 | 700,000 | 600,000 | 500,000 |
  | → level 3 | 1,500,000 | 1,300,000 | 1,100,000 | 1,050,000 | 900,000 |

* Doomsday pieces: built by tech level and shipped by freighter (`0x57ab9d`, debug log at 0x6a472c);
  received pieces 1–3 at the Doomsday site. Build costs and timing **not traced**.

### 6.2 Side missions `0x57fbd3` (verified; mission names from the AI log strings)

Checked every **rand[6, 9) days**. The first that qualifies is added (`0x57f5d3(k)`, sets flag +0x94+k):

| k | Mission | Month value (as §4.3, no +1) | Chance | Other conditions |
|---|---|---|---|---|
| 1 | Alien raiders | 3–8 | 10 % | no mission in progress (`0x57fb44`) |
| 2 | Rogue freighter | 6–18 | 20 % | a freighter flight plan exists |
| 3 | Pirate rescue | 10–24 | 20 % | 1 and 2 not active, `0x57fb44` |
| 4 | Nova Bomb hijack | any | 25 % | Nova Bomb known or > 1000 progress; an AI flag set (derived: a pirate base is known) |
| 5 | Arms dealer | > 6 | `100 × f` % (f = AI float +0x10, derived: aggression) | an AI flag clear |
| 6 | Science station rescue | elsewhere (not this routine) | | |
| 7 | Recon (new outpost system) | > 1, day < 15 | 75 + 3·difficulty % (as read) | outposts / bases < 5, outposts < 16 |

**Fluff missions** (`0x57fbd3` top): at most 4 at a time; one more when more than
`1,008,000 + 72,000 × (6 − difficulty)` s (11.7 days + (6 − d) × 20 h) passed since the last, with a 1-in-5
chance per check.

### 6.3 Detection and reaction (`0x578456`; derived)

When an X-COM base or outpost detects an alien flight plan, the plan's fleet reacts with chance
**25 + 10 × difficulty %** (`0x5785a5`: derived, it re-tasks against the detector). Detection range of a
base = its scan range (§3.2). The detection roll per range band (`Sensor*DetectionProb`) was not traced.

### 6.4 Endgame (verified)

The MacArthur, once requested, arrives **rand[24, 48) hours** later (`0x562bd9`).

## 7. Save game (derived)

Three files per slot in `Saved Games\`: `Header.NNN`, `General.NNN`, `GameState.NNN` (`0x466cc7`,
LoadSave.cpp). The header is a 168-byte record: a check dword 0xef62e at +4, a 9-character magic at +8,
the version `%d.%d.%d`, and the save's name (25 characters) at +0x1c. The other two files were not traced.

## 8. Still open

* Which `MissionScores` index is which event (the counter writers in `0x4da000–0x4e6000`).
* Delivery travel time, interception of deliveries, per-seller stock and restock, sell rules.
* Strategic ship speed between systems; launch/return rules; the alien tactical planner (`TacticalAI`,
  base attacks, raids), alien base growth (Think.ini module priorities), the Doomsday piece costs.
* Recruiting, rank promotion, pilot death outside combat; the `*Goodies` salvage use.
* Module limits per base, dismantling, the Communications and ProbeLaunch effects, outpost rebuild delays.
* The other two save files.
