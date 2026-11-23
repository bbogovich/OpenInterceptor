# Campaign (strategy layer)

`src/campaign/` (`mod.rs`, `logistics.rs`, `story.rs`, `save.rs`) with its data in
`src/content/strategy.rs`, `src/content/logistics.rs` and `src/content/story.rs`, its expression
context in `src/script/campaign.rs`, its story actions in `src/script/action.rs` and its screen
in `src/app/strategy.rs`: the real-time strategy game around the battles (Stage 4, app 0.7.0 –
0.9.2, filled in by app 0.16.1 – 0.16.14 and 0.19.0 – 0.20.0) and the story on top of it (Stage 5, app 0.10.0 – 0.10.3; its
testbed tools in 0.16.13). The modder's view is
[Strategy layer](../../modding/strategy.md) and [Story layer](../../modding/story.md); the
player's is [User guide › Campaign](../../user-guide/campaign.md). Requirements: R-STR-001 –
R-STR-027 on [Strategy & logistics](../../specification/requirements/strategy-logistics.md),
R-CPN-007 – R-CPN-010 on [Campaigns](../../specification/requirements/campaigns.md).

## Architecture Overview

### Where it sits

The campaign is a **simulation layer beside `sim`**: deterministic, plain data, no macroquad, no
rendering, no input. It reads the content layer (`StrategyDef`) and the script layer (it is a
`script::Context`), and the application layer drives it.

```mermaid
flowchart TB
    subgraph APP["7 · Application"]
        ST["app::strategy::StrategyScreen<br/>top bar, map, info, lists, prompts"]
        FL["app::flight::FlightSession<br/>(campaign_battle)"]
        AM["app::run<br/>Screen, Next, AppContext"]
    end
    subgraph SIM["4 · Simulation"]
        CA["campaign::Campaign<br/>advance, orders, battle_result"]
        LG["campaign::logistics<br/>step, Logistics, convoy_result"]
        STO["campaign::story<br/>step, check_end, StoryState"]
        SV["campaign::save<br/>write / read"]
        W["sim::World"]
    end
    subgraph CON["3 · Content"]
        SD["content::strategy::StrategyDef"]
        LD["content::logistics::LogisticsDef<br/>(StrategyDef::logistics)"]
        SC["script::campaign<br/>QUERIES, schema"]
        SYD["content::story::StoryDef<br/>(StrategyDef::story)"]
        EA["script::action::EVENT_ACTIONS"]
    end
    ST --> CA & SV
    AM --> ST & FL
    FL --> W
    CA --> SD & SC & LG & STO
    LG --> LD
    STO --> SYD
    SYD --> SC & EA
    SV --> CA & SD
    SD --> SC & LD
```

`campaign` never calls `sim`: a battle is handed to the app, which flies it in an ordinary
`FlightSession` and hands the debrief back. The two simulations meet only through strings (the
mission id, the result, the outcome's action texts), a kill count, the share of cargo still
aboard (app 0.9.2) and, going in, the factions' stances (app 0.10.3).

### Data flow: from YAML to the screen

```mermaid
flowchart LR
    Y[("&lt;pack&gt;/strategy/*.yaml<br/>start, maps, research, items, hulls,<br/>base_modules, base_types, raids, …")] --> RS["content::rules::build<br/>resolved definitions + globals"]
    RS --> B["StrategyDef::build<br/>name table → script::campaign::schema<br/>expressions parsed and checked"]
    B --> X["StrategyDef::check<br/>start, templates, unlocks exist"]
    X --> RU["Ruleset::strategy: Option&lt;StrategyDef&gt;<br/>(None without start:)"]
    RU --> N["Campaign::new(def, difficulty, seed)<br/>or save::read(text, def, packs)"]
    N --> SCR["StrategyScreen { campaign, … }"]
    SCR -->|"orders"| N
    SCR -->|"F5, Esc"| SAVE[("&lt;user data&gt;/saves/&lt;campaign&gt;.yaml")]
```

1. **Rules.** The strategy sections are ordinary rules sections: definition sections (`maps`,
   `star_classes`, `research`, `items`, `hulls`, `base_modules`, `base_types`, `raids`, `incomes`,
   `expenses`: `content::strategy::DEF_SECTIONS`) layer across packs like ships do; globals
   (`start`, `research_model`, `pilots`, `scores`, `economy`, `flags`, `tech_ids`, `logistics`)
   are read once; the logistics definitions (`content::logistics::DEF_SECTIONS`: `resources`,
   `components`, `recipes`, `facility_types`, `research_sites`) layer like the others (app
   0.9.0). `start.money_from: original:<file>/<key>` is resolved here, while the original's files
   are at hand: `content::rules::build_at` reads the key from the section named after the
   difficulty level and merges it into `start` as `money` (app 0.8.0). Their
   schema (`content::strategy::sections`) joins the rules file schema, so `--check` and the
   editor JSON Schema know every key.
2. **Names.** `StrategyDef::build` first collects every id it defines, plus factions, ship
   classes, missions and the flags missions set, into one sorted table (`names`). That table
   becomes the constants of `script::campaign::schema`, so `research(heat_rays)` compiles to
   *query 0 with argument 7* and a misspelling is a load error.
3. **The original's numbers** (app 0.8.0). Data inherited from the original names prerequisites
   by number (`prereq1` – `prereq3` on topics, `tech_required`, `tech_required2` on items, hulls
   and modules). When a definition has no `requires:` and the campaign has a `tech_ids` table,
   `StrategyDef::build` turns the numbers it lists into `research(<id>) and …` text before
   parsing it like any written condition, so the result is checked the same way. Costs, prices,
   upkeep and maintenance are multiplied by `economy.cost_scale`; with `economy.module_time:
   days`, a module's `prod_time` becomes its build time in days.
4. **Logistics** (app 0.9.0). `LogisticsDef::build` reads the logistics definitions, the start
   map's systems (their `deposits` and `features`), each hull's `components` and each topic's
   `lab`, `site` and `start_cost` — into vectors **in the same order** as the map's systems, the
   hulls and the topics, so a hull index or topic index finds its logistics data directly. It
   returns `None` when no facility type is defined: `StrategyDef::logistics` is an `Option`, and
   everything downstream starts with `let Some(l) = def.logistics.as_ref() else { return }`.
5. **Check.** Cross-references (start bases, systems, craft, templates, unlocks) are checked;
   `content::rules` adds the outside ones (a raid's mission and faction, a hull's ship class).
6. **Play.** `Campaign::new` builds the starting state: the map's systems (generated now for a
   generated map), the starting bases with finished modules, craft, pilots who take the craft in
   order, recruits, the first times for every income, expense and raid roll; with logistics, an
   empty `Logistics` with every system unsurveyed and each deposit's full reserve.

### Logic flow: the hour step

`Campaign::advance(def, hours)` loops `step` in slices of **at most one hour**, and returns as
soon as a step produced an event, so the screen can stop the clock. The order inside `step` is
fixed, which is what makes a campaign replay exactly from a save:

```mermaid
flowchart TD
    A["hours += dt"] --> B["construction finished → Notice"]
    B --> C["research: each base, rate = eval(research_model.rate, base)<br/>topic done → known, Notice"]
    C --> D["manufacture: first job first; a unit not yet started is paid (cost)<br/>or, a hull with components, waits for logistics,<br/>or, an item without room() in store, waits unpaid (0.19.0);<br/>production_rate × dt; finished unit → receive (item or craft)"]
    D --> E["deliveries due → receive<br/>(or lost: roll < economy.delivery_loss → Notice; rolled once)<br/>items beyond store_free → held, retried next hour (0.19.0)"]
    E --> F["incomes and expenses due → money ± eval(amount);<br/>pilots in training: stats += gain × dt/24, money −= cost × dt/24"]
    F --> G{"month boundary<br/>(720 h)?"}
    G -->|yes| H["maintenance (not of wrecks) + upkeep + salaries;<br/>evaluations: first that holds → its actions, rating;<br/>monthly report → Notice; recruits refreshed"]
    G -->|no| I
    H --> I["injured pilots heal"]
    I --> J["raids: roll each due raid (requires, chance) → launch contact"]
    J --> K["contacts move; within a base's sensor range → detected, Notice;<br/>hidden systems in range revealed"]
    K --> L["interceptions fly to their contacts → Event::Battle"]
    L --> M["contacts that arrived unopposed → score(undefended), Notice;<br/>roll < 1 − 0.85^defence → driven off,<br/>else newest working module wrecked → on_destroyed losses (0.19.0)"]
    M --> N["logistics::step (app 0.9.0; campaigns with logistics):<br/>construction strikes → Event::Strike (0.16.4);<br/>demand, mining, recipes, stations, convoys, surveys"]
    N --> O["story::step (app 0.10.0): events, plans,<br/>monthly drift and aggression, the end conditions"]
```

Since app 0.10.0 a raid's roll uses `chance × story.aggression`, and is skipped while
`raids_allowed` says its faction is no longer hostile. The evaluation (app 0.16.8), training
(0.16.7), delivery losses (0.16.11) and the `defence` roll (0.16.10) joined the step later; each
is skipped when its data is absent, so an older pack plays as before.

**Notices outside the clock** (app 0.16.2). `advance` returns its events, but some changes
happen in a debrief, outside `advance` — a `relation(…)` outcome that changes a stance, a
`lose(<facility>)`, a construction strike's result — and a save loaded with a changed pack
version. These push their text onto `Campaign::notices: Vec<String>` (not saved: the screen
shows them before the next save is possible); the strategy screen pops one into its Report
prompt whenever no other prompt is up, and stops the clock.

Every roll (raids, launch points, pilots, recruits, generated maps, `random(p)`) draws from one
**xorshift** generator stored in the campaign (`Campaign::rng`), so `same_seed_same_campaign`
can assert that two campaigns from one seed are equal after the same time.

### Expressions: the campaign as a `script::Context`

`Campaign` answers the 30 queries of `script::campaign::QUERIES` (16 until app 0.9.2; `arc`,
`plan`, `relation`, `fired` and `battles_won` came with the story; the eight logistics queries
with app 0.16.3, answered by `Campaign::logistics_query` — `lane_distance` by `Campaign::lane_hops`,
a breadth-first search counting lanes, not distance; `base_distance` with app 0.19.0) through a
small context struct (`Cx`) that holds `&Campaign`, an optional base (so `module_count` in a
research rate counts the base being evaluated), since app 0.19.0 an optional candidate system
(the `system` of a [`placement:` condition](#placement-a-condition-about-a-candidate-system))
and the generator in a `Cell<u64>` — `Context::query` takes `&self`, but
`random(p)` must advance the generator, the same interior-mutability trick as
`sim::ai::ObjectContext` ([Conditions](script.md#the-object-context)). After evaluating,
`Campaign::eval` copies the advanced generator back.

### The battle hand-off

```mermaid
sequenceDiagram
    participant S as StrategyScreen
    participant A as app::run
    participant C as AppContext
    participant F as FlightSession
    S->>S: advance → Event::Battle{contact} | Convoy{convoy} | Mission{mission}<br/>| Strike{facility} | Survey{survey}; banner or prompt
    S-->>A: Enter → flying = PendingBattle::Contact | Convoy | Story | Strike | Survey;<br/>Next::CampaignBattle{mission, ship, loadout, stances, stand_in}
    A->>C: mem::replace(screen, Launcher); ctx.campaign = Some(strategy screen)
    A->>F: stand_in? start_mission_def(mission.with_stand_in(si), ship)<br/>: start_mission(mission, ship);<br/>apply_loadout(loadout); apply_stances(stances); campaign_battle = true
    F-->>C: debrief + Enter/Esc → ctx.battle_report = {result, actions, kills, cargo}
    F-->>A: Next::Strategy
    A->>C: ctx.campaign.take(), ctx.battle_report.take()
    A->>S: battle_done(report) → battle_result | convoy_result | strike_result<br/>| survey_result | mission_result; story::check_end
```

**Five kinds of campaign battle** (app 0.9.2 – 0.20.0). `StrategyScreen::flying` is an enum,
`PendingBattle`, saying what the battle was *for*, so that the one report coming back is routed
to the right place:

| `PendingBattle` | Started by | The report goes to |
|---|---|---|
| `Contact(id)` | `Event::Battle` — an interception reached its contact | `Campaign::battle_result(def, id, …)` |
| `Convoy(v)` | `Event::Convoy` — an ambushed convoy (with `logistics.ambush_mission`) | `Campaign::convoy_result(def, v, share)`: `lose` → `Some(0.0)`, `retreat` → `None` (fights on alone), anything else → `Some(report.cargo.unwrap_or(1.0))` |
| `Story` | `Event::Mission` — a story event's `mission(x)` offer | `Campaign::mission_result(def, …)`: `battle_result` with an id no contact has, so the outcome's actions and the win apply |
| `Strike(f)` (app 0.16.4) | `Event::Strike { facility }` — a construction strike, flown with `logistics.construction_strikes.mission`; since 0.20.0 with the struck facility as the stand-in (`Campaign::strike_stand_in`) | `Campaign::strike_result(def, f, won)`: `win` → `Some(true)` (beaten off), `lose` → `Some(false)` (the site removed), anything else → `None` (set back); then `mission_result` for the outcome's actions |
| `Survey(s)` (app 0.20.0) | `Event::Survey { survey }` — an attacked survey ship waiting at its site, flown with `logistics.survey_mission` and one `survey_class` as the stand-in (`Campaign::survey_stand_in`) | `Campaign::survey_result(def, s, won)`: `lose` → `Some(false)` (lost), `retreat` → `None` (lost with `survey_loss`), anything else → `Some(true)` (it reports); then, unless a retreat, `mission_result` for the outcome's actions |

A flown **convoy** escort also runs `mission_result` since app 0.16.3, so its outcome's score and
actions apply like any battle's. **Auto-resolve** (app 0.16.12) never leaves the screen: the
Interception prompt's second button calls `balance::win_chance(rules, mission, hull ids)`, rolls
`Campaign::roll` against it and hands `battle_result` the mission's own outcome actions for the
rolled `win` or `lose` (rendered back to text with `Action::display`), with the hostile roles'
minimum counts as kills on a win. **Prizes** (app 0.16.3): the flight screen adds
`captured(<class id>)` to the report's actions for each prize the player's tractor brought home;
`battle_result` keeps them as salvage only when the outcome also says `capture`.

#### The stand-in: a campaign object in a ready-made mission (app 0.20.0)

A struck construction site and an attacked survey ship are campaign objects, but their battles
are ordinary missions written once (`construction_defence`, `convoy_ambush`). Rather than
generate a mission, the strategy layer **substitutes** the object into the mission's **first
protected role** (`MissionDef::stand_in_role`, the first `protect: true`), on a copy made for
that one battle:

```mermaid
sequenceDiagram
    participant S as StrategyScreen
    participant K as Campaign
    participant A as app::run
    participant M as MissionDef (ruleset)
    participant F as FlightSession
    S->>S: prompt answered "Fly" → fly(def, craft, mission, PendingBattle::Strike(f) | Survey(s))
    S->>K: strike_stand_in(def, f) → StandIn{class: hub, state: stage, count: None}<br/>survey_stand_in(def) → StandIn{class: survey_class, state: None, count: Some(1)}
    S-->>A: Next::CampaignBattle{mission, ship, loadout, stances, stand_in: Some(si)}
    A->>M: rules.missions.find(mission)?.with_stand_in(&si) → a clone, role r replaced
    A->>F: FlightSession::start_mission_def(ctx, copy, ship)
    Note over F: "fly again" repeats the mission by its id (start_with), unlike start_custom
```

`StandIn` (`content/mission.rs`) has three optional fields — `class`, `state` (a model-set
state such as `build_60`) and `count` — and `with_stand_in` replaces only the ones that are
`Some`, leaving the role's id, placement, AI and objectives alone. It returns
`Result<MissionDef, String>`: a mission without a protected role is an error, not a silent
no-op — and `content::rules::build_with` refuses such a mission at load for both keys, so the
error should never reach the battle. Why a **copy**: the ruleset's missions are shared by every
battle that flies them (the launcher, the simulator, the next strike), so the substitution must
not leak into them. It is the same reason `simulator::apply` works on a clone. Any failure
(the mission gone, no protected role) goes down the same `Err` branch of `match started { … }`
as a failed `start_mission`: the error is printed to the terminal and the battle comes back as
a `retreat`, so a strike is set back and a survey ship settled with `survey_loss`.

After any of them `story::check_end` runs, because a battle (a flag set by a finale's win, a
last base lost) can end the campaign.

- The strategy screen cannot stay the current `Screen` while a flight runs, and it must not be
  dropped (it owns the campaign). `app::run` moves it out of `screen` with
  `std::mem::replace` and parks it in `AppContext::campaign` (`Option<Box<StrategyScreen>>`).
  `Next::Strategy` takes it back with `Option::take` ([Learn Rust › Ownership](../../learning/rust/ownership.md#moving-a-value-out-of-a-place-memreplace-and-take)).
- **The loadout** (app 0.7.2). `Campaign::loadout(def, craft)` turns the first craft's `fit` into
  beam ids (one per fitted beam slot) and missile ids with rounds. `flight::apply_loadout` sets
  them on the player's class — but beams belong to a ship *class* in the shared `Ruleset`,
  which every other screen also reads. The battle's `World` holds the rules as an `Arc<Ruleset>`,
  so `Arc::make_mut` gives the battle **its own copy** the first time it is changed
  (copy-on-write) and the shared content stays untouched. Ids that are not weapons of the
  campaign are skipped; with no beams fitted the class's own are kept. Since app 0.15.0 the
  loadout is a `campaign::Loadout` struct — `beams`, `missiles`, and `shields` / `armor` as
  `Option<f32>`: the strongest fitted `shield` item's `protection` and the sum of the fitted
  `armor` items' (`None`: keep the class's own). `apply_loadout(world, player, &Loadout)` writes
  them into the battle's copy of the class and fills the player's facings and armour.
  An item's protection is `content::strategy::protection_of(kind, Get)`: its `protection:` key,
  else, for an item inheriting the original's equipment, `Special` (shield) or `HitPoints`
  (armour) — *derived*. The same function serves `fitted:` on ship classes, applied while the
  rules are resolved (`rules::apply_fitted`, on each definition once its `inherit:` chain is
  merged; a `shields` / `armor` already present, own or inherited, wins), so a class names its
  default equipment instead of copying numbers ([Strategy layer › items](../../modding/strategy.md#items-and-hulls)).
- **The stances** (app 0.10.3). With dynamic relations, `StrategyScreen::stances` lists every
  faction with a standing as `(faction, hostile)` — hostile when its stance is `hostile`.
  `flight::apply_stances` calls `Ruleset::set_hostile(player_faction, faction, hostile)` for
  each (never for the player's own faction), which edits both factions' `hostile` lists — in the
  battle's own copy of the rules through `Arc::make_mut`, exactly like the loadout. Without
  dynamic relations the list is empty and the factions' own hostility stands.
- The flight screen fills `AppContext::battle_report` (`BattleReport {result, actions, kills,
  cargo}`) from the debrief: the result, the outcome's campaign actions as written (`score(x)`,
  `money(n)`, `salvage`, `set_flag(x)`, …), the kill count and — when the mission names a
  `cargo:` role — the share of its cargo still aboard (`mission::cargo_share`). Leaving from the pause menu
  is `retreat`; a mission that cannot start is counted as `retreat` too, with the error printed.
- `Campaign::battle_result` applies the actions it knows (`score`, `money`, `set_flag`, `salvage`,
  since app 0.12.1 `relation(f, n)` through `change_standing` — since 0.16.2 a stance change it
  causes becomes a notice; since 0.16.3 `capture` / `captured(<class>)`, `lose(<facility type>)`
  through `Campaign::remove_facility` — which removes the facility's orders and inbound convoys
  and moves every later facility index down by one — and `cargo_lost` / `cargo_captured`,
  logged),
  marks a battle for `after_battle` events, counts a win for the story, credits the party's pilots, removes a
  `killed` leader's craft (re-indexing pilots' craft handles), sends the rest home, and removes
  the contact on `win` or `draw`.

!!! note "Fixed in app 0.9.0: `salvaged(x)` and manufacture costs"
    Until app 0.9.0 `salvage` recorded the raid's display **name** while `salvaged(x)` compared
    the **id**, so the default campaign's `saucer_hulls` and `robot_brains` could never start;
    `battle_result` now records `def.raids[contact.raid].id`. Manufacture was free; each `Job` now
    has a `started` flag, and the hour step takes the unit's `cost` when it starts (or waits for
    the money). `campaign::tests::manufacture_is_paid_per_unit_and_salvage_names_the_raid_by_id`
    covers both.

### Travel on the map

Positions are map units and everything moves in straight hops at its speed per hour
(`travel`). On a `travel: lanes` map (app 0.7.3) contacts and craft carry a `route`: the system
positions still to pass, consumed one by one before the final leg. `Campaign::lane_route(def,
from, to)` finds the shortest way along the lanes with **Dijkstra's algorithm** — repeatedly
settle the unsettled system with the shortest known distance and relax its lanes, then walk the
`prev` links back. With seven systems a linear scan for the minimum is simpler than a heap and
fast enough. A raid enters at the free system farthest from its target; an interception flies
the lanes to the system nearest the contact (`nearest_system`), then straight to it, and home
along the lanes. Routes are part of the saved state.

### Logistics

`campaign::logistics` (app 0.9.0, plan items 4.14 – 4.17) is the runtime of
`content::logistics::LogisticsDef`. Its state is one more plain struct inside the campaign,
`Campaign::logistics: Logistics` — facilities, open orders, convoys and survey ships in flight,
which systems are surveyed, what each deposit has left, each base's store of components and
(app 0.20.0) the running totals `stats: LogisticsStats` —
and `logistics::step(campaign, def, dt, events)` runs one slice of it at the end of the hour step.
The modder's view is [Strategy layer › Logistics](../../modding/strategy.md#logistics).

#### Data flow: demand pulls, convoys carry

```mermaid
flowchart RL
    subgraph BASE["Main base"]
        J["hull job (not started)<br/>needs hulls.x.components"]
        BS[("base_goods[b]")]
    end
    subgraph MC["Manufacturing center"]
        MO["orders it must fill"]
        LN["lines: recipe runs<br/>(≤ lines at once)"]
        MS[("store ≤ storage")]
    end
    subgraph MF["Mining facility"]
        DP["system deposits<br/>rate / 24 per hour, reserve"]
        FS[("store ≤ storage")]
    end
    subgraph RS["Research station"]
        PR["project: station-only topic<br/>start_cost once, then research_rate"]
        RSS[("store ≤ storage")]
    end
    J -- "want(component, short)<br/>Order → nearest ready center" --> MO
    MO -- "inputs short:<br/>want(resource) → nearest mine with a deposit" --> FS
    PR -- "want(start_cost)" --> FS
    DP --> FS
    MO --> LN
    LN -- "output" --> MS
    MS -- "filled order → Convoy(s)<br/>(≤ vessels × cargo_capacity each,<br/>lane route, ambush roll each)" --> BS
    FS -- "filled order → Convoy" --> MS
    FS -- "filled order → Convoy" --> RSS
    BS -- "all components in store →<br/>job.started = true" --> J
    RSS --> PR
```

An **order** is `{goods, amount, to: Dest, source}`: `amount` of `goods` wanted at `to` (a base
or a facility, the `Dest` enum of two index variants), to be filled by facility `source`.
`Campaign::want(l, goods, amount, to)` tops up the open orders for that destination and goods
to `amount`; the shortfall is computed by the caller as *needed − in store − already in convoys*
(`inbound`), so the same demand never produces two orders. A resource goes to the nearest
ready mining facility whose system has a deposit of it, a component to the nearest ready
manufacturing center (`Campaign::supplier`, by straight-line distance between systems). No
supplier means no order: the demand waits and is asked again next hour.

#### Logic flow: `logistics::step`

```mermaid
flowchart TD
    A["facilities whose ready_at fell in this slice → Notice;<br/>construction_strikes: first site past from, its system under max,<br/>roll < chance × dt/24 → attacks += 1, Event::Strike (one at a time)"] --> B["1 · demand: each base's unstarted hull jobs<br/>→ want(component, need − store − inbound)"]
    B --> C{"2 · each ready facility, by role"}
    C -->|mining| M["each deposit: min(rate/24 × dt, room, reserve) → store; reserve −="]
    C -->|manufacturing| P["finished lines → output to store;<br/>per order it fills: store + running ≥ amount? else<br/>free line and inputs in store → start recipe (inputs used)<br/>inputs short → want(inputs)"]
    C -->|research| R["project not started: start_cost in store? use it : want(start_cost)<br/>started: progress[topic] += research_rate × dt → known, Notice"]
    M & P & R --> D["3 · orders whose source store holds the amount →<br/>while left > 0: Convoy {cargo = min(left, vessels × cargo_capacity), to,<br/>route = lane_route, vessels, ambush_in = roll < ambush_chance ? roll × length};<br/>stats.convoys += 1, stats.shipped += load"]
    D --> E["4 · convoys travel convoy_speed × dt along the route;<br/>ambush_in reaches 0 → each vessel lost with ambush_loss, Notice;<br/>at the end → deliver cargo × surviving / vessels"]
    E --> F["5 · hull jobs whose components are in base_goods → use them, started = true"]
    F --> G["6 · survey ships not held travel survey_speed × dt;<br/>arrived and attacked with a survey_mission → held = true, Event::Survey (waits);<br/>else attacked and roll < survey_loss → lost, else surveyed;<br/>survey_report: deposits (rate, reserve), features, sites, suits: placement_ok per type"]
```

Design points worth noticing:

- **Everything is an index.** Facilities refer to their type and system by index, orders to
  their source facility, `Dest` to a base or facility, `Line` to a recipe; stores are short
  `Vec<(String, f32)>` lists of goods and amounts (`logistics::amount`, `add`). A save is then a
  direct walk, and the borrow checker never sees two live references into the campaign.
- **Copy out, then mutate.** `step` takes `&mut Campaign` and often needs to read one part while
  changing another (an order's amount while changing the source facility's store). It clones
  the small thing it reads first — `let o = c.logistics.orders[k].clone();`, the list of
  `(goods, amount)` a center must fill — and then mutates freely. With a few dozen orders the
  clones cost nothing, and the code stays free of `RefCell` and of index juggling.
- **Rolls are made when something launches**, not when it arrives: the ambush and its point
  along the route (one per convoy when an order is split), and a survey ship's attack. The outcome is then part of the saved state
  (`Convoy::ambush_in`, `Survey::attacked`), so a save loaded before the ambush replays it the
  same way.
- **An ambush waits for the player** (app 0.9.2). With `logistics.ambush_mission` set, a convoy
  reaching its ambush point is marked `held` (saved), a message names its cargo, and
  `Event::Convoy { convoy }` goes to the screen; a held convoy does not move. The screen either
  flies the ambush mission (`PendingBattle::Convoy`) or calls `Campaign::convoy_result(def,
  convoy, None)`, which auto-resolves it per vessel as before. `convoy_result(…, Some(share))`
  scales every cargo amount by the share that got through. Without an ambush mission the
  ambush is auto-resolved on the spot, as in app 0.9.0.
- **Placement** (`Campaign::facility_allowed`): one facility of each type per system, and the
  type's `placement:` condition must hold for the system — by default its role's rule, a
  condition string (`Role::default_placement`) parsed by `StrategyDef::build` like any other.
  Until app 0.19.0 these were fixed rules in Rust (DEV-049); see
  [Placement](#placement-a-condition-about-a-candidate-system).
- **Room in a facility's store** (app 0.19.0, R-STR-014): `Campaign::facility_room(l, f)` is its
  type's `storage` less its goods, the orders and convoys bound for it and the output of its
  running lines. `want` clamps a facility's order to it, and a recipe starts only when
  `facility_room + inputs >= output` (the inputs leave the store as the run starts). Every
  store — the manufacturing center's and research station's too — is then at most `storage`.
- **Hulls with components** are not paid in money: the manufacture step in `Campaign::advance`
  sees `hull_components` and leaves the job for `logistics::step` to start once its components
  are in the base's store.
- **Construction strikes** (app 0.16.4, R-STR-023): `LogisticsDef::strikes: Option<Strikes>`
  (`from`, `max`, `chance`, `mission`, `setback`). The roll is per hour for each site whose
  progress `1 − (ready_at − now) / time` has passed `from` and whose system's facilities have
  suffered fewer than `max` strikes in all (`Facility::attacks`, saved); the first hit raises
  `Event::Strike` and stops the loop, so strikes come one at a time. `strike_result` settles it:
  beaten off, destroyed (`remove_facility`), or set back (`ready_at += time × setback`), with a
  message and a notice. The automatic player lets every strike be.
- **Cargo capacity** (app 0.20.0, R-STR-026, decision D1): `LogisticsDef::cargo_capacity:
  Option<f32>` is per vessel. Step 3 splits a filled order into loads of at most `vessels ×
  cargo_capacity` (`f32::INFINITY` when `None`, so the loop runs once), each its own `Convoy`
  with its own ambush roll and a clone of the route. Demand is unchanged: `inbound` already sums
  every convoy bound for a destination, so several convoys count as one shipment.
- **Construction stages** (app 0.20.0, R-STR-022): `LogisticsDef::stages: Vec<(String, f32)>`
  — `(model-set state, start)`, sorted by start, `DEFAULT_STAGES` when the key is absent.
  `LogisticsDef::stage_at(progress)` is the last stage whose start has been reached
  (`rposition`), else the first. `Campaign::construction_stage(l, f)` computes the progress the
  same way as the strikes do and returns `None` once the facility is built. Nothing is stored:
  the stage is derived from `ready_at` whenever it is needed (the map draws its pips from it
  every frame; the Supply list names it), so a setback moves the stage back with no extra state.
- **Survey ships and their battle** (app 0.20.0, R-STR-021, decision D3): an attacked ship that
  arrives is marked `Survey::held` (saved) and raises `Event::Survey { survey }` when the
  campaign has a `survey_mission`; a held ship does not move. `Campaign::survey_result(def, s,
  won)` settles it — `Some(true)` reports, `Some(false)` loses it, `None` rolls `survey_loss` —
  removes it, and writes the message and a notice. Without a survey mission the same roll is
  made on arrival. Both paths end in the private `survey_report`, which marks the system
  surveyed and lists, after the deposits, features and sites, the **facility types whose
  placement holds** there (`placement_ok` with the type's condition or its role's default) —
  asked *after* `surveyed[s]` is set, since every default rule starts with `survey_known(system)`.
- **Running totals** (app 0.20.0, #47): `LogisticsStats` counts convoys, goods shipped and
  delivered, ambushes, vessels lost, and surveys sent, attacked and lost. They are counted where
  the thing happens (step 3, the ambush point, delivery, `convoy_result`, `survey`,
  `survey_report`) and saved, so a long game's totals survive a reload; `--strategy-soak` prints
  them.
- **The automatic player's supply moves** (app 0.20.0, `logistics::autoplay_day`, called once a
  day by `autoplay_from`): one survey ship out to the nearest unsurveyed system any base can
  reach; at most one build a day, in supply-chain order (manufacturing, mining ×2, research),
  where `facility_allowed` says yes, keeping 500 000 in hand (it waits rather than skipping
  ahead); an idle research station set on the first unknown station topic; and one hull — the
  cheapest with `components` — kept on order at the first base, so the chain always has demand.
  Every order may be refused; refusals are ignored. `autoplay_from` also clears
  `Campaign::notices` each day: the screen normally shows and clears them, the automatic player
  has no screen, and they are never saved.
- **Station-only topics** (`station_topics[t]` is `Some`) are filtered out of
  `Campaign::researchable`, so bases never offer them; `start_station_research` starts one at a
  research station of the right `lab` type on the right `site`.

### The story (app 0.10.0)

`campaign::story` is the runtime of `content::story::StoryDef` (`StrategyDef::story`, always
present, empty for a campaign without story sections). Its state is one more plain struct,
`Campaign::story: StoryState` — per event the times fired and the hour it last fired, per arc the
steps done, per plan whether it has started, components done, the next roll's hour and the
warnings given, the standings, the aggression, battles won (in all and this month), two
"something happened" latches for `after_battle` and `on_research`, and the ending reached. As
everywhere in the campaign these are vectors parallel to the definitions, indexed by position.

#### Data flow

```mermaid
flowchart LR
    Y[("strategy/story.yaml<br/>events, arcs, plans, endings,<br/>encyclopaedia; relations,<br/>strategist, campaign_end")] --> R["content::rules::build_at<br/>story::DEF_SECTIONS layer;<br/>globals read once"]
    R --> N["StrategyDef::build<br/>story ids join the name table"]
    N --> B["StoryDef::build<br/>conditions parsed (campaign schema);<br/>actions checked (EVENT_ACTIONS);<br/>arcs, raids, endings resolved to indices"]
    B --> T["content::resolve_text<br/>@keys in names, messages, choices,<br/>endings, entries resolved"]
    T --> ST["StoryState::new<br/>(Campaign::new, save::read)"]
```

The story ids are added to the strategy layer's **name table** before any expression is parsed,
so `arc(serial)`, `plan(eclipse_engine)` and `fired(ch07)` compile to checked constants like
every other name. Actions are parsed with `script::action::parse(text, EVENT_ACTIONS)`; a
single-text action (`message`, `news`) keeps its whole argument, commas included, while every
other action splits on commas.

#### Logic flow: `story::step`

```mermaid
flowchart TD
    Z{"ending reached?"} -->|yes| END["nothing more happens"]
    Z -->|no| A["1 · events in order: due by check period<br/>(day / week / month boundary crossed, or a battle / topic since last)<br/>not spent (once), not cooling down, next step of its arc,<br/>when holds, roll < chance → fired, arc step +1,<br/>apply_actions(do); choices → Event::Choice"]
    A --> B["2 · plans: not started and start_when → started, log;<br/>started, not complete, next roll due → roll < chance and a base →<br/>launch(raid), contact.plan = Some(p)"]
    B --> C{"3 · month boundary?"}
    C -->|yes| D["standings drift towards 0;<br/>aggression −down if month_wins ≥ calm_after_wins else +up;<br/>month_wins = 0"]
    C -->|no| E
    D --> E["4 · day boundary or an ending event →<br/>check_end: defeats, then victories; first that holds → Event::Ending"]
```

- **One place changes the campaign.** `Campaign::apply_actions(def, actions, events)` is the only
  code that turns story actions into changes — money, flags, standings, research, raids, plan
  progress, aggression, endings — and into events for the screen (`Notice`, `Mission`,
  `Ending`, and since app 0.11.1 – 0.13.0 `Card { title, text }`, `Video { reference }` and
  `Cutscene { id }`, which change nothing in the campaign: `card` only logs its title). Events, choices (`Campaign::choose`), plan warnings and completion all call it.
- **Shipments reuse raids.** A plan's shipment is an ordinary `Contact` with `plan: Some(p)`, so
  detection, interception and the battle need nothing new. Only the arrival differs:
  `Campaign::unopposed` sees `plan` and calls `shipment_arrived` (one more component) instead of
  striking. A shipment destroyed or driven off is simply removed with its contact.
- **Relations reach two places.** `Campaign::raids_allowed` stops a non-hostile faction's raids
  in the hour step; `StrategyScreen::stances` carries the stances into battle (above). A change
  of stance is announced in `change_standing`.
- **Arc order by data, not code.** An event's arc index is set from the arc's `steps` while
  building, so the step check is one comparison: `arcs[a].steps.get(arc_step[a]) == Some(&e)`.

The screen (`app::strategy`) holds what is waiting for the player — `convoy`, `choice`, `offer`,
`ending` — as `Option`s on `StrategyScreen`, checked in that order before normal input: an
ending takes Enter/Esc to leave, a choice takes the number keys, an offer and a convoy take
Enter (fly) or Esc (decline); a title card (`card`, app 0.11.1) waits for Enter, Esc or a click.
A video or cutscene event makes the screen return `Next::Video` / `Next::Cutscene` and the app
parks it until the film is over ([Presentation](presentation.md#who-plays-what-the-apps-routing)).
An ending's `video` plays before its banner; loading a save whose story has ended sets
`StrategyScreen::ending` again, so a finished campaign shows its ending (app 0.12.1). The
automatic player (`autoplay_from`) ignores the three presentation events. The **Archive** tab (`Tab::Archive`) lists the started plans, the
standings with their stances, the aggression and battles won, and `Campaign::encyclopaedia`'s
unlocked entries.

### The X-COM data (app 0.8.0)

`tools/interceptor_strategy.py` (Python, standard library) is a build-time helper, not part of
the engine: it reads the player's `GameSettings.ini` and `Interceptor.exe` and writes
`packs/interceptor/strategy/references.yaml` — the `tech_ids:` table and `inherit:` references
to the original's sections, with no values. The engine then treats the X-COM campaign like any
other: inheritance reads the values from the player's file at load, `tech_ids` turns the numeric
prerequisites into conditions (above), and `cost_scale`, `module_time` and `money_from` adapt the
original's units. The numbering is `GameSettings.ini`'s numbered topic list, which the executable
indexes its topics by (*verified*, app 0.22.3); see [Strategy layer › the `interceptor`
campaign](../../modding/strategy.md#the-interceptor-campaign-from-the-players-data).

### The economy's later rules (app 0.16.6 – 0.16.11)

- **Hidden and either-or research** (0.16.6): `Topic::hidden` filters a topic out of
  `researchable` until the flag `discovered.<topic>` exists, which the story action `discover`
  sets (with a notice). `requires_any: [a, b]` is not a new mechanism: while the rules are built
  it becomes the text `research(a) or research(b)`, joined to `requires:` as `(…) and (…)`, and
  is parsed like any condition — so a misspelt topic is reported at `research.<id>.requires`.
- **Training and dismissal** (0.16.7): `Pilot::training` (saved as `yes`/`no`; older saves read
  as not training); `PilotRules::train_gain` / `train_cost` from `pilots.training`.
  `Campaign::dismiss(p)` removes the pilot; no other structure holds a pilot index (a craft's
  pilot is the `Pilot::craft` back-reference), so nothing needs renumbering.
- **Evaluations** (0.16.8): `StrategyDef::evaluations: Vec<Evaluation {rating, when, actions}>`;
  at the month the first whose `when` holds runs its actions through `apply_actions` (the story's
  action runner) and names the rating in the report. `evaluations` must be in the rules' list of
  strategy globals (`rules::build_at`) to reach `StrategyDef` at all — it was missing until
  0.16.9, which the unit test did not catch because it builds the strategy layer directly.
- **Base effects** (0.16.10): `Campaign::quarters(def)` (all bases' type `quarters` plus
  `quarters` effects; 0 = no limit) checked by `hire`; `Campaign::storage(def, base) ->
  (Option<capacity>, used)` — `None` when no module in the campaign has `storage` (since
  0.16.15: when the base type has no hub `storage`) — checked by `buy` for items (since 0.19.0
  everywhere, [below](#bases-storage-wrecks-and-placement-app-0190)); the raid's `defence` roll in the unopposed-strike code;
  `Campaign::dismantle_module(def, base, m)` refunding `StrategyDef::dismantle_refund`. On the
  screen `Row::default_act` keeps `Dismantle`, `Dismiss` and (0.16.11) `CancelJob` off Enter
  and the double click.
- **Queues and deliveries** (0.16.11): `Campaign::raise_job(base, j)` swaps a job with the one
  above; `cancel_job(base, j)` removes an unstarted job, or cuts a started one to its paid unit;
  `StrategyDef::delivery_loss` rolls each arriving delivery.

### Bases: storage, wrecks and placement (app 0.19.0)

Batch 4.1 (main `c28fc2c`; R-STR-004, R-STR-006, R-STR-014) finished the base lifecycle and made
the store limits hold everywhere. The modder's view is
[Strategy layer › Bases: storage, wrecks and placement](../../modding/strategy.md#bases-storage-wrecks-and-placement).

#### Storage flow

Three numbers describe a base's store, all in item `size`s, and each caller uses the one that
answers its question:

| Function | Value | Used by |
|---|---|---|
| `storage(def, base)` | `(Option<capacity>, used)`: the hub's `storage` plus working `storage` modules (`None` = no limit), and the stock's size | the Build list's *Store* line, `over_capacity`, the `on_destroyed: [stock]` loss |
| `store_free(def, base)` (private) | capacity − used − the item being made (`f32::MAX` with no limit) | a delivery arriving now; stock moved by `dismantle_base` |
| `room(def, base)` | `store_free` − every delivery still bound for the base, held ones included (`None` with no limit) | `buy`; whether a manufacture job may start its next item |

The item being made keeps its place (it is subtracted everywhere), so a finished unit always
fits; what is ordered counts against `room` before it arrives, so a purchase can never
over-fill the store. Only deliveries can meet a smaller store than they were bought for (a
`storage` module wrecked meanwhile), and those wait:

```mermaid
flowchart LR
    BUY["buy(item, n)"] -->|"size × n > room → Err"| X1["no room in store:<br/>used of cap, k more on its way"]
    BUY -->|fits| DEL["Delivery {held: false}<br/>arrive_at = now + delivery"]
    MAN["manufacture: next item"] -->|"size > room"| WAIT["job waits, unpaid"]
    MAN -->|fits| PAY["paid, started → receive"]
    DEL -->|"due: loss roll (once)"| ARR{"fits =<br/>⌊store_free / size⌋"}
    ARR -->|"all"| ST[("stock")]
    ARR -->|"some or none"| HOLD["receive(fits);<br/>count −= fits; held = true<br/>(Notice once); arrive_at = now + 1 h"]
    HOLD -->|next hour| ARR
```

`Campaign::over_capacity(def) -> Vec<String>` lists every base and facility store above its
limit. The year-long default test asserts it stays empty, and `--strategy-soak` (in
`main.rs`) now advances day by day with `autoplay_from` so that it can check it every day and
print the first breach. The automatic player stocks the first base with the cheapest item that
takes room, so the soak really meets full stores.

#### The base lifecycle

```mermaid
stateDiagram-v2
    [*] --> Building: build_module / build_base (template_modules)
    Building --> Working: ready_at passed
    Working --> Wrecked: unopposed raid → wreck_module (on_destroyed losses)
    Wrecked --> Building: repair_module (repair share of cost and build_time)
    Working --> [*]: dismantle_module (dismantle share back)
    Wrecked --> [*]: dismantle_module (nothing back)
    Building --> [*]: dismantle_module (dismantle share back)
```

- `ModuleInst { module, ready_at, wrecked }`. `ModuleInst::working(now)` (built and not a
  wreck) replaces the old `ready_at <= now` test everywhere an effect is counted — `effect`,
  `module_count`, the raid's choice of module — so a wreck adds nothing; the monthly maintenance
  skips wrecks. A wreck keeps its place in `Base::modules`, so it still counts towards its
  group's `limits`.
- `wreck_module(def, base, m, events)` marks the module, then applies its
  `ModuleDef::on_destroyed: Vec<Loss>`. Each `Loss` removes what the base can no longer hold
  now that the module's effect is gone — `Craft` (home craft beyond `hangar`, the highest
  index first; `remove_craft` keeps pilot indices valid; the pilots are set `Injured`),
  `Pilots` (beyond `quarters`, those without a craft first), `Stock` (whole units off the end
  of `Base::stock` until `storage` fits). It reads the limits *after* setting `wrecked`, so the
  same functions that the screen uses decide what is lost.
- `repair_module` charges `def.repair_share` × cost and sets `ready_at = max(now, base ready) +
  repair_share × build_time` — a repair is a short rebuild, so the existing *building* state and
  the *&lt;module&gt; is complete* notice serve it.
- `dismantle_module` refuses a working module whose `hangar`, `quarters` or (0.19.0) `storage`
  is in use; a wreck or a module still building contributes nothing, so it is never refused,
  and a wreck refunds nothing.
- `dismantle_base(def, base)` is the one operation that **removes a base from the middle of
  `Campaign::bases`**. Everything that names a base by index must follow: it computes
  `after(b) = if b > base { b − 1 } else { b }` and rewrites each craft's base, each delivery's
  (the removed base's go to the nearest other), each contact's `target_base` (cleared when it
  was this base), `Logistics::base_goods` (removed, its goods merged into the nearest base's),
  orders (those for the base dropped, the rest renumbered) and convoys (redirected), then
  removes the base. Craft are rehoused first and stock moved first, while the indices are
  still the old ones. This is the price of index handles: removing from the middle means one
  explicit renumbering pass, which a unit test checks
  (`a_whole_base_is_dismantled_with_its_craft_and_stock_moved`).
- `template_modules(def, base_type) -> Vec<usize>`: an entry that is a module id is that module;
  an entry that is a group gives the costliest module of the group that is `available` now
  (`max_by` on `cost`, which keeps the last of equal elements: a later module wins a tie), or
  nothing. `Campaign::new` uses it for the starting bases after the start's research is known,
  and `build_base` when the base is ordered (not when it is finished).

#### Placement: a condition about a candidate system

`BaseType::placement` and `FacilityType::placement` are `Option<Expr>`, parsed against the
campaign's name table like every condition. A facility type's text is its own `placement:` or
its role's `Role::default_placement()`; it is parsed in `StrategyDef::build` (not in
`LogisticsDef::build`), because only the strategy builder owns the name table.

```mermaid
flowchart TD
    A["build_base / build_facility / Supply list row"] --> B["placement_ok(def, placement, system)"]
    B -->|None| OK["true (anywhere)"]
    B --> C["Cx { c, def, base: None, system: Some(s), rng: Cell::new(rng) }"]
    C --> D["Expr::eval → Bool / Num ≠ 0"]
    D --> E["system query, argument name:<br/>'system' → Cx::system<br/>'nearest_base' → nearest_base_system(def, s)<br/>other → a map id"]
    E --> F["logistics_query: deposit/research_site with 'any',<br/>base_distance = min straight-line distance to a base (1e9 with none)"]
```

- `system`, `nearest_base` and `any` are `script::campaign::RESERVED`, added to every
  campaign's name table so that they compile as constants. `Cx::query` turns `system` and
  `nearest_base` into a system id **before** the system lookup, so every system query accepts
  them; outside a placement condition `Cx::system` is `None` and they name no system (0).
- `nearest_base_system(def, s)` is the base fewest lane jumps from `s` (`lane_hops`), ties and
  unreachable systems broken by straight-line distance — `min_by` with `Ordering::then`.
- `placement_ok` takes `&self` and evaluates with a **copy** of the generator in the `Cell`,
  never written back: the screen asks it for every facility type every frame, and asking must
  not change the game (the same rule as the condition console). `eval` now goes through
  `eval_at(…, system)`, which does copy the generator back.
- `facility_allowed` keeps the one-per-system rule in Rust and words a refusal as
  *&lt;system&gt; does not suit a &lt;facility&gt;* — the condition is the modder's, so the engine cannot
  say which part failed; it adds *(not surveyed yet)* when the system is unsurveyed, the usual
  reason.

### Testbed cheats (app 0.8.1)

`Campaign::cheat_payday`, `cheat_knowitall`, `cheat_quickbase` and `cheat_jump(contact)` are the
original's own cheat list as plain methods on the campaign (so they are unit-tested like any
order); `app::strategy` binds them to Ctrl+M, Ctrl+K, Ctrl+B and Ctrl+J. `cheat_jump` puts the
first ready craft with a ready pilot at the contact (`away = Some((contact, pos))`, route
cleared), so the next step's interception check reports `Event::Battle` at once. Each cheat is
logged in the message list.

**The event debugger and start-at-chapter** (app 0.16.13): `Campaign::fire_event(def, e) ->
Vec<Event>` counts the event as fired, advances its arc only when it is the arc's next step,
runs its actions and offers its choice — the same steps as `story::step`, minus the checks;
`campaign::autoplay_to_chapter(def, c, step, max_days)` repeats `autoplay_from` five days at a
time until `story.arc_step[0] >= step` or `max_days` pass ([Testbed](testbed.md)).

### Saves

`campaign::save` writes the whole `Campaign` as a YAML document (`write`) and reads it back
(`read`), using the engine's own YAML reader and writer (`formats::yaml`). Because the state
holds only numbers, text and **indices** into the rules, the save is a direct walk:

```yaml
# OpenInterceptor campaign save
save: 1
campaign: default
packs: [{id: default, version: 0.7.1}]
hours: 1234
money: 2150000
…
```

Reading checks the format version, then that **every pack listed is loaded** (else "this save
needs packs that are not loaded: …"), then every index against the loaded rules (module, item,
topic, income/expense/raid counts; else "save: … does not match the loaded rules (was a mod
changed?)"). Since app 0.9.0 a `logistics:` block follows (`write_logistics` / `read_logistics`):
facilities with their stores, lines and projects, orders, convoys with their routes, vessels and
pending ambush and (app 0.9.2) whether it is `held`, survey ships (with `held` since 0.20.0),
the surveyed flags, the reserves, the bases' stores and (app 0.20.0) `stats`, the eight running
totals as one number list in a fixed order (older saves read zeros);
destinations are written `base:<i>` / `facility:<i>`. Its indices are checked like the rest, a
campaign without logistics saves an empty block, and a save with facilities is refused by a
campaign that has no logistics. Since app 0.10.0 a `story:` block follows too (`write_story` /
`read_story`): events fired and when, arc steps, plan state, standings, aggression, battles won,
the two latches and the ending; a contact's `plan`. Never-fired events hold `-inf` for their last
firing, which YAML numbers cannot carry, so it is written as a very old hour (−10¹²) and read
back as `-inf`. A save whose events, arcs or plans no longer match the rules is refused. A changed mod is refused with a message instead of crashing or silently changing
the game. Since app 0.16.2 the pack **versions** are compared too: a pack still loaded at another
version loads, with a notice per pack (*Pack x is now version a (this save was made with b)*) in
`Campaign::notices`. Since 0.16.3 each facility saves its `attacks`, since 0.16.7 each pilot
`training`; older saves read them as 0 and not training. Since 0.19.0 a wrecked module is saved
as `[module, ready_at, 1]` (a third number only for a wreck) and a delivery has `held`; older
saves read both as false. The screen saves to
`<user data>/saves/<campaign id>.yaml` (`strategy::save_path`, slot 0) on F5, when leaving with
Esc and at the ending; *Campaign: continue* loads it. Since app 0.16.14 the top bar's *Save*
writes slot 1 – 3, `<campaign id>-<n>.yaml` (`strategy::slot_path`), loaded from the main
menu's *Load game* page ([Presentation](presentation.md#the-main-menu-and-credits-appmenu-appcredits)).

## Key Symbols

| Symbol | Where | What it is |
|---|---|---|
| `StrategyDef` | `content/strategy.rs` | The strategy layer's rules: `start`, `map`, `star_classes`, `research`, `research_model`, `items`, `hulls`, `modules`, `base_types`, `raids`, `incomes`, `expenses`, `pilots`, `scores`, `evaluations` (0.16.8), `dismantle_refund` (0.16.10), `delivery_loss` (0.16.11), `repair_share` (0.19.0), `delivery`, `salvage`, the `names` table and `logistics: Option<LogisticsDef>` (app 0.9.0). |
| `Evaluation`, `Topic::hidden`, `PilotRules::train_gain`, `train_cost` | `content/strategy.rs` | A monthly rating; a hidden topic; training (app 0.16.6 – 0.16.8). |
| `Strikes`, `LogisticsDef::strikes` | `content/logistics.rs` | `logistics.construction_strikes` (app 0.16.4). |
| `Date` | `content/strategy.rs` | A Gregorian calendar date (`parse`, `plus_days`). |
| `ClockDef`, `StrategyDef::clock` | `content/strategy.rs` | App 0.21.0: the time controls (`clock:`) — `speeds` (campaign hours per real second), `labels`, `on_event`, `run`; `parse`, `short(hours)` (a button caption: the pause sign, `5s`, `30m`), `describe(hours)` (`5 secs/s`). `StrategyScreen` advances the campaign by `speeds[speed] × frame time` and drops to `on_event` on every event; it replaces the old `app::strategy::SPEEDS` constant. |
| `MapKind`, `Travel`, `RaidTarget`, `ResearchMode` | `content/strategy.rs` | Fixed / generated maps; free / lane travel; base / system raids; download / labs. |
| `Campaign` | `campaign/mod.rs` | One game in progress: `hours`, `money`, `systems`, `bases`, `known`, `progress`, `craft`, `pilots`, `recruits`, `contacts`, `deliveries`, flow and raid timers, score, flags, salvaged, messages, and `notices` (app 0.16.2, not saved). |
| `Base`, `ModuleInst`, `Job`, `Craft`, `Pilot`, `Contact`, `Delivery` | `campaign/mod.rs` | The parts of the state, each referring to the others by index. |
| `Event` | `campaign/mod.rs` | `Notice(String)` (stop the clock, show it), `Battle { contact }`, and since app 0.9.2 – 0.10.0 `Convoy { convoy }` (an ambush waiting for the player), `Choice { event }`, `Mission { mission }` (an offer) and `Ending { ending }`; since 0.16.4 `Strike { facility }`; since 0.20.0 `Survey { survey }`. |
| `StoryDef`, `EventDef`, `Check`, `Choice`, `ArcDef`, `PlanDef`, `Stage`, `Relations`, `Strategist`, `Ending`, `Entry` | `content/story.rs` | The story rules (app 0.10.0); `story::DEF_SECTIONS`, `GLOBALS`, `sections()` (the schema). |
| `StoryState` | `campaign/story.rs` | The story state inside `Campaign::story`; `standing`, `plan_progress`. |
| `story::step`, `story::check_end` | `campaign/story.rs` | One slice of story time; the end conditions. |
| `Campaign::apply_actions`, `choose`, `mission_result`, `raids_allowed`, `encyclopaedia` | `campaign/story.rs` | Story actions; a choice; a story mission's result; whether a faction raids; unlocked entries. |
| `Campaign::convoy_result`, `Convoy::held` | `campaign/logistics.rs` | Settling an ambushed convoy (app 0.9.2). |
| `Campaign::strike_result`, `remove_facility`, `lane_hops`, `logistics_query`, `Facility::attacks` | `campaign/logistics.rs` | Settling a construction strike; losing a facility safely; lane jumps; the logistics queries (app 0.16.3 – 0.16.4). |
| `Campaign::fire_event`, `campaign::autoplay_to_chapter` | `campaign/story.rs`, `campaign/mod.rs` | The event debugger; start-at-chapter (app 0.16.13). |
| `LogisticsDef::cargo_capacity`, `survey_mission`, `survey_class`, `survey_loss`, `stages`, `stage_at`, `DEFAULT_STAGES` | `content/logistics.rs` | Cargo capacity, the survey battle and the construction stages (app 0.20.0). |
| `MissionDef::stand_in_role`, `MissionDef::with_stand_in`, `StandIn` | `content/mission.rs` | The first protected role; a copy of a mission with a campaign object in it (app 0.20.0). |
| `Campaign::construction_stage`, `strike_stand_in`, `survey_stand_in`, `survey_result`, `Survey::held`, `LogisticsStats`, `Logistics::stats`, `logistics::autoplay_day` | `campaign/logistics.rs` | Stages, stand-ins, settling an attacked survey ship, the running totals and the automatic player's supply moves (app 0.20.0). |
| `Event::Survey { survey }`, `PendingBattle::Survey`, `Next::CampaignBattle::stand_in`, `FlightSession::start_mission_def` | `campaign/mod.rs`, `app/strategy.rs`, `app/mod.rs`, `app/flight.rs` | The survey-battle hand-off and the stand-in's way into the flight screen (app 0.20.0). |
| `Campaign::set_training`, `dismiss`, `quarters`, `storage`, `dismantle_module`, `raise_job`, `cancel_job` | `campaign/mod.rs` | Pilots, base effects and queues (app 0.16.7 – 0.16.11). |
| `ModuleDef::on_destroyed`, `Loss`, `BaseType::placement`, `BaseType::template` | `content/strategy.rs` | What a wreck loses (`Craft`, `Pilots`, `Stock`); where a base may be built (an `Option<Expr>`); template entries that may name a group (app 0.19.0). |
| `ModuleInst::wrecked`, `ModuleInst::working`, `Delivery::held` | `campaign/mod.rs` | A wreck; built and not a wreck; a delivery waiting for room (app 0.19.0, saved). |
| `Campaign::room`, `store_free`, `over_capacity`, `template_modules`, `wreck_module`, `repair_module`, `dismantle_base`, `placement_ok` | `campaign/mod.rs` | Storage, the base lifecycle and placement (app 0.19.0, [above](#bases-storage-wrecks-and-placement-app-0190)). |
| `Cx` (`system` field) | `campaign/mod.rs` | The expression context; since 0.19.0 with the candidate system of a placement condition. |
| `FacilityType::placement`, `Role::default_placement` | `content/logistics.rs` | A facility type's placement condition; each role's default rule as condition text (app 0.19.0). |
| `Campaign::facility_room`, `facility_allowed`, `nearest_base_system` | `campaign/logistics.rs` | Room in a facility's store; the placement check; the base fewest lanes from a system (app 0.19.0). |
| `script::campaign::RESERVED`, `id::BASE_DISTANCE` | `script/campaign.rs` | `system`, `nearest_base`, `any`; query 29 (app 0.19.0). |
| `balance::power`, `balance::win_chance` | `balance.rs` | Auto-resolve odds (app 0.16.12). |
| `script::action::EVENT_ACTIONS` | `script/action.rs` | The story actions: 14 in app 0.10.0, 17 with `card`, `video`, `play_cutscene`, 18 with `discover` (0.16.6). |
| `PendingBattle` | `app/strategy.rs` | `Contact`, `Convoy`, `Story`, (0.16.4) `Strike` or (0.20.0) `Survey`: what the battle being flown is for. |
| `flight::apply_stances`, `Ruleset::set_hostile` | `app/flight.rs`, `content/rules.rs` | The campaign's relations in the battle's own rules (app 0.10.3). |
| `LogisticsDef`, `FacilityType`, `Role`, `Recipe`, `Deposit`, `SystemGoods`, `StationTopic` | `content/logistics.rs` | The logistics rules (app 0.9.0): resources, components, recipes, facility types, sites, per-system goods, per-hull components, per-topic station restrictions, supply settings. |
| `Logistics`, `Facility`, `Line`, `Order`, `Convoy`, `Survey`, `Dest` | `campaign/logistics.rs` | The logistics state inside `Campaign::logistics`. |
| `logistics::step`, `amount` | `campaign/logistics.rs` | One slice of logistics time; goods in a store. |
| `Campaign::cheat_payday`, `cheat_knowitall`, `cheat_quickbase`, `cheat_jump` | `campaign/mod.rs` | Testbed cheats (app 0.8.1). |
| `campaign::autoplay`, `autoplay_from` | `campaign/mod.rs` | The automatic player of `--strategy-soak` and the six-month and year-long tests (public since app 0.8.0; `autoplay_from` continues an existing campaign, app 0.10.3). It lets ambushed convoys fight through, takes every choice's first option and flies offered missions as two wins and a loss in turn. |
| `Campaign::fit_best`, `loadout`, `lane_route`, `nearest_system` | `campaign/mod.rs` | Fitting a craft from its base's store (0.7.2); its battle weapons; the shortest lane route (0.7.3). |
| `flight::apply_loadout` | `app/flight.rs` | Refits the player's class in the battle's own copy of the rules (beams, missiles, and since 0.15.0 shields and armour). |
| `Loadout`, `content::strategy::protection_of` | `campaign/mod.rs`, `content/strategy.rs` | A craft's battle fit (0.15.0); an item's protection. |
| `MONTH` | `campaign/mod.rs` | 720 hours: reports, salaries, maintenance. |
| `save::write`, `save::read`, `save::PackRef`, `save::FORMAT` | `campaign/save.rs` | Saves and their pack header. |
| `script::campaign::QUERIES`, `id`, `schema` | `script/campaign.rs` | The campaign context. |
| `StrategyScreen`, `Tab` (eleven with `Reports` since 0.16.8, and the testbed's `Debug`), `BattleReport` (with `cargo`), `save_path`, `slot_path`, `map_transform` | `app/strategy.rs` | The screen. |
| `Next::NewCampaign`, `LoadCampaign`, `LoadSlot` (0.16.14), `CampaignBattle` (with `stances`; `stand_in` since 0.20.0), `Strategy`; `AppContext::campaign`, `battle_report` | `app/mod.rs` | The screen changes around a campaign battle. |

## API Reference

- `Campaign::new(&StrategyDef, difficulty, seed) -> Campaign`.
- `campaign.advance(&def, hours) -> Vec<Event>` — the only way time passes.
- Orders, each `-> Result<(), String>` with a message for the player: `start_research(def, base,
  topic)`, `build_module(def, base, module)`, `build_base(def, base_type, system, name)`,
  `buy(def, base, Product, count)`, `sell(def, base, item, count)`, `manufacture(def, base,
  Product, count)`, `hire(def, recruit)`, `intercept(def, contact, &[craft])`, `fit_best(def, craft) -> Result<String, String>`; `assign(pilot,
  Option<craft>)`.
- Logistics orders (app 0.9.0), each `-> Result<(), String>`: `survey(def, base, system)`,
  `build_facility(def, facility_type, system)`, `start_station_research(def, facility, topic)`;
  the check alone: `facility_allowed(def, facility_type, system)`.
- Cheats (app 0.8.1): `cheat_payday()`, `cheat_knowitall()`, `cheat_quickbase()`,
  `cheat_jump(contact) -> Result<usize, String>` (the craft sent).
- `campaign::autoplay(&def, seed, days) -> Campaign` — plays with the automatic player;
  `autoplay_from(&def, campaign, days)` continues one until it is `days` old.
- Story (app 0.10.0): `apply_actions(def, &[Action], &mut events)`, `choose(def, event, choice)
  -> Vec<Event>`, `mission_result(def, result, &actions, kills)`, `raids_allowed(def, faction)`,
  `encyclopaedia(def) -> Vec<(title, text)>`; `story::step(c, def, dt, &mut events)`,
  `story::check_end(c, def, &mut events)`.
- Convoys (app 0.9.2): `convoy_result(def, convoy, Option<share>)`.
- Strikes (app 0.16.4): `strike_result(def, facility, Option<won>)`; `remove_facility(f)`;
  `lane_hops(def, a, b) -> Option<u32>`.
- Stages, stand-ins and surveys (app 0.20.0): `LogisticsDef::stage_at(progress) ->
  Option<usize>`; `construction_stage(&LogisticsDef, facility) -> Option<usize>`;
  `strike_stand_in(def, facility) -> Option<StandIn>`; `survey_stand_in(def) -> StandIn`;
  `survey_result(def, survey, Option<won>)`; `MissionDef::stand_in_role() -> Option<usize>`,
  `MissionDef::with_stand_in(&StandIn) -> Result<MissionDef, String>`;
  `FlightSession::start_mission_def(ctx, MissionDef, Option<&str>) -> Result<FlightSession,
  String>`; `logistics::autoplay_day(&mut Campaign, &StrategyDef)`.
- Pilots, bases and queues (app 0.16.7 – 0.16.11), each `-> Result<(), String>`:
  `set_training(def, pilot, on)`, `dismiss(pilot)`, `dismantle_module(def, base, module)`,
  `raise_job(base, job)`, `cancel_job(base, job)`; queries `quarters(def) -> u32`,
  `storage(def, base) -> (Option<f32>, f32)`.
- Bases (app 0.19.0): `repair_module(def, base, module)`, `dismantle_base(def, base)`, each
  `-> Result<(), String>`; `template_modules(def, base_type) -> Vec<usize>`; queries
  `room(def, base) -> Option<f32>`, `over_capacity(def) -> Vec<String>`,
  `placement_ok(def, Option<&Expr>, system) -> bool`; logistics `facility_room(&LogisticsDef,
  facility) -> f32`, `nearest_base_system(def, system) -> Option<usize>`. (`wreck_module` is
  private: only a raid wrecks.)
- Testbed (app 0.16.13): `fire_event(def, event) -> Vec<Event>`;
  `campaign::autoplay_to_chapter(&def, campaign, step, max_days) -> Campaign`.
- Auto-resolve (app 0.16.12): `balance::win_chance(&Ruleset, &MissionDef, &[&str]) -> f32`.
- Queries: `date(def)`, `eval(def, &Expr, base)`, `holds(def, Option<&Expr>)`, `stock_of(item)`,
  `effect(def, base, name)`, `sensor_range(def, base)`, `hangar(def, base)`, `available(def,
  "items.x")`, `researchable(def)`, `Campaign::rank(def, &pilot)`, `battle_party(contact)`.
- `campaign.battle_result(def, contact, result, &actions, kills)`.
- `save::write(&campaign, campaign_id, &[PackRef]) -> String`; `save::read(text, &def,
  &[PackRef]) -> Result<Campaign, String>`.

## Debugging

- `openinterceptor --check --campaign default` reports every strategy-data error with where it
  came from (`research.x.requires: unknown name 'y'`, `raids.x: unknown mission 'y'`).
- `openinterceptor --no-original --strategy` opens a new default campaign straight away.
- A save is readable YAML: open `<user data>/saves/default.yaml` to see the whole state.
- The message log (Messages tab) records every payment, arrival, research result, battle and
  report with its date; the Reports list (app 0.16.8) keeps the monthly reports with their
  ratings.
- **Ctrl+D** on the strategy screen (app 0.16.13) lists every arc's step and every event's fired
  count and whether its condition holds now; *Fire now* fires one. `--strategy --chapter N`
  starts a new campaign played forward to chapter N.
- `openinterceptor --campaign <id> --strategy-soak 180` plays six months headless with
  `campaign::autoplay` and prints a one-line summary (app 0.8.0): the quickest check that a
  modded strategy layer runs — six-month results at app 0.8.0: `interceptor` 4 of 71 topics on
  one downlink, `default` 12 of 14. Since app 0.10.1 – 0.10.2 it also prints the last monthly
  report and a story line — `story: arcs [serial 12/12], plans [eclipse_engine 61%],
  aggression 1.20, ending none` (an example of the form) — so a story that stalls shows which arc
  stopped where.
- The story: the Archive tab shows plan progress, standings and aggression; the message log
  records every chapter, choice (*Chose: …*), shipment that got through, stance change and the
  ending. An arc that never moves usually has a step whose `when` cannot hold (a flag nobody
  sets, a plan that never starts because its `start_when` is never true).
- `campaign::tests::the_default_campaign_runs_a_year_with_its_story_and_stays_consistent` is the
  year-long consistency soak: it plays a month at a time with `autoplay_from` and checks after
  each month that every craft, pilot, base, contact and facility index is in range, money and
  score are finite, arcs never go backwards and (app 0.19.0) `over_capacity` is empty.
- Storage (app 0.19.0): `--strategy-soak` ends with *storage: every store within its capacity*
  or names the first day and store over its limit. A purchase refused with *k more on its way*
  counts deliveries not yet arrived (held ones too); a manufacture job that never starts may be
  waiting for room (sell or build a `storage` module); a held delivery's notice comes once, and
  the delivery stays in `Campaign::deliveries` with `held: true` (visible in the save). Results at
  app 0.19.0 (main `c28fc2c`), 180 days: `default` 14 of 16 topics, 47 battles, every store
  within its capacity; `interceptor` 4 of 71 topics, 1 battle, every store within its capacity
  (it has no store limit).
- Placement: a refused facility says only *does not suit*. To see which part fails, put each
  part of the condition in a test event's `when`, with the system's id in place of `system` and
  the base's system id in place of `nearest_base`, and read the Debug list (Ctrl+D) as in
  MT-4-49.
- Logistics: the Supply tab's last row counts open orders, convoys and survey ships; the message
  log records every survey, completed facility and ambush. Since app 0.20.0 `--strategy-soak`
  prints a *logistics:* line — facilities (and how many still building), systems surveyed,
  surveys sent / attacked / lost, convoys (ambushed, vessels lost), goods shipped and delivered
  (summed over every unit), open orders. At app 0.20.0 (main `9064f59`), `default`, 180 days: 3
  facilities, 3 systems surveyed, 3 surveys (none attacked), 26 convoys (11 ambushed, 11 vessels
  lost), 134 shipped, 108 delivered, 2 orders open, every store within its capacity; 12 of 16
  topics and 39 battles. Shipped minus delivered is what ambushes cost plus what is still on
  its way. In a save, `stats:` under `logistics:` holds the same totals.
- A construction-strike defence or survey battle showing the wrong object: the stand-in goes
  into the mission's **first** `protect: true` role — check the role order in the mission file;
  a type without `hub` keeps the mission's own class (only the stage changes). A battle that
  will not start prints *error: …* in the terminal (for example *mission 'x' has no protected
  role*) and counts as a retreat. An order that never appears usually
  means no ready facility can fill it (a resource no mined system has, or no manufacturing
  center); a hull job that never starts is waiting for its `components`.
- The campaign is unit-testable without a window: `campaign::autoplay` plays any
  `StrategyDef` with a simple automatic player (keeps every base researching, builds a lab and a
  workshop when it can, intercepts every contact it sees, alternates battle results). The tests
  build small synthetic strategy layers from one YAML document (`content::strategy::tests::build`) and also play the real default campaign for six months.

## Save sections

Since app 0.22.33 (RQ-S-28, after the original's tagged chunks, unknowns §12.6) a save lists its
subsystems' versions under `sections:` — `player`, `bases`, `craft`, `pilots`, `contacts`,
`deliveries`, `outposts`, `history`, `story` (`save::SECTIONS`). Reading refuses a save whose
section has another version than this app reads, naming it ("its story section is version 3;
this app reads version 2"), instead of half reading it; a save from before the sections has none
and loads as before. A change to a section's layout that older readers would misread raises its
version. Test: `campaign::save::tests` (the round trip and a refused version).

