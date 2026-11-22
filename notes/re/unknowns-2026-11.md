# The parity plan's "Still unknown" list, settled (issue #136, 2026-11-22)

Facts read from the player's own `Interceptor.exe` (CD build, 3,008,512 bytes) and `GameSettings.ini` /
`Think.ini`, by static analysis only (Hopper 5 whole-binary decompile and `objdump`, kept outside the repo).
Nothing from the originals is reproduced: these are addresses, offsets, constants, formulas and behaviour, in
our own words. **Verified** = read from the machine code. **Derived** = inferred from how the code uses a
value (all field *names* are derived). **Unknown** = not traced. Addresses are CD-build virtual addresses.

Units: game time in seconds (`0x412181`); real-time ticks are 1/60 s (`0x61926a` = GetTickCount × 0.06);
strategic map units, 6,000 per light year; colours are RGB.

| # | Question | Section | Status |
|---|---|---|---|
| 1 | CSD star map: icons, labels, zoom, transform, lamps | §1 | settled (verified, a few names derived) |
| 2 | Deliveries, freighters, stock, ship travel speed | §2 | settled (verified) |
| 3 | Alien raid planner and alien base growth | §3 | settled (verified; some helper meanings derived) |
| 4 | `MissionScores` indices | §4 | partly: rating formula and the hull-class counters verified; ~60 counters still unnamed |
| 5 | Pilot "temperament" | §5 | settled: it is a pilot **grade** 0–4 (verified; the name is derived) |
| 6 | Gimballed turret aim | §6 | settled (verified) |
| 7 | Save format | §7 | settled enough for a compatible-in-spirit save (verified) |
| — | Station shield pool when the generator dies; outpost/OPP pools (for #113) | §8 | settled (verified) |

## 1. The CSD star map (CSD.cpp)

### 1.1 Geometry and transform (verified)

* **Map window** (`0x438101`, CSD member +0x553c): a 333 × 333 square, offset so its centre is near
  (232, 227) on the 640 × 480 screen. Labels and hover only count inside the **inscribed circle**: the
  label routine `0x4398c2` rejects the cursor when its distance from the window centre exceeds half the
  width, so the visible map is a round viewport (derived from that test).
* **Minimap** rect [502, 43, 596, 137] (+0x555c).
* **The galaxy is 60,000 × 60,000 map units** (10 ly square). A "view rect" (+0x56b4) is the whole galaxy's
  projection in screen pixels. World → screen (`0x44c1e0`):
  `sx = view.left + view.width × x / 60000`, `sy = view.top + view.height × y / 60000`;
  screen → world (`0x44c240`) is the inverse, `x = (sx − view.left) × 60000 / view.width`.
* **Zoom levels** = the view knob's position (knob byte, read by `0x44c510`; stored at CSD +0x56c4),
  set up by `0x4444fd`:

  | Level | View rect | Scale | Visible across the window |
  |---|---|---|---|
  | 0 sector | window inflated by 1.5 × its width on every side → **4 × 333 = 1,332 px** | 45 units/px | 15,000 units = **2.5 ly** |
  | 1 close | inflated by 2.5 × → **6 × 333 = 1,998 px** | 30 units/px | 10,000 units ≈ **1.67 ly** |
  | 2 object | the selected system or base drawn as a picture (`0x440a61`, `0x43d19e`) | — | — |
  | 3 | removed: draws a fixed "zoom level removed" message (`0x43ed13`) | — | — |

  Changing level keeps the world point under the window centre (it converts the centre to world
  co-ordinates before resizing and re-centres after). Panning (`0x447a14`, `0x447c42`) re-centres the view
  rect on a world point, then clamps it so the view never shows past the galaxy's edge. Clicking a black
  hole (star class 8) only re-centres; any other system zooms to level 2.
* **Level-2 picture** (`0x440a61`): a system shows an animated star portrait chosen by star class
  (sprite rows 454 / 474 / 494 / 514 / 534, 120 × 120, by class through the jump table at 0x440b35, 554
  for an unknown class); a base shows 566, or 586 for an alien or pirate base.

### 1.2 What is drawn, per pass (verified)

The draw (`0x439b9b` → `0x43ad59` at level 0, `0x43a3cb` at level 1) runs passes: pass 0 records clickable
hit boxes and queues labels, pass 1 draws icons. Objects outside the view or the window are skipped.

**Stars** by star class (system record +4; names from the class strings `0x5db3fc`): 0 Red Giant,
1 Blue Giant, 2 White Dwarf, 3 Blue, 4 White, 5 Yellow, 6 Orange, 7 Red Dwarf, 8 Black Hole.

| Class | Level 0 icon (9 × 9, static) | Level 1 icon (40 × 30, 20 frames) |
|---|---|---|
| Red Giant | 899 | 702 |
| Blue Giant | 896 | 662 |
| White Dwarf | 893 | 622 |
| Blue | 895 | 642 |
| White | 894 | 762 (same art as White Dwarf) |
| Yellow | 897 | 682 |
| Orange | 898 | 782 |
| Red Dwarf | 900 | 722 |
| Black Hole | 901 | 742 |

(Numbers are rows of the sprite table at 0x69bc18, group 1.) Level-1 stars animate with
frame = (map counter + system index) mod 20, so neighbouring stars are out of step. Systems that are
hidden (`0x44c160`) are drawn only when a debug/cheat switch (`0x4a58c0`) is on.

**Markers on a star (level 1):** an online X-COM or neutral outpost in the system → static 5 × 5 marker 916
under the star; an alien outpost → 7 × 5 marker 917, **blinking** (on for 3 of every 6 counter steps); the
current side-mission system (global 0x6b0854) without an outpost → 11 × 11 marker 914 at star y + 15,
blinking the same way.

**Bases** (`0x43b649`, up to 20; shown only when known to the player, `0x422107`):

| Owner | Level 0 (static) | Level 1 (animated) |
|---|---|---|
| X-COM (default) | 902 | 802 + (counter + base) mod 20 |
| Alien (side 9) | 903 | 822 + (counter + base) mod 20 |
| Pirate (side 10), subtype 1 (arms dealer) | 912 | 859 + (counter + base) mod 16 |
| Pirate, other subtypes | 911 | 843 + (counter + base) mod 16 |
| Side 11 (unknown owner; derived: neutral station) | 913 | 875 + (counter + base) mod 16 |

A base under construction gets a progress bar under its icon: 15 × 5 px at y + 10 (level 0) or 25 × 10 px at
y + 20 (level 1), red RGB(150,0,0) background and green RGB(0,150,0) filled to elapsed / 29 days. Sensor
range circles (bases `0x43bd33`, outposts `0x43bdef`) are drawn when the map's sensor toggle is on, radius =
range × view.width / 60000; an outpost's range is halved while its hull is at 0 (`0x42a744`).

**Fleets** (flight plans, `0x43c330…0x43d067`): 7 × 7 icons that flip between two frames on the fast
counter (`+0x5e10` bit 0). Alien plans with an identified lead ship get a 5 px ring RGB(100,0,0) and icon
608/609 (moving) or 612/613 (holding); unidentified contacts use 616/617 (7 × 6). Pirates 618/619 or
620/621; side 11 (corporate) ring RGB(100,200,0) and 606/607, 610/611 or 614/615; X-COM 606…615 likewise.
The selected plan blinks (drawn on 2 of every 3 counter steps). Labels: X-COM squadron name RGB(0,120,0),
corporate "corporate squadron" text RGB(100,200,0), alien "alien" / "alien probe" (or "jammer probe")
RGB(200,100,50), pirates RGB(255,50,50). At level 2 the plan's map marker 910 is drawn in the system picture.

**Battles in progress** (`0x43c051`): a square, 15 px (level 0) or 25 px (level 1), filled translucent
RGB(255,50,50) at level 25 and outlined RGB(100,100,100), shown for 5 of every 10 counter steps; label
"attack in progress" RGB(200,200,200).

### 1.3 Labels (verified)

* Labels are **shown on hover only**: `0x4398c2` queues a label when the object lies inside a 30 × 30 box
  around the cursor (globals 0x6b0530/0x6b0534, written by the mouse-move handler `0x48c0ba`, derived) and
  the cursor is inside the round window.
* Text: the object's name, **upper-cased**, at most 50 characters; system names get " - <side-mission
  title>" for the side-mission system. Drawn by `PortDrawString` (`0x5ffa81`) in the **built-in 6 px font**,
  **centred** on the x anchor (flag 1).
* Position: x = the object's x, pushed inward so the text stays 3 px inside the circle's chord at that row;
  y = object y − 20 for stars, y − 8 (level 0) or y − 16 (level 1) for bases, y − 8·k for fleets.
* Colours: stars RGB(160,160,0); bases RGB(120,120,0), with " building" appended while under construction;
  fleets and battles as above.
* When several labels are queued (objects overlapping under the cursor) only **one is shown at a time,
  cycling every 15 map-counter steps (1 s)** (`0x4397aa`).
* The status bar shows the hovered object's name (§2.7 of `wrappers/ui-framework.md`).

### 1.4 Counters and lamps (verified)

* **Map counter** CSD +0x5e0c: +1 when more than 3 ticks have passed (every 4 ticks ≈ **67 ms**, 15 Hz),
  in the CSD frame routine `0x43ff0f`. Level-1 star and base animations: 20 frames → 1.33 s per cycle.
  Blinks "3 of 6" = 200 ms on, 200 ms off.
* **Fast counter** +0x5e10: +1 every 7 ticks (≈ 117 ms); fleet icons flip on it.
* **Corner lamps** (CSD +0x61bc, +0x61dc, +0x61fc, +0x621c; SpritesDLL 1900–1909): each advances one frame,
  looping, **every 4 ticks (≈ 67 ms)**, 10 frames → 0.67 s per cycle (`0x440240`: `now > last + 3`). They
  are a frame-list object (`0x46d548`, step `0x46d8cd`), not the timed Animation class.
* Status lamp: unchanged (6 ticks, `ui-framework.md` §2.5).

## 2. Deliveries, freighters and strategic speed

### 2.1 The shipment queue (verified; `GameData.cpp`)

Every purchase, transfer, hire and finished manufacture becomes a **0x24-byte shipment record** in a linked
list (`gShipmentQueue`):

| Offset | Size | Field (names derived) |
|---|---|---|
| +0x00 | byte | kind: 1 ship, 2 pilot, 3 item |
| +0x04 | int | item / ship / pilot id |
| +0x08 | int | quantity (1 for ships and pilots) |
| +0x10 | byte | owner side whose station must load it (the player's side for purchases) |
| +0x11 | byte | flag: on loading, also clear the destination's pending marker (`0x4174d0` / `0x417440`) |
| +0x14 | int | destination base |
| +0x18 | byte | 1 = load at **any outpost** of that side; 0 = load at the base in +0x1c |
| +0x1c | int | source base (forced to −1 when +0x18 = 1) |
| +0x20 | byte | flag: the carrying freighter gets 1–3 extra escorts (`0x55d4aa(plan, 1 + rand 3)`, derived) |

Creators: item `0x414740`, pilot `0x41481a`, ship `0x414908`. Callers:

* **Buying** (Equipment & Personnel, `0x56b950`): the price (Cost × 1000 × quantity, or the pilot's hire
  cost) is **paid at once** when ordering; the record is "load at any X-COM outpost, deliver to the current
  base". There is **no stock limit or restock**: the only check is money (verified: nothing else is read).
* **Transfer between bases** (`0x56bdfd`): items leave the source base's stores immediately (`0x42350d`);
  the record is "load at base A, deliver to base B".
* **Manufacturing output** (`0x413801`, `0x413865`): finished items and ships are shipped from the outposts
  to the ordering base the same way as purchases.

So "the seller" is **X-COM's own outpost network**, not a vendor with stock (verified).

### 2.2 Loading (verified, `0x4152c1`, every 1 s real time with the clock running)

For each idle freighter flight plan that is parked at an outpost or base (`0x416bd0`) owned by the player:
records matching that station are loaded in list order into **100 cargo units**:

* a **ship** takes the **whole freighter** (only when the hold is still empty; one ship per trip);
* each **pilot** takes 1 unit;
* an **item** takes `size × quantity` units (`0x42b2ff`); a stack that does not fit is **split** and the
  rest waits for the next trip.

Then the freighter is routed to the record's destination (`0x55e263`). On arrival the existing delivery
checks run (cargo/quarters/hangar space, refunds; `strategy/README.md` §2.6). At a stop a freighter spends
**1,800 s (30 min) unloading** and **3,600 s (1 h) refuelling** (`0x560ed6`) before it can be loaded again.

**Delivery time** = wait for an idle X-COM freighter at an outpost + outpost→base distance ÷ the
freighter's speed (§2.3) + unloading. Each outpost owns one outpost freighter (§2.6 of the strategy notes).

### 2.3 Strategic ship speed (verified, `0x407f6d`)

A ship's map speed in **map units per game second** comes from its **mass** (`0x407880`) by bracket (table
0x65f020: upper mass bound, speed), × 10:

| Mass ≤ | 7,001 | 15,001 | 25,001 | 50,001 | 75,001 | 100,000 | heavier |
|---|---|---|---|---|---|---|---|
| speed (units/s) | 0.1875 | 0.165 | 0.15 | 0.135 | 0.075 | 0.06 | 0.0375 |
| hours per light year | 8.9 | 10.1 | 11.1 | 12.3 | 22.2 | 27.8 | 44.4 |

× **1.15** with EngineType2 and × **1.4** with EngineType3 fitted (ship +0x12f = 8 / 9).

Mass = base by hull class + armour type (≈ 150–450) + 250 with a shield + Σ missile mass × rounds
(table 0x65f210) + special systems. Base masses: IntType1 6,700; IntType2 12,700; IntType3 22,700;
X-COM/Outpost freighters 157,000; AlienType1 5,500; AlienType2 and 4 11,000; AlienType3 19,000; AlienType5
110,000; AlienType6 157,000; AlienFreighter 110,000; PirateFreighter 157,000; pirate and corporate 1–3 as
IntType1–3; Patton 250,000. **Freighters are always in the slowest band: 0.0375 units/s, ≈ 1.85 days per
light year.** A plan moves at `min(ship speed) × factor` (plan +0xd4 × +0xd8, factor 0–1; alien raids set
it to a random 0.75–0.95), at least 1 unit per step; a probe plan uses its probe's speed.
At the fastest clock rate the plan update is split into 10 sub-steps (`0x553414`).

### 2.4 Interception of freighters and fleets (verified)

Freighters are ordinary flight plans, so they can be met by any hostile plan. In addition the alien AI's
**harassment check** (`0x573181`, every (6 − difficulty) hours) looks at every X-COM plan in space: one
within **4,000 units (⅔ ly) of an alien outpost** (or 2,000 of an alien base, derived from the same loop)
is attacked with chance **50 + 5·d %** (`0x57338e`). Alien bases also send fleets after detected plans
(`strategy/README.md` §6.3).

## 3. The alien strategist (Agents.cpp)

### 3.1 Main alien AI tick `0x5756c8` (verified)

| Every | Action |
|---|---|
| raid timer (below) | choose a target and launch a raid (`0x5745e6`), then `0x574bce` |
| (6 − d) hours | harassment of X-COM plans near alien outposts (§2.4) |
| 20 h (72,000 s) | `0x5753dc`; consider a **new alien base** (below) |
| 5 days | if no alien base is alive: found one (`0x572771`), a second with 10 + 7d %; with 50 + 5d % also grow outposts |
| 25 days | reset the period counters and **grow alien outposts** (`0x575274`) |
| 12.5 days | (timer only) |

**Raid interval** (`0x5728e4`): next raid = now + rand[a, b) days by month index m (`0x41276c`):
m < 2 → [9, 12); m < 6 → [8, 12); m < 10 → [7, 11); m < 16 → [7, 10); later → [6, 9).

**New alien base** (20-hourly): when there are fewer than 8 alien bases, more than 12.5 days since the last,
and any of: 3 × bases < alien outposts; bases − d/2 < X-COM bases; m / 8 > bases; or a period counter rule
(`rand[1,3) + 4` vs counter +0x40, `8 − d/2` vs +0x44); and the AI can afford a CentralHub (`0x5a5260`).

**Outpost growth** (`0x575274`): count = m < 4: 0 or 1 (50 %); otherwise rand[0, min(m / 8, 3)); with fewer
than 2 alien outposts rand[1, 2). Each goes to a free system without an outpost near an alien base
(`0x574505`, `0x5f2e47`) and is founded by `0x425f1d`.

### 3.2 Target choice `0x5745e6` (verified)

1. **Base attack** with chance 5 + 5d % (m < 3), 25 + 5d % (m < 6), 40 + 5d % later: a random X-COM base.
2. else with 45 %: an **outpost** — the richest by monthly income (`0x428b76`) with chance 40 + 10d %,
   otherwise a random one; then a random OPP slot of it (if that OPP exists, the OPP is the target).
3. else with 25 %: an outpost as in 2 (no OPP).
4. else, when no tactical agent is already on mission 3 and m > 1: a random X-COM outpost as **mission 3**;
   otherwise with 35 + 5d % a random outpost.
5. In every branch, when no target was set and m > 6: with 10 + 15d % a random X-COM base.

Missions 2 (attack) and 3 (derived: outpost raid with escort rules) are launched by `0x573998`.

### 3.3 Raid fleet `0x573998` (verified)

* Source: the alien base (agent `Agent006v4`) **nearest the target** that has at least **3 ships queued**
  in its build slots.
* Size: from jump tables by the target's location type (0x573dd7) and difficulty (0x573d99), capped by the
  queued ships; ships are taken from the base's queue by class (up to 3 classes) and created as owner 9 with
  a pilot of grade `0x5a44f9` (§5). On mission 3 the first 4 ships are normal; later ones, with a falling
  chance (100 − 25·k %), get special armament (AlienType1: slot +0xa8 = 8; AlienType2: three slots = 8).
* Escorts by alien tech level (`0x5a2f30`): at level 3 and m > 5, location type 0/1: 25 % an AlienType4
  (pilot grade 4); otherwise, at level > 2, 25 + 5d % an AlienType3.
* Speed factor rand[0.75, 0.95) (`0x416f70`); a tactical agent (`0x577892`) flies the plan, 500 ms think.
* **Mission 8 patrol** (`0x5787b5`): 85 % per arrival to hop to another system within rand[15,000,
  25,000) units for up to 7 days, waiting rand[10,000, 50,000) s at each.

### 3.4 Alien base agent `0x57e80b` (each alien base; verified)

| Every | Action |
|---|---|
| 6 h | **queue a ship** into one of 9 build slots (variant-A bases only, while queued ships < 3 × hangars) (`0x57e51e`) |
| 40 h + rand[0, 12) h | **build a module** (`0x57d86c`) when the base has fewer modules under construction than its limit (+0x2c); its Cost is taken from the base's fund (+0x1c) |
| 5 days | with ≥ 5 queued ships: launch up to 3 queued AlienType1 as a scout to a random non-black-hole system **without an outpost** within rand[15,000, 25,000) units (`0x57d439`) |
| 12.5 days (+0–6 s: a quirk, `rand × 72,000 mod 7`) | advance the base's **stage** (0→1→2) when every module count reaches `Think.ini` DCountA/B[stage] (`0x57d77c`) |
| 20 h | nothing (`0x57e7f5` is empty) |

**Ship queue choice** (verified): by alien tech level t and research progress p toward the next level
(`0x576be0`, 0–1; 1 at level 3), and the fleet share of each class (`0x57e47e`):
* t = 3: AlienType4 if its share < 25 %, else AlienType3 if its share < 25 %, else as t = 2.
* t = 2: with chance 100·p %, AlienType3 when its share < 50 % and m > 6; otherwise AlienType2.
* t = 1: with chance 100·p %, AlienType2 when its share < 75 % and m > 2; otherwise AlienType1.

**Module choice** (`0x57d86c`, verified as a hard-coded decision tree, not data): every candidate needs a
free slot of its size class (`0x420845`) and its Cost ≤ the base's fund. The first steps: Hangar (45) while
fewer than 2, with chance 75 − 10n %; then gated by alien tech level (`0x5a52e0`) Downlink 2/3 (41/42) and
Shield 2/3 (63/64); Personnel (59) while fewer than hangars / 2 (chance 60 − 10n %, derived from the same
shape); Phase (56) up to d + 3 and PlasmaPulse (57) up to d + 5. Other modules it can return: 35, 36, 39
(Communications, 35 sites, 10 % filler), 40, 43/44 (ECM, 140 sites each), 48, 60, 61, 62, 65, 66. The
exact order past the first steps was not tabulated. (Module ids: exe enum, `strategy/README.md` §2.1.)

**Think.ini module sections** (loaded by `0x49b2d8` into 0x67c5e0, stride 0x48): only **DCountA/B** are
read, for the stage counter above; **PriorityA/B and ReservesA/B are loaded but never read** (verified: no
other references), and nothing found reads the stage. So `Think.ini` has almost no effect on base growth.

## 4. MissionScores and the mission event counters

### 4.1 Rating (verified, debriefing `0x53ee96`)

The mission rating 1–5 starts at +1 (won) or −1 (lost) and adds:

| Term | Counters (index into 0x71c240) |
|---|---|
| +1 if > 0, else −1 | debrief field +0x30c (derived: objectives) |
| −0.5 if any | 23, 24, 25 (X-COM IntType1–3 lost) |
| −1 if any | 26 (counted three times: an original bug) |
| +1 if > 5, +0.5 if > 2, −0.5 if 0 | Σ 2…8 (alien craft destroyed) |
| +1 if any | Σ 1, 9…14 (alien craft — second list, derived: disabled/captured) |
| −1 / −0.25 / −0.5 / −0.75 | 34 / 33 / 31 / 32 |
| +1 / +0.5 / +0.5 | 19 (station destroyed) / 16 (OPP destroyed) / 17 (outpost destroyed) |

Rating 5 when the total > 3, 4 when > 1, 3 when > −1, 2 when > −2, else 1 (constants at 0x661c64…0x661c90). The score itself is
Σ counter × `MissionScores[k]` (`strategy/README.md` §2.4).

### 4.2 Counters by hull class (verified, `0x4dcc6a`)

Objects destroyed in the mission (list A, rt type 1): IntType1–3 → 23/24/25; X-COM/Outpost freighter → 21;
AlienType1–3 → 3/4/5; AlienFreighter → 2; PirateType1–3 → 92/93/94; Corporate 1–3 → 135/136/137 (beyond the
134 the debriefing sums: never scored). Other rt types: 2 station → 19, 4 outpost → 17, 5 OPP → 16,
6 → 20 and 35, 8 → 15. List B (derived: craft disabled/captured): IntType1–3 → 26/27/28, freighters → 22,
AlienType1–5 → 9/10/11/12/13, AlienType6 and AlienFreighter → 1. Pilots killed: 110–117.
**Unknown:** counters 36–90 (objective events in `0x4df000–0x4e3000`) and 95–104.

## 5. Pilot "temperament" is a pilot grade (verified; name derived)

The field is **pilot record +0x28** (byte; the "+0xa0" in `flight/README.md` was an indexing slip), read by
`0x410a56`, which returns 0 for pilot indices below 10. It is set from the generator's argument
(`0x40e8b6(grade)`):

| Grade | Who |
|---|---|
| 0 | X-COM recruits (all player pilots; the generator also keeps grade-0 names unique) |
| 1 | pirates; AlienType1/2 early on |
| 2 | outpost freighter crews; AlienType1/2 with chance 50 + 2m % from month 2 |
| 3 | AlienType3 |
| 4 | AlienType4 (and Type4 raid escorts) |

`0x5a44f9(2, class)` picks the alien grade; from month 8 − d/2 on it can return higher grades.
Effects: the generator adds a grade bonus to generated stats (byte tables 0x65f0a8 / 0x65f0b0 / 0x65f0b8:
0/3/6/9/12, 0/5/10/15/20, 0/5/15/20/30 scaled by Intelligence/100, derived), and in flight the look-ahead
(0 → 225 m, 2 → 100 m, else 250 m) and the sprint rule (`a = 1.0625 S` when grade ≠ 0). No strategy screen
shows it.

## 6. Gimballed turret aim (verified, `0x5992b9`, `0x59970f`, `0x599d0a`, solution `0x59878d`)

* **Solution:** the target's position in the turret base's local frame, minus the mount point, plus the
  target's velocity × **(distance / 2,000 m/s)** — a fixed projectile speed, not the beam's `Speed`.
* **Limits:** a solution only counts when the target is in front of the mount (local forward > 0); kinds 2
  and 3 also reject solutions past ±180° (0x8000) on the second axis.
* **Rates:** two joints (yaw, then pitch) slew toward the solution by at most **0x4000 angle units per
  frame** (65,536 units per turn → 90° per frame, effectively instant).
* **Fire:** only when both joints sit exactly on the solution; barrels alternate at ±1.2 m; with the LOS flag
  a 5,000 m line of sight must not hit the own station; the interval and target choice (range +0x60) come
  from the caller `0x59a962` / `0x598899`.
* **No accuracy error:** the gimballed routines make no random calls, so the turret's accuracy (+0x2c) is
  unused for kinds 1–3 (only fixed turrets, kind 0, apply it).

## 7. Save format (LoadSave.cpp; verified)

* Three files per slot in `Saved Games\`: `Header.NNN`, `General.NNN`, `GameState.NNN`.
* **Header** (168 bytes, read by `0x466aa2`): check dword 0xef62e at +4, magic "XCOM INTR" at +8, the
  version text `%d.%d.%d` (from the build's version bytes at 0x6a6a58), the save name (25 chars) at +0x1c.
  A save is rejected when the magic or check differs.
* **General** is created and closed **empty** (`0x4672ac`).
* **GameState** (`0x467384`) is a sequence of **chunks**, each a NUL-terminated tag followed by raw dumps of
  the in-memory records (arrays as count + fixed-size records; lists as records until a terminator).
  Order: `gPlayerInfo`, `gShipInfo` (ship records 0x13c bytes), `gPilotInfo`, `gGameClock`, `gHistory`
  (0x38-byte entries), `gShipmentQueue` (0x24-byte records, §2.1), `Globals` (difficulty, player side, …),
  `gUniverse` (0x4c98 bytes of systems), `gOptions`, `gBaseInfo`, `gOutpostInfo` (per-system scores,
  outposts), `gAIManager` (agents `Agent000v3` … `Agent035v2`), probes (`gProbeInfo`),
  `gFlightPlanManager_v5` (with `SquadronSet_v3`), the combat simulation (`CombatSimulation_V4`,
  `gSimManager`), then a trailer (`0x4e900e`). `gManufactureQueue` is written by `0x4139cd` inside the
  player block (derived).
* **Versioning** is per chunk by tag suffix (`_v5`, `v3`, `_V4`); a load fails on any tag mismatch ("game
  state has changed").

## 8. Station, outpost and OPP shield pools (for #113; verified)

* **Generator destroyed → the pool drops to 0.** Module damage `0x4204cd` removes a module whose HP reaches
  0 through `0x4244ff`; when it is the shield-generator slot, that routine sets slot +0xc4 = −1 **and the
  pool +0x38 = 0** (`0x4246db`). Adding a generator (`0x421271`) sets the pool to the full 2,000 / 3,000 /
  4,000. The recharge `0x45a17a` has no explicit `max > 0` check; it adds only when `max > pool`, so with no
  generator (max 0) it does nothing.
* Recharge runs once per second only for the (up to two) station rts listed at 0x6fd784, and for stations
  reads the maximum through the rt at `0x684720` (quirk already noted).
* **Outposts (rt type 4)**: pool at outpost record +0x6bc (85 records); **OPPs (rt type 5)**: pool at
  +0x6f0 + 12·k (20 per outpost). Same rule, +min(max − pool, 2) per second, floor 0, no generator. Their
  maxima come from the INI charts after all (copied at load, `0x5f0bc1`):

  | | HP max | Shield max | two more ints (unknown use) |
  |---|---|---|---|
  | X-COM outpost | `XCOMOutpostHP` HP1 | `XCOMOutpostHP` SD | 6, 4 |
  | X-COM OPP | `XCOMOPPHP` HP1 | `XCOMOPPHP` SD | 2, 2 |
  | Alien outpost | `AlienOutpostHP` HP1 | `AlienOutpostHP` SD | 8, 8 |
  | Alien OPP | `AlienOPPHP` HP1 | `AlienOPPHP` SD | 3, 3 |

  (In the player's INI: 3,500 / 1,500; 2,800 / 1,000; 3,500 / 1,500; 2,500 / 1,000.) Outposts of other
  owners use the X-COM row. This corrects `flight/README.md` §8.1: the Outpost/OPP charts **are** used
  (HP1 and SD only).

## 9. Follow-ups for #114 (verified unless marked)

* **Pirate turrets** (`0x59d8fc`, `0x59d6c9`, subtype switch at `0x59df3a`): arms dealer (subtype 1) =
  5 turrets, always. Raider (2) and pirate base (5) use a 10-row table, but rows 3, 5, 6 and 8 are never
  built. Raider: 6 turrets, always. Pirate base: each of the 10 rows first rolls `chance(25 + 5d)` and is
  **skipped** on success (`0x405070` = `rand % 100 < p`), so each of the 6 buildable rows is present with
  75 − 5d %. Subtypes 3 and 4 get none.
* **Alien base:** see `flight/README.md` §8.4 — tendrils are the children in attach slots 1–3 of the
  `al-base01` object (hit through their own geometry); destruction is hull **or** all hard points; no hard
  point damage spills to the hull.
* **Mines:** rings by yaw steps; contact trigger (derived), INI `Range` 0.

## 10. What remains unknown

* Counter meanings 36–90 and 95–104 (objective events); the debrief field +0x30c.
* Raid fleet sizes per target type and difficulty (two jump tables in `0x573998`, not tabulated).
* The variant A/B flag of alien base agents (+0x28) and who sets it.
* Alien ship build time in a slot; Doomsday piece costs (unchanged).
* The two small ints per outpost/OPP chart (§8).
* The byte layout inside each save chunk (only tags, order and record sizes were read).
* Where xcgen fills the `al-base01` attach slots 1–3 (tendril objects) from.
