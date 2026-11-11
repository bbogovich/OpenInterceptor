# Strategy layer

The campaign around the battles — the star map, bases, research, the economy, craft, pilots and
the enemy raids that start battles — is **pack data** (app 0.7.0 – 0.7.3). A campaign that has a
`start:` section has a strategy layer: the launcher then leads with *Campaign: continue* and
*Campaign: new game* ([User guide › Campaign](../user-guide/campaign.md)). A campaign without
`start:` is battles only, as before.

Requirements: [R-STR-001 – R-STR-011](../specification/requirements/strategy-logistics.md).
Design (the full planned format, of which this page is the built part):
[Format: strategy layer](../specification/design/format-strategy.md). How the engine plays these
rules: [Architecture › Campaign](../implementation/architecture/campaign.md).

## Where the files go

Strategy sections are ordinary rules sections, so they may sit in any rules file; by convention
they go in `<pack>/strategy/*.yaml`, which the loader reads like `rules/`, `campaign/` and
`missions/`. The default campaign splits them by subject:

| File | Sections |
|---|---|
| `resources/default/strategy/start.yaml` | `start`, `flags` |
| `resources/default/strategy/map.yaml` | `maps`, `star_classes` |
| `resources/default/strategy/research.yaml` | `research_model`, `research` |
| `resources/default/strategy/economy.yaml` | `economy`, `incomes`, `expenses`, `items`, `hulls`, `base_modules`, `base_types` |
| `resources/default/strategy/raids.yaml` | `raids` |
| `resources/default/strategy/pilots.yaml` | `pilots`, `scores` |

Each file starts with `# yaml-language-server: $schema=../../../schemas/rules.schema.json`, so
an editor offers every key below. Definition sections (`maps`, `research`, `items`, …) layer
across packs like any other definitions: an addon may add a research topic, `inherit` an item or
`delete: true` a raid ([Rulesets](rulesets.md#layering-inherit-delete-and-list-edits)).

## Units

- **Times** — `build_time`, `every`, `delivery` — take a time unit: `10d` (days, new in app
  0.6.7 for every time quantity), `36h`, `90min`; a plain number is hours in the strategy layer.
- **Map units** — positions, `speed`, `sensor_range`, `max_distance` — are the map's own units;
  `unit_ly` says how many light years one is (for display). `speed` is map units per hour.
- **Money** is a plain number. `upkeep`, `maintenance` and `salary` are per month (720 hours).
- **Work units** — research `size`, `prod_time` — are consumed at a rate per hour: a base's
  research rate (`research_model.rate`) or production rate (module effect `production_rate`).

## Expressions: the campaign context

Prerequisites (`requires:`), amounts (`incomes`, `expenses`) and the research rate are written in
the [condition language](behaviours.md#the-condition-language) with the **campaign context**
(`script::campaign`, app 0.6.7). Plain numbers are expressions too. Every id the strategy layer
defines — topics, items, hulls, modules, base types, raids, flags — and every faction, ship class
and mission id is a named constant, so a misspelt name (`research(lazers)`) is a load error that
names it. Difficulty level ids are constants as well (`difficulty >= veteran`).

| Query | Result | Meaning |
|---|---|---|
| `research(x)` | true/false | Topic `x` has been researched. |
| `researching(x)` | true/false | A base is working on topic `x`. |
| `has(x)` | number | Items `x` in store, all bases. |
| `module_count(x)` | number | Working modules `x` — at the base being evaluated (in a research rate), else at all bases. |
| `bases` | number | The player's bases. |
| `money` | number | The player's money. |
| `day` | number | Days since the campaign began. |
| `month` | number | Whole months (30 days) since the campaign began. |
| `difficulty` | number | The difficulty level (compare with level names). |
| `pilots` | number | Pilots on the roster. |
| `score` | number | The campaign score so far. |
| `month_score` | number | Score gained since the last monthly report. |
| `random(p)` | true/false | True with probability `p` (0..1), from the campaign's seeded generator. |
| `flag(x)` | true/false | Campaign flag `x` is set (by a mission outcome's `set_flag(x)`). |
| `salvaged(x)` | true/false | Something named `x` has been salvaged (see [Battles](#raids-and-battles)). |
| `craft` | number | The player's craft, all bases. |

```text
research(ion_rays) and research(saucer_hulls)
6 + module_count(laboratory) * 12
400000 + 60000 * bases + month_score * 200
```

## Sections

### `start`

How a new campaign begins. **Without `start:` the campaign has no strategy layer.**

| Key | Type | Default | Meaning |
|---|---|---|---|
| `date` | text | *required* | The first day, `YYYY-MM-DD` (Gregorian calendar, leap years counted). A missing or malformed date is a load error. |
| `money` | number | 0 | Starting money. |
| `map` | id | the first map | The map (`maps.<id>`). |
| `bases` | list of `{type, system, name, modules}` | none | Bases the player starts with: a base type, the system it is in, its name, and modules beyond the type's `template`. Starting bases and their modules are finished. |
| `craft` | list of `{hull, base, count}` | none | Craft the player starts with; `base` is an index into `bases` (0 = the first). |
| `pilots` | number | 0 | Pilots on the starting roster; they take the starting craft in order. |
| `research` | ids | none | Topics already known. |
| `stock` | mapping item: count | none | Items in store at the first base. |

### `maps` and `star_classes`

`maps: {<id>: …}` — the star map.

| Key | Type | Default | Meaning |
|---|---|---|---|
| `kind` | `fixed` / `generated` | `fixed` | Hand-made (the systems as written) or generated when a campaign starts. |
| `systems` | fixed: mapping id: `{name, pos, class, hidden}`; generated: `[min, max]` or a count | — | Fixed: each system's display `name` (default the id), `pos: [x, y]` (required), star `class` (default `yellow`) and `hidden` (not shown until a base's sensors reach it; default false). Generated: how many systems (default 30). |
| `lanes` | list of `{between: [a, b]}` | none | Lanes between two of the map's systems (drawn on the map). |
| `travel` | `free` / `lanes` | `free` | Straight lines anywhere (the original) or along the lanes (app 0.7.3; see [Travel](#travel)). |
| `size` | `[w, h]` | `[60000, 60000]` | Generated: the rectangle, map units. |
| `names` | list of text | none | Generated: system names to draw from (then numbered). |
| `seed` | number | the campaign's | Generated: a fixed seed, so every campaign gets the same map. |
| `black_holes` | number | 1 | Generated: how many systems are hidden black holes. |
| `unit_ly` | number | 0.001 | Light years per map unit (display only). |

`star_classes: {<id>: {color: [r, g, b], weight}}` — how a star of that class is drawn, and
(for generated maps) how often it appears relative to the others (`weight`, default 1; 0 = never
rolled). Generated systems are scattered at least a little apart, take a class by weight and a
name from `names`.

### `research` and `research_model`

`research: {<id>: …}` — the research tree.

| Key | Type | Default | Meaning |
|---|---|---|---|
| `name` | text | the id | Display name. |
| `size` | number | 1000 | Work units to finish (also read from the original's `file_size`, in thousands, through inheritance). |
| `requires` | condition | none | Available to research only while this holds: `research(x)`, `salvaged(x)`, any campaign query. |
| `unlocks` | list | none | What finishing it makes available: `items.<id>`, `hulls.<id>`, `modules.<id>` (or `base_modules.<id>`), `base_types.<id>`. Something a topic unlocks cannot be bought, made or built until the topic is known. |
| `description` | text | empty | Encyclopaedia text (Stage 5 shows it). |

`research_model:` — how research progresses.

| Key | Type | Default | Meaning |
|---|---|---|---|
| `mode` | `download` / `labs` | `download` | How it is described: the original's downloads, or staff in labs. The rule is the same: each base works on **one** topic at a time. |
| `rate` | expression | 10 | Work units per hour at one base, evaluated per base (so `module_count` counts that base's modules). |
| `switching` | `keep_progress` / `lose_progress` | `keep_progress` | Whether switching a base to another topic keeps the work done on the old one (with `lose_progress` it is lost unless another base is still working on it). |

A topic finishes when its work reaches `size`; every base working on it stops, and a message
says so.

### `items` and `hulls`

`items: {<id>: …}` — equipment, ammunition and goods.

| Key | Type | Default | Meaning |
|---|---|---|---|
| `name` | text | the id | Display name. |
| `kind` | id | `goods` | `beam`, `launcher`, `missile`, `armor`, `shield`, `engine`, `sensor`, `special`, `probe`, `mine` or `goods`; matched against hull `slots`. |
| `cost` | number | 0 | Purchase price; 0 = cannot be bought. |
| `sell_price` | number | 0 | What one fetches on the market. |
| `prod_time` | number | 0 | Work units to manufacture one; 0 = bought only. |
| `size` | number | 1 | Storage space (kept for the storage rules to come). |
| `rounds` | number | 1 | Missiles: the rounds one missile slot holds; fitting a slot takes this many from the store (app 0.7.2). |
| `requires` | condition | none | Available only while this holds (as well as any topic that unlocks it). |

`hulls: {<id>: …}` — the strategy side of a ship class. **The id must be a ship class's id**
(`hulls.comet` is `ships.comet`); another id is a load error.

| Key | Type | Default | Meaning |
|---|---|---|---|
| `cost` | number | 0 | Purchase price. |
| `prod_time` | number | 0 | Work units to manufacture one. |
| `upkeep` | number | 0 | Per month, per craft (also read from the original's `maint_cost` through inheritance). |
| `speed` | number | 1000 | Map units per hour on the star map. |
| `slots` | mapping kind: count | none | Equipment slots: `{beam: 2, missile: 2, special: 1}` (also read from the original's mount counts through inheritance). |
| `requires` | condition | none | Available only while this holds. |

### `base_modules` and `base_types`

`base_modules: {<id>: …}` — what a base is built from.

| Key | Type | Default | Meaning |
|---|---|---|---|
| `name` | text | the id | Display name. |
| `group` | id | the id | Its limits group (`research`, `hangar`, `sensor`, …): a base type's `limits` cap modules per group. |
| `cost` | number | 0 | Build price, paid when ordered. |
| `build_time` | time | 72 h | Time to build; the module works from then on. |
| `maintenance` | number | 0 | Per month. |
| `effects` | mapping name: number | none | Campaign quantities it adds while working: `research_rate`, `production_rate`, `sensor_range`, `hangar` (craft housed), `quarters` (pilots housed), `storage`, `defence`, `income`. |
| `requires` | condition | none | Available only while this holds. |

As built, the engine reads `production_rate` (manufacturing), `sensor_range` (detection and
revealing hidden systems) and `hangar` (craft a base houses). `research_rate` is a number the
`research_model.rate` expression may use through `module_count`; `quarters`, `storage`,
`defence` and `income` are accepted and summed but not yet acted on.

`base_types: {<id>: …}` — kinds of base.

| Key | Type | Default | Meaning |
|---|---|---|---|
| `name` | text | the id | Display name. |
| `cost` | number | 0 | Build price. |
| `build_time` | time | 168 h | Time to build the hub and its template modules; the base is under construction until then. |
| `max_bases` | number | 8 | The most bases of this type. |
| `limits` | mapping group: count | none | The most modules of each group. |
| `template` | ids | none | Modules a new base comes with. |
| `sensor_range` | number | 0 | The hub's own sensor range, map units (modules add to it). |
| `hangar`, `quarters` | number | 0 | Craft and pilots the hub alone houses. |
| `max_distance` | number | 0 | A new base must be within this many map units of one of yours (0 = anywhere). |

### `raids`

`raids: {<id>: …}` — enemy flights the campaign launches now and then. Meeting one is a battle.

| Key | Type | Default | Meaning |
|---|---|---|---|
| `name` | text | the id | Shown on the map and in messages. |
| `mission` | id | *required* | The mission flown when it is intercepted (`missions.<id>`; unknown = load error). |
| `faction` | id | *required* | Who flies it (`factions.<id>`; unknown = load error). |
| `every` | time | 72 h | Time between rolls. |
| `chance` | fraction | 50 % | Chance per roll that a raid launches (`45%`). |
| `requires` | condition | none | Rolled only while this holds (`day >= 10`). |
| `target` | `base` / `system` | `base` | What it heads for: one of the player's bases, or any system. |
| `speed` | number | 600 | Map units per hour. |

On a free-travel map a launched raid appears well outside the map, in a random direction, and
flies straight for its target; on a lane map it enters at a system and follows the lanes
([Travel](#travel)). See [Raids and battles](#raids-and-battles).

### `incomes` and `expenses`

`incomes: {<id>: {name, every, amount}}` and `expenses: {<id>: {name, every, amount}}` — an
amount (an expression) added or taken every period (`every`, default 30 days). Each payment is a line in the message log.
Module `maintenance`, hull `upkeep` and pilot `salary` are charged monthly on their own; use
`expenses` for anything else.

### `pilots` and `scores`

`pilots:` — pilot rules.

| Key | Type | Default | Meaning |
|---|---|---|---|
| `stats` | mapping stat: `[min, max]` | none | Each pilot's stats, rolled in the range. Any names. |
| `ranks` | mapping rank: experience | none | Ranks in ascending order with the experience each needs. |
| `salary` | number | 0 | Per month, per living pilot. |
| `recruit_fee` | number | 0 | Cost to hire one. |
| `pool` | number | 6 | Recruits offered at a time (refreshed every month). |
| `names` | list of text | none | Names to draw from. |
| `heal_days` | number | 14 | Days an injured pilot is out. |
| `survive` | fraction | 60 % | Chance that a pilot shot down ejects (injured) rather than dies. |

`scores: {<name>: points}` — mission score names. A mission outcome's `score(<name>)` adds the
points to the campaign score and the month's score; the name `undefended` is used when a raid
reaches its target unopposed.

### `flags` and `economy`

`flags: [<id>, …]` — campaign flags that conditions may test with `flag(x)` before any mission
sets them (`set_flag(x)` names in mission outcomes are added automatically).

`economy:` — `delivery` (time for a purchase to arrive, default 48 h) and `salvage_per_kill`
(money per enemy destroyed when an outcome says `salvage`, default 0).

## How a campaign plays the data

- **Time** passes an hour at a time (smaller at the end of a jump). Each hour: construction
  finishes, each base researches its topic at `research_model.rate`, each base manufactures its
  queue (first job first) at its `production_rate`, deliveries arrive, incomes and expenses pay,
  pilots heal, raids roll and move, sensors detect, interceptions fly.
- **The month** (every 720 hours): maintenance, upkeep and salaries are charged; the **monthly
  report** gives the month's score, the total, the costs and the funds; a new pool of recruits is
  offered.
- **Bases.** A new base is built in a system that has none, at most `max_bases` of a type,
  within `max_distance` of one of yours; it comes with its `template` modules and is finished
  after `build_time`. A module is refused beyond its group's limit or while not available.
- **Buying** pays now and delivers to the current base after `economy.delivery`. **Selling**
  takes one item from the current base's store at `sell_price`. **Manufacture** queues at the
  base; finished units go into its store (hulls become craft there). *As built, manufacture
  charges no money* (see [Limits](#limits-in-app-074)).
- **Interception** sends every craft of the current base that is at home and has a ready pilot.
- **Fitting** (app 0.7.2): *fit best* fills each of the craft's hull `slots` with the costliest
  item of that `kind` in its base's store, as many as the hull has slots of that kind (a missile
  slot takes the item's `rounds`); what was fitted before goes back into the store first. A
  campaign battle flies the fit: fitted **beam** items become the player's class's beams and
  fitted **missile** items its racks, matched to `beams:` and `missiles:` of the **same id**
  (`items.blaster` ↔ `beams.blaster`); ids with no such weapon are skipped, and with no beams
  fitted the class keeps its own. The refit changes only that battle's copy of the rules
  (wingmen of the same class fly it too).

## Travel

On a `travel: free` map everything flies straight. On a `travel: lanes` map (app 0.7.3):

- a **raid** enters at the system farthest from its target that has none of the player's bases,
  and follows the shortest way along the lanes to the system nearest its target;
- an **interception** flies the lanes from its base to the system nearest the contact, then
  straight to the contact, and comes home along the lanes;
- the map draws each contact's way ahead. Routes are kept in saves.

The shortest way is by distance along the lanes (Dijkstra's algorithm, `Campaign::lane_route`);
systems no lane joins are reached in one straight hop. Endurance and stranding are not built.

## Raids and battles

1. A raid's roll succeeds: a **contact** starts outside the map and heads for its target.
2. When a contact comes within a base's sensor range (hub plus `sensor_range` effects) it is
   **detected** — the clock stops with a message — and shows on the map.
3. The player sends the current base's ready craft to **intercept** it. They fly at their hull's
   `speed`; when they reach it the clock stops and the player chooses: **fly** the raid's
   `mission`, with the first craft's hull as the player's ship, or **break off** (the contact
   flies on).
4. The battle's debrief comes back to the campaign: the outcome's actions apply —
   `score(<name>)`, `money(<n>)`, `set_flag(<id>)`, `salvage` (money: `salvage_per_kill` × kills,
   and the raid is marked salvaged for `salvaged(x)` — see the known issue under [Limits](#limits-in-app-074)), `lose(<what>)` (logged).
   The leader is credited the kills and experience (kills + 1); every pilot in the party a
   mission. A `killed` result loses the craft, and its pilot ejects injured (`survive`) or dies.
   `win` and `draw` remove the contact; `lose` and `retreat` leave it flying. Leaving a campaign
   battle early from the pause menu is a `retreat`.
5. A contact that reaches its target **unopposed**: a base target loses its newest finished
   module, and the `undefended` score applies.

## The default campaign as the example

*Perils of the Void* (`resources/default/strategy/`) uses every section:

- **Start**: 1 March 2084, 2 500 000 in funds, the Gatehouse base (`guard_base`) at Mercury
  Gate, three Comets, four pilots, `rocketry` known, six blasters and twelve seeker rockets.
- **Map** `reach`: fixed, lane travel, 1 unit = 0.001 ly; six systems — Mercury Gate, Ajax,
  Cinder, Vesta Reach, Tycho Drift, Nebula Haven — joined by seven lanes, and **The Dark Moon**,
  a hidden black hole far to the east, found only when a base's sensors reach it.
- **Research**: labs (`6 + module_count(laboratory) * 12` per hour, `keep_progress`); twelve
  topics from `rocketry` through `heat_rays`, `ion_rays`, `saucer_hulls` (requires
  `salvaged(saucer_raid) or salvaged(scout_probe)`), `saucer_propulsion`, `disintegrators`,
  `deep_radar`, `shipyards`, `thunderbolt_design`, `robot_brains` (`salvaged(robot_raid)`),
  `victory_design` and `regents_engine`, unlocking beams, missiles, the Meteor, Thunderbolt and
  Victory hulls and the deep-radar and shipyard modules.
- **Economy**: delivery 48 h, salvage 15 000 a kill; the **Senate stipend** every 30 days,
  `400000 + 60000 * bases + month_score * 200`; cosmium sales every 7 days,
  `12000 * module_count(cosmium_refinery)`; fuel and stores `4000 * craft` a week. Eight items
  (four beams, three missiles, cosmium), four hulls, nine module kinds (hangar, quarters, radar,
  deep radar, laboratory, workshop, shipyard, cosmium refinery, ray battery) and the `guard_base`
  type (template hangar, radar, laboratory; at most 6 bases, 3500 units apart).
- **Raids**: Dominion scouts (`intercept_scouts`), saucer raids (`defend_station`, from day
  10), colony raids (`colony_defence`, day 20), robot assaults (`outpost_raid`, day 45) and the
  Black Comet ambush by raiders (`pocket_ambush`, day 15) — each a mission of the default
  campaign ([Missions](missions.md#the-missions-shipped)).
- **Pilots**: flying, gunnery and nerve; five ranks from cadet to commander; salary 6000,
  recruit fee 40 000, five recruits a month, 12 days to heal, 70 % survive; twelve mission
  scores.

The unit test `campaign::tests::the_default_campaign_runs_six_months` plays it for six months
with an automatic player: it stays solvent, researches at least four topics, fights raids and
round-trips through a save.

## Errors

| Message | Cause |
|---|---|
| `research.x.requires: unknown name 'y'` (and the same for other expressions) | A name in an expression that nothing defines. |
| `maps.x.systems.y: pos: expected [x, y]` | A system without two coordinates. |
| `maps.x.lanes: between needs two of the map's systems` | A lane naming an unknown system. |
| `maps.x: kind: 'y' is not fixed or generated` | A wrong map kind. |
| `raids.x: unknown mission 'y'` / `unknown faction 'y'` | A raid naming a mission or faction that does not exist. |
| `hulls.x: no ship class with this id` | A hull without a ship class of the same id. |
| `incomes.x.amount: expected a number, not a condition` | An amount (or rate) written as a true/false condition. |
| `research.x.unlocks: unknown item, hull, module or base type 'y'` | An unlock naming something not defined. |
| `start.bases[0].system: unknown system 'y'` (and `type`, `modules`, `start.craft[0].hull`, `start.map`) | The start names something the strategy data does not define. |
| `start.date: expected YYYY-MM-DD` | A missing or malformed start date. |
| `this save needs packs that are not loaded: …` | Loading a save made with a mod that is not active. |
| `save: … does not match the loaded rules (was a mod changed?)` | Loading a save whose research, modules, items, incomes or raids no longer fit the rules. |

## Limits in app 0.7.4

- No endurance or stranding on the map; no fitting rules (`fits`), power draw or stock designs —
  fitting is *best available* only, and slots of kinds other than `beam` and `missile` are filled
  but have no effect in battle.
- Outcome actions `relation(…)` and cargo actions are **logged only**; they belong to Stage 5's
  relations and the logistics of plan items 4.14 – 4.17.
- **`salvaged(<raid id>)` never becomes true**: a salvage records the raid's display `name`, while
  the query compares the raid's id, so in the default campaign `saucer_hulls`, `robot_brains` and
  the topics after them cannot be researched yet. A known issue, to be fixed in the code.
- **Manufacture charges no money**: the item's or hull's `cost` is not taken when a unit starts,
  although the code's own comment says it is. To be fixed with plan item 4.4.
- No **logistics** (resources, facilities, supply convoys, surveys), no designs, training,
  bravery, dismantling, funding evaluations or the 3D base view yet.
- Names and descriptions in the default campaign's strategy files are plain English, not text
  keys, so they are not yet translatable.
- The `interceptor` campaign has no strategy data yet (plan item 4.12): the original's research
  prerequisites are numeric tech ids that do not map to its research sections in file order, so a
  tech-id table has to be worked out first.
