# The original game, system by system

**Status: analysis, 2026-11-10.** What *X-COM: Interceptor* does, how it decides it, and
where the data lives — the requirements the mod format must meet. Facts are marked
**V** verified (seen in the CD data), **D** derived (inferred from data, strings or a source),
**U** unknown.

**Sources.**

- **[CD]**: the player's CD. Files used:
    - `GameSettings.ini` (259 sections)
    - `Think.ini`
    - `MPSettings.INI` and `NetSettings.ini`
    - `soundfx.ini`
    - `Text/LangFile1.txt`, with 3,782 keyed strings
    - `Text/starcorp1.txt`, `Names1.txt`, `news1.txt` and `short1.txt`
    - `Animations/mfdanim.mps`
    - the `Loops/*.wtl` music tables
    - strings and constants in `Interceptor.exe`
- **[G]**: *X-COM Interceptor Official Strategies & Secrets* (Sybex, 1998), the designer-written strategy guide.
- **[UR]** UFOpaedia: Research, Background and Hypernews pages.
- **[F]**: a GameFAQs walkthrough.

Nothing here reproduces original text or art; names and numbers are cited as facts.

## Summary: data versus code

The original keeps **numbers** in data and **rules** in code.

| In data (V) | Hard-coded in the executable (V/D) |
|---|---|
| Prices, build times, maintenance, hit points, weapon and missile stats, hull handling, research sizes and prerequisites, module stats and size classes, rank thresholds, salvage chances, sensor tables, mission score values, alien budget priorities, Doomsday defence counts, all text, names, news, sound cue table | Star map generation; every mission's composition, waves and objectives; all AI (strategic, financial, tactical, Doomsday, "fluff" encounters, 32 agent classes); the campaign arc (messages, components, final mission); research **yields** (what a topic unlocks: all `Yield` keys are 0); the meaning of each score index; funding and evaluation formulas; report and news triggers; the debris table; module meshes and textures; star and planet visuals; limits (85 systems, 20 bases (8 in the UI), 33 modules per base, 8 players) |

**Consequence:** the `interceptor` mod must express everything in the right-hand column as
data. That column is therefore the hard part of the format, and each row has a section below.

## Campaign arc

| Element | Rule | Src |
|---|---|---|
| Start | 1 Jan 2067. X-COM: 1 base, 3 corporate outpost systems nearby, 3 Lightning II, 5 pilots, cargo of 5 Sensor Probes and 10 X-Winders. Aliens: 2 bases, 6 outposts, at least 1 parsec away | G (D) |
| Starting base | Cargo, Communications, Hangar, 2 laser turrets, Personnel, Probe Launcher, Standard Downlink, Standard Sensor | G |
| Story gate chain | Alien Message 1 (not before 1 Feb; chance per combat 7/7/5/5/4% by difficulty, or automatic later) → aliens ship Structural Components → Message 2 (≥1 Mar; needs Msg 1 + a captured Structural Component + Wraith examined) → Message 3 (≥1 May; + Spectre examined; Weapon Components start) → Message 4 (≥1 Jul; + Phantasm examined + Weapon Component; Drive Components start) → Message 5 (≥1 Oct; + Drive Component; gateway map) | G (D) |
| Doomsday components | Built one type at a time. Totals per difficulty: 25/25/25/20/18 components, split Structural/Weapon/Drive (e.g. Beginner 8/8/9, Superhuman 7/6/5). Convoys pass near X-COM space with probability 65→45%. On the three easiest levels, a fully shipped Structural or Weapon type you never captured is granted. All Drive Components delivered = **loss** | G (D) |
| Final preparation | Alien Hull Alloys (Super Avenger), Message 5, Dimension Jump Drive, Nova Bomb researched; gateway (a black hole) found by probes; "Summon MacArthur" (arrives in about 1 week) | G |
| Final mission I | Outside the black hole vs a large fleet. Fail if the MacArthur dies or no Nova-armed Super Avenger survives | G, CD strings `FINAL1_MACDEAD`, `FINAL_NOVASGONE` |
| Final mission II | Inside the "pocket universe": 7 squadrons, a base, maybe mines; Doomsday weapon untargetable; ships rearmed (repaired on low levels). Total victory: Nova Bomb delivered and you return to within 1 km of the MacArthur; partial victory: bomb delivered, you are lost | G |
| Defeat | Doomsday completed (`DD_TOOLATE`), last base destroyed (`S_LASTBASEDEAD`), final mission failed | CD V, G |
| Warnings | `DD_WARNING1..3`; MacArthur prompts `MAC_PROMPT1..7` | CD V |

## Strategy map

| Element | Rule | Src |
|---|---|---|
| Generation | Procedural each campaign (`universe.cpp`); no map file. Star names from a pool of 100. Limits: 85 systems, 2D grid of 60000 units | CD V |
| Stars | d100 spectral class: red giant 1–2, blue giant 3–6, black hole 7–10, white dwarf 11–13, blue 14–24, white 25–35, yellow 36–70, orange 71–85, red dwarf 86–100. At least one black hole (the gateway). Black holes hidden; probes lost in them | G |
| Bodies | 0–5 satellites in 5 orbit slots; asteroid fields mostly inner; planet size small/medium/large/huge; mineral richness low/medium/high by spectral class | G |
| Locations | `DEEPSPACE{x,y}`, `SYSTEM{id,x,y,orbit}`, `BASE`, `OUTPOST`, `OPP{outpost,opp}`, `DOOM_PORTAL`, `POCKET`, `FLIGHTPLAN` | CD V |
| Travel | Free hyperjumps on the 2D map (no lanes). Range limited by engine endurance: 24/36/48 h. Ships stranded without fuel are lost. Ambush chance grows with distance from home | G |
| Time | Real time, 6 speeds, auto-pause on events; weekly reports (`WEEKLY1..40`); monthly evaluation | G, CD |
| Sensors | Rings around bases (long), ships, outposts, probes. Sensor Probe 0.75 ly, 4–6 weeks; Hunter Probe roams, 3–4 weeks. Destroyed base sensor → about 0.25 ly | G |
| Orders in flight | Need a Communications module at the squadron's base | G |

## Factions and relations

| Party | Mechanics | Src |
|---|---|---|
| X-COM | The player | — |
| Aliens | Full strategic opponent with the same rules plus cheats (rough knowledge of X-COM's start, off-the-shelf equipment, outposts always rebuilt). Five races, lore only | G, CD |
| Corporations | 87 names; each outpost system's corporation sets a **funding share** 0–95%, moved monthly by evaluation; at 0% it leaves for good. Only source of money, supplies, manufacturing, recruits. Corporate fighters guard outposts; escorts can be hired (10% of an order) | G, CD |
| Kabron pirates | Hostile; one base. Either destroy the base ($1M) or defend it from aliens; both end organised piracy | G |
| Cult of Sirius | Rogue freighter side mission | G |
| Arms dealer | Secret buyer of everything sold; resells to aliens; one bust mission | G |
| GSC | Research body; science stations; provides the MacArthur | G |

## Economy

| Element | Rule | Src |
|---|---|---|
| Treasury | Starting 5.0/4.75/4.5/3.75/3.0 M by difficulty (CD `StartingTreasury` values V) | CD, G |
| Outpost yield | Kilotons/month by richness × planet size (200–1300; asteroid OPP 800–1600) × `ExchangeRate` (250→150 $/kt by difficulty) × 0.75 X-COM share × funding % | CD, G (D) |
| Evaluation | Monthly per system: (X-COM activity here + overall/10) − (alien activity here + overall/5) → funding change bands (±1…±20%) | G (D) |
| New outposts | Sum of evaluations → number of d10 rolls (none/one/two/three); +1 per won Recon mission | G (D) |
| Expenses | Module maintenance; hull upkeep 12k/18k/22k; pilot salaries (formula from stats and rank); recruitment fees | CD, G |
| Buying | Items with `ProdTime=0` bought outright; else manufactured by outpost technicians (10 + 10 per OPP) on one global queue with priorities 10/25/50/100% | G |
| Delivery | One freighter per outpost, 100 CSU; a lost freighter loses its cargo | G |
| Selling | To the arms dealer; interceptors and Nova items never resold; ships sell at 0.8 × (hull + parts) | G |
| Storage | Cargo module 1000 CSU; full storage blocks orders and jettisons loot | G |
| Score | `MissionScores` S0–S99 (−5000…+4500); index meanings hard-coded; ratings Phenomenal … Dismal by monthly total | CD V, G |

## Research

| Element | Rule | Src |
|---|---|---|
| Topics | 71 `R_*` sections; tech ids 0–73 (3 placeholders), 74–100 alien ships and installations, 127 = special. `Level` 0–3, `FileSize` 8–75 (k TQ), `Prereq1..3` | CD V |
| Process | Research happens on Earth; each base **downloads** one topic through up to 3 downlinks (30/35/40 TQ/h); switching loses progress; jammer probes pause a downlink | G |
| Discovery | Prerequisites; salvage (per-item survival chance by difficulty, `*Goodies`); **capture** of disabled ships (Wraith, Spectre, Phantasm mandatory); alien messages | CD, G |
| Yields | What each topic unlocks is hard-coded (the `Yield` keys are all 0); the guide's tree gives it | CD V, G |
| Text | `short1.txt`: per topic a found name, a researched name and a description | CD V |

## Bases and modules

| Element | Rule | Src |
|---|---|---|
| Limits | 8 bases (UI); 33 attachment points: 15 small, 12 medium, 3 large (hangars), 3 special (downlinks); per base 1 shield, 1 sensor, 1 comms, 3 hangars, 3 downlinks, 10 weapons | G, CD |
| New base | Within 1.5 ly of an existing base; cost (1.5M + standard modules) × 0.75; build time = slowest module + 1 week; dismantle refunds 50% | G |
| Modules | 68 `M_*` sections (35 X-COM, 33 alien): `Cost`, `ProdTime` (days), `SizeClass`, `Maintenance`, `HitPoints`, tech requirements, `Rounds` for missile defence | CD V |
| Effects | Hangar 3 ships, Personnel 5 pilots, Medical heals ×2, Training raises stats, Cargo 1000 CSU, Comms, Probe Launcher, Downlinks, Shields 2000/3000/4000, Sensors (range, which weapons the base fields), ECM 1/2 km, Damage Control ×2 repair, Tractor, beam and missile defence | G |
| Destruction | Hangar destroyed → ships inside lost; Personnel/Medical → occupants die; Cargo → contents lost | G |
| Visuals | Generic module mesh retextured per type with normal / construction / destroyed textures; alien pods normal / `-b` | CD V |

## Ships, equipment and pilots

| Element | Rule | Src |
|---|---|---|
| Hulls | 20 `Ship_*` sections (player, freighters, 6 alien types, drone, pirate, corporate, Patton): handling, mounts (`MaxBeamWeapons`, `MaxMissileWeapons`, `SpecialSystemMounts`), hit points, cost, upkeep | CD V |
| Slots | Beam and missile hardpoints; one armour, engine, sensor, shield slot; special systems; missiles need a launcher | G |
| Equipment | 23 sections: armour ×3, shields ×4, engines ×3 (power 75/175/300, endurance), ECM ×2, sensors ×3 (lock range, time, cone; which missiles), cloak, jump drive, damage control, tractor, VCT, 3 probes | CD V, G |
| Power | 100 points shared between engine, beam recharge, shield recharge, specials (tractor 33, ECM 25, cloak 50 + 5%/s); capacitor 1000; afterburner 50/s | G, CD |
| Stock designs | Named configurations; stock price = hull + 0.9 × parts | G |
| Pilots | 8 stats with generation ranges; rank rolled d100; experience from kills and missions; rank thresholds by difficulty (`RankAdvancement*` V); training; injury and health; bravery and fleeing; 10 voice sets, MFD talking heads | CD, G |
| Missions | Up to 5 ships; player picks which to fly; wingman orders | G |

## Missions

| Element | Rule | Src |
|---|---|---|
| Types | 27 briefing types + 3 Doomsday; 56 debrief outcomes; 12 combat-simulator types; 7 side missions | CD V |
| Generation | From the strategic simulation: alien squadrons on flight plans meeting X-COM assets, corporate and GSC requests. Squadron sizes Small 1–3 … Huge 10+ | G |
| Defensive | Deep-space ambush, probe, freighter (win if it jumps out), OPP, outpost, base (corporate fighters and station guns help); refusing costs score | G |
| Offensive | Intercept, attack probe (later with mines and hidden escorts), freighter convoy (drones from about month 6), mining platform, outpost, base (damage = one pod; destroy = all; hangars grounded about 20 days) | G |
| Side missions | Hyperwave anomaly (Doomsday freighter), corporate recon (+1 outpost), rogue freighter ($250k), Kabron pirates ($1M base), pirate base rescue, arms dealer, hijacked Nova Bomb; cheat codes confirm discrete triggers | G |
| Leaving | Hyperspace out with countdown (no salvage) | G |
| Outcome | Win/draw/lose; salvage roll; capture; score; pilot experience, injury | G, CD |

## Alien strategy

| Element | Rule | Src |
|---|---|---|
| Priorities | Exploration, expansion, aggression (rises when X-COM is passive or losing) | G |
| New bases | On several conditions; each brings 2 outposts; a lost last base is replaced | G |
| Outposts | Monthly roll, more likely later; cap 16 | G |
| Offence ladder | Attack probe → freighter → OPP → outpost → cripple base → destroy base → decoy-and-strike | G |
| Economy | 100% of outpost yield; spending Doomsday > ships and bases > research; `Think.ini` budgets (`ResearchDefault`, `GoodsDefault`, `DoomsdayDefault`, 15-day periods) and per-module priorities | CD V, G |
| Tech | 3 levels bought with money (≈0.8–4 M); data receptor pods fund up to $20k/week | G |
| Gateway defence | `Think.ini` `MaxPortalDefenses*`, `MaxDoomsdayDefenses*` per type and difficulty | CD V |

## Space objects and models

| Element | Rule | Src |
|---|---|---|
| Class → mesh | String table in the exe (21 entries) | CD V |
| Debris | 28 named sets × up to 5 chunk meshes; generic chunks; alien corpse parts; `shokwave` | CD V |
| Damage | No ship damage textures; modules: destroyed texture | CD V |
| Effects | Three-ring meshes per hull size (`1ASH*`, `1SH3*`, `FRSH*`…: shield or jump effect, U); explosion sprites | CD |
| Liveries | `DM1–8 A/B/C`: multiplayer colours of the three hulls | CD V |
| Stations | X-COM base (hub + modules), corporate outpost, OPP, alien base (hub + pods; damage zones), alien outpost, alien mining platform, pirate base, arms dealer, science station, Nova station, Doomsday weapon, landing pad | CD, G |
| Station HP | 11 charts `HP1..HP4`, `SD` (parts and a value of unknown meaning) | CD V |

## Interface

| Screen | Function | Src |
|---|---|---|
| CSD (star map) | Sensor rings; zoom map/system/object; time controls; object info and squadron reports; Launch Probe; Launch Mission (pick ships, destination; Summon MacArthur); event prompts | G |
| Bases | 3D base view with attachment points; add/dismantle modules; build/dismantle bases; base status (overlays normal/defences/damage) | G, CD |
| Research | Topics, current download, size, progress, ETA, jammed downlinks | G |
| Equip Ship | Hardpoint editor; stock designs; Assign Pilots (training, medical, sack) | G |
| Equipment & Personnel | Purchase/manufacture per base; production queue with priorities and destination; escorts; selling; recruiting | G |
| Status Reports | Monthly rating and funding, player status, alien activity, outpost data | G, CD |
| UFOpedia | 9 categories, about 150 entries (`DATABANK_*` strings) | CD V |
| Briefing / debriefing, Hypernews, combat simulator, load/save, configuration, difficulty, multiplayer | — | CD |
| Comms | Wingman orders; MFD talking-head clips (113 Smacker clips in `mfdanim.mps`); interactive music sets with cruise/threat/battle states | CD V |

## Difficulty

Five levels: **Beginner, Experienced, Veteran, Genius, Superhuman**. They change: starting
money, exchange rate, alien ship counts and tiers, alien shields, armour, weapons, rate of fire,
skill and aggression, cloak-breaks-lock chance (50→30%), alien tech speed, Doomsday size, salvage
survival, message chance, convoy visibility, rank thresholds, final-mission repair, auto-granted
components (G; CD tables V where noted above).
