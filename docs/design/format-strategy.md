# Format: strategy layer

**Status: design; implemented in [Stage 4](../implementation-plan/stage-4-strategy.md) (map,
economy, research, bases, ships, pilots) and [Stage 5](../implementation-plan/stage-5-campaign.md)
(relations, strategist).**
Files in `strategy/*.ini`. Numbers that vary by difficulty take one value per level; any
quantity the original computes with a formula takes an **expression** in the
[condition language](format-behaviour.md#conditions) (arithmetic over campaign queries), so the
formulas are data too.

## Star map

### `[map:<id>]`

```ini
[map:frontier]                       ; interceptor: generated like the original
Kind = generated
Size = 60000, 60000                  ; map units (2D)
Unit = 0.000167 ly                   ; map unit → light years (so 1.5 ly ≈ 9000 units)
Systems = 30-45                      ; or a table rolled on d100
StarClasses = table:spectral         ; d100 table → star class
Bodies = 0-5, slots: 5, table:bodies
Names = text:star_names              ; pool
Guarantee = black_hole >= 1          ; the gateway
Travel = free                        ; free hyperjumps, or `lanes`
Hidden = black_hole                  ; not drawn until found
Seed = random                        ; or a number for a fixed campaign

[map:reach]                          ; default campaign: a fixed, hand-made map
Kind = fixed
System.mercury_gate = pos: 1200, 800, class: yellow, bodies: planet:rocky, belt:thin
System.ajax = pos: 4300, 1500, class: red_dwarf, bodies: planet:ice, belt:rich
Lane.1 = mercury_gate - ajax, length: 2 days
Travel = lanes
```

| Construct | Covers |
|---|---|
| `[starclass:<id>]` | Visual (sun model, colour, size), d100 range, mineral richness table, hazard (black hole: hidden, swallows probes, battle gravity). |
| `[body:<id>]` | Planet, moon or asteroid field: size, richness, the backdrop model used in battles in its orbit, what can be built there (OPP, outpost). |
| `Travel` | `free` (any point, range-limited: the original) or `lanes` (only along lanes: simpler, used by the default campaign). |
| `[location:<kind>]` | Location kinds a mission can happen at: `deep_space`, `system`, `orbit`, `base`, `outpost`, `opp`, `gateway`, `pocket` (a separate battle space reached only by story). |

### Sensors and probes

`Sensor` keys on bases, ships, outposts and probes give a **map range**; contacts are visible
inside the union of ranges. Probes are object classes with `MapRange`, `Lifetime`, movement
(`station`, `roam`) and effects (`reveal`, `jam: downlink`, `hunt`).

## Factions and relations

### `[faction:<id>]` (extended)

```ini
[faction:corporate]
Name = @faction.corporate
Kind = patron                ; player, rival, patron, independent, hostile
Relation.xcom = funding      ; relation is a per-holding funding share
Funding = 0% - 95%, start: 50%
Leaves = funding <= 0%       ; holding lost for good
Names = text:corporations
```

| `Kind` | Meaning | Interceptor | Default campaign |
|---|---|---|---|
| `player` | The player | X-COM | Interplanetary Guard |
| `rival` | A strategic opponent with its own economy, research, plan and AI | Aliens | Umbral Dominion |
| `patron` | Funds the player; relation per holding or overall | Corporations | Solar Senate |
| `independent` | Has a relation score; can trade, help or turn hostile | Arms dealer, GSC | Nebula Cartel, Belt Prospectors |
| `hostile` | Attacks everyone; can be bribed or destroyed by events | Kabron pirates, Cult of Sirius | Black Comet raiders |

Relations are numbers (−100…100) or funding shares, changed by actions (`add(relation(x),
n)`) and by evaluations. Conditions read them.

### Dynamic relations (optional per campaign)

The original's relations are fixed except for corporate funding. A campaign may instead switch
on **dynamic relations**: every pair of factions has a standing that drifts with what happens,
and the standing decides who supports whom and who fights whom, so the player's actions decide
their allies over time.

```ini
[relations]
Dynamic = yes                         ; no = hostility only from [faction] Hostile (Stage 1 rule)
Scale = -100, 100
Stances = hostile <= -40, wary <= -10, neutral < 20, friendly < 60, allied
; Initial standings (unlisted pairs start at 0); symmetric unless written both ways
Start = guard-cartel -10, guard-senate 30, cartel-senate -50, dominion-* -100

; How events move standings: who did what to whom
[relationrule:defended]
On = battle_won where: ally_of_victim(asset_owner)     ; you saved their base, convoy or ship
Change = asset_owner +8 (base +15, convoy +6, ship +2)
Also = enemies_of(asset_owner) -3

[relationrule:attacked]
On = destroyed(by: player) or captured(by: player)
Change = owner -10 (base -40, outpost -25, freighter -10, fighter -3)
Also = enemies_of(owner) +4

[relationrule:refused]
On = defence_request_refused
Change = requester -6

[relationrule:drift]
Every = 1 month
Change = towards 0 by 2               ; old grudges fade
```

| Stance | Effect in battle | Effect in strategy |
|---|---|---|
| `allied` | Their ships fight on your side; you may call for help | They share sensor data, sell at a discount, may fund you, ask you to defend them |
| `friendly` | Their ships help if attacked nearby | Trade; occasional requests |
| `neutral` | Ignore you unless attacked | Trade at normal prices |
| `wary` | Ignore you; return fire | Refuse to trade; no requests |
| `hostile` | Attack on sight; their bases and convoys become targets | Raid your convoys; may ally with your enemies |

Stance changes fire events (`event:on_stance_change` with the faction and old and new
stances), so a campaign can react with messages, new missions or story steps. Battle hostility
is taken from the stances at the moment the battle starts. The `interceptor` campaign sets
`Dynamic = no` to match the original, with corporate funding as its only moving relation; the
default campaign sets `Dynamic = yes`.

### Evaluations: `[evaluation:<id>]`

The monthly funding review is an evaluation: a period, an expression per holding, and a table
mapping the result to changes.

```ini
[evaluation:funding]
Period = 1 month
Each = outpost system
Value = activity(xcom, here) + activity(xcom) / 10 - activity(alien, here) - activity(alien) / 5
Change = < -1000: -5..-20%, < -500: -2..-10%, < -250: 0..-5%, < 0: 0..-1%, == 0: 0, \
         <= 250: 0..1%, <= 500: 0..5%, <= 1000: 0..10%, > 1000: 2..10%
Then = new_outposts(rolls: sum / 1000, table: d10 1-3 none, 4-7 one, 8-9 two, 10 three)
```

## Economy

| Section | Defines | Interceptor | Default |
|---|---|---|---|
| `[resource:<id>]` | Something mined, stored and sold | Ore (kt) | Cosmium |
| `[income:<id>]` | A recurring income: source, period, expression | Outpost yield × exchange rate × 0.75 × funding | Senate stipend per protected colony + cosmium sales |
| `[expense:<id>]` | Recurring costs | Module maintenance, hull upkeep, salaries | Same, fewer |
| `[market:<id>]` | Who sells what, prices, availability, and who buys | Outposts sell; the arms dealer buys (and arms the aliens) | Senate depot sells; the Cartel buys and sells (and may supply the Dominion) |
| `[factory:<id>]` | Manufacturing capacity and queue rules | Outpost technicians (10 + 10/OPP), priorities 10/25/50/100% | One shipyard per base |
| `[delivery:<id>]` | How goods arrive | Outpost freighters (100 CSU; lost with cargo) | Supply rocket |
| `[storage]` | Units and limits | CSU; cargo module 1000; full = no orders, loot jettisoned | Same rule |

Item economy keys go on the items themselves: `Cost`, `SellPrice`, `ProdTime` (0 = bought off
the shelf), `Size` (storage), `Maintenance`, `Requires`.

## Research

```ini
[research:tri_titanium]
Name = @research.tri_titanium            ; found name / researched name / text keys
Size = 18000                              ; work units (the original's TQ)
Level = 1
Requires = research(wraith_examination)
RequiresAny =                              ; alternative prerequisites
Appears = salvage(wraith_hull) or captured(wraith)   ; how it becomes available
Unlocks = item:tri_titanium_armor, hull:firestar, module:...  ; the original's hard-coded yields
Encyclopaedia = @db.tri_titanium

[research_model]
Mode = download                           ; or `labs`
Rate = module_count(downlink_std) * 30 + module_count(downlink_hyper) * 35 + ...
PerBase = 1 topic, max 3 downlinks
Switching = lose_progress
Jammed = probe(data_jammer) near base: pause one downlink
```

- `Mode = download` is the original (research done elsewhere, bandwidth-limited);
  `Mode = labs` is the familiar X-COM model (staff in base labs) for other campaigns.
- **Discovery** routes: prerequisites, salvage, capture, events (`unlock(research: x)`).
- **Salvage** is `[salvage:<id>]`: per item type, survival chance by difficulty (the
  original's 13 `*Goodies` sections), what it yields (items, research topics, score).
- **Capture**: a disabled object can be towed or held until victory; its class's `Capture`
  key gives what it yields.

## Bases and modules

```ini
[basetype:xcom]
Hub = object:xcom_hub
MaxBases = 8
Placement = within(1.5 ly, own base)
Cost = (1500000 + template_cost) * 0.75
BuildTime = slowest_module + 7 days
Dismantle = refund 50%, sell stock, dismiss pilots
Limits = shield 1, sensor 1, comms 1, hangar 3, downlink 3, weapon 10
Template.new = best(beam_defence)*2, best(downlink), best(shield), best(sensor)
```

Modules are object classes ([objects](format-objects.md#modules-moduleid)) with `Effects`:
named campaign quantities they add while working (`hangar_capacity +3`, `pilot_capacity +5`,
`heal_rate x2`, `storage +1000`, `research_rate +30`, `base_shield 2000`, `sensor_range
40000`, `orders_in_flight`, `probe_launch`, `repair_rate x2`, `ecm 1km`), and `OnDestroyed`
actions (`lose(ships in hangar)`, `kill(pilots in module)`, `lose(stock)`). A destroyed module
stays as a wreck until repaired or dismantled.

## Ships and equipment

```ini
[hull:firestar]                ; strategy data for a ship class (same id as [ship:firestar])
Slots = beam 4, missile 3, special 4, armor 1, engine 1, sensor 1, shield 1
Cost = 550000
ProdTime = 18000
Upkeep = 18000 / month
Requires = research(tri_titanium)

[item:graviton_pulse]
Kind = engine                  ; beam, launcher, missile, armor, shield, engine, sensor, special, probe, mine
Special = power: 300, endurance: 48h
Cost = 185000
Requires = research(graviton_pulse)

[design:lightning_stock]       ; named configuration
Hull = lightning
Fit = beam: laser_cannon*2, missile: xwinder_launcher*2, engine: conversion_drive, ...
Auto = best                    ; stock designs upgrade to the best available item
Price = hull + 0.9 * parts
```

Loadout rules (`Fits` on items: which slots, minimum sensor for a missile, power draw) are
item keys; the Equip Ship screen enforces them.

## Pilots

```ini
[pilots]
Stats = intelligence 50-100, accuracy 25-100, flying 25-100, aggression 25-100, \
        bravery 25-100, psionics 1-100, stamina 50-100, health 6-15
Ranks = rookie 40%, ensign 20%, lieutenant 20%, lt_commander 15%, commander 5%
RankUp = experience >= 2, 10, 24, 38 @ experienced …          ; per difficulty
Experience = kills / missions + missions
KillValue = fighter 1, freighter 1, outpost 10, opp 5, probe 5, mine 0.25
Salary = (48000 + modifiers) / 12
Training = accuracy, flying, psionics + ceil(0.1 * stat * intelligence / 100) per month
Injury = health 5: accuracy, flying x0.5; health 2: x0.25; 0 unconscious; < 0 dead
Bravery = kill +5, outpost +10, base +25, own_base_lost -25; <= 10: flee 50% per 30s
Recruit = pool 10, refresh monthly, fee 10000 + 24000 + modifiers
Names = text:pilot_names; Voices = voice:m1..m5, voice:f1..f5
```

Every formula is an expression, so another campaign can use completely different pilot rules
(or set `Systems` without `pilots` and have none).

## Rival strategy: `[strategist:<id>]`

The rival is driven by a strategist: budgets, priorities and an offence ladder, with
conditions deciding when it expands or attacks. It replaces the original's hard-coded strategic
AI and reads its `Think.ini` numbers.

```ini
[strategist:alien]
Faction = alien
Income = 100% of outpost yield
Budget = plan 50%, forces 35%, research 15%     ; Think.ini ResearchDefault/GoodsDefault/DoomsdayDefault
Period = 15 days
Aggression = start: think:StartingAggression; +when: player_passive; -when: player_winning
Explore = probes, patrols
Expand = new_base when: player_aggressive or lost_base or plan_starved or outposts_exposed
Outposts = monthly chance: 5% + day / 30 * 1%, max 16
Offence = attack_probe, attack_freighter, attack_opp, attack_outpost, cripple_base, \
          destroy_base, decoy_strike                 ; ordered by aggression
Defence = outposts > bases > opps > freighters
Tech = levels 3, cost: 800000-4000000, funded_by: module(data_receptor) * 20000 / week
Cheats = knows_player_start_area, instant_equipment, outposts_rebuilt
BuildPriorities = think:module_priorities       ; the 31 module sections of Think.ini
```

The strategist launches **squadrons** on **flight plans**; when squadrons and player assets
meet, a [mission](format-missions.md) is generated.
