# Coverage matrix

**Status: design check, 2026-11-10.** For every system of the original
([analysis](original-analysis.md)): the construct that expresses it, where the `interceptor`
mod gets the data, and the simplified equivalent in the [default campaign](default-campaign.md).
The second table goes the other way: features the default campaign adds, and what they mean for
`interceptor`. A row with no construct would be a gap in the format. **There are none**; the
open items at the end are data the original hides in code, which must be recovered, not format
features that are missing.

Data source key: **INI** = imported from the original INI files by `inherit: original:…` (mod files themselves are YAML);
**TXT** = original text by reference; **AUTH** = authored in the mod from the strategy guide and
play-testing (the original hard-codes it); **EXE** = a table read from the executable during
analysis and written into the mod as data.

## Original → format → default campaign

| Original system | Format construct | `interceptor` data | Default campaign equivalent |
|---|---|---|---|
| Start state (date, base, outposts, ships, pilots, stock) | `starts:`, base templates | AUTH (guide) | Start at one base with two colonies, 3 *Comets* |
| Five difficulty levels and per-level tables | `campaign: difficulty` list; `{per_level: […]}` values; `variants:` (all implemented) | INI (difficulty sections, `*Goodies`, `RankAdvancement*`, `Think.ini`) + AUTH | Four levels |
| Doomsday story chain (messages 1–5, gates, chances) | `arcs:` of `events:` | AUTH; text TXT | 12-chapter serial arc |
| Doomsday device (3 component types, counts per difficulty, convoys, visibility, auto-grant, completion = loss) | `plans:` with stages, shipments, `grant_if_missed`, `complete` | AUTH (guide) + INI (`Think.ini` defence counts) | The Eclipse Engine plan (3 stages, cargo orb convoys) |
| MacArthur, final mission I and II, total/partial victory | Events, mission types with `environment: pocket`, `victory` list, `endings:` | AUTH; strings TXT | Flagship *Victory*, two-part final assault |
| Defeats (Doomsday complete, last base, final failed) | `defeat` list | AUTH | Engine complete, last base, Senate withdraws |
| Generated star map (classes, bodies, richness, black holes, names) | `maps:` with `kind: generated`, `star_classes:`, `bodies:`, name pools | AUTH (guide tables) + TXT (names) | **Fixed** map, 6 systems |
| Free hyperjump travel, engine endurance, stranding | `travel: free`, item `endurance` | INI (engines) + AUTH | `travel: lanes` |
| Time compression, auto-pause | `time_speeds`, `pause` | AUTH | same |
| Sensors, probes (range, lifetime, roam, jam, black-hole loss) | `sensor`, probe classes, `star_classes:` hazards | INI (probe equipment) + AUTH | Radar probe, jammer probe |
| Corporations: funding share per outpost, monthly evaluation, leaving at 0% | Faction `kind: patron`, `relation: funding`, `evaluations:` | AUTH (guide formula and bands) + TXT (87 names) | Solar Senate stipend driven by dynamic relations |
| New outposts from evaluations and recon | Evaluation `then`, event actions | AUTH | Prospector colonies join when relations are friendly |
| Kabron pirates, Cult of Sirius, arms dealer, GSC | Factions (`hostile`, `independent`) + side-mission events | AUTH | Black Comet raiders; Nebula Cartel; Belt Prospectors |
| Treasury, exchange rate, outpost yield | `resources:`, `incomes:` expressions | INI (difficulty) + AUTH (yield table) | Cosmium yield and sales |
| Maintenance, upkeep, salaries, recruitment fees | `expenses:`, item `maintenance`/`upkeep`, pilot formulas | INI + AUTH | same, fewer items |
| Purchase vs manufacture, technicians, priorities, global queue | `markets:`, `factories:`, item `prod_time` | INI (`ProdTime`, `Cost`) + AUTH | One shipyard per base |
| Freighter delivery, cargo units, losses | `deliveries:`, `storage:` | AUTH | Supply rocket |
| Selling to the arms dealer (feeds the aliens) | Market `buys`, actions on sale | AUTH | Selling to the Cartel (feeds the Dominion while relations are poor) |
| Score events and monthly ratings | `scores:`, `ratings` | INI (`MissionScores` values) + AUTH (index meanings) | Same mechanism, fewer events |
| Weekly and monthly reports, news | `reports:`, `news:` events | TXT (`WEEKLY*`, `news1.txt`) + AUTH triggers | Newsreel-style bulletins |
| Research topics, sizes, prerequisites | `research:` | INI (71 `R_*`) | 30 topics per side |
| Research yields (what a topic unlocks) | `unlocks` | AUTH (guide tree; the INI yields are all 0) | Authored |
| Download research via downlinks, jammers, switching loss | `research_model: {mode: download}` | AUTH (rates) + INI (modules) | `mode: labs` (simpler) |
| Salvage chances by difficulty | `salvage:` | INI (13 `*Goodies`) | Same, 4 categories |
| Capture of disabled ships (mandatory alien captures) | Class `capturable` (implemented) / `capture`, tractor (implemented), research `appears` | AUTH | Reverse-engineering captured saucers |
| Alien messages from battles | Event with `chance` per battle | AUTH | Intercepted broadcasts (chapter 3) |
| Bases: max 8, placement 1.5 ly, cost and time formulas, dismantling | `base_types:` | AUTH | Max 4 bases, placement in owned systems |
| 33 slots in 4 size classes; per-base module limits | Model set `slots` with size; `limits` | EXE (hub manipulators) + INI (`SizeClass`) | Hub with 12 slots, 3 sizes |
| 68 modules: cost, days, maintenance, HP, effects | `modules:` + `effects` | INI (`M_*`) + AUTH (effects) | 8 Guard modules, 6 Dominion pods |
| Module destruction effects (ships, pilots, stock lost) | Module `on_destroyed` | AUTH | same |
| Module textures normal/construction/destroyed | Model set `states: {building, destroyed}` | EXE (texture triples) | Procedural scaffolding and scorch textures |
| 20 hulls: handling, mounts, HP, cost, upkeep | `ships:` (implemented) + `hulls:` | INI (`Ship_*`) | 3 rockets, flagship, 6 Dominion craft, 4 third-party |
| 23 equipment items; power budget | `items:`, `power` | INI | Fewer items, same slots |
| 9 beams, 10 launchers, 10 missiles, 3 mines | `beams:`, `items:` with `kind: launcher`, `missiles:`, mine classes | INI | 4 ray guns, 4 rockets, 2 mines |
| Missile special behaviours (sidestep, permeator, parallax, cloaked wait, doppelganger, enveloping) | Missile guidance behaviours | AUTH (behaviour params) + INI (stats) | Seeker, decoy, cluster, atomic |
| Special systems (ECM, cloak, tractor, VCT, damage control, jump drive) | Built-in systems with parameters (`systems:`, implemented) | INI + AUTH | Jammer, tractor ray, cloak, auto-pilot |
| Stock designs | `designs:` | AUTH | same |
| Pilots: 8 stats, ranks, experience, training, injury, bravery, recruiting, voices | `pilots:` formulas, `voices:` | INI (rank thresholds) + AUTH (formulas) + TXT (names) | 5 stats, 4 ranks |
| Up to 5 craft per mission; wingman orders | `max_craft_per_mission`; behaviours with order transitions | AUTH | 4 craft |
| 27 mission types, 56 debriefs, 7 side missions, simulator types | `missions:`, `simulator: true`, events | AUTH (compositions: guide and play-testing) + TXT | 11 generated types + 12 chapter missions |
| Squadron sizes, flight plans, ambush chance | `squadrons:`, `convoys:`, strategist | AUTH | same, smaller |
| Hyperspace out, retreat costs | `outcomes: {retreat}` | AUTH | same; plus escape to orbit in atmospheres |
| Alien strategic AI (aggression, expansion, offence ladder, defence priority, cheats, tech levels) | `strategists:` | INI (`Think.ini`) + AUTH (guide rules) | Dominion strategist |
| Per-class combat AI | `behaviours:` + difficulty variants (implemented) | AUTH | Saucer and robot behaviours |
| Stations in battle (guns, shields, damage zones, pods) | Object classes, modules, behaviours | INI (HP charts) + AUTH | same |
| Debris table (28 sets), shock wave, corpses | `debris:`, `effects:` | EXE | Procedurally cut pieces |
| Effect rings per hull size (shield/jump, U) | `effects:` | EXE + AUTH | Ray shield shimmer |
| Black-hole battles (gravity pull), asteroid hazards, minefields | Mission `hazards` | AUTH | Ion storm (chapter 4), asteroid belts |
| Interactive music sets, CD tracks | `music:` states and switches | original files by reference + AUTH | Synthesised loops |
| MFD talking heads, chatter voices | `instruments: {comms}`, `voices:` | original files by reference | Synthesised radio beeps + text |
| UFOpedia (9 categories, ~150 entries) | `encyclopaedias:`, `entries:` | TXT | ~60 entries |
| Strategy screens (CSD, bases, research, equip, market, reports) | Screens + `screens:`/`skins:` | original wrapper art by reference | Art-deco skin |
| Briefing/debriefing, hypernews, load/save, options | Screens; `news:` | TXT | same |
| Multiplayer liveries and arenas | Model set `textures`, scenarios | EXE + INI (`MPSettings`) | Not used (multiplayer deferred) |

## Default campaign → format → `interceptor`

| Default campaign feature | Construct | Use in `interceptor` |
|---|---|---|
| Chapters with title cards and cliffhangers | `arcs:`, `chapter` screen, events | The Doomsday arc (no title cards) |
| Story triggered by **both sides' development** | Conditions on `plan()`, rival research, holdings | Doomsday gates on captures and research |
| Dynamic relations between all factions | `relations: {dynamic: true}`, `relation_rules:` | `dynamic: false`; corporate funding only |
| Senate and Cartel as third parties | `patron`, `independent` factions | Corporations, arms dealer |
| Player choices in events | `choices` | Not used (the original has accept/refuse prompts: `offer_mission`) |
| Fixed map with lanes | `maps:` with `kind: fixed`, `travel: lanes` | Generated, free travel |
| Ground and atmospheric battles | `environments:` with `physics: atmosphere`, terrain, ground classes | Not used |
| Labs research model | `mode: labs` | `mode: download` |
| Fully original procedural assets | Generator tools; OBJ/PNG/WAV | `original:` references |

## Open items (data the original keeps in code)

These need the strategy guide, careful play of the original, or both; each becomes a work item
in Stage 5 with its source recorded (verified/derived) in the mod's comments:

1. The meaning of each `MissionScores` index (values are data, indices are code).
2. Exact squadron compositions per mission type, tech level and difficulty (the guide's
   tables are partly illegible).
3. Star count, planet size and sensor-radius tables for map generation.
4. Salary, recruitment and psionics-by-rank modifiers.
5. The alien tech-level cost mapping and the Doomsday cost/time columns.
6. Station hit-point chart meaning (`HP1..HP4`, `SD`).
7. The three-ring effect meshes (`1ASH*` etc.): shield flash or jump effect.
8. Whether overlay-face low bits on station hubs are state groups.

Each is a **data** question with a reasonable fallback (the guide's legible values, or a
documented estimate marked *derived*), so none blocks the format or the stages.
