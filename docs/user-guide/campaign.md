# Campaign

A campaign with a strategy layer is played on the **strategy screen**: a star map with your
bases and the enemy's raids, a clock, and panels for research, building, buying, making,
craft and pilots (app 0.7.1 – 0.9.0), and — in the default campaign — a **supply chain** of
mines, factories and research stations (app 0.9.0). When your craft meet a raid you fly the
battle; the debrief brings you back to the map. The default campaign, *Perils of the Void*, has
one; so does the `interceptor` campaign since app 0.8.0, built from your own copy's data (not
yet play-tested, and without the supply chain, which the original does not have).

The screen is **functional and plain** in this version: every rule is playable, but it is not
yet dressed in the campaign's look (that is Stage 6).

## Starting

- In the launcher, a campaign with a strategy layer lists **Campaign: continue** and
  **Campaign: new game** first, and the selection starts on *continue*.
- *New game* starts at the launcher's difficulty level. *Continue* loads the campaign's save
  (there is one save per campaign); with no save, the launcher shows the error.
- From the command line: `openinterceptor --no-original --strategy` opens a new default campaign
  directly ([Getting started](getting-started.md#run)).

## The screen

```text
 2084-03-14   1 day/s   Funds 2,140,000   Score 310   [Space pause, 1-4 speed, Tab panels, F5 save, Esc leave]
┌──────────────────────────────────────┐┌─ Bases │ Research │ Build │ Market │ … ─┐
│   ·Ajax ─────── ·Vesta Reach         ││ * Gatehouse - Mercury Gate: 3 modules   │
│  /                   ▲ contact       ││   (0 building); researching Heat rays    │
│ ■Mercury Gate                        ││                                          │
└──────────────────────────────────────┘└──────────────────────────────────────────┘
```

- **Top line**: the date, the clock speed, your funds and the campaign score.
- **Map** (left): star systems (hidden ones appear when your sensors reach them), lanes, your
  bases with their sensor ranges, enemy contacts once detected, and your craft in flight. With a
  supply chain also: your **facilities** (a small diamond beside the system, grey while it is
  being built, orange when working), **supply convoys** (gold dots) and **survey ships** (sky-blue
  rings).
- **Panel** (right): nine tabs, each a list of rows. The current base is marked `*` on the
  Bases tab; orders apply to it.

## Keys

| Key | Does |
|---|---|
| Space | Pause, or resume at 1 day a second |
| 1 – 4 | Clock speed: 1 hour, 6 hours, 1 day or 5 days a second |
| Tab / Shift+Tab | Next / previous tab |
| Up / Down | Choose a row |
| Enter | Act on the row (below) |
| Left / Right | Change the current base |
| S | On the Market tab: sell one of the item on the row from the current base |
| F | On the Hangar tab, on a craft: fit it with the best equipment in its base's store |
| F5 | Save |
| Ctrl+M, Ctrl+K, Ctrl+B, Ctrl+J | Testbed cheats ([below](#testbed-cheats)) |
| Esc | Save and leave to the launcher |
| Mouse | Click a tab or a row to choose it; click a system on the map to choose it (and its base, if it has one) |

The clock **stops by itself** whenever something needs you: a building or base is finished,
research is complete, a contact is detected, a raid strikes, the monthly report, an
interception reaching its contact, or — with a supply chain — a facility completed, a survey's
report or a convoy ambushed. The message is shown in a banner; Enter, Esc or a click
dismisses it.

## The tabs

| Tab | Rows | Enter |
|---|---|---|
| **Bases** | Each base: system, modules (and how many are building), what it researches. With a system chosen on the map that has no base: *Build a <type> at <system>* with its cost | Make that base current; or order the new base |
| **Research** | What the current base researches; every topic that can be researched now, with progress and size | The current base researches that topic |
| **Build** | Modules the current base can build, with cost and build time; then its modules and their state (building, days left / working) | Order the module |
| **Market** | Items and hulls on sale, with prices and what you have | Buy one, delivered to the current base after the delivery time. **S** sells one |
| **Factory** | The current base's production rate; what it can make, with the work it takes; its queue | Queue one |
| **Hangar** | Every craft with its base, pilot (and *away* when out) and fit (*standard fit* or the items fitted); then your pilots with rank, kills, missions and state; then recruits with their stats and the fee | On a craft: give it the next free, ready pilot (**F** fits it). On a recruit: hire |
| **Contacts** | Detected enemy flights and where they head | Send the current base's craft that are at home and have a ready pilot to intercept |
| **Supply** | With a system chosen on the map: *Survey* it, and *Build* each kind of facility there (with the cost, and why not when it is not allowed). Then your facilities and what they hold or how long they take to build; on a working research station, the topics it can research; the current base's store of components; how many orders, convoys and survey ships are under way ([Supply](#supply)) | Send the survey ship; order the facility; start the research |
| **Messages** | The log, newest first, with dates | — |

Each order answers *Done.* or says why not (not enough money, not yet researched, the limit
for that kind of module, too far from your other bases, no free pilot, …).

## Interceptions and battles

1. A raid is detected: the clock stops with *Contact: <raid>, heading for <base>*.
2. On the Contacts tab, Enter on it sends your ready craft. They fly to it on the map — along
   the lanes to the system nearest it, then straight to it (app 0.7.3). Raiders, too, come in
   along the lanes, and the map draws each one's way ahead.
3. When they reach it, the clock stops and a banner asks: **Enter** to fly the battle, **Esc** to
   break off (the contact flies on).
4. The battle is the raid's mission, with the first craft's hull as your ship and its fitted
   weapons (app 0.7.2). Fly it as usual. When the result is decided the battle runs on until the
   debrief ([Debrief](getting-started.md#after-a-battle-the-debrief)); at the debrief, **Enter** or
   **Esc** returns to the map. Leaving from the pause menu (L or Q)
   counts as a retreat.
5. Back on the map, a banner gives the result and the kills. The outcome's score and money
   apply; the leader gains the kills and experience; if you were shot down, the craft is lost and
   its pilot ejected injured — or was killed.

A raid you do not stop strikes its target: a base loses its newest finished module, and the
score drops.

## Supply

*Default campaign, app 0.9.0.* The Guard builds its craft from **components** — hull plating,
rocket motors, ray emitters — made from mined **resources**. When you order a hull on the
Factory tab, the base orders the components it lacks; a **manufacturing center** makes them
from resources it orders in turn from your **mining facilities**; everything is carried by
automatic government **supply convoys** along the lanes. You do not move the convoys; you
decide where to survey and what to build.

1. **Survey.** Click a system on the map, open the Supply tab and choose *Survey <system> from
   <base>*. The system must be within the current base's radar range. The survey ship flies
   there (a sky-blue ring); when it arrives a message lists the deposits it found, with their
   rates a day, and any research site. A survey ship may be attacked on the way: it fights off
   the attack or is lost (then survey again).
2. **Build.** With a system chosen, the Supply tab offers each kind of facility with its cost:
   - a **mining facility** needs a surveyed system with deposits;
   - a **manufacturing center** must be within two lanes of one of your bases;
   - a **research station** needs a surveyed system that is a research site.

   The row says why when it cannot be built. A facility takes days to build (a grey diamond on
   the map) and a message says when it is complete.
3. **Make.** Queue a hull on the Factory tab as usual. Its row waits until the components have
   arrived; the Supply tab shows the base's store and the orders and convoys under way.
4. **Research at a station.** A finished research station lists the station-only topics of
   its kind (*Research Theremin shielding here*); a topic is refused when the station is not on
   its research site or its prerequisites are not known. The station first orders the topic's
   start materials; once they arrive it researches on its own.

**Convoys can be ambushed** on the way. In this version you do not fly those battles: each
vessel may be lost, a message says how many, and the survivors deliver their share; what is
missing is simply ordered again.

## Money and the monthly report

Incomes (the default campaign's Senate stipend grows with your bases and your month's score)
and expenses are paid on their own schedules and logged. Every 30 days the **monthly report**
charges module maintenance, craft upkeep and pilot salaries, shows the month's score, the total
and your funds, and a new set of recruits is offered.

## Saves

- **F5** saves; **Esc** saves and leaves. The save is `<user data folder>/saves/<campaign>.yaml`
  (for example `saves/default.yaml`); the user data folder is described on
  [Settings](settings.md).
- A save records the packs it was played with. Continuing refuses a save that needs a pack that
  is not active, or one whose rules no longer fit it (a mod was changed), with a message.

## Testbed cheats

The testbed keeps the original game's own cheat list as debug keys on the strategy screen (app
0.8.1). Each says so in the banner and the message log.

| Key | Cheat | Does |
|---|---|---|
| Ctrl+M | *payday* | A million more in funds. |
| Ctrl+K | *knowitall* | Every research topic known; bases stop researching. |
| Ctrl+B | *quickbase* | Every base and module under construction finished now (not supply facilities). |
| Ctrl+J | *jump to battle* | With a contact chosen on the Contacts tab: a ready craft with a pilot is put at it at once, and the fly-or-break-off banner appears. Otherwise it says *Choose a contact on the Contacts tab first*, or *no ready craft with a pilot*. |

## Limits in app 0.9.0

- Fitting takes the best items in store; you cannot choose them one by one yet.
- Supply-convoy ambushes are decided automatically; you cannot fly them or escort convoys yet,
  and facilities have no fighters of their own. The Supply tab is plain text.
- The `interceptor` campaign's strategy layer has not been play-tested; its research is slow
  with one downlink.
- Fixed in app 0.9.0: manufacturing now costs money as each unit starts, and the default
  campaign's salvage topics (*Saucer hull analysis*, *Robot brains*) can be started after
  salvaging the right raid.
