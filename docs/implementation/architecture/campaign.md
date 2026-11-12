# Campaign (strategy layer)

`src/campaign/` (`mod.rs`, `logistics.rs`, `story.rs`, `save.rs`) with its data in
`src/content/strategy.rs`, `src/content/logistics.rs` and `src/content/story.rs`, its expression
context in `src/script/campaign.rs`, its story actions in `src/script/action.rs` and its screen
in `src/app/strategy.rs`: the real-time strategy game around the battles (Stage 4, app 0.7.0 –
0.9.2) and the story on top of it (Stage 5, app 0.10.0 – 0.10.3). The modder's view is
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
        ST["app::strategy::StrategyScreen<br/>map, tabs, clock, banners"]
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
    C --> D["manufacture: first job first; a unit not yet started is paid (cost)<br/>or, a hull with components, waits for logistics;<br/>production_rate × dt; finished unit → receive (item or craft)"]
    D --> E["deliveries due → receive"]
    E --> F["incomes and expenses due → money ± eval(amount)"]
    F --> G{"month boundary<br/>(720 h)?"}
    G -->|yes| H["maintenance + upkeep + salaries;<br/>monthly report → Notice; recruits refreshed"]
    G -->|no| I
    H --> I["injured pilots heal"]
    I --> J["raids: roll each due raid (requires, chance) → launch contact"]
    J --> K["contacts move; within a base's sensor range → detected, Notice;<br/>hidden systems in range revealed"]
    K --> L["interceptions fly to their contacts → Event::Battle"]
    L --> M["contacts that arrived unopposed → module lost, score(undefended), Notice"]
    M --> N["logistics::step (app 0.9.0; campaigns with logistics):<br/>demand, mining, recipes, stations, convoys, surveys"]
    N --> O["story::step (app 0.10.0): events, plans,<br/>monthly drift and aggression, the end conditions"]
```

Since app 0.10.0 a raid's roll uses `chance × story.aggression`, and is skipped while
`raids_allowed` says its faction is no longer hostile.

Every roll (raids, launch points, pilots, recruits, generated maps, `random(p)`) draws from one
**xorshift** generator stored in the campaign (`Campaign::rng`), so `same_seed_same_campaign`
can assert that two campaigns from one seed are equal after the same time.

### Expressions: the campaign as a `script::Context`

`Campaign` answers the 21 queries of `script::campaign::QUERIES` (16 until app 0.9.2; `arc`,
`plan`, `relation`, `fired` and `battles_won` came with the story) through a small context struct
(`Cx`) that holds `&Campaign`, an optional base (so `module_count` in a research rate counts the
base being evaluated) and the generator in a `Cell<u64>` — `Context::query` takes `&self`, but
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
    S->>S: advance → Event::Battle{contact} | Convoy{convoy} | Mission{mission}; banner
    S-->>A: Enter → flying = PendingBattle::Contact | Convoy | Story;<br/>Next::CampaignBattle{mission, ship, loadout, stances}
    A->>C: mem::replace(screen, Launcher); ctx.campaign = Some(strategy screen)
    A->>F: start_mission(mission, ship); apply_loadout(loadout);<br/>apply_stances(stances); campaign_battle = true
    F-->>C: debrief + Enter/Esc → ctx.battle_report = {result, actions, kills, cargo}
    F-->>A: Next::Strategy
    A->>C: ctx.campaign.take(), ctx.battle_report.take()
    A->>S: battle_done(report) → battle_result | convoy_result | mission_result;<br/>story::check_end
```

**Three kinds of campaign battle** (app 0.9.2 – 0.10.0). `StrategyScreen::flying` is an enum,
`PendingBattle`, saying what the battle was *for*, so that the one report coming back is routed
to the right place:

| `PendingBattle` | Started by | The report goes to |
|---|---|---|
| `Contact(id)` | `Event::Battle` — an interception reached its contact | `Campaign::battle_result(def, id, …)` |
| `Convoy(v)` | `Event::Convoy` — an ambushed convoy (with `logistics.ambush_mission`) | `Campaign::convoy_result(def, v, share)`: `lose` → `Some(0.0)`, `retreat` → `None` (fights on alone), anything else → `Some(report.cargo.unwrap_or(1.0))` |
| `Story` | `Event::Mission` — a story event's `mission(x)` offer | `Campaign::mission_result(def, …)`: `battle_result` with an id no contact has, so the outcome's actions and the win apply |

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
  campaign are skipped; with no beams fitted the class's own are kept.
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
- `Campaign::battle_result` applies the actions it knows (`score`, `money`, `set_flag`, `salvage`;
  `lose` is logged; `relation`, `capture` and the cargo actions are still ignored at app 0.10.3),
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
which systems are surveyed, what each deposit has left, and each base's store of components —
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
        MS[("store")]
    end
    subgraph MF["Mining facility"]
        DP["system deposits<br/>rate / 24 per hour, reserve"]
        FS[("store ≤ storage")]
    end
    subgraph RS["Research station"]
        PR["project: station-only topic<br/>start_cost once, then research_rate"]
        RSS[("store")]
    end
    J -- "want(component, short)<br/>Order → nearest ready center" --> MO
    MO -- "inputs short:<br/>want(resource) → nearest mine with a deposit" --> FS
    PR -- "want(start_cost)" --> FS
    DP --> FS
    MO --> LN
    LN -- "output" --> MS
    MS -- "filled order → Convoy<br/>(vessels, lane route, ambush roll)" --> BS
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
    A["facilities whose ready_at fell in this slice → Notice"] --> B["1 · demand: each base's unstarted hull jobs<br/>→ want(component, need − store − inbound)"]
    B --> C{"2 · each ready facility, by role"}
    C -->|mining| M["each deposit: min(rate/24 × dt, room, reserve) → store; reserve −="]
    C -->|manufacturing| P["finished lines → output to store;<br/>per order it fills: store + running ≥ amount? else<br/>free line and inputs in store → start recipe (inputs used)<br/>inputs short → want(inputs)"]
    C -->|research| R["project not started: start_cost in store? use it : want(start_cost)<br/>started: progress[topic] += research_rate × dt → known, Notice"]
    M & P & R --> D["3 · orders whose source store holds the amount →<br/>Convoy {cargo, to, route = lane_route, vessels, ambush_in = roll < ambush_chance ? roll × length}"]
    D --> E["4 · convoys travel convoy_speed × dt along the route;<br/>ambush_in reaches 0 → each vessel lost with ambush_loss, Notice;<br/>at the end → deliver cargo × surviving / vessels"]
    E --> F["5 · hull jobs whose components are in base_goods → use them, started = true"]
    F --> G["6 · survey ships travel survey_speed × dt;<br/>arrived: attacked and roll < 0.5 → lost, else surveyed; Notice with deposits and sites"]
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
  along the route, and a survey ship's attack. The outcome is then part of the saved state
  (`Convoy::ambush_in`, `Survey::attacked`), so a save loaded before the ambush replays it the
  same way.
- **An ambush waits for the player** (app 0.9.2). With `logistics.ambush_mission` set, a convoy
  reaching its ambush point is marked `held` (saved), a message names its cargo, and
  `Event::Convoy { convoy }` goes to the screen; a held convoy does not move. The screen either
  flies the ambush mission (`PendingBattle::Convoy`) or calls `Campaign::convoy_result(def,
  convoy, None)`, which auto-resolves it per vessel as before. `convoy_result(…, Some(share))`
  scales every cargo amount by the share that got through. Without an ambush mission the
  ambush is auto-resolved on the spot, as in app 0.9.0.
- **Fixed placement rules** (`Campaign::facility_allowed`): mining needs a surveyed system with
  deposits, research a surveyed research site, manufacturing at most two lanes
  (`lane_route(...).len() <= 2`) from a base; one facility of each type per system. The design's
  `placement:` expressions are not built (DEV-049).
- **Hulls with components** are not paid in money: the manufacture step in `Campaign::advance`
  sees `hull_components` and leaves the job for `logistics::step` to start once its components
  are in the base's store.
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
  `Ending`). Events, choices (`Campaign::choose`), plan warnings and completion all call it.
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
Enter (fly) or Esc (decline). The **Archive** tab (`Tab::Archive`) lists the started plans, the
standings with their stances, the aggression and battles won, and `Campaign::encyclopaedia`'s
unlocked entries.

### The X-COM data (app 0.8.0)

`tools/interceptor_strategy.py` (Python, standard library) is a build-time helper, not part of
the engine: it reads the player's `GameSettings.ini` and `Interceptor.exe` and writes
`packs/interceptor/strategy/references.yaml` — the `tech_ids:` table and `inherit:` references
to the original's sections, with no values. The engine then treats the X-COM campaign like any
other: inheritance reads the values from the player's file at load, `tech_ids` turns the numeric
prerequisites into conditions (above), and `cost_scale`, `module_time` and `money_from` adapt the
original's units. The executable's table layout (12-byte entries: a pointer to the `R_` section
name, a string id, the tech id) is *derived*; see [Strategy layer › the `interceptor`
campaign](../../modding/strategy.md#the-interceptor-campaign-from-the-players-data).

### Testbed cheats (app 0.8.1)

`Campaign::cheat_payday`, `cheat_knowitall`, `cheat_quickbase` and `cheat_jump(contact)` are the
original's own cheat list as plain methods on the campaign (so they are unit-tested like any
order); `app::strategy` binds them to Ctrl+M, Ctrl+K, Ctrl+B and Ctrl+J. `cheat_jump` puts the
first ready craft with a ready pilot at the contact (`away = Some((contact, pos))`, route
cleared), so the next step's interception check reports `Event::Battle` at once. Each cheat is
logged in the message list.

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
pending ambush and (app 0.9.2) whether it is `held`, survey ships, the surveyed flags, the
reserves and the bases' stores;
destinations are written `base:<i>` / `facility:<i>`. Its indices are checked like the rest, a
campaign without logistics saves an empty block, and a save with facilities is refused by a
campaign that has no logistics. Since app 0.10.0 a `story:` block follows too (`write_story` /
`read_story`): events fired and when, arc steps, plan state, standings, aggression, battles won,
the two latches and the ending; a contact's `plan`. Never-fired events hold `-inf` for their last
firing, which YAML numbers cannot carry, so it is written as a very old hour (−10¹²) and read
back as `-inf`. A save whose events, arcs or plans no longer match the rules is refused. A changed mod is refused with a message instead of crashing or silently changing
the game. The screen saves to `<user data>/saves/<campaign id>.yaml` (`strategy::save_path`) on
F5 and when leaving with Esc; *Campaign: continue* loads it.

## Key Symbols

| Symbol | Where | What it is |
|---|---|---|
| `StrategyDef` | `content/strategy.rs` | The strategy layer's rules: `start`, `map`, `star_classes`, `research`, `research_model`, `items`, `hulls`, `modules`, `base_types`, `raids`, `incomes`, `expenses`, `pilots`, `scores`, `delivery`, `salvage`, the `names` table and `logistics: Option<LogisticsDef>` (app 0.9.0). |
| `Date` | `content/strategy.rs` | A Gregorian calendar date (`parse`, `plus_days`). |
| `MapKind`, `Travel`, `RaidTarget`, `ResearchMode` | `content/strategy.rs` | Fixed / generated maps; free / lane travel; base / system raids; download / labs. |
| `Campaign` | `campaign/mod.rs` | One game in progress: `hours`, `money`, `systems`, `bases`, `known`, `progress`, `craft`, `pilots`, `recruits`, `contacts`, `deliveries`, flow and raid timers, score, flags, salvaged, messages. |
| `Base`, `ModuleInst`, `Job`, `Craft`, `Pilot`, `Contact`, `Delivery` | `campaign/mod.rs` | The parts of the state, each referring to the others by index. |
| `Event` | `campaign/mod.rs` | `Notice(String)` (stop the clock, show it), `Battle { contact }`, and since app 0.9.2 – 0.10.0 `Convoy { convoy }` (an ambush waiting for the player), `Choice { event }`, `Mission { mission }` (an offer) and `Ending { ending }`. |
| `StoryDef`, `EventDef`, `Check`, `Choice`, `ArcDef`, `PlanDef`, `Stage`, `Relations`, `Strategist`, `Ending`, `Entry` | `content/story.rs` | The story rules (app 0.10.0); `story::DEF_SECTIONS`, `GLOBALS`, `sections()` (the schema). |
| `StoryState` | `campaign/story.rs` | The story state inside `Campaign::story`; `standing`, `plan_progress`. |
| `story::step`, `story::check_end` | `campaign/story.rs` | One slice of story time; the end conditions. |
| `Campaign::apply_actions`, `choose`, `mission_result`, `raids_allowed`, `encyclopaedia` | `campaign/story.rs` | Story actions; a choice; a story mission's result; whether a faction raids; unlocked entries. |
| `Campaign::convoy_result`, `Convoy::held` | `campaign/logistics.rs` | Settling an ambushed convoy (app 0.9.2). |
| `script::action::EVENT_ACTIONS` | `script/action.rs` | The 14 story actions. |
| `PendingBattle` | `app/strategy.rs` | `Contact`, `Convoy` or `Story`: what the battle being flown is for. |
| `flight::apply_stances`, `Ruleset::set_hostile` | `app/flight.rs`, `content/rules.rs` | The campaign's relations in the battle's own rules (app 0.10.3). |
| `LogisticsDef`, `FacilityType`, `Role`, `Recipe`, `Deposit`, `SystemGoods`, `StationTopic` | `content/logistics.rs` | The logistics rules (app 0.9.0): resources, components, recipes, facility types, sites, per-system goods, per-hull components, per-topic station restrictions, supply settings. |
| `Logistics`, `Facility`, `Line`, `Order`, `Convoy`, `Survey`, `Dest` | `campaign/logistics.rs` | The logistics state inside `Campaign::logistics`. |
| `logistics::step`, `amount` | `campaign/logistics.rs` | One slice of logistics time; goods in a store. |
| `Campaign::cheat_payday`, `cheat_knowitall`, `cheat_quickbase`, `cheat_jump` | `campaign/mod.rs` | Testbed cheats (app 0.8.1). |
| `campaign::autoplay`, `autoplay_from` | `campaign/mod.rs` | The automatic player of `--strategy-soak` and the six-month and year-long tests (public since app 0.8.0; `autoplay_from` continues an existing campaign, app 0.10.3). It lets ambushed convoys fight through, takes every choice's first option and flies offered missions as two wins and a loss in turn. |
| `Campaign::fit_best`, `loadout`, `lane_route`, `nearest_system` | `campaign/mod.rs` | Fitting a craft from its base's store (0.7.2); its battle weapons; the shortest lane route (0.7.3). |
| `flight::apply_loadout` | `app/flight.rs` | Refits the player's class in the battle's own copy of the rules. |
| `MONTH` | `campaign/mod.rs` | 720 hours: reports, salaries, maintenance. |
| `save::write`, `save::read`, `save::PackRef`, `save::FORMAT` | `campaign/save.rs` | Saves and their pack header. |
| `script::campaign::QUERIES`, `id`, `schema` | `script/campaign.rs` | The campaign context. |
| `StrategyScreen`, `Tab` (ten, with `Archive`), `SPEEDS`, `BattleReport` (with `cargo`), `save_path`, `map_transform` | `app/strategy.rs` | The screen. |
| `Next::NewCampaign`, `LoadCampaign`, `CampaignBattle` (with `stances`), `Strategy`; `AppContext::campaign`, `battle_report` | `app/mod.rs` | The screen changes around a campaign battle. |

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
  report with its date.
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
  score are finite, and arcs never go backwards.
- Logistics: the Supply tab's last row counts open orders, convoys and survey ships; the message
  log records every survey, completed facility and ambush. An order that never appears usually
  means no ready facility can fill it (a resource no mined system has, or no manufacturing
  center); a hull job that never starts is waiting for its `components`.
- The campaign is unit-testable without a window: `campaign::autoplay` plays any
  `StrategyDef` with a simple automatic player (keeps every base researching, builds a lab and a
  workshop when it can, intercepts every contact it sees, alternates battle results). The tests
  build small synthetic strategy layers from one YAML document (`content::strategy::tests::build`) and also play the real default campaign for six months.
