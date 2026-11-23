# Story layer

The story layer (app 0.10.0 – 0.10.3, plan items 5.1 – 5.9) is what turns a
[strategy layer](strategy.md) into a campaign with a beginning, a middle and an end: **events**
that fire on conditions and may offer the player a choice or a mission, **arcs** that put events
in order, a rival's **plans** built from shipments the player can intercept, **relations** with
the other factions, a **strategist** whose aggression answers the player, **endings**, and an
**encyclopaedia**. All of it is pack data in the strategy files; a campaign without these
sections plays exactly as before.

Requirements: [R-CPN-007 – R-CPN-010](../specification/requirements/campaigns.md#r-cpn-007-events-and-arcs).
Design (the full planned format, of which this page is the built part):
[Format: campaign and game flow](../specification/design/format-campaign.md#arcs-story-progression),
[Campaign events](../specification/design/format-missions.md#campaign-events-eventid),
[Dynamic relations](../specification/design/format-strategy.md#dynamic-relations-optional-per-campaign).
How the engine runs it: [Architecture › Campaign › The story](../implementation/architecture/campaign.md#the-story-app-0100).
What the player sees: [User guide › Campaign › The story](../user-guide/campaign.md#the-story).

## Where the files go

Like every strategy section the story sections may sit in any rules file; by convention they go
in `<pack>/strategy/story.yaml`:

| File | What it holds |
|---|---|
| `resources/default/strategy/story.yaml` | The default campaign's twelve-chapter serial, side events, the Eclipse Engine plan, relations, the Dominion's strategist, four endings, five encyclopaedia entries (app 0.10.1) |
| `packs/interceptor/strategy/story.yaml` | The X-COM Doomsday arc, the Doomsday plan, two side missions, the alien strategist, three endings (app 0.10.2) |

`events`, `arcs`, `plans`, `endings` and `encyclopaedia` are **definition sections**: they
layer across packs like any other definitions, so an addon may add an event, `inherit` a plan
or `delete: true` an ending ([Rulesets](rulesets.md#layering-inherit-delete-and-list-edits)).
`relations`, `strategist` and `campaign_end` are **global sections**: the last pack to write one
wins. Every id these sections define is a named constant in the campaign's conditions, so a
misspelt event, arc, plan or ending id is a load error that names it.

The campaign's **presentation** keys sit beside the story: `campaign.skin` ([Skins](skins.md)),
`campaign.intro` and `campaign.intro_video` ([Cutscenes](cutscenes.md)), and the `music` sets
([Music](music.md)).

## A worked example

A three-step arc with a choice, a rival plan and two endings:

```yaml
# mypack/strategy/story.yaml
flags: [rival_revealed, beacon_destroyed]   # every flag a condition reads; one flags: list per campaign

arcs:
  main: {steps: [opening, reveal, finale]}

events:
  opening:
    when: day >= 0
    do: ["message(The frontier is quiet. Too quiet.)"]
  reveal:
    when: day >= 30 or research(deep_radar)
    do: ["message(Their fleet is building a beacon!)", set_flag(rival_revealed)]
    choices:
      - {text: "Ask the Senate for funds", do: ["money(200000)", "relation(senate, -10)"]}
      - {text: "Go it alone", do: ["relation(senate, 5)"]}
  finale:
    when: plan(beacon) >= 0.5
    do: ["message(Strike the beacon now!)", "mission(beacon_assault)"]

plans:
  beacon:
    owner: dominion
    stages: [{name: Frame, count: 4}, {name: Core, count: 4}]
    start_when: flag(rival_revealed)
    every: 7d
    chance: 60%
    raid: beacon_shipment                # a raid with chance: 0%, launched only by the plan
    warnings: [{at: 75%, do: ["message(The beacon is nearly complete.)"]}]
    complete: ["defeat(beacon_lit)"]

endings:
  beacon_destroyed: {victory: true, text: "The beacon is gone."}
  beacon_lit: {text: "The beacon is lit, and the frontier is theirs."}

campaign_end:
  victory: [{when: "flag(beacon_destroyed)", ending: beacon_destroyed}]
  defeat: [{when: "bases == 0", ending: beacon_lit}]
```

The mission `beacon_assault` sets the flag in its win outcome
(`outcomes: {win: [set_flag(beacon_destroyed)]}`), which the next daily check turns into the
victory.

## `events`

An event is a condition, a chance, a list of actions and, optionally, choices for the player.

| Key | Type | Default | Meaning |
|---|---|---|---|
| `when` | condition | always | When it may fire, in the [campaign context](#conditions). |
| `check` | `daily`, `weekly`, `monthly`, `after_battle`, `on_research`, `scheduled` | `daily` | How often it is checked: at each day, week (168 h) or month (720 h) boundary, after any battle the player flies, or after any research topic is finished. `scheduled` (app 0.22.31): only the [side-mission scheduler](#side_missions) fires it. |
| `chance` | fraction | 100 % | The chance per check, once `when` holds. |
| `do` | list of [actions](#actions) (or one action) | none | What it does when it fires. |
| `choices` | list of `{text, do}` | none | A choice for the player, made after `do` has run: `text` is the option (a `@key` or words), `do` its actions. Up to five are offered on number keys. |
| `once` | bool | `true` | Fires only once. |
| `cooldown` | time | 0 | For a repeating event (`once: false`): the least time between firings (`60d`). |
| `delay` | `[min, max]` times | none | App NEXT: once the event would fire (its check, condition and chance passed), it fires a random time in this range later instead, whatever its `check` (`interceptor`: the MacArthur arrives `[24h, 48h]` after the request, RQ-S-15, **verified**). |
| `arc` | arc id | none | The arc it is a step of. Not needed when the arc lists it in `steps`. |

An event fires when, at a check, it is not spent (`once`) or cooling down, it is its arc's next
step (if it belongs to an arc), `when` holds, and its chance comes up. Events are checked in the
order they are defined.

## `arcs`

```yaml
arcs:
  serial: {steps: [ch01, ch02, ch03]}
```

`steps` lists events in order. A step can fire only after the step before it has fired, so
each step's `when` needs to say only what *this* step waits for; `arc(serial)` reads how many
steps have fired. A campaign may run several arcs side by side. An event listed in `steps`
belongs to that arc whether or not it says `arc:`; an unknown event in `steps` is an error.

## `plans`

A rival's project, built from **shipments** the player can intercept. A plan's completion
usually loses the campaign.

| Key | Type | Default | Meaning |
|---|---|---|---|
| `owner` | faction id | — | Whose plan it is (shown in the Archive; not otherwise used yet). |
| `stages` | list of `{name, count}` | — | The plan's parts in order; `count` components each (at least 1; `{per_level: …}` allowed). The plan's size is the sum. |
| `start_when` | condition | at once | When work begins (logged: *Intelligence: the rival's … has begun*). |
| `every` | time | `7d` | How often a shipment is rolled once work has begun. |
| `chance` | fraction | 100 % | The chance per roll that a shipment leaves (`{per_level: …}` allowed). |
| `work` | number | 0 | With [`alien_bases`](strategy.md#alien_bases) (app 0.22.34): each piece takes this many **technician-hours** — every 20 game hours the hours since the last update × the rival's technicians (10 + 10 per platform for each alien outpost, from [`manufacture`](strategy.md), app 0.22.35) go to the current piece — and is shipped (the plan's `raid`, from an alien base) when done; `every` and `chance` then do not apply. 0 = the rolls. `interceptor` (app 0.22.35): 100,000 / 80,000 / 70,000 / 50,000 / 30,000 by level (unknowns §12.2, **verified**); an alien outpost's platforms are one per planet of its system (*derived*). |
| `raid` | raid id | none | The [raid](strategy.md#raids) a shipment flies as: a contact on the map, detected, intercepted and fought like any raid. Without it the plan grows only through the `plan(…)` action. |
| `warnings` | list of `{at, do}` | none | Actions run once when progress passes `at` (a fraction), in order. |
| `complete` | actions | none | Actions run when the last component arrives (usually `defeat(<ending>)`). |

A shipment that **reaches its destination unopposed adds one component**; one the player
intercepts and wins (or draws) against is gone and adds nothing; one the player loses to or
breaks off from flies on and may still arrive. A plan's shipments go only while the player has
a base. Give the shipment raid `chance: 0%` so that the ordinary raid roll never launches it:
only the plan does.

`plan(<id>)` in a condition reads the progress, 0 to 1.

## `relations`

The player's standing with each other faction, from −100 to 100 (a global section).

| Key | Type | Default | Meaning |
|---|---|---|---|
| `dynamic` | bool | `false` | Standings change: actions move them, they drift, and stances take effect. Without it standings are fixed and the factions' own `hostile` lists decide everything. |
| `start` | mapping faction: number | none | Starting standings; an unlisted faction is 0. |
| `drift` | number | 0 | Each month every standing moves this much towards 0. |
| `stances` | mapping stance: upper bound | hostile −40, wary −10, neutral 20, friendly 60 | Stance names and the highest standing each covers, sorted by bound; above the last bound a faction is `allied`. |

With `dynamic: true`:

- `relation(faction, n)` moves a standing; a change of stance stops the clock with *Relations:
  X are now Y (were Z)*.
- **Raids stop**: a raid of a faction that has a standing flies only while that faction is
  `hostile`.
- **Battles follow the stances** (app 0.10.3): when a campaign battle starts, every faction with
  a standing is made hostile to the player's faction if its stance is `hostile`, and not
  hostile otherwise — in that battle's own copy of the rules. A faction befriended stops
  attacking; one turned hostile attacks.
- `relation(<faction>)` in a condition reads the standing, and expressions may use it — the
  default campaign's Senate stipend adds `relation(senate) * 3000`.

Not built yet: allied factions fighting at the player's side, relation effects on prices and
trade, the design's `relation_rules`, and standings between factions other than the player.

## `strategist`

The rival's mood (a global section). Its **aggression** multiplies the chance of **every raid**.

| Key | Type | Default | Meaning |
|---|---|---|---|
| `aggression.start` | number | 1 | Starting aggression. |
| `aggression.min`, `aggression.max` | number | 0.5, 2 | Its limits. |
| `aggression.up` | number | 0.1 | Monthly rise while the player wins fewer than `calm_after_wins` battles that month (sits back). |
| `aggression.down` | number | 0.1 | Monthly fall when the player wins at least that many. |
| `calm_after_wins` | number | 2 | Battles won in a month that count as winning. |

Without a `strategist` section aggression stays at 1. The `aggression(n)` action moves it
directly. `interceptor` starts at 0.7, `Think.ini`'s `StartingAggression` (D-25, **verified**).

`strategist.economy` (app 0.22.31, RQ-S-14) — the rival's treasury and tech level:

| Key | Type | Default | Meaning |
|---|---|---|---|
| `income_every`, `income` | time, expression | 25 d, `125000 * (difficulty + 1)` | Money the treasury gets each period. |
| `split_every`, `ratios` | time, `[research, goods, doomsday]` | 6.25 d, `[0.3, 0.4, 0.3]` | Each period the treasury is split into the three funds by these shares. |
| `spend_every`, `research_per_downlink`, `downlinks` | time, number, number | 20 h, 20000, 0 | Each period up to `research_per_downlink` × `downlinks` / 7 (/ 14 with none) moves from the research fund into research. |
| `tech_levels` | list of `[to 2, to 3]` per difficulty level | none | Research money at which the tech level rises to 2 and to 3. |

Conditions read the level as `alien_tech` (1 – 3). With [`alien_bases`](strategy.md#alien_bases)
and its `outpost_income` (app 0.22.34) the rival's outposts pay instead, each once old enough; with one
outpost or none the `income` formula applies. `interceptor` follows the original (README
§6.1, **verified**): its outposts pay (455,625 each a period, *derived*: the README's example
outpost), else the fallback 125,000 × (difficulty + 1), `Think.ini`'s
ratios, 20,000 per downlink and the thresholds 1.0M / 1.5M (Beginner) … 0.5M / 0.9M (Superhuman);
its 3 alien downlinks and the missing aggression adjustment of the split are *derived*.

## `side_missions`

`side_missions: {check_every, list}` (app 0.22.31, RQ-S-13) — a scheduler for side missions: every
`check_every` (a random time in `[min, max]`) the entries of `list` are tried **in order**, and
the first whose `months` window holds (whole 30-day months since the start), whose event is not
spent (`once`) and whose `when` holds fires, with its own `chance`; at most one a check. The
events are `check: scheduled`.

| Entry key | Type | Meaning |
|---|---|---|
| `event` | event id | The side mission's event (unknown: load error). |
| `months` | `[first, last]` | Months it may fire in (default: any). |
| `chance` | fraction | The chance when it qualifies (default 100 %). |

`interceptor` (README §6.2, **verified** order, months and chances): alien raiders (months 2 – 7,
10 %), rogue freighter (5 – 17, 20 %), pirate rescue (9 – 23, 20 %), arms dealer (from 6, 70 %:
*derived*, the original uses the aliens' aggression), every 6 – 9 days. Which mission stands for
each, and the month offset (the original's month value − 1), are *derived*; the Nova Bomb hijack
and the recon mission are not built. `default`: Dominion scouts and a Black Comet ambush call.

## `endings` and `campaign_end`

```yaml
endings:
  engine_destroyed: {victory: true, text: "…"}
  last_base: {text: "…"}
campaign_end:
  defeat: [{when: "bases == 0", ending: last_base}]
  victory: [{when: "flag(engine_destroyed)", ending: engine_destroyed}]
```

An ending has a `text` (a `@key` or words), `victory` (default `false`) and, since app 0.12.1,
an optional `video` — an asset played full screen when the campaign ends this way, before its
text (the X-COM victory plays the original's closing credits, `original:credits_sm2a.smk`). `campaign_end`
lists conditions for each: they are checked every day and after every battle, **defeats
first**, and the first that holds ends the campaign. The `victory(<ending>)` and
`defeat(<ending>)` actions end it at once. Once ended, the campaign stops; the screen shows
*VICTORY.* or *DEFEAT.* with the text, and Enter saves and leaves. Continuing a finished campaign
shows its ending again (app 0.12.1). A partial victory is simply another ending with
`victory: true` and its own condition.

## `encyclopaedia`

```yaml
encyclopaedia:
  dominion: {title: The Umbral Dominion, text: "…", unlock: day >= 0}
```

Each entry has a `title` (default the id), a `text` and an `unlock` condition; entries whose
condition holds are listed on the strategy screen's **Archive** tab.

## Actions

Every action list in the story — `do`, `choices[].do`, `warnings[].do`, `complete` — and the
strategy layer's [`evaluations`](strategy.md#evaluations) `do` lists are checked when the rules
load against the nineteen event actions (`script::action::EVENT_ACTIONS`, with `script(...)` since app 0.18.0): an unknown action or a
wrong argument is an error with its place. A text argument may be a [text key](text.md) (`message(@story.beacon)`); its key is resolved when the content loads in every one of these lists — in a plan's `complete` list and in evaluations only since app 0.18.1 (before, they kept the raw `@key` and `--check` did not report a missing one).

| Action | Arguments | Does |
|---|---|---|
| `message(text)` | text | A message that stops the clock (and goes in the log). |
| `news(text)` | text | A *News:* line in the log; the clock runs on. |
| `card(Title \| text)` | text | A full-screen **title card** in the campaign's [skin](skins.md) (app 0.11.1): the title large in the skin's `border` colour, the text after `\|` wrapped beneath. The clock stops; Enter, Esc or a click dismisses it. The title goes in the log. Made for chapter openings. |
| `video(ref)` | asset | Play a **video** full screen (app 0.12.0): `original:<file>.smk` from the player's data, or a pack path; Smacker only. The strategy screen waits and comes back afterwards ([Cutscenes › Videos](cutscenes.md#videos)). |
| `play_cutscene(id)` | cutscene | Play the engine **cutscene** `cutscenes/<id>.yaml` (app 0.13.0, [Cutscenes](cutscenes.md)); back to the strategy screen afterwards. |
| `set_flag(x)`, `clear_flag(x)` | flag | Set or clear a campaign flag. |
| `money(n)` | number | Add (or, negative, take) money. |
| `score(x)` | score id | Add a score entry ([`scores`](strategy.md#pilots-and-scores)). |
| `relation(faction, n)` | faction, number | Change a standing (dynamic relations only). |
| `research(x)` | topic | The topic becomes known. |
| `discover(x)` | topic | A [`hidden`](strategy.md#research-and-research_model) topic is discovered: from now on it is offered once its prerequisites hold. A notice says *New research possible: <name>*; kept as the flag `discovered.<x>` (app 0.16.6). An unknown topic is logged and sets no flag (app 0.16.15). |
| `mission(x)` | mission | **Offer** a mission to fly now: the player accepts with Enter (the first ready craft with a pilot flies it) or declines with Esc. The outcome's campaign actions (`score`, `money`, `set_flag`, `salvage`, `relation`, and since app 0.16.3 `capture`, `lose`, `cargo_lost`, `cargo_captured`) apply as for a raid's battle ([Raids and battles](strategy.md#raids-and-battles)), and a win counts towards `battles_won`. |
| `raid(x)` | raid | Launch that raid's contact now. |
| `plan(x, n)` | plan, number | Add `n` components to a plan (warnings and completion follow). |
| `aggression(n)` | number | Change the rival's aggression, within its limits. |
| `script(file:function)` | `scripts/<file>.ois:<function>` | Run a [Tier 2 script](scripts.md) in the campaign context (app 0.18.0): it reads the campaign's queries and takes these actions, each applied at once. |
| `victory(x)`, `defeat(x)` | ending | End the campaign in that ending. |

When one day's events ask for several videos or cutscenes, the last video asked for plays, and
a cutscene only when no video was asked for; a video and a cutscene in the same event's list do
not both play. An unknown video or `play_cutscene` id is reported on standard error when it would
play (`video …: …`, `cutscene x: no cutscene 'x'`), not when the rules load (`campaign.intro` is
checked at load since app 0.14.1).

An action with a single text argument takes everything between the brackets, commas included:
`message(Hold on, Guardsman!)` (app 0.10.1). In a YAML flow list, quote an action that contains
a comma, a colon or a `+`.

**Text keys.** Message, news and card texts, choice texts, ending texts and encyclopaedia titles and
texts may be `@keys` (app 0.10.2), resolved like any other text: `@msg.chapter_1` from the
pack's `text/`, or `@orig.KEY` from the player's original language files.

## Conditions

Story conditions — `when`, `start_when`, `campaign_end` and `unlock` — use the
[campaign context](strategy.md#expressions-the-campaign-context), including its logistics
queries (`deposit`, `feature`, `research_site`, `survey_known`, `owned`, `lane_distance`,
`facility_progress`, `construction_attacks`; app 0.16.3) and `base_distance` (app 0.19.0), with
five more queries:

| Query | Result | Meaning |
|---|---|---|
| `arc(x)` | number | Steps of arc `x` fired so far. |
| `plan(x)` | number | Rival plan `x`'s progress, 0..1. |
| `relation(x)` | number | Faction `x`'s standing with the player (−100..100). |
| `fired(x)` | number | Times event `x` has fired. |
| `battles_won` | number | Battles the player has won, story missions included. |

## The default campaign's serial

`resources/default/strategy/story.yaml` (app 0.10.1) is *Perils of the Void*'s serial, as in
the [campaign definition](../definition/campaigns/default-campaign.md):

- **Arc `serial`**, twelve chapters `ch01` – `ch12`, each a title card (`card(Chapter N -
  Title | hook)`, app 0.11.1; a `message` before that) and a `mission` offer. Chapters are triggered by the Guard's progress (days,
  research, bases), its relations and the Eclipse Engine's progress (`plan(eclipse_engine)`).
  Chapter 3 sets `regent_revealed`, which starts the plan; chapter 7 is **the Cartel's choice** —
  escort the Cartel's shipment (the Cartel becomes an ally, money) or raid it (the Cartel turns
  hostile, the Dominion's aggression falls, a mission); chapter 10 sets `engine_site_found`;
  chapter 12 offers the finale `eclipse_assault`.
- **Chapters whose bespoke mission is not built yet fly the nearest existing mission**
  (`colony_defence`, `convoy_ambush`, `intercept_scouts`, `pocket_ambush`, `ground_strike`,
  `defend_station`, `outpost_raid`); only the finale has its own.
- **Side events**: the Cartel's offers (monthly, 40 %, `once: false`, `cooldown: 60d`), the Belt
  Prospectors' tithe to friends (monthly while `relation(prospectors) >= 20`) and the Black
  Comet's truce, a choice that can buy the raiders off (standing +70 → neutral, so they stop
  raiding and stop attacking in battle).
- **Plan `eclipse_engine`**: Lens, Coils and Heart, six components each; starts with
  `regent_revealed`; a shipment every 9 days at 70 % as the raid `eclipse_shipment`
  (`intercept_scouts`, `chance: 0%`); warnings at 33 %, 66 % and 90 %; complete =
  `defeat(engine_lit)`.
- **Relations**: dynamic; Senate 40, Cartel −5, Prospectors 15, raiders −70, Dominion −100;
  drift 2 a month; the default stances. The Senate stipend follows the Senate's standing.
- **Strategist**: aggression 1.0 (0.6 – 1.8, ±0.1 a month), calm after 3 wins.
- **Endings**: `engine_destroyed` (victory, set by `eclipse_assault`'s win), `engine_lit`,
  `last_base` (no bases left) and `senate_recognises` (`relation(senate) <= -60`).
- **Encyclopaedia**: the Guard, the Dominion, cosmium, the Eclipse Engine (once the Regent is
  revealed) and the Cartel (once chapter 7 has fired).

The finale mission `eclipse_assault` (in `missions/missions.yaml`) is set in the `pocket`
nebula: the Engine (a `dominion_outpost`), three Thunderbolt wingmen, a guard fleet that comes
again every two minutes while the Engine stands, and a dreadnought after 90 s; its win scores
`engine_destroyed` and sets the flag.

The unit test `campaign::tests::the_default_campaign_runs_a_year_with_its_story_and_stays_consistent`
plays it for a year with the automatic player (which takes each choice's first option and
alternates mission results), a month at a time, checking that nothing refers to something that
is not there, that arcs only move forward, and that the serial and the Engine are under way.

## The `interceptor` campaign's Doomsday arc

`packs/interceptor/strategy/story.yaml` (app 0.10.2) expresses the original's story from the
[design analysis](../specification/design/original-analysis.md) — the gate chain the strategy
guide describes. Every estimate is marked `DERIVED` in the file.

- **Arc `doomsday`**: `alien_message_1` – `alien_message_5`, `summon_macarthur`,
  `final_mission`. Since app 0.22.31 (RQ-S-12, README §4.3, unknowns §11.2, **verified**) each message
  is intercepted after a battle with the AlienMessage goodies chance (7 / 7 / 5 / 5 / 4 %), from
  its month (February, April, June, August, November 2067 — here whole 30-day months 1 / 3 / 5 /
  7 / 10, *derived*) once its prerequisites are known: the previous message, the saucer
  examinations and the Doomsday pieces; it opens its (hidden) research topic. MacArthur needs
  three research topics; the final mission (`destroy_doomsday_weapon`) follows 24 – 48 hours later
  (`delay`, **verified**).
- **Plan `doomsday`**: three stages (Structural, Weapon, Drive) with the original's counts by
  difficulty (8/8/9, 9/8/8, 10/8/7, 7/7/6, 7/6/5, unknowns §12.2, **verified** since app 0.22.31),
  shipments as the raid `doomsday_convoy` (the escorted alien freighter ambush) with a chance by
  difficulty, three warnings and `defeat(doomsday_complete)`.
- **Side missions**: the [scheduler](#side_missions) with four of the original's seven; Kabron
  pirates stay as a *derived* stand-in for the minor missions.
- **Strategist**: aggression from 0.7 (rise and fall *derived*) and the [aliens' economy](#strategist).
- **Videos** (app 0.12.0): each alien message also plays the original's transmission film,
  `video(original:alientrans1.smk)` – `alientrans5.smk`; the victory ending plays the credits
  film (`video: original:credits_sm2a.smk`, app 0.12.1); and a new campaign opens with
  `campaign.intro_video: original:animation1.smk` (app 0.13.0) — which of the CD's films is the
  original's opening is *derived*.
- **Endings**: `doomsday_destroyed` (victory), `doomsday_complete` and `last_base`.
- **Texts from the original**: the warnings, MacArthur's prompt and two endings use the
  original's own text keys — `@orig.DD_WARNING1` – `DD_WARNING3`, `@orig.MAC_PROMPT1`,
  `@orig.DD_TOOLATE`, `@orig.S_LASTBASEDEAD` — read from the player's data at load.

The saucer examinations the message gates name are the original's topics 70 – 72 (**verified**
since app 0.22.27).

## Trying a story out

- `openinterceptor --campaign <id> --strategy-soak 365` plays a year headless and now prints the
  last monthly report and a story line: each arc's steps (`serial 12/12`), each plan's progress,
  the aggression and the ending reached (or `none`).
- `--check --campaign <id>` reports every story error below.
- On the strategy screen the **Archive** tab shows the plans under way, every standing and
  stance, the aggression, battles won and the unlocked encyclopaedia.
- **The event debugger** (app 0.16.13): **Ctrl+D** on the strategy screen adds a *Debug* list —
  *Arc <id>: step n of m* for each arc, then every event as *<id> - fired n, condition holds*
  (or *false*), evaluated now. *Fire now* on an event fires it whatever its condition
  (`Campaign::fire_event`): it counts as fired, its arc advances **only if it is the arc's next
  step**, its actions run and its choice is offered; the log records *Debug: fired event <id>*.
  A test event with `when: day >= 99999` is a handy way to run any action list by hand.
- **Start at a chapter** (app 0.16.13): `openinterceptor --campaign <id> --strategy --chapter
  <n>` plays a new campaign forward headless with the automatic player (`--strategy-soak`'s)
  until its **first** arc reaches step *n*, or for at most 1500 days, then opens the strategy
  screen there.

Not built yet: setting flags and plan progress directly (fire an event that does it).

## Errors

| Message | Cause |
|---|---|
| `events.x.check: 'y' is not daily, weekly, monthly, after_battle or on_research` | A wrong check period. |
| `events.x.arc: unknown arc 'y'` / `arcs.x.steps: unknown event 'y'` | An arc or step that is not defined. |
| `events.x.do: unknown action 'y' (known: …)` (also `.choices`, `plans.x.warnings`, `plans.x.complete`) | An action that is not an event action, or a wrong argument. |
| `events.x.when: …` (also `plans.x.start_when`, `encyclopaedia.x.unlock`, `campaign_end.victory`) | A condition that does not parse, or names something not defined. |
| `plans.x.raid: unknown raid 'y'` | A plan shipping as a raid that is not defined. |
| `campaign_end.victory: unknown ending 'y'` (or `defeat`) | An end condition naming an ending that is not defined. |
| `save: the story's events or arcs do not match the loaded rules (was a mod changed?)` (or `plans`) | Continuing a save after events, arcs or plans were added or removed. |
