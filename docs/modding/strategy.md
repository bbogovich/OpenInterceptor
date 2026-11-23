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
| `alien_tech` | number | App NEXT: the rival's tech level, 1 – 3 ([`strategist.economy`](story.md#strategist); 1 without it). |
| `centre_distance(s)` | number | App NEXT: map units from system `s` to the map's centre — a generated map's middle, else the middle of the systems' bounding box. `interceptor`'s bases must be within 30,000 (5 ly) of it (**verified**). |

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
| `names` | list of text | none | Generated: system names to draw from (then numbered). An entry `original:<file>/<section>` (app 0.22.17) stands for the lines of `[section]` in a text file of the player's data — `interceptor` uses `original:starcorp1.txt/STAR NAMES`, the original's 101 star names. A missing file or section is a load warning. |
| `system_counts` | map of count → weight | none | Generated: the system count rolled from this table instead of `systems` (app 0.22.21). `interceptor`: `{65: 11, 70: 10, 75: 70, 80: 9}`, the original's roll (**verified**, `unknowns-2026-11.md` §13.5). |
| `seed` | number | the campaign's | Generated: a fixed seed, so every campaign gets the same map. |
| `black_holes` | number or `[min, max]` | 1 | Generated: how many systems are hidden black holes. |
| `black_hole_spacing` | number | 0 | Generated: with a spacing (app 0.22.22), the black holes are `black_holes` (rolled in its range) existing systems picked at random, each at least this far from the others; without, the last systems become them. `interceptor`: `[4, 5]` and 7,500 (1.25 ly), the original's (**verified**, §14.6). |
| `unit_ly` | number | 0.001 | Light years per map unit (display only). |

Fixed-map systems also take `bodies` (a list of planets, moons and belts; descriptive) and, for
a campaign with [logistics](#logistics), `deposits` and `features` (app 0.9.0).

`star_classes: {<id>: {color: [r, g, b], weight, icon}}` — how a star of that class is drawn, and
(for generated maps) how often it appears relative to the others (`weight`, default 1; 0 = never
rolled). `icon` (asset, app 0.22.17) draws that image at its own size, centred on the star, instead
of a dot; `interceptor` uses the CSD's 9 × 9 class icons (`original:sprite#893` – `#901`,
**verified**; which original class each of its six classes stands for is **derived**). Generated systems are scattered at least a little apart, take a class by weight and a
name from `names`. `portrait` (asset) and `portrait_frames` (number, default 1; app
NEXT) are the system picture a map with [zoom levels](screens.md#zoom-levels) shows for a
system of that class, animated through rows N … N + frames − 1.

Since app 0.22.21 `interceptor` has the original's nine classes (red giant, blue giant, white
dwarf, blue, white, yellow, orange, red dwarf, black hole) weighted by its class roll (**verified**,
§13.5; black holes weigh 0 and are placed by `black_holes`), each with its CSD icon and a
portrait: five 20-frame star pictures (**verified** since app 0.22.22, §14.4): 454 yellow, 474
orange, 494 red giant and red dwarf, 514 blue giant and blue, 534 white dwarf and white; black
holes have none.

`planets` (list of numbers, app 0.22.24): the planet counts a generated system of the class draws one
from at random (empty: none; drawn after the positions and classes, so a map is otherwise the
same). `interceptor` gives each class the original's results (**verified**, `0x5dab1e`): 1 plus the
class's entry in a 10 × 9 table (row `rand % 10`), held to 2 – 5 — a yellow star `[3, 3, 3, 4, 4, 5,
5, 5, 5, 5]`, a red dwarf always 2. The count is kept with the system and saved; no screen shows it
yet.

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
| `requires_all` | list of topic ids | none | App NEXT: topics that must **all** be known as well — `research(a) and research(b)` joined to `requires` (and to the translated `prereq*`). `interceptor`: Nova Bomb's five extra prerequisites, which the original's code adds to its INI ones (**verified**). |
| `level` | number | 0 | App NEXT: the original's `Level` (read through inheritance); a finished topic earns `research_model.score_by_level` at its level. |
| `requires_item` | item id or list | none | App 0.22.25: an **artefact-gated** topic (the original's Prereq 127, RQ-S-03): offered only once one of these items has been *recovered* by salvage (the destroyed ships' [`salvage_fits`](#research-and-research_model), raids' [`salvage_items`](#raids)) and only while one stays in some base's stores. A list (app 0.22.27) accepts any of them — the original takes a launcher or its torpedoes. When none is left (sold, lost) the topic closes, and stays closed until another is recovered. Unknown item: load error. |
| `unlock_on` | list of events | none | App NEXT: an **event-gated** special topic, offered once any of these battle events has happened: `destroyed(<class>)` (an enemy of that class destroyed), `captured(<class>)` (captured or disabled), `won(<raid>)` or `won(<raid>#<n>)` (that raid's n-th won battle), `kind(<kind>)` (a won raid's [`kinds`](#raids) pick, app 0.22.30). It stays open. |
| `complete_on` | list of events | none | App NEXT: the same events finish the topic outright (the original's topics 74 / 75 for captured freighters and AlienType5). |
| `lab`, `site`, `start_cost` | | none | A **station-only** topic: see [Logistics › research stations](#research-stations-and-station-only-topics) (app 0.9.0). |

`research_model:` — how research progresses.

| Key | Type | Default | Meaning |
|---|---|---|---|
| `mode` | `download` / `labs` | `download` | How it is described: the original's downloads, or staff in labs. The rule is the same: each base works on **one** topic at a time. |
| `rate` | expression | 10 | Work units per hour at one base, evaluated per base (so `module_count` counts that base's modules). |
| `switching` | `keep_progress` / `lose_progress` | `keep_progress` | Whether switching a base to another topic keeps the work done on the old one (with `lose_progress` it is lost unless another base is still working on it). |
| `score_by_level` | list of numbers | none | App NEXT: the score a finished topic earns, by its `level` (a level past the list takes the last). `interceptor`: `[250, 250, 500, 750]` (**verified** +250 / +500 / +750 for Levels 1 – 3; Level 0 taking 250 is *derived*). `default`: `[100, 200, 300]`. |

A topic finishes when its work reaches `size`; every base working on it stops, a message
says so, and its `score_by_level` score is added to the month's.

`goodies: {<id>: …}` (app 0.22.25) — salvage chances for raids' `salvage_items`.

| Key | Type | Default | Meaning |
|---|---|---|---|
| `chances` | list of numbers | none | Per cent by difficulty level, easiest first; a level past the list takes the last. |
| `beginner` … `superhuman` | number | none | The same as the original's keys, read through inheritance: `interceptor` inherits `original:GameSettings.ini/BeamGoodies`, `MissileGoodies`, `FreighterGoodies` and `AlienCorpseGoodies` (**verified** tables, `unknowns-2026-11.md` §11.2). |

`interceptor` (app 0.22.27, `unknowns-2026-11.md` §15, **verified** unless marked) follows the exe's
table at 0x66f768: eleven special topics need an item — the five corpses, the three torpedoes (the
launcher or its torpedoes) and the Phase, Plasma and Psi beams. The others open on an event: the
saucer examinations on `captured(wraith | spectre | phantasm)`, Shade on `captured(shade)`, the
probes by the probe's type (`kind(sensor_probe | intel_jammer_probe | comm_jammer_probe)`; of the
original's seven types the other four give nothing; the `alien_probe` raid's `kinds` stand for
them, one per contact, all as likely: *derived*),
the Doomsday pieces on `won(doomsday_convoy#1 | #2 | #3)` (the n-th convoy carrying piece n:
*derived*). Still open as before: the mines (the flight layer does not report destroyed mines
yet), Poltergeist and the Drone (no source known). Topics 74 and 75 are not in this pack's tree.

`salvage_fits: {<ship class>: [{item, goodies}]}` (app 0.22.27) — what a destroyed enemy of that
class may yield: after a **won** battle each enemy destroyed (reported by the battle as
`destroyed(<class>)`) rolls each item of its class with the goodies chance, into the home base
of the first craft in the battle. `interceptor`: the special topics' weapons each alien class
carries (§15.2, **verified** fits; tech levels are not modelled, so each class lists every such
weapon it carries at any level: *derived*).

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
| `rounds` | number | 1 | Missiles: the rounds one missile slot holds; fitting a slot takes this many from the store (app 0.7.2). `interceptor` (app 0.22.19): each `*Launcher`'s `Rounds` — X-Winder 5, Permeator 3, Sidestep 4, Fusion 2, Elerium 4, Enveloping 3, Parallax 3, Passive Cloaking 2, Doppelganger 1, Nova 1 (**verified**). |
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
| `sensor_range` | number | 0 | The hub's own sensor range, map units (modules add to it). `interceptor`: 0 since app 0.22.24 — the original's base sees only through its sensor module: SensorType1 – 3 scan 2,000 / 4,000 / 6,000 map units (⅓, ⅔, 1 ly; **verified**, strategy README §3.2). |
| `hangar`, `quarters` | number | 0 | Craft and pilots the hub alone houses. The default Guard base houses 6 pilots (4 before app 0.16.15, which the opening crew filled). |
| `storage` | number | 0 | The hub's store capacity in item `size`s (app 0.16.15). 0: no store limit at bases of this type; above 0, its `storage` modules add to it. |
| `placement` | condition | none (anywhere) | Where a base of this type may be built: a condition over the candidate `system` (app 0.19.0; [below](#where-a-base-or-facility-may-be-built)). `base_distance(system) <= 3500` is "within 3500 map units of one of your bases". |
| `price_share`, `hub_cost` | number | 0, 0 | App NEXT (RQ-S-08): when `price_share` is above 0, a new base costs `price_share` × (`hub_cost` + its `template`'s modules as they would be chosen now), in place of `cost`. `interceptor`: 0.75 and 1,500,000 (**verified**, README §3.1). |
| `hull` | number | 0 | App NEXT: the hub's hull points (0: none). An unopposed strike takes `strike_damage`; `repair` mends them. `interceptor`: 6,000, the station chart's HP1 (**verified**); `default`: 4,000. |
| `repair` | `{per_minute, min_step, boost}` | none | App NEXT (RQ-S-16): `per_minute` hull points a game minute, at least `min_step` a step (an hour at most), times each `boost: {<module>: factor}` while one works. `interceptor`: 2, 5, Advanced Damage Control × 2 (**verified**, README §3.2). |
| `strike_damage` | number | 0 | App NEXT: hull points an unopposed strike takes (`interceptor`: 1,500, *derived*). |

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
| `kinds` | list of ids | none | App NEXT: what a launch of this raid may be (`interceptor`'s `alien_probe`: the original's seven probe types); one is picked per contact and a **won** battle reports it as the event `kind(<kind>)` for research's `unlock_on`. Every kind is as likely. |
| `salvage_items` | list of `{item, goodies}` | none | App 0.22.25: artefacts a **won** battle with this raid may recover besides the destroyed ships' `salvage_fits` (`interceptor`: the alien corpses, which raid yields which race *derived*): each is rolled once with its [`goodies`](#research-and-research_model) table's chance at the campaign's difficulty and goes to the home base of the first craft in the battle (base 0 if none), opening the topics whose `requires_item` it is. Unknown item or table: load error. |
| `react_on_detect` | fraction (`per_level`) | 0 | App NEXT: once a base's sensors see the contact, with this chance it turns on that base — it heads for it (along the lanes on a lane map) and strikes it if not stopped. A plan's shipment keeps its course. `interceptor`: alien raids `{per_level: [25%, 35%, 45%, 55%, 65%]}` (the original's 25 + 10 × level %, **verified**; that it re-tasks against the detector is *derived*). |

On a free-travel map a launched raid appears well outside the map, in a random direction, and
flies straight for its target; on a lane map it enters at a system and follows the lanes
([Travel](#travel)). See [Raids and battles](#raids-and-battles).

### `incomes` and `expenses`

`incomes: {<id>: {name, every, amount}}` and `expenses: {<id>: {name, every, amount}}` — an
amount (an expression) added or taken every period (`every`, default 30 days). Each payment is a line in the message log.
Module `maintenance`, hull `upkeep` and pilot `salary` are charged monthly on their own; use
`expenses` for anything else.

### `manufacture`

`manufacture: {per_outpost, per_opp}` (app 0.22.29, RQ-S-09) — production by the
[outposts](#outposts)' technicians instead of the bases' `production_rate`: the capacity is
`per_outpost` + `per_opp` × its OPPs technicians for each outpost (default 10 and 10), and a job
gains one work unit per technician-hour, so an item's `prod_time` is in technician-hours. The
capacity is one pool taken in queue order across the bases (the first base's first job first);
what a base leaves over goes on to the next. `interceptor` has it (README §2.5, **verified**: two
outposts with two OPPs are 60 technicians, and a 500-hour item takes 8.3 hours); that the first
job takes the whole capacity is *derived* (the exe gives each job a percentage share). `default`
keeps its workshops.

### `outposts`

`outposts:` (app 0.22.26, RQ-S-04/05) — corporate outposts as campaign objects: they pay the
player, and how well funded they are follows how the player does against the aliens where they
are. Without the section there are none, and `incomes` pay as written. `interceptor` has them in
place of its old flat corporate income; `default` has a small set beside the Senate's stipend.

| Key | Type | Default | Meaning |
|---|---|---|---|
| `exchange_rate` | number | 250 | Money per yield point. `exchange_rate_from: original:<file>/<key>` reads it from the difficulty level's section, as `start.money_from` does (`interceptor`: `GameSettings.ini/ExchangeRate`, 250 / 250 / 225 / 225 / 200). |
| `yield_table` | rows of numbers | none | An OPP's yield by its body class (row 1 – 3) and richness (column 0 – 4). `interceptor`: the exe's table at 0x65f860 (**verified**). |
| `start_funding`, `max_funding` | fraction | 0.75, 0.95 | A new outpost's funding, and its ceiling. |
| `min_age` | time | 638.4 h | An outpost pays only once this old (0.95 × 28 days). |
| `withdraw_below`, `withdraw_cost`, `withdraw_score` | number | 0.03, 1500, 1000 | Funding under `withdraw_below` makes the outpost withdraw: `withdraw_cost` is lost and `withdraw_score` taken from the player's score. |
| `evaluation.pool_weights` | `[w0, w1]` | `[0.1, 0.2]` | Weights of the player's and the aliens' month totals in each system's evaluation. |
| `evaluation.steps` | list of `[threshold, step]` | the original's | The first threshold ≥ the evaluation gives the step; funding moves by 1 … \|step\| percentage points at random, with the step's sign. |
| `growth` | `{per_level, max_per_month, max}` | none | After a month the player out-scored the aliens: `[min, max]` new outposts by difficulty level, at most `max_per_month` a month and `max` in all. |
| `start` | `[min, max]` | `[0, 0]` | Outposts at the start. |
| `min_opps` | number | 1 | OPPs per outpost at least; otherwise one per planet of its system (at most 20). |

Each month, before the report (strategy README §2.2 – §2.3, **verified**):

1. **Income.** Every outpost old enough pays round(`exchange_rate` × Σ its OPPs' yields ×
   funding). Three class-2, richness-2 OPPs at Veteran: 225 × 2,700 × 0.75 = 455,625.
2. **Evaluation.** With X and A the player's and the aliens' month totals (systems and pools), each
   system with outposts gets e = X[i] + X × w0 − (A[i] + A × w1), and its outposts' funding moves
   by the step, clamped to [0, `max_funding`].
3. **Withdrawal** of outposts under `withdraw_below`.
4. **Growth** when X > A: new outposts in the best-scoring systems (each picked once a month).
5. The month's system scores start again.

**Scores belong to systems** (RQ-S-06): a battle's score goes to the system nearest the contact
when an outpost is there, else to a pool; research and story scores go to the pool. A positive
score counts for the player, a negative one for the aliens (*derived*: the engine's score names
carry their side in their sign). The map marks a system with an outpost (`outpost_marker`, see
[Screens](screens.md#widget-keys)); the Outposts report lists them.

**Derived** (DEV-098): the starting outposts (`interceptor`: 3, in the systems nearest the base),
the OPPs' body classes and richness (random), where new outposts go among the candidates (best
score, then nearest a base), and the score sign convention. Mission scores by the original's
`MissionScores` are not read yet: which counter is which event needs the flight layer to report
kills by type.

### `auto_resolve` and `craft_damage`

`auto_resolve:` (app NEXT) — an auto-resolved interception is fought as the original's hourly
combat simulation (flight README §13, **verified**) instead of one roll of odds; without the section
(or with `model: odds`) the roll stays (DEV-057). The craft at the contact, with their fits,
battle damage and pilots, fight the raid's mission's hostile roles (each at a count rolled in its
range, each with a pilot rolled like a recruit); the mission's friendly roles join the craft, and
at a targeted base its station (below) fights too. The contact stops while the battle runs.

1. The first round comes `first_round` after the start, then one every `round`.
2. Each round both sides score `rand % score_random + score_share × N_side / N_all` (N: ships);
   only the hour's winner attacks. Each of its ships duels a random enemy for as many
   exchanges as hours have passed (1 – `max_exchanges`), shot for shot — or, when the player's
   side lost the hour and the station stands, the station (1 – `station_exchanges`).
3. A shot hits with the shooter's **offence** A — its pilot's stats weighted by `offence` over
   its divisor; no pilot, 0 — and does its **full** damage: a missile's impact damage with
   `missile_chance` while rounds last, else its first beam's damage, else `fallback_damage`.
   It goes through a random shield bank, a random armour bank, then the hull. A craft destroyed
   is lost with its pilot. No systems are hit and no ship is disabled or captured (the
   original's simulation has no system hits).
4. At a station the damage is × frac — from the ship's attack C against the station's defence D,
   with the station's evasion B deciding the dodge — held to `frac_range`; the station's pool
   takes it first, then with `module_hit` one of `slots` module places: a module there is
   wrecked when the hit beats its hit points (that damage never reaches the hull), else the hull.
   The station fires back with its offence for `return_fire`. The base keeps the hull damage.
5. It ends when one side is left: the mission's `win` or `lose` outcome applies, plus
   `destroyed(<class>)` for each enemy killed; `give_up_after` (72 h; an engine safeguard)
   ends an undecided battle as a retreat.

| Key | Type | Default | Meaning |
|---|---|---|---|
| `model` | `hourly`, `odds` | `hourly` | `odds`: the one roll. |
| `first_round`, `round` | time | 5 h, 1 h | When the rounds come. |
| `score_random`, `score_share` | number | 50, 50 | The hour's score. |
| `max_exchanges`, `station_exchanges` | number | 10, 5 | Most exchanges a duel. |
| `missile_chance`, `station_missile_chance` | fraction | 2 %, 4 % | A shot is a missile. |
| `fallback_damage`, `station_fallback_damage` | `[min, max]` | [25, 34], [25, 74] | Damage of a ship with no beam (whole numbers). |
| `frac_range` | `[min, max]` | [0.25, 1.5] | Station damage factor. |
| `module_hit`, `slots` | fraction, number | 75 %, 33 | A station hit strikes a module place. |
| `offence`, `evasion` | `{<stat>: weight, divisor, aggression_penalty}` | — | The ratings; `aggression_penalty: [[above, k], ...]` takes k × Aggressiveness off evasion for the first `above` exceeded. |
| `system_attack`, `system_defence` | `{<system id>: number}` | none | C and D bonuses for fitted systems. |
| `give_up_after` | time | 72 h | An undecided battle ends as a retreat (0 = never). |

A ship class's `auto_resolve: {attack, defence}` ([reference](reference.md#ships-and-objects)) is
its base C and D. A base module's `auto_resolve: {attack, defence, hit_points}` adds to its
station's C and D and is a module place with those hit points. A base type's
`auto_resolve: {shields, defence, offence, evasion, return_fire, hull}` gives it a station: its
pool, base D (1,000), A and B drawn from their ranges per battle, its return fire; its hull is
the base's own `hull` (as damaged and repaired), else `hull` here.

`craft_damage: {persist: true, repair_per_day: 10%}` (app NEXT) — battle damage stays on a craft
(flown or off-screen) and mends at home by `repair_per_day` of each bank's maximum a day; a flown
battle starts the craft with it.

`interceptor` has both with the original's values: the ratings `(0.6 Fly + 0.7 Brav + 0.5 Int +
0.35 Sta + 0.75 Agg + 0.95 Acc) / 385` and `(0.4 Fly + 0.6 Brav + 0.75 Int + 0.55 Sta + 0.05 Acc −
k·Agg) / 235`, the hull classes' C and D, the station's module weights, ECM 1's +100 and the
modules' hit points (**verified**); its station pool (the chart's SD) and the craft repair rate
(10 % a day) are *derived*.

### `pilots` and `scores`

`pilots:` — pilot rules.

| Key | Type | Default | Meaning |
|---|---|---|---|
| `stats` | mapping stat: `[min, max]` | none | Each pilot's stats, rolled in the range. Any names. |
| `ranks` | mapping rank: experience | none | Ranks in ascending order with the experience each needs. |
| `salary` | number or table | 0 | Per month, per living pilot. App NEXT: or the original's table `{base, rank, brackets, kills, max_health}` — a month's pay is `base` + (the rank's entry of `rank` + every bracket bonus) / 12, where `brackets: {<stat>: [[from, bonus], …]}`, `kills` and `max_health` give the bonus of the last entry whose `from` the value reaches (the first when none does). `interceptor`: 4,000 + the exe's rank and bracket tables (**verified**, `0x410297`). |
| `recruit_fee` | number | 0 | Cost to hire one. |
| `pool` | number | 6 | Recruits offered at a time (refreshed every month). |
| `names` | list of text | none | Names to draw from. |
| `heal_days` | number | 14 | Days an injured pilot is out. |
| `survive` | fraction | 60 % | Chance that a pilot shot down ejects (injured) rather than dies. |
| `training` | `{gain, cost}` | none | Pilot training (app 0.16.7): a ready pilot put into training gains `gain` points in **every** stat a day, up to the top of that stat's `stats` range, and costs `cost` money a day. Without it (or with `gain: 0`) the Hangar offers no *Train* button. Training continues until stopped; it is saved. |
| `training` (periodic) | `{every, stats, factor, add, scale_by, capacity}` | — | App NEXT, the original's training: every `every` a pilot in training gains, in each of `stats`, `trunc(scale_by / 100 × stat × factor + add)` up to `stat_max`; at most the bases' `capacity` module effect train at once, the rest drop out. `interceptor`: every 5 days, Accuracy, Flying and Psionics, `trunc(Int / 100 × stat × 0.1 / 6 + 1)`, 5 per Training module (**verified**). |
| `generate` | `{rank_roll, bonus}` | none | App NEXT, the original's recruit generator: a starting rank — the last of `rank_roll`'s thresholds a 1 – 100 roll reaches (the pilot gets that rank's experience) — and `bonus: {<stat>: [by rank]}` or `{<stat>: {add: [by rank], times: <stat>}}` (the bonus × that stat / 100, truncated) added up to `stat_max`. `interceptor`: thresholds 41/61/81/96; Stamina +0/3/6/9/12, Accuracy +0/5/10/15/20 and Flying +0/5/15/20/30 times Intelligence / 100, Aggressiveness −5/0/5/10/15 (**verified**, `0x40e8b6`). |
| `health` | `{max, per_day, in_medical, medical, fly_from}` | none | App NEXT: pilots have health. `max: {base, per_rank}` (whole-number ranges) rolls a recruit's maximum (`base` + rank × `per_rank`) and the pilot starts at it; every day each hurt pilot heals `per_day`, or `in_medical` while the bases' `medical` module effect has room; a pilot below `fly_from` (100 %) of the maximum is grounded. A battle's injuries come back with the pilot ([`damage.pilot_injury`](reference.md#damage)); below 0 the pilot is dead. Without it, injuries last `heal_days`. `interceptor`: 5 – 15 + rank × 1 – 5, 1 a day, 2 in a medical bay (5 per Medical module, *derived*), grounded until whole (*derived*). |
| `stat_max` | number | each range's maximum | App NEXT: the most training or a generator bonus raises a stat to. `interceptor`: 100. |

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

### `clock`

The time controls (app 0.21.0): how much campaign time passes per real second at each speed,
and what an event does to the clock. Without `clock:` a campaign keeps the engine's own: a pause
and 1 hour, 6 hours, 1 day and 5 days per second.

| Key | Type | Default | Meaning |
|---|---|---|---|
| `speeds` | list of times | `[0, 1h, 6h, 1d, 5d]` | Campaign time per real second at each speed, slowest first; `0` is a pause. |
| `labels` | list of text | made from the rates | What the `speed` readout shows for each speed (`5 secs/s`, `1 hour/s`, `paused`); speeds without one get one made from their rate. |
| `on_event` | number | 0 | The speed (0 = the first) an event, notice, prompt or return from a battle drops the clock to. |
| `run` | number | 3 | The speed Space toggles to from `on_event`. |
| `notices` | `drop`, `keep`, `ask` | `drop` | What a notice (a Report prompt) does to the speed (app 0.22.12): drop to `on_event`, keep it, or ask with two buttons — the first keeps it, the second drops to `on_event`. Battles, offers and choices still drop it. |
| `notice_buttons` | two captions | `[OK, "OK (slow)"]` | With `ask`: the Report's buttons (text or `@key`). `interceptor`: `[@orig.S_OK, @orig.S_OK5SECS]`, the original's "OK" and "OK (5 secs)" (RQ-S-18, **verified**: 0x46bdc4 → `SetSpeed(0)`). |
| `first_key` | number | `on_event` + 1 | The speed the number key 1 picks; 2 the next, up to 6 (app 0.22.15). `interceptor`: 0 — keys 1 – 6 pick the six speeds (**verified**, OnCommand 49 – 54). |
| `ask_for` | list of notice kinds | all | With `notices: ask`: the kinds of notice that offer the two buttons; the rest are a plain OK that keeps the speed (app 0.22.15). Kinds: `base_built`, `module_built`, `research_complete`, `delivery`, `report`, `contact`, `damage`, `other`. `interceptor`: all but `other` — the original offers "OK (5 secs)" for ships damaged, people killed, modules and bases built, research complete, the reports, deliveries and sightings, and a plain OK otherwise (**verified**, `ui-framework.md` §4.5). An unknown kind is an error. |

The screen-set action `speed(n)` picks speed `n` ([Screen layouts](screens.md#actions)); the
testbed layout shows one button per speed (`||`, `5s`, `30m`, `1d` …); the number keys pick the
speeds after `on_event` (with a pause first, 1 is the first running speed). Errors:
`clock.speeds: 'fast' is not a time per second (e.g. 5s, 1h, 1d)`; `clock.run: 7 is not one of
the 3 speeds (0–2)` (the index is then clamped to the last speed); `clock.notices: 'x' is not drop,
keep or ask`; `clock.notice_buttons: expected two captions [keep, slow]`.

```yaml
# packs/interceptor/strategy/campaign.yaml — the original's dial (verified)
clock:
  speeds: [5s, 1min, 5min, 30min, 1h, 1d]   # six speeds and no pause
  on_event: 0                               # events drop to 5 s per second
  run: 3                                    # Space: 30 min per second
  notices: ask                              # a Report offers OK and OK (5 secs)
  notice_buttons: ["@orig.S_OK", "@orig.S_OK5SECS"]
  first_key: 0                              # keys 1-6 pick the six speeds
  ask_for: [base_built, module_built, research_complete, delivery, report, contact, damage]
```

The original's dial has six holes and no pause: its slowest speed is five times real time, and
its "(5 secs)" buttons select it.

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
it says *&lt;type name&gt; cannot be built at &lt;system&gt;* (app 0.19.1). Asking never changes the game: a
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
   app 0.16.15). With an [`auto_resolve:`](#auto_resolve-and-craft_damage) section (app NEXT)
   the battle is instead **fought off-screen in game time**, as the original's: the result
   arrives hours later as a notice, the craft keep their damage and lost craft are gone with
   their pilots.
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
   outcome's actions too (app 0.16.3, 0.16.4), and so does a flown survey-ship battle unless it
   was left early (app 0.20.0).
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
| `hub` | ship or object class | none | The battle object at the facility. Since app 0.20.0 it **stands in** for the site in its construction-strike defence, in the site's current [construction stage](#construction-stages-and-the-stand-in) (its model set should have those states). Without it, the defence mission's own object is shown, still in the right stage. A class that does not exist is a load error. |
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
| `cargo_capacity` | number | none (unlimited) | Goods **one vessel** carries (app 0.20.0, R-STR-026). A convoy carries `vessels` × `cargo_capacity`; a filled order larger than that leaves as **several convoys**, each with its own ambush roll. Without it (or 0) one convoy carries any order, as before app 0.20.0. Amounts of every resource and component count alike. |
| `ambush_chance` | fraction | 30 % | Chance, rolled when a convoy launches, that it will be attacked on its way; the point along its route is rolled then too. |
| `ambush_loss` | fraction | 35 % | Chance that each vessel of an ambushed convoy is lost when the ambush is auto-resolved (no `ambush_mission`, or the player lets it fight through). |
| `survey_attack_chance` | fraction | 25 % | Chance, rolled when a survey ship launches, that it is attacked — since app 0.20.0 **at the survey site**, on arrival ([Survey ships and their battle](#survey-ships-and-their-battle)). |
| `survey_mission` | mission id | none | The battle an attacked survey ship fights at the site (app 0.20.0). With it, the ship **waits** there and the clock stops: the player flies it or lets it be auto-resolved. Without it every attack is auto-resolved on arrival. The mission needs a protected role (`protect: true`) for the ship to [stand in for](#the-stand-in-rule); an unknown mission, or one without a protected role, is a load error. The default campaign sets `convoy_ambush`. |
| `survey_class` | ship or object class | none | The survey ship's class in that battle (app 0.20.0): **one** of it replaces the mission's first protected role. Without it the role keeps its own class and only its count becomes one. An unknown class is a load error. The default campaign sets `survey_ship`. |
| `survey_loss` | fraction | 50 % | Chance that an attacked survey ship is lost when its battle is **not flown** — auto-resolved, let be, or left early (app 0.20.0; before, a fixed 50 % in the code). |
| `survey_speed` | number | 900 | Survey ship speed, map units an hour. |
| `convoy_speed` | number | 500 | Convoy speed, map units an hour. |
| `construction_strikes` | `{from, max, chance, mission, setback}` | none | Enemy strikes on facilities under construction ([below](#construction-strikes), app 0.16.4). Without it there are none. Its `mission` needs a protected role too (a load error since app 0.20.0). |
| `construction_stages` | mapping model-set state: fraction | `{build_25: 0%, build_60: 60%, build_90: 90%}` (since app 0.20.1; 0.20.0 used 0 / 50 / 80 %) | The construction stages (app 0.20.0, R-STR-022): each **state of the hub's model set** with the progress it starts at. Listed in any order; sorted by where they start. A site shows the last stage it has reached, on the map and in its defence battle; when finished, the intact model. A value that is not a percentage is a load error. See [Construction stages and the stand-in](#construction-stages-and-the-stand-in). |
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
   facility. The ambush roll and point are decided now. With `cargo_capacity` (app 0.20.0) an
   order larger than `vessels` × `cargo_capacity` leaves as several full convoys and one with
   the rest, each rolling its own ambush: with 3 vessels of 40, an order of 100 is one convoy
   and an order of 300 is three, carrying 120, 120 and 60.
6. **Convoys travel** at `convoy_speed`. At the ambush point each vessel may be lost
   (`ambush_loss`) and a message says how many; on arrival the surviving share of the cargo is
   delivered. A short delivery is simply wanted again: the next hour's demand orders the rest.
7. **Hull jobs** whose components are in the base's store start, using them up; the factory
   then works on them as usual.
8. **Survey ships** arrive. One that was attacked **waits at the site** for its battle when there
   is a `survey_mission`, or is lost with `survey_loss` at once when there is none
   ([below](#survey-ships-and-their-battle)); a waiting ship stays put until the player decides.
   Otherwise the system is **surveyed** and a message lists its deposits (with rates a day and
   the reserve, or *unlimited*), its features, the research sites it meets (reserves and
   features since app 0.16.2) and, since app 0.20.0, **which facility types it suits** — those
   whose [`placement:`](#where-a-base-or-facility-may-be-built) holds there now that it is
   surveyed.
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

### Construction stages and the stand-in

*App 0.20.0, R-STR-022.* A facility under construction passes through **stages**, each a
**state** of its hub's model set (the meshes and debris of [Looks](looks.md) — the default
facilities have `build_25`, `build_60` and `build_90`). `logistics.construction_stages` names
them with the progress each starts at:

```yaml
logistics:
  construction_stages: {build_25: 0%, build_60: 60%, build_90: 90%}   # the engine default too (app 0.20.1)
```

Progress is `1 − (time left / the type's time)`, so a strike's `setback` can move a site back a
stage. A site is in the **last stage whose start it has reached** (below the first start,
the first stage). A pack may name its own states and any number of them —
`{frame: 0%, top: 70%}` gives two stages — but every facility type's hub then needs those states
in its model set. Leave the key out for the default three.

Where the stage shows:

- **On the strategy map** a facility under construction (a grey diamond) has one small **pip**
  per stage underneath, the stages it has reached filled in orange; a finished facility (an
  orange diamond) has none.
- **On the Supply list** a building facility reads *building, stage 2 of 3, 6.5 days*.
- **In the construction-strike defence**: the struck facility **stands in** for the mission's
  first protected role — as its type's `hub` class, in its current stage. The default
  `construction_defence` therefore shows the mining facility, manufacturing center or research
  station that was actually struck, at 25, 60 or 90 % built, instead of always its own research
  station at 60 % (before app 0.20.0). A type without `hub` keeps the mission's own class, in
  the site's stage.

#### The stand-in rule

Two campaign battles put a strategy-layer object into a ready-made mission: a **struck
construction site** (`construction_strikes.mission`) and an **attacked survey ship**
(`survey_mission`). The object stands in for the mission's **first protected role** — the first
role, in the mission file's order, with `protect: true`:

| What stands in | Class | Model-set state | Count |
|---|---|---|---|
| A struck construction site | the facility type's `hub` (else the role's own) | its construction stage | the role's own |
| An attacked survey ship | `survey_class` (else the role's own) | the role's own | **one** |

The rest of the mission is unchanged: the role keeps its id, so objectives and conditions that
name it (`destroyed(site)`, `cargo(clippers)`) now mean the stand-in; later protected roles keep
their own classes. The change is made on a **copy** of the mission for that battle only — the
same mission flown from the launcher or the simulator, or the next campaign battle, is the
original. A mission named by either key must have a protected role: `--check` reports
*logistics.survey_mission: mission 'x' has no protected role (protect: true) to stand in for*
(and the same for `logistics.construction_strikes.mission`). When writing such a mission, make
the role the stand-in replaces the first `protect: true` role and keep its objectives about that
role.

### Survey ships and their battle

*App 0.20.0, R-STR-021 (decision D3).* A survey ship is rolled for attack at launch
(`survey_attack_chance`); the attack happens **at the survey site** when it arrives.

- **With `survey_mission`** the ship stops at the site (the message *Survey ship under attack
  at &lt;system&gt;*) and the clock stops with the **Survey ship attacked** prompt:
  - **Fly the escort** — the first ready craft flies `survey_mission`, with one `survey_class`
    standing in for the mission's first protected role ([the stand-in rule](#the-stand-in-rule)).
    A **loss** loses the survey ship; leaving early (a **retreat**) settles it as if not flown;
    any other result — a win, a draw, or even the player's craft destroyed — means it got away,
    and it reports. The mission's outcome actions apply for every result but a retreat.
  - **Let it be** (or no ready craft) — the ship is lost with `survey_loss`, else it reports.
- **Without `survey_mission`** the attack is settled on arrival: lost with `survey_loss`, else
  it reports.

A ship that got away reports as usual with *(after fighting off an attack)*; a lost one leaves
the message *Survey ship attacked and lost at &lt;system&gt;*, and the system stays unsurveyed
(survey again). A ship waiting at its site is saved (`held`).

**The report** (every survey) ends with the facility types the system now suits:

```text
Survey of Ajax: Cosmium 6/day (1800 in reserve), Iron 20/day (unlimited); features: none; research site: none; suits: Mining facility, Manufacturing center
```

(The default campaign's Ajax, from its own data.) A type is listed when its
`placement:` condition — or its role's default rule — holds for the system at that moment, so a
rule that depends on other things (money, bases, lanes) can change later; the Supply list's
*Build* rows always give the current answer.

The default campaign's survey ship is the `survey_ship` class (`rules/logistics.yaml`, text keys
`ship.survey_ship.name` and `.desc`, model set and debris in `rules/looks.yaml`, the model
generated by `tools/default_assets/models.py`): a light Guard hull with a sensor dish and **two
sample containers** (`supply_pod` modules), behaviour `supply_run`. Its `survey_mission` is the
[convoy ambush](missions.md#the-convoy-ambush-app-092): the survey ship replaces the two
clippers (`clippers`, the first protected role), so the grapplers go for its two containers and
it wins by jumping out with cargo aboard. The mission's optional objective *both clippers*
(`count(clippers) == 2`) cannot be met by one ship.

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
a Guard player who wants more craft must survey, mine and manufacture first. Since app 0.20.0:
`cargo_capacity: 40` (a convoy of three carries 120), the survey battle (`survey_mission:
convoy_ambush`, `survey_class: survey_ship`, `survey_loss: 50%`) and the default
construction stages spelled out; every facility type names its own hub (`mining_facility`,
`manufacturing_center`, `research_station`).

### Not built yet

Convoy ambushes can be **flown** since app 0.9.2 (`ambush_mission`; the battle is described in
[Missions › The convoy ambush](missions.md#the-convoy-ambush-app-092)), construction sites are
struck since app 0.16.4 ([above](#construction-strikes)), and since app 0.20.0 cargo capacity,
construction stages and the survey battle are built. Not built: facility fighter pools and
escorts (R-STR-025), captured cargo and enemy supply depots (R-STR-020), third-party convoys
(R-STR-027) and facility upgrades. `placement:` conditions and storage at every facility are
built since app 0.19.0.

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

- The tech-id numbering is the numbered topic list in a comment of the player's
  `GameSettings.ini` (`00 ACCUMULATOR_SHIELDS` … `73 VECTOR_CONTROL_THRUSTERS`): the executable
  indexes its topic records by it and its loader is called with that index (*verified* from the
  executable, app 0.22.3). A numbered name maps to the `R_<NAME>` section; numbers without a
  section (the minefield placeholders 21/32/41, the alien craft and modules from 74, and 127,
  "special": an artefact must be recovered) are left out of conditions. With it, 4 `Prereq`
  links point to a topic of a higher `Level`, as in the original's own data. Up to app 0.22.2
  the tool read a 12-byte table in the executable whose third field is a databank entry number,
  not the tech id; that shifted every id from 4 on by one and made 16 topics unreachable.
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
  store each day (`Campaign::over_capacity`). Since app 0.20.0, in a campaign with logistics,
  the automatic player also runs the **supply chain** once a day (`logistics::autoplay_day`):
  it keeps a survey ship out to the nearest unsurveyed system it can reach, builds one facility
  of each type (two mines) in supply-chain order — manufacturing, then mining, then research —
  where placement allows and while it keeps 500 000 in hand, sets an idle research station on a
  station-only topic, and keeps one hull (the cheapest built from components) on order so the
  chain has demand. Attacked survey ships are let be. A **logistics** line follows:
  *logistics: 3 facilities (0 building), 3 systems surveyed; surveys 3 sent, 0 attacked, 0
  lost; convoys 26 (11 ambushed, 11 vessels lost); goods shipped 134, delivered 108; 2 orders
  open* — the running totals are part of the save (`stats` in its `logistics:` block).
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
  flown as `construction_defence`, a quarter of the build time set back when let be (app 0.16.4),
  the struck facility shown in its construction stage (app 0.20.0); convoys of three vessels of
  40 each; an attacked survey ship flies the convoy ambush as the `survey_ship` (app 0.20.0).

The unit test `campaign::tests::the_default_campaign_runs_six_months` plays it for six months
with an automatic player: it stays solvent, researches at least four topics, fights raids and
round-trips through a save; the year-long test also asserts that no store is ever above its
capacity (app 0.19.0). `--strategy-soak 180` with the same player (app 0.8.0) reached 12 of
the 14 topics then; at app 0.19.0 (main `c28fc2c`) it reaches **14 of the 16** (the twelve
topics plus `force_fields` and `saucer_alloys`) and fights 47 battles, with every store within
its capacity — the automatic player built no facilities then, so the two station-only topics
stayed unknown. At app 0.20.0 (main `9064f59`) the automatic player also runs the supply chain:
**12 of 16** topics and 39 battles (money now goes into facilities and hulls built from
components), 3 facilities, 3 systems surveyed, 26 convoys (11 ambushed, 11 vessels lost), 134
goods shipped and 108 delivered, every store within its capacity.

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
| `facility_types.x.hub: no ship or object class 'y'` / `logistics.survey_class: no ship or object class 'y'` | A hub or survey class that is not a ship or object class (app 0.20.0). |
| `logistics.survey_mission: unknown mission 'y'` / `logistics.construction_strikes.mission: unknown mission 'y'` | A survey or defence mission that does not exist (app 0.20.0). |
| `logistics.survey_mission: mission 'y' has no protected role (protect: true) to stand in for` (and `construction_strikes.mission`) | The mission has no role for the survey ship or the struck site to [stand in for](#the-stand-in-rule) (app 0.20.0). |
| `logistics.construction_stages.x: not a percentage` | A stage's start that is not a fraction (app 0.20.0). |
| `evaluations[0].when: …` / `evaluations[0].do: …` | A rating's condition or action that does not parse or names something not defined (app 0.16.8). |
| `research.x.requires: unknown name 'y'` | Also for a misspelt topic in `requires_any` (it becomes part of `requires`). |
| `base_modules.x.on_destroyed: 'y' is not craft, pilots or stock` | A wrong `on_destroyed` entry (app 0.19.0). |
| `base_types.x.template: unknown module or module group 'y'` | A template entry that is neither a module id nor a module group (app 0.19.0). |
| `base_types.x.placement: …` / `facility_types.x.placement: …` | A placement condition that does not parse or names something not defined (app 0.19.0). |
| `base_types.x.max_distance: unknown key` | A mod written before app 0.19.0: use `placement: base_distance(system) <= <n>`. |
| `this save needs packs that are not loaded: …` | Loading a save made with a mod that is not active. |
| `save: this save has logistics but the loaded campaign has none` | Loading a save with facilities into a campaign whose logistics was removed. |
| `save: … does not match the loaded rules (was a mod changed?)` | Loading a save whose research, modules, items, incomes or raids no longer fit the rules. |

## Limits in app 0.20.0

- No endurance or stranding on the map; no fitting rules (`fits`), power draw or stock designs —
  fitting is *best available* only, and slots of kinds other than `beam`, `missile` and (since app
  0.15.0) `shield` and `armor` are filled but have no effect in battle.
- `cargo_lost` and `cargo_captured` are only recorded: captured cargo goes to no enemy depot.
  `lose(<x>)` acts only on a facility type; a base type or anything else is logged.
- **Logistics**: see [Not built yet](#not-built-yet) — fighter pools, depots, third-party
  convoys and facility upgrades are not built. Construction stages are one list for every
  facility type (`logistics.construction_stages`), not a list per type.
- No designs, bravery, stat effects in battle, or the 3D base view yet. Without `auto_resolve:`,
  auto-resolve counts hull classes only (not fitted items or pilots) and does not damage the
  player's craft (DEV-057).
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
