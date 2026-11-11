# Campaign

A campaign with a strategy layer is played on the **strategy screen**: a star map with your
bases and the enemy's raids, a clock, and panels for research, building, buying, making,
craft and pilots (app 0.7.1 – 0.7.4). When your craft meet a raid you fly the battle; the debrief brings
you back to the map. The default campaign, *Perils of the Void*, has one; the `interceptor`
campaign does not yet.

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
  bases with their sensor ranges, enemy contacts once detected, and your craft in flight.
- **Panel** (right): eight tabs, each a list of rows. The current base is marked `*` on the
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
| Esc | Save and leave to the launcher |
| Mouse | Click a tab or a row to choose it; click a system on the map to choose it (and its base, if it has one) |

The clock **stops by itself** whenever something needs you: a building or base is finished,
research is complete, a contact is detected, a raid strikes, the monthly report, or an
interception reaching its contact. The message is shown in a banner; Enter, Esc or a click
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

## Limits in app 0.7.4

- Fitting takes the best items in store; you cannot choose them one by one yet.
- Only the default campaign has a strategy layer.
- Two research topics of the default campaign that need salvage (*Saucer hull analysis*, *Robot
  brains*) cannot be started yet (a known issue).
