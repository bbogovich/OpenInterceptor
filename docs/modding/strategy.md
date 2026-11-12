# Strategy layer

The campaign around the battles — the star map, bases, research, the economy, craft, pilots and
the enemy raids that start battles — is **pack data** (app 0.7.0 – 0.7.3), and so is the optional
**logistics** layer of mining, manufacturing, research stations and supply convoys (app 0.9.0,
[Logistics](#logistics)). The story on top of it — events, arcs, the rival's plans, relations,
the strategist, endings and the encyclopaedia (app 0.10.0) — has its own page,
[Story layer](story.md). A campaign that has a
`start:` section has a strategy layer: the launcher then leads with *Campaign: continue* and
*Campaign: new game* ([User guide › Campaign](../user-guide/campaign.md)). A campaign without
`start:` is battles only, as before.

Requirements: [R-STR-001 – R-STR-027](../specification/requirements/strategy-logistics.md).
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
| `resources/default/strategy/logistics.yaml` | `resources`, `components`, `recipes`, `facility_types`, `research_sites`, `logistics`; and, merged into the `reach` map, the hulls and the research tree: `deposits`, `features`, hull `components`, two station-only topics (app 0.9.0) |
| `resources/default/strategy/story.yaml` | `events`, `arcs`, `plans`, `relations`, `strategist`, `endings`, `campaign_end`, `encyclopaedia` (app 0.10.1; [Story layer](story.md)) |

The `interceptor` campaign has two (app 0.8.0, [below](#the-interceptor-campaign-from-the-players-data)):
`packs/interceptor/strategy/references.yaml`, generated from the player's own files, and
`packs/interceptor/strategy/campaign.yaml`, written by hand; and since app 0.10.2 its story,
`packs/interceptor/strategy/story.yaml` ([Story layer](story.md#the-interceptor-campaigns-doomsday-arc)).

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
  `economy.cost_scale` multiplies every item, hull and module cost, sell price, upkeep and
  maintenance (app 0.8.0; for data in thousands, like the original's).
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
| `arc(x)`, `plan(x)`, `relation(x)`, `fired(x)`, `battles_won` | number | The story's queries (app 0.10.0): see [Story layer › Conditions](story.md#conditions). |

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
| `money_from` | `original:<file>/<key>` | none | The starting money read from the player's original file, from the section named after the difficulty level (its id with a capital first letter: `veteran` → `[Veteran]`); the key's name is matched without regard to case. Replaces `money`. Not found = load error (app 0.8.0). |

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

Fixed-map systems also take `bodies` (a list of planets, moons and belts; descriptive) and, for
a campaign with [logistics](#logistics), `deposits` and `features` (app 0.9.0).

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
| `prereq1`, `prereq2`, `prereq3` | numbers | none | The original's numeric prerequisites, read through inheritance; with a [`tech_ids`](#tech_ids) table and no `requires`, they become `research(a) and research(b)` (app 0.8.0). |
| `lab`, `site`, `start_cost` | | none | A **station-only** topic: see [Logistics › research stations](#research-stations-and-station-only-topics) (app 0.9.0). |

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
| `tech_required`, `tech_required2` | numbers | none | The original's numeric tech ids, read through inheritance; with a [`tech_ids`](#tech_ids) table and no `requires`, they become the prerequisite (app 0.8.0). Also on hulls and modules. |

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
| `components` | mapping component: amount | none | With [logistics](#logistics): what a main base builds one from. A hull with components is not paid for in money when its manufacture starts; it waits until the components are in the base's store (app 0.9.0). |

### `base_modules` and `base_types`

`base_modules: {<id>: …}` — what a base is built from.

| Key | Type | Default | Meaning |
|---|---|---|---|
| `name` | text | the id | Display name. |
| `group` | id | the id | Its limits group (`research`, `hangar`, `sensor`, …): a base type's `limits` cap modules per group. |
| `cost` | number | 0 | Build price, paid when ordered. |
| `build_time` | time | 72 h | Time to build; the module works from then on. With `economy.module_time: days`, a module without `build_time` takes its `prod_time` (the original's key) in days (app 0.8.0). |
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

`economy:`

| Key | Type | Default | Meaning |
|---|---|---|---|
| `delivery` | time | 48 h | Time for a purchase to arrive. |
| `salvage_per_kill` | number | 0 | Money per enemy destroyed when an outcome says `salvage`. |
| `cost_scale` | number | 1 | Multiplies every item and hull `cost`, `sell_price`, hull `upkeep` and module `cost` and `maintenance` (app 0.8.0). The `interceptor` campaign sets 1000: the original's money is in thousands. |
| `module_time` | `days` | — | Modules without `build_time` take their `prod_time` in days (app 0.8.0; the original's module times are days). |

### `tech_ids`

`tech_ids: {<number>: <research id>}` (app 0.8.0) — for data inherited from a game that names
prerequisites by **number**. A topic's `prereq1` – `prereq3`, and an item's, hull's or module's
`tech_required` and `tech_required2`, are looked up in the table and become
`research(<id>)` conditions joined with `and`. Rules:

- A written `requires:` always wins; the numbers are used only when there is none.
- A number the table does not list is **left out** of the condition (the original's ids for
  alien craft and installations, 75 and up, are not research topics).
- Without a `tech_ids` table the numeric keys are ignored.
- The research ids are checked like any other name: a table entry naming an unknown topic is a
  load error once a condition uses it.

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
  base; finished units go into its store (hulls become craft there). Each unit is **paid for as
  it starts** (its `cost`); a job waits while the money is short (app 0.9.0; until then
  manufacture was free). With logistics, a hull that has `components` is paid in components
  instead: it starts when they are in the base's store.
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
   and the raid's **id** is recorded, so `salvaged(<raid id>)` holds from then on — fixed in app
   0.9.0; before, the display name was recorded and never matched), `lose(<what>)` (logged).
   The leader is credited the kills and experience (kills + 1); every pilot in the party a
   mission. A `killed` result loses the craft, and its pilot ejects injured (`survive`) or dies.
   `win` and `draw` remove the contact; `lose` and `retreat` leave it flying. Leaving a campaign
   battle early from the pause menu is a `retreat`.
5. A contact that reaches its target **unopposed**: a base target loses its newest finished
   module, and the `undefended` score applies.

## Logistics

*App 0.9.0, plan items 4.14 – 4.17.* An optional layer of the strategy game: raw **resources**
are mined from **deposits** in star systems, **manufacturing centers** turn them into
**components** by **recipes**, and main bases build hulls from components. Nothing is shipped
on speculation: **demand pulls supply** up the chain, and automatic government **supply
convoys** carry each filled order along the lanes. **Survey ships** find out what a system
holds; **research stations** work on topics no ordinary base can. The design is
[Logistics: facilities and supply lines](../specification/design/logistics.md); requirements
R-STR-012 – R-STR-027.

**A campaign has logistics only if it defines at least one `facility_types` entry.** Without
one, every key below is ignored and the campaign plays as before (the `interceptor` campaign
has none, deliberately). The definition sections (`resources`, `components`, `recipes`,
`facility_types`, `research_sites`) layer across packs like any other definitions.

Amounts are plain numbers in the resource's unit. Mining `rate` is **per day**; everything
else in time takes the usual units (`24h`, `10d`).

### `resources` and `components`

| Section | Key | Type | Default | Meaning |
|---|---|---|---|---|
| `resources.<id>` | `name` | text | the id | Display name of a raw resource (mined). |
| | `unit` | text | empty | The unit shown after amounts (`t`, `kg`). |
| `components.<id>` | `name` | text | the id | Display name of a manufactured component. |

### `recipes`

`recipes: {<id>: …}` — what a manufacturing center makes.

| Key | Type | Default | Meaning |
|---|---|---|---|
| `component` | id | the recipe's id | The component made. One recipe per component is used (the first). |
| `inputs` | mapping resource or component: amount | none | What one run consumes, taken from the center's store when the run starts. |
| `time` | time | 24 h | How long one run takes (at least 0.1 h). |
| `output` | number | 1 | How much one run makes. |

An input or output that is neither a resource nor a component is a load error.

### `facility_types`

`facility_types: {<id>: …}` — what the player may build in a star system.

| Key | Type | Default | Meaning |
|---|---|---|---|
| `name` | text | the id | Display name. |
| `role` | `mining` / `manufacturing` / `research` | *required* | What it does. Another value is a load error. |
| `cost` | number | 0 | Build price, paid when ordered. |
| `time` | time | 10 d | Time to build; it works from then on. |
| `storage` | number | 500 | Mining: the most goods its store holds; extraction stops when it is full. |
| `lines` | number | 1 | Manufacturing: recipes running at once. |
| `research_rate` | number | 0 | Research: work units an hour on its project. |
| `hub` | id | none | The battle object at the facility, with its construction-stage looks (read; not yet used — R-STR-022). |

**Where a facility may be built** (fixed rules per role, app 0.9.0; the design's `placement:`
expressions are not built — DEV-049). At most one facility of each type per system, and:

| Role | Needs |
|---|---|
| `mining` | A **surveyed** system with at least one deposit. |
| `research` | A **surveyed** system that meets at least one research site's features. |
| `manufacturing` | A system with one of the player's bases, or at most **two lanes** from one (a system no lane reaches counts as one straight hop). |

### `research_sites`

`research_sites: {<id>: {features: [<tag>, …]}}` — a site is any system whose `features` include
**all** the listed tags. Station-only topics name the site they need (below).

### On the map, hulls and topics

| Where | Key | Type | Meaning |
|---|---|---|---|
| A map system (`maps.<id>.systems.<id>`) | `deposits` | mapping resource: `{rate, reserve, body}` | What a mining facility there extracts: `rate` per day (default 1), `reserve` a number or `unlimited` (the default), `body` descriptive. An unknown resource is a load error. Only the **start map's** systems are read. |
| | `features` | list of tags | Matched against `research_sites`. |
| A hull (`hulls.<id>`) | `components` | mapping component: amount | What a main base builds one from (instead of money). An unknown component is a load error. |
| A topic (`research.<id>`) | `lab` | facility type | Only a facility of this type may research it; the topic is **not offered to bases**. Unknown = load error. |
| | `site` | research site | Only a station in a system meeting this site. Unknown = load error. |
| | `start_cost` | mapping resource or component: amount | Materials delivered **once** to begin; then research runs without further supply. |

### `logistics`

The supply settings (a global section).

| Key | Type | Default | Meaning |
|---|---|---|---|
| `vessels` | number | 3 | Vessels in each supply convoy; each carries an even share of the cargo. |
| `ambush_chance` | fraction | 30 % | Chance, rolled when a convoy launches, that it will be attacked on its way; the point along its route is rolled then too. |
| `ambush_loss` | fraction | 35 % | Chance that each vessel of an ambushed convoy is lost when the ambush is auto-resolved (no `ambush_mission`, or the player lets it fight through). |
| `survey_attack_chance` | fraction | 25 % | Chance, rolled when a survey ship launches, that it is attacked on the way. |
| `survey_speed` | number | 900 | Survey ship speed, map units an hour. |
| `convoy_speed` | number | 500 | Convoy speed, map units an hour. |
| `ambush_mission` | mission id | none | The battle flown when the player escorts an ambushed convoy (app 0.9.2). With it, an ambushed convoy **waits** where it is and the clock stops: the player flies its escort, and the share of the mission's [`cargo:`](missions.md#the-mission) role still aboard at the end is delivered — or lets it fight through (auto-resolved as below). Without it every ambush is auto-resolved. An unknown mission is a load error. The default campaign sets `convoy_ambush`. |

### How the chain runs

Every hour, after the rest of the hour step ([Architecture › Campaign](../implementation/architecture/campaign.md#logistics)):

1. **Demand at bases.** Each hull job not yet started needs its hull's `components`. What the
   base's store lacks, less what is already on its way, is **ordered** from the nearest ready
   manufacturing center (nearest by straight-line distance).
2. **Mining facilities** extract each deposit of their system at `rate` / 24 an hour, until the
   store is full (`storage`) or the reserve runs out.
3. **Manufacturing centers** finish runs (the output goes into the store), then look at the
   orders they must fill: when the store plus the runs under way do not cover an order, a free
   line starts the component's recipe if its inputs are in store; if they are not, the missing
   inputs are ordered from the nearest mining facility whose system has a deposit of them.
4. **Research stations** with a project first order its `start_cost`; when it is in store it
   is used up, and from then on the station adds `research_rate` an hour to the topic.
5. **Filled orders leave as convoys.** When a facility's store holds an order's amount, it is
   taken out and a convoy of `vessels` vessels leaves along the lanes for the order's base or
   facility. The ambush roll and point are decided now.
6. **Convoys travel** at `convoy_speed`. At the ambush point each vessel may be lost
   (`ambush_loss`) and a message says how many; on arrival the surviving share of the cargo is
   delivered. A short delivery is simply wanted again: the next hour's demand orders the rest.
7. **Hull jobs** whose components are in the base's store start, using them up; the factory
   then works on them as usual.
8. **Survey ships** arrive. One that was attacked is lost half the time; otherwise the system is
   **surveyed** and a message lists its deposits (with rates) and the research sites it meets.

An order is placed only when a facility exists that can fill it: a base without a reachable
manufacturing center simply waits. Facilities report when they are complete.

### The player's orders

On the strategy screen's **Supply** tab ([User guide › Campaign](../user-guide/campaign.md#supply)):

- **Survey** the chosen system from the current base — it must be within that base's sensor
  range (hub plus `sensor_range` effects) and not already surveyed.
- **Build** a facility of any type in the chosen system, paying its `cost` (the row says why
  not when it is not allowed).
- **Research here**: on a finished research station, start a station-only topic whose `lab`
  is its type, whose `site` its system meets and whose `requires` holds.

### Research stations and station-only topics

```yaml
research:
  theremin_shielding:
    name: Theremin shielding
    size: 24000
    requires: research(heat_rays)
    lab: research_station        # only this facility type researches it
    site: nebula_lab             # only in a system meeting this site
    start_cost: {cosmium: 20, dilithium: 2}
```

The topic's progress is shared with the rest of the tree: when it reaches `size` it is known,
like any other topic, and its `unlocks` apply.

### The default campaign's logistics

`resources/default/strategy/logistics.yaml`: four resources (iron, gold, cosmium, dilithium),
three components (hull plating, rocket motors, ray emitters) with a recipe each, the three
facility types (mining facility, manufacturing center, research station), two research sites
(`nebula_lab`, `pulsar_array`), deposits in every one of the Reach's six systems and two features, the
components of all four hulls, and two station-only topics (`theremin_shielding`,
`pulsar_navigation`). Every hull of the default campaign is therefore built from components:
a Guard player who wants more craft must survey, mine and manufacture first.

### Not built yet

Convoy ambushes can be **flown** since app 0.9.2 (`ambush_mission`; the battle is described in
[Missions › The convoy ambush](missions.md#the-convoy-ambush-app-092)). Not built: facility fighter pools and escorts
(R-STR-025), strikes on facilities under construction from 50 % (R-STR-023), construction-stage
looks on the map and in battle (R-STR-022), captured cargo and enemy supply depots (R-STR-020),
third-party convoys (R-STR-027), supply-vessel cargo capacity (R-STR-026), facility upgrades,
and the design's `placement:` expressions (fixed rules per role instead). The manufacturing
center's and research station's `storage` is not enforced.

## The `interceptor` campaign from the player's data

*App 0.8.0, plan item 4.12.* The X-COM campaign's strategy layer is built from the player's
own `GameSettings.ini` and `Interceptor.exe`; the pack holds **no values** from them.

- `tools/interceptor_strategy.py` (Python, standard library) reads the player's files and
  writes `packs/interceptor/strategy/references.yaml`: the **tech-id table** (`tech_ids:`) and
  `inherit: original:GameSettings.ini/<section>` references for 71 research topics, 55 items and
  34 X-COM base modules — section names and ids only. Run it on the player's data and review
  the diff:

  ```sh
  python3 tools/interceptor_strategy.py resources/xcom-cd/Game/ProgramFiles \
      > packs/interceptor/strategy/references.yaml
  ```

- The tech-id table comes from the executable, which keeps it as 12-byte entries of a pointer
  to an `R_` section name, a string id and the tech id; the tool finds the run of entries whose
  pointers lead to `R_` names. *Derived* (the layout is inferred, not documented). With it, 67
  of the 69 `TechRequired` values resolve to topics; eight `Prereq` links point to a topic of a
  higher `Level`; ids 75 and up (alien craft and installations) are not in it and are left out
  of conditions.
- `packs/interceptor/strategy/campaign.yaml` is written by hand on top: a generated **Frontier**
  map, **downlink** research (`research_model.rate` from the three downlink module types,
  switching loses progress), hulls inheriting `Ship_IntType1` – `Ship_IntType3`, missile
  `rounds`, module groups and effects, the X-COM base type, corporate funding, raids flying the
  original mission types, pilots and scores. `economy.cost_scale: 1000` and
  `economy.module_time: days` adapt the original's units; `start.money_from` reads the starting
  treasury for the chosen difficulty. **Every estimate is marked `DERIVED`** in the file
  (DEV-050): the start, the map's size in systems, delivery, salvage, hull speeds, rounds, module
  groups and effects, the base type's cost and hub sensor range, funding and raids. Pilot rules
  and score points are authored values too.
- Research has no authored `unlocks`: items, hulls and modules are gated by their own
  `TechRequired` through `tech_ids`.

It has **not been play-tested**. A six-month `--strategy-soak` with the automatic player
researches 4 of the 71 topics on the starting base's one downlink (the default campaign: 12 of
14), so the research pace in particular needs the developer's judgement.

## Trying a campaign out

- `openinterceptor --campaign <id> --strategy-soak <days>` (app 0.8.0) plays the strategy layer
  without a window for that many days with a simple automatic player (`campaign::autoplay`:
  every base researches the first topic it can, a lab and a workshop are built when there is
  money, every detected contact is intercepted, battles alternate win, win, retreat) and prints
  one line: the date, funds, score, bases, craft, topics known, battles and messages.
- On the strategy screen the testbed has the original's own **cheats** (app 0.8.1): Ctrl+M
  *payday* (a million), Ctrl+K *knowitall* (every topic known), Ctrl+B *quickbase* (every base
  and module under construction finished — not facilities) and Ctrl+J *jump to battle* from
  the contact chosen on the Contacts tab ([User guide › Campaign](../user-guide/campaign.md#testbed-cheats)).

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
  Victory hulls and the deep-radar and shipyard modules; and two station-only topics,
  `theremin_shielding` and `pulsar_navigation` (app 0.9.0, [Logistics](#the-default-campaigns-logistics)).
- **Economy**: delivery 48 h, salvage 15 000 a kill; the **Senate stipend** every 30 days,
  `400000 + 60000 * bases + month_score * 200 + relation(senate) * 3000` (the standing term since
  app 0.10.1); cosmium sales every 7 days,
  `12000 * module_count(cosmium_refinery)`; fuel and stores `4000 * craft` a week. Eight items
  (four beams, three missiles, cosmium), four hulls, nine module kinds (hangar, quarters, radar,
  deep radar, laboratory, workshop, shipyard, cosmium refinery, ray battery) and the `guard_base`
  type (template hangar, radar, laboratory; at most 6 bases, 3500 units apart).
- **Raids**: Dominion scouts (`intercept_scouts`), saucer raids (`defend_station`, from day
  10), colony raids (`colony_defence`, day 20), robot assaults (`outpost_raid`, day 45) and the
  Black Comet ambush by raiders (`pocket_ambush`, day 15) — each a mission of the default
  campaign ([Missions](missions.md#the-missions-shipped)); and since app 0.10.1 the Eclipse
  Engine's shipments (`eclipse_shipment`, `chance: 0%`: launched only by the
  [plan](story.md#plans)).
- **Pilots**: flying, gunnery and nerve; five ranks from cadet to commander; salary 6000,
  recruit fee 40 000, five recruits a month, 12 days to heal, 70 % survive; twelve mission
  scores.

The unit test `campaign::tests::the_default_campaign_runs_six_months` plays it for six months
with an automatic player: it stays solvent, researches at least four topics, fights raids and
round-trips through a save. `--strategy-soak 180` with the same player (app 0.8.0) reached 12 of
the 14 topics — the automatic player builds no facilities, so the two station-only topics stay
unknown.

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
| `start.money_from: original:<file>/<key> not found for level <Level>` | The original file, the difficulty level's section or the key is missing (app 0.8.0). |
| `facility_types.x.role: 'y' is not mining, manufacturing or research` | A wrong facility role (app 0.9.0). |
| `recipes.x: unknown resource or component 'y'` | A recipe input or output that is not defined. |
| `maps: system x: unknown resource 'y'` | A deposit of an undefined resource. |
| `hulls.x.components: unknown component 'y'` | A hull built from an undefined component. |
| `research.x.lab: unknown facility type 'y'` / `research.x.site: unknown research site 'y'` | A station-only topic naming something not defined. |
| `this save needs packs that are not loaded: …` | Loading a save made with a mod that is not active. |
| `save: this save has logistics but the loaded campaign has none` | Loading a save with facilities into a campaign whose logistics was removed. |
| `save: … does not match the loaded rules (was a mod changed?)` | Loading a save whose research, modules, items, incomes or raids no longer fit the rules. |

## Limits in app 0.10.3

- No endurance or stranding on the map; no fitting rules (`fits`), power draw or stock designs —
  fitting is *best available* only, and slots of kinds other than `beam` and `missile` are filled
  but have no effect in battle.
- Mission outcome actions `relation(…)`, `capture`, `cargo_lost` and `cargo_captured` are still
  **ignored** by the campaign (app 0.10.3): a story event's `relation(…)` moves a standing
  ([Story layer](story.md#actions)), a mission's does not yet. A flown convoy escort delivers its
  surviving share through `cargo:` instead.
- **Logistics**: see [Not built yet](#not-built-yet) — fighter pools, construction strikes, depots, third-party convoys, cargo capacity, facility
  upgrades and `placement:` expressions are not built.
- No designs, training, bravery, dismantling, funding evaluations or the 3D base view yet.
- Names and descriptions in the default campaign's strategy files are plain English, not text
  keys, so they are not yet translatable.
- The `interceptor` strategy layer is built but not play-tested, and its estimates are
  `DERIVED` ([above](#the-interceptor-campaign-from-the-players-data)).
- Save files record pack versions but do not compare them.
- Fixed in app 0.9.0: manufacture is paid per unit as each starts (it was free), and
  `salvaged(<raid id>)` now holds after a salvage (it compared the display name).
