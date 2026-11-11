---
spec_version: 1.0.0
status: current
updated: 2026-11-11
---

# The default campaign

**Status: design; built up stage by stage ([plan](../../implementation/plan/index.md)). First
playable content delivered in Stage 2** (`packs/default`, which replaced the Stage 1 `sample`
pack). Mod id `default`, kind `campaign`, **no original content**. Working title: ***Perils of
the Void***.

It is what the engine runs when no CD is present, the reference implementation of a complete
campaign, and the template a modder copies to make their own. It uses every format construct
the `interceptor` campaign uses, in a simpler form ([coverage](../../specification/design/coverage.md)).

## Aesthetic: a 1950s film serial

The look and the story come from the **science-fiction cinema serials and pulp magazine covers
of the late 1940s and 1950s** — as a genre, not any particular serial or character:

- **Ships**: stylised rocket ships (needle and cigar hulls, swept tail fins, nose cones,
  porthole rows, chrome trim, exhaust bells) against **flying saucers** (domed discs, rim
  lights, antenna spikes) and **tin-can robots**. Low-poly forms with smooth shading and
  saturated two-tone paint.
- **Palette**: rocket-red, cream, chrome and teal for the player; violet, acid green and black
  for the villain; Technicolor nebulae and painted-backdrop planets with rings.
- **Interface**: art-deco panels, Bakelite knobs, radar scopes with phosphor green sweeps,
  Nixie-tube counters, ticker-tape messages; screen titles in poster lettering.
- **Story**: told in **chapters**. Each opens with a title card ("Chapter 4: Trapped in the
  Ion Storm!") and a narrator's recap. Each ends on a **cliffhanger**: a special mission that
  begins at the worst possible moment. The next chapter resolves it.
- **Planets**: battles also happen *on* worlds — jungle planets under violet skies, airless
  cratered moons — where rockets fly like aircraft: drag slows them, gravity pulls them down,
  and fights are slower and closer to the ground
  ([environments](../../specification/design/format-missions.md#battle-environments-environmentid)).
- **Sound**: theremin and organ stings, ray-gun zaps, rocket roars, a brassy title fanfare,
  all synthesised.

## Premise

The inner worlds are powered by **cosmium**, a glowing crystal found only in the asteroid
belts and moons of six frontier systems. The **Interplanetary Guard** (the player) protects the
mining colonies. From a dark moon at the edge of the map, **Regent Vexor of the Umbral
Dominion** has begun building the **Eclipse Engine**: a colossal lens that, fed with enough
cosmium, will cast the inner worlds into permanent night unless they kneel.

Both sides race to mine the belts, research new technology, build stations and fight for every
system. The Regent needs cosmium for his Engine; the Guard needs it to stop him.

## Factions

| Faction | Role | Kind | Notes |
|---|---|---|---|
| **Interplanetary Guard** | The player | `player` | Rocket ships; bases with modules |
| **Umbral Dominion** | The opponent; its diabolical plan drives the campaign | `rival` | Saucers and robots; own economy, research tree, bases, strategist |
| **Solar Senate** | Third party: the government | `patron` | Funds the Guard per protected colony; can cut funding, demand missions, or (if relations collapse) recognise the Regent and stop funding altogether |
| **Nebula Cartel** | Third party: smugglers and traders | `independent` | Sells rare parts to whoever pays; secretly supplies the Dominion while relations with the Guard are poor; can be turned into an ally or broken |
| **Belt Prospectors** | Third party: free miners | `independent` | Their mines yield cosmium to whoever protects them |
| **Black Comet raiders** | Third party: pirates | `hostile` | Raid everyone; may be hired by the Cartel or the Regent |

**Relations are dynamic** ([format](../../specification/design/format-strategy.md#dynamic-relations-optional-per-campaign)).
Defending a faction's bases, convoys and ships raises its standing with the Guard; attacking
them lowers it; and each faction's enemies react the opposite way. Over the campaign the player
can make the Cartel an ally (its convoys then supply the Guard and stop supplying the Regent),
lose the Senate's backing by neglecting its colonies, or drive the Prospectors into the
Regent's arms. Stances change who fights alongside or against the player in battle, who
trades, and which story branches open.

## Structure: chapters driven by both sides' progress

The campaign is one [arc](../../specification/design/format-campaign.md#arcs-story-progression) of **12 chapters**. A
chapter's cliffhanger mission is triggered by conditions on **the player's progress and the
Regent's plan**, not by the calendar alone, so a strong player meets the story faster and a
struggling one sees the villain advance.

The **Eclipse Engine** is a [plan](../../specification/design/format-campaign.md#rival-plans-planid) with three stages
(**Lens**, **Coils**, **Heart**). Each stage needs shipments of cosmium and parts carried by
saucer convoys, which the player can intercept, and the plan's progress is visible to every
chapter condition.

| Ch. | Title (working) | Cliffhanger triggered when | Special mission | Outcome |
|---|---|---|---|---|
| 1 | The Shadow Falls | Campaign start | **Tutorial defence**: saucers strike a mining colony | Unlocks the star map, first base |
| 2 | Raiders of the Belt | First mine built, or day 15 | **Convoy escort** through raider space | Prospectors' relation; cosmium income |
| 3 | The Regent's Ultimatum | Dominion builds its 2nd base, or Guard researches 4 topics | **Broadcast intercept**: catch a saucer carrying the Regent's message | Reveals the Eclipse Engine; plan stage 1 begins |
| 4 | Trapped in the Ion Storm! | Plan ≥ 15% and a Guard squadron is in deep space | **Survive**: an ion storm cripples sensors while saucers attack | Storm map hazard learned; derelict found |
| 5 | The Secret of the Derelict | Chapter 4 done | **Surface salvage**: the derelict has crashed on a jungle planet; fly down through the atmosphere and tractor it out under fire from saucers and ground robots | Research: *saucer propulsion* |
| 6 | Senate in Peril | Senate relation < 20 **or** plan ≥ 33% | **Defend the Senate station** from a robot boarding fleet | Senate relation ±; funding changes |
| 7 | The Cartel's Price | Cartel relation crosses friendly or wary | **Choice**: escort a Cartel shipment (Cartel ally) **or** raid it (cut the Regent's supply) | Branches: Cartel ally or Cartel hostile |
| 8 | Lens of Doom | Plan stage 1 complete | **Strike** the lens assembly yard | Delays stage 2 (or loses a colony) |
| 9 | The Traitor in the Guard | Guard researches *saucer propulsion* and Dominion research ≥ level 2 | **Pursuit**: chase a defector's stolen rocket before it reaches the Regent | One rival research topic stolen or kept |
| 10 | Into the Dark Moon's Shadow | Plan ≥ 66% **or** Guard holds 4 systems | **Surface recon**: fly low over the dark moon's airless craters, under the fortress guns, to photograph the Engine site | Map of the Engine site |
| 11 | The Eclipse Begins! | Plan stage 3 starts | **Desperate defence**: the Engine fires a test beam at a colony; save its people | Colony saved or lost; Senate reaction |
| 12 | Duel at the Eclipse Engine | Map found + *atomic torpedo* researched + flagship built | **Final assault**, two parts: break the fleet, then fly into the Engine's core | Victory (total or partial) |

**Victory**: destroy the Eclipse Engine (total victory if the flagship survives, partial if it
is lost). **Defeat**: the Engine completes; the Guard's last base falls; or the Senate withdraws
recognition (funding at 0 for 60 days).

## Logistics: mines, research stations and foundries (planned)

Features: [Strategy & logistics](../strategy-logistics.md) (F-STR-005 – F-STR-009).

The user has asked for the default campaign's strategy layer to run on **supply lines**
([design](../../specification/design/logistics.md); decisions recorded 2026-11-11):

- **New base types**: mining facility, research station, manufacturing center, built in star
  systems according to their resources; research stations only in systems meeting a research
  site's criteria (a nebula, a pulsar…).
- **Supply and demand**: the main base orders components (for fighters and other technology)
  from manufacturing centers; manufacturing orders materials from mining facilities; nothing is
  produced until its materials have arrived. Resources and components are campaign data.
- **Supply convoys** are automatic, provided by the government, and travel the fixed lanes as
  convoys of several large vessels (a core vessel with attached cargo containers), each carrying
  an even share of the cargo, so a partly destroyed convoy still delivers the surviving share.
  Where along its route a convoy will be attacked is chosen at random at launch; destroyed cargo
  is lost, captured cargo goes to the Dominion's supply depots (destroy a depot to seize it).
- **Research stations** open tech branches that ordinary bases cannot research (for example
  theremin shielding at a nebula lab); materials are needed only to begin a project.
- **Survey ships** — a different mission and craft from supply ships — are sent to systems
  within the base's radar range to find which station types they suit; when one launches, a
  25 % roll decides whether it will be attacked.
- **Fighters** are built at main bases and transferred to a facility's own pool, defending it
  and escorting its supply ships.
- **Construction** takes time and passes through three construction stages, each with its own
  mesh and debris (implemented: `build_25` from 0 %, `build_60` from 60 %, `build_90` from 90 %,
  thresholds confirmed); from 50 % the Dominion may
  attack a facility under construction, at most three times per location.
- **Third-party convoys**: Prospectors fly expeditions (to a system, pause, home); the Cartel's
  destinations are random, even uncharted stars.

This fits the premise — both sides race to mine the belts — and gives the Escort and Convoy raid
mission types a strategic cause. The Regent's Eclipse Engine shipments use the same convoys.
Owners: Stage 4 (economy and logistics), Stage 3 (convoy-ambush and construction-defence
missions), Stage 5 (the Dominion's depots and third-party convoys). Already built in Stage 2: the
facility and supply-craft objects, their construction-stage models and debris, and the
`construction-defence` and `convoy-ambush` testbed arenas. Still open: supply-vessel cargo
capacity (TBD, set by tech-tree balance) ([design page](../../specification/design/logistics.md#open-questions)). The `interceptor` campaign stays without
logistics.

## Content at a glance

| Area | Default campaign | Interceptor equivalent |
|---|---|---|
| Star map | **Fixed** map: 6 systems, 7 lanes, 2–4 bodies each, plus the dark moon | Generated Frontier, free travel |
| Player craft | 3 rockets: *Comet* (light), *Meteor* (medium), *Thunderbolt* (heavy) + the *Victory* flagship (story) | Lightning, Firestar, Super Avenger; MacArthur |
| Rival craft | *Scout saucer*, *Raider saucer*, *Dreadnought disc*, *Robot sentinel* (drone), *Grappler* (tug), *Cargo orb* (freighter) | Wraith, Spectre, Phantasm, Drone, Harbinger, Spirit |
| Third-party craft | Senate patrol rocket, Cartel freighter, prospector hauler, raider hot-rod | Corporate and pirate hulls, freighters |
| Stations | Guard base (hub + 8 module types), Senate station, Cartel depot, raider asteroid, Dominion base (hub + pods), Eclipse Engine; **ground**: mining colonies, research outposts, Dominion surface mines, the dark moon fortress | X-COM base, outposts, OPPs, alien bases, pirate base, arms dealer, science station, Doomsday weapon (no ground installations) |
| Weapons | Ray guns: *blaster*, *heat ray*, *disintegrator*, *ion ray*; rockets: *seeker*, *atomic torpedo*, *decoy*, *cluster*; equipment: radar jammer, tractor ray, cloak, auto-pilot (VCT) | 9 beams, 10 missiles, equipment |
| Research | 30 topics per side in 4 branches (Drives, Weapons, Defence, Industry) + 4 reverse-engineering topics from captured saucers | 71 topics |
| Missions | 11 generated types (2 on planet surfaces) + 12 chapter missions | 27 types + 7 side missions + final (space only) |
| Difficulty | 4 levels: *Cadet*, *Ranger*, *Captain*, *Space Ace* | 5 levels |
| Relations | Dynamic, 6 factions | Corporate funding only |

### Generated mission types

| Type | Goal |
|---|---|
| Intercept | Destroy a Dominion strike group before it reaches a colony |
| Defend | Hold a base, colony, mine or allied station |
| Escort | Bring a convoy (prospector, Cartel or Senate) through — and, with [logistics](../../specification/design/logistics.md), the Guard's own supply rockets (`convoy_ambush`) |
| Construction defence | Hold a facility under construction against a Dominion strike ([logistics](../../specification/design/logistics.md#mission-types-tied-to-logistics-stage-3), planned) |
| Convoy raid | Destroy or capture Dominion cargo orbs (slows the Engine) |
| Strike | Destroy a Dominion mine, pod or base |
| Patrol | Sweep a system; random contacts |
| Salvage | Tractor home a disabled saucer or derelict |
| Survey | Scan a body to reveal its cosmium |
| Raider hunt | Clear a raider nest (relation effects on Cartel and Prospectors) |
| Colony defence (surface) | Defend a mining colony or research outpost on a planet: atmospheric flight, gravity, anti-air guns, terrain |
| Ground raid | Destroy a Dominion surface mine or research outpost |

## Assets: all generated by this project

| Asset | How | Tool |
|---|---|---|
| Rockets, saucers, robots, stations, modules, probes, mines | Procedural low-poly OBJ from parametric parts: lathed hulls (rocket bodies, saucer discs, domes), fins, portholes, antennae, rings; LODs, slots and overlay groups | `tools/default_assets/models.py` (replaced the Stage 1 `make_sample_models.py`) |
| Debris families | Each model cut procedurally into 3–5 pieces along planes, giving the `Piece.n` models | same tool |
| Textures | Procedural two-tone paint with chrome trim, rivets, porthole glow, faction insignia, scorch overlays for damage states | `tools/default_assets/textures.py` (PNG, standard library) |
| Cockpits | A 3D cockpit per rocket (chrome dash, round dials, radar scope) and one 2D art-deco cockpit image | models.py / textures.py |
| UI art | Art-deco panels, buttons, dials, chapter title cards, poster-style screen titles, star map glyphs | `tools/default_assets/ui.py` |
| Sounds | Synthesised: ray zaps, rocket roar, saucer hum, explosions, theremin and organ stings, UI clicks | `tools/default_assets/sounds.py` (WAV) |
| Music | A title fanfare, an exploration loop and a battle loop, synthesised | sounds.py |
| Text | Chapter cards and narrator recaps, briefings, encyclopaedia, messages, names: written for the project in a pulp register, in `text/en/` | by hand |

All tools are Python standard library only and deterministic (fixed seeds); their outputs are
committed so the mod works without running them. Everything is MIT-licensed with the engine.
Names, characters and designs are original to this project; the serial genre is the
inspiration, not any specific work.

### Delivered so far (Stage 2, first delivery)

`python3 tools/default_assets/make_all.py` regenerates everything below (`--check` builds twice
and proves the output is byte-identical); `common.py` validates every model (closed solids wound
outward, no degenerate faces, triangle budgets) before it is written.

| Asset | Delivered |
|---|---|
| Ships | Guard rockets Comet, Meteor and Thunderbolt (playable) and the Victory; Dominion scout and raider saucers, the Dreadnought disc, the robot sentinel, the Grappler and the cargo orb; Senate patrol rocket, Cartel freighter, Prospector hauler, Black Comet hot-rod |
| Stations | Guard station, Dominion outpost, cosmium asteroid |
| Debris | Three to five pieces per class, plus generic rubble chunks |
| Textures | Hull paint per faction plus *scorched* and *burnt* damage variants |
| Cockpit | The 3D art-deco Guard cockpit `guard` (no 2D art), with bezels for every instrument including `power` and `systems`; rectangles measured by projecting the bezels with the cockpit camera |
| Sounds | Synthesised weapon, explosion, hit, lock, afterburner, saucer hum, theremin sting, UI, and Stage 2 system sounds (system damage, tractor, cloak, hyperspace; ECM reuses the target click) |
| Rules | YAML (mod format 2): seven factions, six beams and two missiles, systems and power rules (`rules/systems.yaml`), equipment fits by `systems+`, flight rules (`hyperspace_time: 8s`), behaviours for every enemy type and the Guard pilots (`rules/behaviours.yaml`) with aim, reaction and fire discipline per level, the campaign ruleset with four difficulty levels (`campaign/campaign.yaml`; default *ranger*) |
| Cockpit layout | The Guard cockpit's instruments are also **anchored** on the dashboard (drawn in perspective); `models/guard_cockpit.instruments.yaml` is generated with the model |
| Scenarios | `rocket-vs-saucer`, `dominion-raid`, `black-comet-ambush`, `systems-trial` |

Not yet: model sets with damage states, the 2D cockpit, UI
art, music, text.

## Built up by stage

Every stage's features must work in the default campaign as well as in `interceptor` (project
rule), so the campaign grows with the engine:

| Stage | Default campaign content added |
|---|---|
| 2 | The rockets, saucers and robots with model sets, damage states and debris; behaviours with difficulty variants; weapons and equipment; a "rocket vs saucer" testbed arena |
| 3 | The generated mission types (including the two surface types, with atmospheric flight and terrain) and chapter missions 1, 2, 4 and 5 as playable missions; the condition/action language exercised by objectives |
| 4 | The fixed star map, economy, research trees, bases and modules, purchase and loadout, pilots |
| 5 | Factions with dynamic relations, the Dominion strategist, the Eclipse Engine plan, all 12 chapters: a complete playable campaign |
| 6 | Screens skinned with the art-deco UI, chapter title cards, music, narrator text |
| 7 | Balance pass, complete encyclopaedia, the modding tutorial built on this campaign |

## Revision history

| Version | Date | Change |
|---|---|---|
| 1.0.0 | 2026-11-11 | Adopted into Definition 1.0.0 as background for the campaign features; logistics updated with the revised decisions (survey ships separate from supply ships, supply convoys, construction thresholds 0 / 60 / 90 %) |
