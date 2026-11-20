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
- **Map units** — positions, `speed`, `sensor_range`, `base_distance(…)` — are the map's own units;
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
| `deposit(s, r)` | number | The daily rate of resource `r` in system `s` — **0 until the system is surveyed** ([logistics](#logistics), app 0.16.3). |
| `feature(s, tag)` | true/false | System `s` has the feature `tag` (known without a survey). |
| `research_site(s, site)` | true/false | System `s` meets research site `site`'s features. |
| `survey_known(s)` | true/false | System `s` has been surveyed. |
| `owned(s)` | true/false | The player has a base or a facility (finished or not) in system `s`. |
| `lane_distance(a, b)` | number | Jumps along the map's lanes from `a` to `b` (breadth-first; each lane one jump); 0 for the same system, −1 when no route exists; on a map without lanes every pair is 1 apart. |
| `facility_progress(s)` | number | The construction progress (0..1) of the least finished facility in system `s`; 1 when none is building (also when there is none). |
| `construction_attacks(s)` | number | [Construction strikes](#construction-strikes) suffered by the facilities built in system `s`. |
| `base_distance(s)` | number | Map units from system `s` to the player's nearest base, in a straight line; 1 000 000 000 with no base (app 0.19.0). |

Systems are named by their map id (`ajax`, `nebula_haven`) and are checked names like everything
else, as are resource ids, feature tags and research-site ids (app 0.16.3).

**Reserved names** (app 0.19.0). Three words have a meaning of their own as a system query's
argument and are in every campaign's name table, so a modder should not use them as ids:

| Name | Meaning |
|---|---|
| `system` | In a [`placement:`](#where-a-base-or-facility-may-be-built) condition, the candidate system being asked about. Elsewhere it names no system, and the query answers 0 (false). |
| `nearest_base` | In a `placement:` condition, the system of the player's base fewest lane jumps from the candidate (ties, and systems no lane reaches, by straight-line distance). With no base it names no system. |
| `any` | As `deposit`'s resource: the richest deposit's rate (`deposit(system, any) > 0`: the system has a deposit). As `research_site`'s site: any research site (`research_site(system, any)`). |

```text
research(ion_rays) and research(saucer_hulls)
6 + module_count(laboratory) * 12
400000 + 60000 * bases + month_score * 200
survey_known(nebula_haven) and deposit(nebula_haven, dilithium) > 10
owned(ajax) and lane_distance(mercury_gate, tycho_drift) <= 3
base_distance(system) <= 3500                      # a placement: condition
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
| `requires_any` | list of topic ids | none | Available once **any one** of these topics is known — shorthand for `research(a) or research(b)`, joined to `requires` with `and` (app 0.16.6). |
| `hidden` | bool | `false` | Not offered at all until a story event's [`discover(<topic>)`](story.md#actions) action has run (then its `requires` must still hold; a notice says *New research possible: <name>*). The discovery is kept as the campaign flag `discovered.<topic>` (app 0.16.6). |
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
| `size` | number | 1 | Storage space: what one takes of a base's store at a base type with hub [`storage`](#base_modules-and-base_types) (app 0.16.10; since 0.16.15 the base type decides). |
| `rounds` | number | 1 | Missiles: the rounds one missile slot holds; fitting a slot takes this many from the store (app 0.7.2). |
| `protection` | number | 0 | A `shield`: shield points per facing; an `armor`: armour points (app 0.15.0). An item that inherits the original's equipment and sets none takes the original's `Special` (shields) or `HitPoints` (armour) — *derived*. In battle a craft flies with the **strongest** fitted shield and the **sum** of its fitted armour, in place of its class's `shields` / `armor`; with none fitted it keeps the class's own (which `fitted:` on the [ship class](reference.md#ships-and-objects) may supply). |
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
| `slots` | mapping kind: count | none | Equipment slots: `{beam: 2, missile: 2, shield: 1, armor: 1}` (also read from the original's mount counts through inheritance). `beam`, `missile`, `shield` and `armor` slots change the craft in battle; other kinds are filled but have no effect yet. The `interceptor` hulls have one `shield` and one `armor` slot (app 0.15.0, *derived*). |
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
| `effects` | mapping name: number | none | Campaign quantities it adds while working: `research_rate`, `production_rate`, `sensor_range`, `hangar` (craft housed), `quarters` (pilots housed), `storage` (store capacity, in item `size`s), `defence` (points), `income`. |
| `requires` | condition | none | Available only while this holds. |
| `on_destroyed` | list of `craft`, `pilots`, `stock` | none | What is lost with it when a raid wrecks it (app 0.19.0; [Wrecks and repair](#wrecks-and-repair)). Another word is a load error. |

As built, the engine reads `production_rate` (manufacturing), `sensor_range` (detection and
revealing hidden systems) and `hangar` (craft a base houses). `research_rate` is a number the
`research_model.rate` expression may use through `module_count`. Since app 0.16.10:

- **`quarters`**: the pilots the player can keep — every base type's `quarters` plus every
  working `quarters` module, all bases together. Hiring is refused at the limit (*no quarters:
  room for n pilots*). A campaign that gives no quarters anywhere has no limit.
- **`storage`**: a base's store capacity, counted in item `size`s — its base type's hub
  `storage` plus its working `storage` modules. Since app 0.16.15 only a base type with hub
  `storage` above 0 has a limit; at any other base there is none, and its `storage` modules add
  nothing. (In 0.16.10 – 0.16.14 one `storage` module anywhere in the campaign limited every
  base, so an `interceptor` base without a `cargo` module could buy nothing.) What the limit
  does is under [Storage](#storage) below.
- **`defence`**: each point of a base's working modules' defence turns away 15 % of unopposed
  raids on that base on its own — 1 − 0.85^points in all (two points: 28 %, four: 48 %). A raid
  turned away is reported and destroys nothing.
- **`income`** stays a tag: an `incomes:` formula counts the modules with `module_count`.

**Dismantling** (app 0.16.10): any module of a base can be dismantled from the Build list; it is
removed at once and `economy.dismantle` of its `cost` (50 %) is refunded. Since app 0.16.15 a
`hangar` or `quarters` module is refused when the craft or pilots it houses would have nowhere to
go (*… houses craft that would have nowhere to go*), and since app 0.19.0 a `storage` module
whose stock would have nowhere to go (*… holds stock that would have nowhere to go*). A wrecked
module is cleared for nothing; a module still building, or a wreck, houses nothing, so it is
never refused. A whole base can be dismantled too ([below](#dismantling-a-whole-base)).

`base_types: {<id>: …}` — kinds of base.

| Key | Type | Default | Meaning |
|---|---|---|---|
| `name` | text | the id | Display name. |
| `cost` | number | 0 | Build price. |
| `build_time` | time | 168 h | Time to build the hub and its template modules; the base is under construction until then. |
| `max_bases` | number | 8 | The most bases of this type. |
| `limits` | mapping group: count | none | The most modules of each group. |
| `template` | ids | none | Modules a new base comes with. An entry names a module, or (app 0.19.0) a module **group**: then the best module of that group available when the base is ordered — the costliest whose `requires` holds, the later one in the file on a tie — and nothing while none is. A starting base's template is taken at the start, with the start's research known. |
| `sensor_range` | number | 0 | The hub's own sensor range, map units (modules add to it). |
| `hangar`, `quarters` | number | 0 | Craft and pilots the hub alone houses. The default Guard base houses 6 pilots (4 before app 0.16.15, which the opening crew filled). |
| `storage` | number | 0 | The hub's store capacity in item `size`s (app 0.16.15). 0: no store limit at bases of this type; above 0, its `storage` modules add to it. |
| `placement` | condition | none (anywhere) | Where a base of this type may be built: a condition over the candidate `system` (app 0.19.0; [below](#where-a-base-or-facility-may-be-built)). `base_distance(system) <= 3500` is "within 3500 map units of one of your bases". |

!!! warning "Breaking change in app 0.19.0: `max_distance` is gone"
    `base_types.<id>.max_distance` was removed in app 0.19.0; `placement:` replaces it. A mod
    that still sets `max_distance: 3500` gets *`base_types.<id>.max_distance: unknown key`* when
    the content loads and from `--check`, and the value is ignored — its bases may then be built
    **anywhere**. Write
    `placement: base_distance(system) <= 3500` instead (`max_distance: 0`, "anywhere", needs no
    `placement:`).

Both campaigns' base types since app 0.19.0:

```yaml
base_modules:
  supply_depot: {name: Supply depot, group: storage, cost: 150000, build_time: 7d, maintenance: 3000,
                 effects: {storage: 150}, on_destroyed: [stock]}

base_types:
  guard_base:                                   # the default campaign
    storage: 120                                # the hub's store, in item sizes
    placement: base_distance(system) <= 3500
    limits: {…, storage: 2}                     # at most two supply depots
    template: [hangar, sensor, laboratory]      # sensor: a group — the deep radar once known
```

The `interceptor` base type sets `placement: base_distance(system) <= 9000` (1.5 light years)
and `template: [beam_defense_laser, beam_defense_laser, downlink, shield, sensor]` — the best
downlink, shield and sensor available (*derived* from the guide). It sets no hub `storage`, so
its `cargo` modules limit nothing: the original hub's own cargo room is unknown.

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
| `training` | `{gain, cost}` | none | Pilot training (app 0.16.7): a ready pilot put into training gains `gain` points in **every** stat a day, up to the top of that stat's `stats` range, and costs `cost` money a day. Without it (or with `gain: 0`) the Hangar offers no *Train* button. Training continues until stopped; it is saved. |

`scores: {<name>: points}` — mission score names. A mission outcome's `score(<name>)` adds the
points to the campaign score and the month's score; the name `undefended` is used when a raid
reaches its target unopposed.

### `evaluations`

`evaluations: [{rating, when, do}]` (app 0.16.8; a global section like `scores`) — the patron's
monthly evaluation. At each monthly report the ratings are tried **in order** and the **first**
whose `when` holds rates the month: its `do` actions run (the [event actions](story.md#actions):
`money(-50000)`, `news(…)`, `relation(…)`, `set_flag(…)`, …) and the report ends *rated
<rating>*. A rating without `when` always applies, so put a catch-all last. No rating holds: the
report has none.

| Key | Type | Meaning |
|---|---|---|
| `rating` | text | The month's rating, shown in the report and the Reports list. |
| `when` | condition | When it applies (the campaign context; `month_score` is the month being rated). None: always. |
| `do` | list of actions | Run when it rates the month. Checked when the rules load. |

```yaml
evaluations:
  - {rating: Outstanding, when: month_score >= 400, do: ["money(150000)", "news(The Senate commends the Guard's month)"]}
  - {rating: Satisfactory, when: month_score >= 0}
  - {rating: Poor, when: month_score >= -300, do: ["money(-50000)"]}
  - {rating: Unacceptable, do: ["money(-150000)", "news(The Senate cuts the Guard's funding)"]}
```

Text in the actions may be a [text key](text.md) (`"news(@eval.cut)"`): since app 0.18.1 the
keys in `do` — and in the scripts it calls — are resolved when the content loads, and `--check`
reports a missing one (before, they stayed as the raw `@key`).

Before app 0.16.9 the section was dropped when the rules were built (it was missing from the list
of strategy globals), so a pack's ratings never reached the campaign.

### `flags` and `economy`

`flags: [<id>, …]` — campaign flags that conditions may test with `flag(x)` before any mission
sets them (`set_flag(x)` names in mission outcomes are added automatically).

`economy:`

| Key | Type | Default | Meaning |
|---|---|---|---|
| `delivery` | time | 48 h | Time for a purchase to arrive. |
| `delivery_loss` | fraction | 0 % | Chance that a purchase is lost on the way (app 0.16.11): it is paid for all the same, and a notice says so. |
| `dismantle` | fraction | 50 % | The share of a module's `cost` refunded when it is dismantled (app 0.16.10); since app 0.19.0 also of a whole base's hub and working modules, and of the hulls of craft sold when it is dismantled. |
| `repair` | fraction | 50 % | The share of a wrecked module's `cost` paid, and of its `build_time` taken, to repair it (app 0.19.0). |
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
- **The month** (every 720 hours): maintenance, upkeep and salaries are charged; the first
  [evaluation](#evaluations) that holds rates the month and runs its actions (app 0.16.8); the
  **monthly report** gives the month's score, the total, the costs, the funds and the rating; a
  new pool of recruits is offered.
- **Training** (app 0.16.7): every hour each ready pilot in training gains `training.gain` / 24
  in every stat and the campaign pays `training.cost` / 24.
- **Bases.** A new base is built in a system that has none, at most `max_bases` of a type,
  where its type's `placement:` holds; it comes with its `template` modules and is finished
  after `build_time`. A module is refused beyond its group's limit or while not available.
- **Buying** pays now and delivers to the current base after `economy.delivery` — unless the
  base's store has no room for it, counting what is already on its way
  ([Storage](#storage)) — and with the chance `economy.delivery_loss` the delivery is lost.
  **Selling** takes one item from the current
  base's store at `sell_price`. **Manufacture** queues at the base; finished units go into its
  store (hulls become craft there). The player can move a queued job up one place and cancel a
  job: a job not yet started goes without cost, a job at work keeps only the unit already paid
  for (app 0.16.11). Each unit is **paid for as
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

## Bases: storage, wrecks and placement

*App 0.19.0, plan items 4.4, 4.6 and 4.14 (Batch 4.1).*

### Storage

A base's store holds items up to its capacity in item `size`s: the base type's hub `storage`
plus its working `storage` modules (no hub `storage`: no limit). The **Build** list shows
*Store: n of m used* for a base with a limit. Since app 0.19.0 the limit is kept everywhere an
item enters the store:

- **Purchases** count what is already on its way. A purchase is refused when the items bought,
  the stock, the deliveries still coming to that base (those waiting for room too) and the item
  being made there do not fit: *no room in store: 100 of 120 used, 16 more on its way*.
- **Manufacture** of an item waits, unpaid, until the item it will make has room (counting the
  deliveries on their way). Hulls take no store room.
- **Deliveries** that find no room — a `storage` module was wrecked or dismantled while they
  were on their way — **wait at the base**: what fits is stored, the rest is held, and a
  notice says so once (*Gatehouse: no room in store for 3 x Blaster; it waits until there is*).
  A held delivery is tried again every hour and stored as soon as there is room (sell something,
  build or repair a supply depot). Its loss roll (`delivery_loss`) was made when it first
  arrived, never again. Held deliveries are saved.
- **Salvage** and captured prizes are not items in store, so they are not limited.

With logistics, facilities keep their own `storage`
([`facility_types`](#facility_types)):

- A **mining facility** stops extracting when its store is full (since app 0.9.0).
- A **manufacturing center** or **research station** orders only what its store has room for —
  its `storage` less its goods, its open orders, the convoys bound for it and the output of its
  running recipes — and a recipe starts only when its output fits once its inputs are used. A
  large order is then delivered in several smaller shipments.

A base's store of **components** (`base_goods`, what hull jobs are built from) has no limit.

A store can still end up above its capacity when a raid wrecks a `storage` module that has no
`on_destroyed: [stock]`: nothing more is stored there until it fits again.
`--strategy-soak` checks every store every day and prints *storage: every store within its
capacity* or the first day one was over ([Trying a campaign out](#trying-a-campaign-out)).

### Wrecks and repair

An unopposed raid **wrecks** the newest working module of the base it reaches. A wreck does
nothing (its `effects` stop), costs no `maintenance`, and still fills its slot (it counts towards
the group's `limits`). The Build list shows it as *Hangar: wrecked (repair 125,000)* with two
buttons:

- **Repair** pays `economy.repair` (50 %) of the module's `cost` now; the module works again
  after the same share of its `build_time` (from when the base itself is finished).
- **Dismantle** clears the wreck for nothing (no refund) and frees the slot.

`on_destroyed` on the module decides what is lost with it — whatever **no longer has room** once
it stops working:

| Entry | Lost |
|---|---|
| `craft` | Craft at the base, not away, beyond its remaining hangar room — the newest first. Their pilots eject, injured for `pilots.heal_days`. |
| `pilots` | Living pilots beyond the remaining quarters (all bases together) — those without a craft first, the newest first. They are killed. A campaign with no quarters anywhere has no limit, and loses none. |
| `stock` | Items from the end of the base's store, one at a time, until the stock fits the remaining capacity. |

A module without `on_destroyed` loses nothing with it. One notice names the module and what was
lost: *Gatehouse: Hangar wrecked; lost: COMET-3* (craft are named by hull and number). Wrecks
are saved.

```yaml
base_modules:
  hangar:   {…, effects: {hangar: 3}, on_destroyed: [craft]}
  quarters: {…, effects: {quarters: 6}, on_destroyed: [pilots]}
  supply_depot: {…, effects: {storage: 150}, on_destroyed: [stock]}
```

The `interceptor` pack sets `on_destroyed` on its `hangar` (`craft`), `personnel` (`pilots`) and
`cargo` (`stock`) modules — authored, from what the guide says is lost with them.

### Dismantling a whole base

With more than one base, the Build list's row *&lt;base&gt;: the whole base* has **Dismantle base**
(a button only; Enter and a double click never do it). It is refused for the last base, and when
the base's quarters house pilots who would have nowhere else to sleep. Otherwise:

- `economy.dismantle` (50 %) of the base type's `cost` and of each working module's `cost` comes
  back (wrecks bring nothing);
- its **craft** move to the other bases with hangar room (or no hangar limit), nearest first;
  those with nowhere to go
  are sold for the same share of their hull's `cost`;
- its **stock** moves to the nearest bases with room; what does not fit is sold at `sell_price`;
- **deliveries** and **convoys** bound for it, and its store of components, go to the nearest
  other base; its open orders are dropped (they are ordered again from the new base's demand);
- its manufacturing queue stops, and its research stops (the topic keeps its progress);
- a raid heading for it loses its target.

The log says *&lt;base&gt; dismantled (&lt;refund&gt; back)*, adding *; 2 craft sold for want of hangar
room* when some were.

### Where a base or facility may be built

`placement:` on a base type or a facility type is a [campaign condition](#expressions-the-campaign-context)
about the **candidate system**: in it `system` names the system being asked about, and
`nearest_base` the player's base fewest lanes from it. It is checked when the player orders the
base or facility, and on the Supply list and the info panel for every facility type (the
Supply list offers a refused facility with the reason, *Tycho Drift does not suit a
Manufacturing center*, adding *(not surveyed yet)* whenever the system has not been surveyed,
whichever part of the condition failed; the info panel leaves it out). A base refused by
it says *A &lt;type name&gt; base cannot be built at &lt;system&gt;*. Asking never changes the game: a
`random(p)` in a placement condition rolls a copy of the generator.

Rules that hold besides `placement:`: one base per system and at most `max_bases` of a type; at
most one facility of each type per system.

A facility type without `placement:` follows its **role's default rule**, which is itself a
condition (app 0.19.0; until then they were fixed rules in the code):

| Role | Default `placement:` | Meaning |
|---|---|---|
| `mining` | `survey_known(system) and deposit(system, any) > 0` | A surveyed system with at least one deposit. |
| `research` | `survey_known(system) and research_site(system, any)` | A surveyed system that meets at least one research site. |
| `manufacturing` | `bases > 0 and lane_distance(system, nearest_base) <= 2` | A system at most two lanes from one of the player's bases (0 for the base's own system; a system no lane reaches answers −1, so it is allowed, as before). |

To change a rule, copy the default and edit it:

```yaml
facility_types:
  # a mine only on a rich deposit, and not in the middle of nowhere
  mining_facility: {…, placement: "survey_known(system) and deposit(system, any) >= 3 and base_distance(system) <= 6000"}
  # a plant only in a nebula
  manufacturing_center: {…, placement: "feature(system, nebula) and lane_distance(system, nearest_base) <= 3"}
```

A base type without `placement:` may be built anywhere (but one per system). The default
campaign's manufacturing center writes its role's rule out in full, as an example.

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
   `mission`, with the first craft's hull as the player's ship, **auto-resolve** it (app
   0.16.12), or **break off** (the contact flies on). Auto-resolve rolls the result from both
   sides' strength: each craft's power is √(effective hit points × sustained damage per second)
   of its class (the [`--balance`](#trying-a-campaign-out) numbers), a side's power the sum —
   the player's craft at the contact against the mission's hostile roles at their average
   counts — and the player wins with p = P² / (P² + E²). The mission's own outcome actions for
   the rolled `win` or `lose` apply as for a flown battle (DEV-057). A won auto-resolve credits
   the kills of the hostile roles only, each at its smallest count (`balance::hostile_count`,
   app 0.16.15).
4. The battle's debrief comes back to the campaign: the outcome's actions apply —
   `score(<name>)`, `money(<n>)`, `set_flag(<id>)`, `relation(<faction>, <n>)`, `salvage` (money:
   `salvage_per_kill` × kills, and the raid's **id** is recorded, so `salvaged(<raid id>)` holds
   from then on — fixed in app 0.9.0; before, the display name was recorded and never matched),
   and since app 0.16.3:
   - `capture` — every prize the player's tractor beam brought home (the battle reports each as
     `captured(<class>)`) is kept as **salvage**, so `salvaged(<class id>)` holds and research
     can start from a capture; without `capture` in the outcome nothing is kept;
   - `lose(<facility type>)` — the player's facility of that type in the system of the raid's target base is
     lost (since app 0.16.15; else the most recently built one of that type) — its orders and the
     convoys bound for it go too, and a notice says so; any other argument is only logged;
   - `cargo_lost`, `cargo_captured` — recorded in the log (a convoy escort's own `cargo:` share
     settles the cargo).

   A stance change that a `relation(…)` outcome causes is announced as soon as the map is back,
   and stops the clock (app 0.16.2). A flown convoy escort or construction defence applies its
   outcome's actions too (app 0.16.3, 0.16.4).
   The leader is credited the kills and experience (kills + 1); every pilot in the party a
   mission. A `killed` result loses the craft, and its pilot ejects injured (`survive`) or dies.
   `win` and `draw` remove the contact; `lose` and `retreat` leave it flying. Leaving a campaign
   battle early from the pause menu is a `retreat`.
5. A contact that reaches its target **unopposed**: a base target's newest working module is
   **wrecked** ([Wrecks and repair](#wrecks-and-repair); until app 0.19.0 it was removed), and
   the `undefended` score applies — unless the base's [`defence`](#base_modules-and-base_types)
   turns it away (app 0.16.10).

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
| `storage` | number | 500 | The most goods its store holds. Mining: extraction stops when it is full. Since app 0.19.0 manufacturing centers and research stations keep it too: they order only what fits, and a recipe starts only when its output fits ([Storage](#storage)). |
| `lines` | number | 1 | Manufacturing: recipes running at once. |
| `research_rate` | number | 0 | Research: work units an hour on its project. |
| `hub` | id | none | The battle object at the facility, with its construction-stage looks (read; not yet used — R-STR-022). |
| `placement` | condition | its role's rule | Where it may be built: a condition over the candidate `system` (app 0.19.0). Without it, its role's default rule. |

**Where a facility may be built**: at most one facility of each type per system, where its
`placement:` holds — by default its role's rule (a surveyed system with a deposit for mining, a
surveyed research site for research, at most two lanes from a base for manufacturing). The
rules, the reserved names and examples are under
[Where a base or facility may be built](#where-a-base-or-facility-may-be-built). Until app
0.19.0 these were fixed rules in the code (DEV-049).

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
| `construction_strikes` | `{from, max, chance, mission, setback}` | none | Enemy strikes on facilities under construction ([below](#construction-strikes), app 0.16.4). Without it there are none. |
| `ambush_mission` | mission id | none | The battle flown when the player escorts an ambushed convoy (app 0.9.2). With it, an ambushed convoy **waits** where it is and the clock stops: the player flies its escort, and the share of the mission's [`cargo:`](missions.md#the-mission) role still aboard at the end is delivered — or lets it fight through (auto-resolved as below). Without it every ambush is auto-resolved. An unknown mission is a load error. The default campaign sets `convoy_ambush`. |

### How the chain runs

Every hour, after the rest of the hour step ([Architecture › Campaign](../implementation/architecture/campaign.md#logistics)):

1. **Demand at bases.** Each hull job not yet started needs its hull's `components`. What the
   base's store lacks, less what is already on its way, is **ordered** from the nearest ready
   manufacturing center (nearest by straight-line distance). A manufacturing center or research
   station orders no more than its store has room for (app 0.19.0).
2. **Mining facilities** extract each deposit of their system at `rate` / 24 an hour, until the
   store is full (`storage`) or the reserve runs out.
3. **Manufacturing centers** finish runs (the output goes into the store), then look at the
   orders they must fill: when the store plus the runs under way do not cover an order, a free
   line starts the component's recipe if its inputs are in store and (app 0.19.0) its output
   fits the store once they are used; if the inputs are not there, the missing ones are ordered
   from the nearest mining facility whose system has a deposit of them.
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
   **surveyed** and a message lists its deposits (with rates a day and the reserve, or
   *unlimited*), its features and the research sites it meets (reserves and features since app
   0.16.2).
9. **Construction strikes** (app 0.16.4, with `construction_strikes`): see below.

An order is placed only when a facility exists that can fill it: a base without a reachable
manufacturing center simply waits. Facilities report when they are complete.

### Construction strikes

`logistics.construction_strikes` (app 0.16.4, R-STR-023) lets the enemy strike a facility while
it is being built:

| Key | Type | Default | Meaning |
|---|---|---|---|
| `from` | fraction | 50 % | Construction progress from which a site can be struck. |
| `max` | number | 3 | Strikes at most per **location** (all facilities in one system together). |
| `chance` | fraction | 10 % | Chance per day for each eligible site. |
| `mission` | mission id | none | The defence the player flies (`construction_defence`). None: every strike is let be. |
| `setback` | fraction | 25 % | A strike not flown sets construction back this share of the facility type's `time`. |

```yaml
logistics:
  construction_strikes: {from: 50%, max: 3, chance: 12%, mission: construction_defence, setback: 25%}
```

Each hour every site past `from` whose system has had fewer than `max` strikes rolls
`chance` (per day); one strike at a time stops the clock with a prompt:

- **Fly the defence** — the `mission` with the first ready craft: a **win** beats the strike
  off; a **loss** destroys the site (removed with its orders); a retreat or draw counts as not
  flown. The mission's own outcome actions apply too.
- **Let it be** — or no ready craft, or no `mission` — the strike sets the site back by
  `setback` × its build time.

Every strike counts towards `construction_attacks(<system>)` (saved).

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
[Missions › The convoy ambush](missions.md#the-convoy-ambush-app-092)), and construction sites are
struck since app 0.16.4 ([above](#construction-strikes)). Not built: facility fighter pools and escorts
(R-STR-025), construction-stage looks on the map and in battle (R-STR-022: the
`construction_defence` mission shows its own research station at 60 %, whatever was struck), captured cargo and enemy supply depots (R-STR-020),
third-party convoys (R-STR-027), supply-vessel cargo capacity (R-STR-026) and facility
upgrades. `placement:` conditions and storage at every facility are built since app 0.19.0.

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
researches 4 of the 71 topics on the starting base's one downlink (the default campaign: 14 of
16) and fights one battle (app 0.19.0, main `c28fc2c`), so the research pace in particular needs
the developer's judgement.

## Trying a campaign out

- `openinterceptor --campaign <id> --strategy-soak <days>` (app 0.8.0) plays the strategy layer
  without a window for that many days with a simple automatic player (`campaign::autoplay`:
  every base researches the first topic it can, a lab and a workshop are built when there is
  money, every detected contact is intercepted, battles alternate win, win, retreat) and prints
  one line: the date, funds, score, bases, craft, topics known, battles and messages. Since app
  0.19.0 the automatic player also stocks the first base's store with the cheapest item that takes
  room (buying while it has more than 3 000 000, making it while the queue is idle), the soak
  plays day by day, and a line *storage: every store within its capacity* — or *storage: OVER
  CAPACITY first on day n: &lt;place&gt;: &lt;used&gt; of &lt;capacity&gt;* — checks every base's and facility's
  store each day (`Campaign::over_capacity`).
- On the strategy screen the testbed has the original's own **cheats** (app 0.8.1): Ctrl+M
  *payday* (a million), Ctrl+K *knowitall* (every topic known), Ctrl+B *quickbase* (every base
  and module under construction finished — not facilities) and Ctrl+J *jump to battle* from
  the contact chosen on the Contacts tab ([User guide › Campaign](../user-guide/campaign.md#testbed-cheats-and-tools)).
- **Ctrl+D** shows the **Debug** list (app 0.16.13): each arc's step and every story event with
  how often it has fired and whether its condition holds now — the quickest way to check a
  condition you have written (the [logistics queries](#expressions-the-campaign-context)
  included); *Fire now* fires one ([Story layer › Trying a story out](story.md#trying-a-story-out)).
- `openinterceptor --campaign <id> --strategy --chapter <n>` (app 0.16.13) plays a new campaign
  forward headless with the automatic player until its first arc reaches step *n* (at most 1500
  days), then opens the strategy screen there.

## The default campaign as the example

*Perils of the Void* (`resources/default/strategy/`) uses every section:

- **Start**: 1 March 2084, 2 500 000 in funds, the Gatehouse base (`guard_base`) at Mercury
  Gate, three Comets, four pilots, `rocketry` known, six blasters, twelve seeker rockets, three
  deflector screens and three steel platings (app 0.16.1).
- **Map** `reach`: fixed, lane travel, 1 unit = 0.001 ly; six systems — Mercury Gate, Ajax,
  Cinder, Vesta Reach, Tycho Drift, Nebula Haven — joined by seven lanes, and **The Dark Moon**,
  a hidden black hole far to the east, found only when a base's sensors reach it.
- **Research**: labs (`6 + module_count(laboratory) * 12` per hour, `keep_progress`); twelve
  topics from `rocketry` through `heat_rays`, `ion_rays`, `saucer_hulls` (requires
  `salvaged(saucer_raid) or salvaged(scout_probe)`), `saucer_propulsion`, `disintegrators`,
  `deep_radar`, `shipyards`, `thunderbolt_design`, `robot_brains` (`salvaged(robot_raid)`),
  `victory_design` and `regents_engine`, plus `force_fields` (after `heat_rays`) and
  `saucer_alloys` (after `saucer_hulls`) since app 0.16.1 — fourteen in all — unlocking beams,
  missiles, the force field and saucer-alloy plating, the Meteor, Thunderbolt and Victory hulls and
  the deep-radar and shipyard modules; and two station-only topics,
  `theremin_shielding` and `pulsar_navigation` (app 0.9.0, [Logistics](#the-default-campaigns-logistics)).
- **Economy**: delivery 48 h, salvage 15 000 a kill; the **Senate stipend** every 30 days,
  `400000 + 60000 * bases + month_score * 200 + relation(senate) * 3000` (the standing term since
  app 0.10.1); cosmium sales every 7 days,
  `12000 * module_count(cosmium_refinery)`; fuel and stores `4000 * craft` a week. Twelve items
  (four beams, three missiles, cosmium, and since app 0.16.1 two shields — deflector screen 300,
  force field 480 — and two armours — steel plating 180, saucer-alloy plating 380), four hulls
  (each with a `shield` and an `armor` slot since 0.16.1), ten module kinds (hangar, quarters, radar,
  deep radar, laboratory, workshop, shipyard, cosmium refinery, ray battery and, since app 0.19.0,
  the supply depot: 150 more store room, at most two a base) and the `guard_base` type
  (template hangar, the best `sensor` — the radar, the deep radar once known — and laboratory;
  at most 6 bases, `placement: base_distance(system) <= 3500`; a hub store of 120 since app
  0.19.0). The hangar loses its craft and the quarters their pilots when wrecked
  (`on_destroyed`); `economy.dismantle` and `economy.repair` are both 50 %.
- **Raids**: Dominion scouts (`intercept_scouts`), saucer raids (`defend_station`, from day
  10), colony raids (`colony_defence`, day 20), robot assaults (`outpost_raid`, day 45) and the
  Black Comet ambush by raiders (`pocket_ambush`, day 15) — each a mission of the default
  campaign ([Missions](missions.md#the-missions-shipped)); and since app 0.10.1 the Eclipse
  Engine's shipments (`eclipse_shipment`, `chance: 0%`: launched only by the
  [plan](story.md#plans)).
- **Pilots**: flying, gunnery and nerve; five ranks from cadet to commander; salary 6000,
  recruit fee 40 000, five recruits a month, 12 days to heal, 70 % survive; training 0.4 points
  and 400 a day (app 0.16.7); twelve mission scores. The Gatehouse's `guard_base` houses six
  pilots (four until app 0.16.15, which the four you start with filled); more need a Crew
  quarters module (app 0.16.10).
- **Evaluations** (app 0.16.8): the Senate rates each month *Outstanding* (a month's score of 400
  or more: +150 000 and praise), *Satisfactory* (0 or more), *Poor* (−300 or more: −50 000) or
  *Unacceptable* (−150 000 and a funding-cut notice).
- **Logistics**: construction strikes at 12 % a day from half-built, three at most per system,
  flown as `construction_defence`, a quarter of the build time set back when let be (app 0.16.4).

The unit test `campaign::tests::the_default_campaign_runs_six_months` plays it for six months
with an automatic player: it stays solvent, researches at least four topics, fights raids and
round-trips through a save; the year-long test also asserts that no store is ever above its
capacity (app 0.19.0). `--strategy-soak 180` with the same player (app 0.8.0) reached 12 of
the 14 topics then; at app 0.19.0 (main `c28fc2c`) it reaches **14 of the 16** (the twelve
topics plus `force_fields` and `saucer_alloys`) and fights 47 battles, with every store within
its capacity — the automatic player builds no facilities, so the two station-only topics stay
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
| `evaluations[0].when: …` / `evaluations[0].do: …` | A rating's condition or action that does not parse or names something not defined (app 0.16.8). |
| `research.x.requires: unknown name 'y'` | Also for a misspelt topic in `requires_any` (it becomes part of `requires`). |
| `base_modules.x.on_destroyed: 'y' is not craft, pilots or stock` | A wrong `on_destroyed` entry (app 0.19.0). |
| `base_types.x.template: unknown module or module group 'y'` | A template entry that is neither a module id nor a module group (app 0.19.0). |
| `base_types.x.placement: …` / `facility_types.x.placement: …` | A placement condition that does not parse or names something not defined (app 0.19.0). |
| `base_types.x.max_distance: unknown key` | A mod written before app 0.19.0: use `placement: base_distance(system) <= <n>`. |
| `this save needs packs that are not loaded: …` | Loading a save made with a mod that is not active. |
| `save: this save has logistics but the loaded campaign has none` | Loading a save with facilities into a campaign whose logistics was removed. |
| `save: … does not match the loaded rules (was a mod changed?)` | Loading a save whose research, modules, items, incomes or raids no longer fit the rules. |

## Limits in app 0.19.0

- No endurance or stranding on the map; no fitting rules (`fits`), power draw or stock designs —
  fitting is *best available* only, and slots of kinds other than `beam`, `missile` and (since app
  0.15.0) `shield` and `armor` are filled but have no effect in battle.
- `cargo_lost` and `cargo_captured` are only recorded: captured cargo goes to no enemy depot.
  `lose(<x>)` acts only on a facility type; a base type or anything else is logged.
- **Logistics**: see [Not built yet](#not-built-yet) — fighter pools, construction stages, depots, third-party convoys, cargo capacity and facility
  upgrades are not built.
- No designs, bravery, stat effects in battle, or the 3D base view yet. Auto-resolve counts hull classes only (not fitted items or pilots) and does not damage
  the player's craft (DEV-057).
- A store limit applies only at a base type with hub `storage`. The default campaign's Guard base
  has 120 (and supply depots add 150 each); the `interceptor` base type sets none, so its
  `cargo` modules limit nothing — the original hub's cargo capacity is unknown (an open fidelity
  item).
- Names and descriptions in the default campaign's strategy files are plain English, not text
  keys, so they are not yet translatable.
- The `interceptor` strategy layer is built but not play-tested, and its estimates are
  `DERIVED` ([above](#the-interceptor-campaign-from-the-players-data)).
- A save made with another version of a loaded pack loads with a notice naming the versions
  (app 0.16.2); its rules may differ.
- Fixed in app 0.9.0: manufacture is paid per unit as each starts (it was free), and
  `salvaged(<raid id>)` now holds after a salvage (it compared the display name).
