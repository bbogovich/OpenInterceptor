# Campaign

A campaign with a strategy layer is played on the **strategy screen**: a star map with your
bases and the enemy's raids, a clock, and panels for research, building, buying, making,
craft and pilots (app 0.7.1 – 0.9.0), and — in the default campaign — a **supply chain** of
mines, factories and research stations (app 0.9.0). When your craft meet a raid you fly the
battle; the debrief brings you back to the map. The default campaign, *Perils of the Void*, has
one; so does the `interceptor` campaign since app 0.8.0, built from your own copy's data (not
yet play-tested, and without the supply chain, which the original does not have).

Since app 0.10.0 each campaign also has a **story** ([below](#the-story)): chapters or
messages that arrive as you play, choices to make, missions offered to you, a rival building
something you must stop, factions whose standing with you changes, and endings — victory or
defeat.

Since app 0.16.0 the screen is played with the **mouse**: a map you zoom and drag, a panel for
whatever you click on with its actions as buttons, lists that scroll with a button for every
order, and prompts with buttons. The keys still work. It is drawn in the campaign's **skin** —
the default campaign's art-deco navy and gold, `interceptor`'s green on steel (app 0.11.1) — and
fills the window. Since app 0.11.0 – 0.13.0 the campaign has **music**, chapter **title cards**,
**films** and **cutscenes** ([Films, cutscenes and music](#films-cutscenes-and-music)).

## Starting

- Since app 0.14.0 the game opens on the campaign's **main menu**: **Continue** (when there is a
  save) and **New campaign** ([Getting started › The main menu](getting-started.md#the-main-menu)).
  The testbed launcher (*Battles & tools*) still lists **Campaign: continue** and
  **Campaign: new game** first.
- *New game* starts at the launcher's difficulty level. If the campaign has an introduction it
  plays first: the default campaign's opening cutscene, or `interceptor`'s opening film from your
  data (app 0.13.0). Esc skips it. *Continue* loads the campaign's save
  (there is one save per campaign); with no save, the launcher shows the error.
- From the command line: `openinterceptor --no-original --strategy` opens a new default campaign
  directly ([Getting started](getting-started.md#run)).

## The screen

```text
┌ 2084-03-14   Funds 2,140,000   Score 310     [<] Gatehouse [>]     [||][1h][6h][1d][5d][Save][Menu] ┐
┌──────────────────────────────────────────┐ ┌ Bases │ Research │ Build │ Market │ Factory ┐
│        ·Ajax ───────── ·Vesta Reach       │ │ Hangar│ Contacts │ Supply│ Archive│ Messages│
│       /            ▲ Saucer raid          │ ┌ Mercury Gate ──────────────────────────────┐
│  ■Mercury Gate  ◆                         │ │ yellow dwarf star. Gatehouse: 3 modules;   │
│                     ● convoy              │ │ researching Heat rays.                     │
│                                           │ │ [Survey]                                   │
│                                           │ └────────────────────────────────────────────┘
│                                           │ ┌ Market ────────────────────────────────────┐
│                                           │ │ Heat ray - 120,000 (have 2)                │
│ square: base  red: hostile  gold: convoy …│ │ Rocket rack - 45,000 (have 0)          ▮   │
└───────────────────────────────────────────┘ │ ─────────────────────────────────────────  │
                                              │ Heat ray - 120,000 (have 2)                │
                                              │ [Buy 1] [Buy 5] [Sell 1]                   │
                                              └────────────────────────────────────────────┘
```

Five parts (a mod may move them: [Screen layouts](../modding/screens.md)):

- **Top bar.** The date, your funds and the campaign score. In the middle the **current base**
  between `<` and `>` buttons: orders apply to it. On the right the clock — `||` pause, then
  `1h`, `6h`, `1d` and `5d` (campaign time per real second; the running speed is outlined) —
  then **Save** and **Menu** (save and go to the main menu).
- **Map** (left). Star systems (hidden ones appear when your sensors reach them), lanes, your
  bases (squares) with their sensor ranges, enemy contacts once detected (red, with their way
  ahead), and your craft in flight. With a supply chain also your **facilities** (a diamond beside
  the system, grey while building, orange when working), **supply convoys** (gold dots) and
  **survey ships** (sky-blue rings). A legend runs along the bottom.
  - **Wheel**: zoom in and out about the mouse pointer — the point under it stays put.
  - **Right or middle drag**: pan.
  - **Hover**: the name of what is under the pointer.
  - **Click**: select the nearest system, contact, convoy, survey ship or craft in flight
    (moving things win when they are as close as a system). Clicking a system with one of your
    bases also makes that base current.
- **Info panel** (right, top). What you selected, its details and its **actions as buttons**:
  - a system without a base: *Build <base type> (<cost>)*; with logistics, *Survey* and
    *Build <facility> (<cost>)* for each facility allowed there;
  - a system with a base that is not current: *Make <base> current*;
  - a contact: where it is heading, how many craft are ready at the current base, and
    **Intercept**;
  - a convoy, a survey ship or a craft: what it is doing.

  With nothing selected it says how to use the map.
- **List buttons**: ten buttons that open a list in the panel below; the open one is outlined.
- **Panel** (right, bottom). The open list ([The lists](#the-lists)). Rows that are too long end
  in `...`; the **selected row is shown in full** under the list, with **its actions as
  buttons**. Click a row to select it; **double-click** to do its first action. The wheel, the
  scroll bar or Up/Down scroll the list. Grey rows are information only.

Every order answers in a short yellow line at the bottom of the map: *Done.*, or why not (not
enough money, not yet researched, the limit for that kind of module, too far from your other
bases, no free pilot, …).

## Prompts

When something needs you, the clock **stops by itself** and a **prompt** opens in the middle of
the screen with buttons; nothing else can be clicked until you answer.

| Prompt | Buttons |
|---|---|
| **Report** — a building or base finished, research complete, a contact detected, a raid struck, the monthly report, a facility completed, a survey's findings, a story message, a stance changed | **OK** (or Enter / Esc) |
| **Interception** — your craft reached a contact | **Fly the battle**, **Break off** (Enter / Esc) |
| **Convoy ambushed** | **Fly the escort**, **Let it fight** (Enter / Esc) |
| **A decision** — a story choice | One button per option (or the keys 1 – 5) |
| **Mission offered** | **Fly it**, **Decline** (Enter / Esc) |
| **Title card** — a chapter opens | **Continue** (or a click anywhere) |
| **Victory** / **Defeat** — the end | **Leave** (saves and goes to the main menu) |

## Keys

The mouse does everything; the keys are shortcuts.

| Key | Does |
|---|---|
| Space | Pause, or resume at 1 day a second |
| 1 – 4 | Clock speed: 1 hour, 6 hours, 1 day or 5 days a second |
| Tab / Shift+Tab | Next / previous list |
| Up / Down | Choose a row |
| Enter | Do the selected row's first action |
| Left / Right | Change the current base |
| 1 – 5 | While a choice is offered: pick that option |
| Enter / Esc | In a prompt: the first / the last button |
| F5 | Save |
| Ctrl+M, Ctrl+K, Ctrl+B, Ctrl+J | Testbed cheats ([below](#testbed-cheats)) |
| Esc | Save and leave to the main menu |

Since app 0.16.0 there are no hidden keys: selling (formerly **S**) and fitting (formerly **F**)
are the *Sell 1* and *Fit best* buttons.

## The lists

| List | Rows | Buttons on the selected row |
|---|---|---|
| **Bases** | Each base: system, modules (and how many are building), what it researches. With a system selected that has no base: *Build a <type> at <system>* with its cost | *Select* (make it current); *Build* |
| **Research** | What the current base researches; every topic that can be researched now, with progress and size; the topics known | *Research* |
| **Build** | Modules the current base can build, with cost and build time; then its modules and their state (building, days left / working) | *Build* |
| **Market** | Items and hulls on sale, with prices and what you have | *Buy 1*, *Buy 5* (delivered to the current base after the delivery time), *Sell 1* (items, from the current base's store) |
| **Factory** | The current base's production rate; what it can make, with the work it takes; its queue | *Make 1*, *Make 5* |
| **Hangar** | Every craft with its base, pilot (and *away* when out) and fit (*standard fit* or the items fitted); then your pilots with rank, kills, missions and state; then recruits with their stats and the fee | On a craft: *Assign pilot* (the next free, ready pilot), *Fit best* (the best equipment in its base's store). On a recruit: *Hire* |
| **Contacts** | Detected enemy flights and where they head | *Intercept*: send the current base's craft that are at home and have a ready pilot |
| **Supply** | With a system selected: *Survey* it, and *Build* each kind of facility there (with the cost, and why not when it is not allowed). Then your facilities and what they hold or how long they take to build; on a working research station, the topics it can research; the current base's store of components; how many orders, convoys and survey ships are under way ([Supply](#supply)) | *Survey*; *Build*; *Start* (research at a station) |
| **Archive** | The rival's plans you know of, with how far each is built; every faction's stance and standing with you; the rival's aggression and the battles you have won; the encyclopaedia entries unlocked so far (app 0.10.0) | — |
| **Messages** | The log, newest first, with dates | — |

**Fit best** fills every slot of the craft's hull with the best item of that kind in its base's
store — beams, missile racks and, since app 0.15.0, **shields and armour**. In battle the craft
flies with the strongest shield fitted and all its armour plates added up, in place of its hull's
standard protection (*standard fit* means the hull's own).

## Interceptions and battles

1. A raid is detected: the clock stops with *Contact: <raid>, heading for <base>*.
2. Click the contact on the map and press **Intercept** in the info panel — or select it in the
   Contacts list and press *Intercept*. Your ready craft fly to it on the map — along the lanes
   to the system nearest it, then straight to it (app 0.7.3). Raiders, too, come in along the
   lanes, and the map draws each one's way ahead.
3. When they reach it, the clock stops and the **Interception** prompt asks: **Fly the battle**,
   or **Break off** (the contact flies on).
4. The battle is the raid's mission, with the first craft's hull as your ship and its fitted
   weapons, shields and armour (app 0.7.2, 0.15.0). Fly it as usual. When the result is decided the battle runs on until the
   debrief ([Debrief](getting-started.md#after-a-battle-the-debrief)); at the debrief, **Enter** or
   **Esc** returns to the map. Leaving from the pause menu (L or Q)
   counts as a retreat.
5. Back on the map, a report gives the result and the kills. The outcome's score and money
   apply; the leader gains the kills and experience; if you were shot down, the craft is lost and
   its pilot ejected injured — or was killed.

A raid you do not stop strikes its target: a base loses its newest finished module, and the
score drops.

## Supply

*Default campaign, app 0.9.0.* The Guard builds its craft from **components** — hull plating,
rocket motors, ray emitters — made from mined **resources**. When you order a hull on the
Factory list, the base orders the components it lacks; a **manufacturing center** makes them
from resources it orders in turn from your **mining facilities**; everything is carried by
automatic government **supply convoys** along the lanes. You do not move the convoys; you
decide where to survey and what to build.

1. **Survey.** Click a system on the map and press **Survey** in the info panel (or open the
   Supply list and choose *Survey <system> from <base>*). The system must be within the current base's radar range. The survey ship flies
   there (a sky-blue ring); when it arrives a message lists the deposits it found, with their
   rates a day, and any research site. A survey ship may be attacked on the way: it fights off
   the attack or is lost (then survey again).
2. **Build.** With a system selected, the info panel offers each kind of facility that may be
   built there as a button with its cost, and the Supply list offers every kind:
   - a **mining facility** needs a surveyed system with deposits;
   - a **manufacturing center** must be within two lanes of one of your bases;
   - a **research station** needs a surveyed system that is a research site.

   The row says why when it cannot be built. A facility takes days to build (a grey diamond on
   the map) and a message says when it is complete.
3. **Make.** Queue a hull on the Factory list as usual. Its row waits until the components have
   arrived; the Supply list shows the base's store and the orders and convoys under way.
4. **Research at a station.** A finished research station lists the station-only topics of
   its kind (*Research Theremin shielding here*); a topic is refused when the station is not on
   its research site or its prerequisites are not known. The station first orders the topic's
   start materials; once they arrive it researches on its own.

**Convoys can be ambushed** on the way. Since app 0.9.2 the clock stops with the **Convoy
ambushed** prompt, and the convoy waits:

- **Fly the escort** — fly its escort, *Convoy ambush*, with the first ready craft that has a pilot (from
  any base). Two cargo clippers, each with twelve cargo pods, make for their jump point about ten
  minutes away. Dominion grappler tugs come every couple of minutes, each with a squadron of
  saucers flying formation on it; a tug tears a pod off with its tractor beam, hauls it clear
  and jumps away with it, or takes a whole clipper once raiders have crippled it. Kill the tugs
  before they get away, keep the raiders off the clippers, and the clippers' own laser helps
  against missiles. The battle is won when the clippers have jumped with cargo aboard, lost if a
  clipper is captured or every pod is gone, and a draw after 15 minutes if a crippled clipper
  never makes its jump. Back on the map, the **share of the cargo still aboard** is delivered
  (a loss delivers nothing; leaving the battle early lets the convoy fight on alone).
- **Let it fight** — let it fight through alone: each vessel may be lost, a message says how many, and
  the survivors deliver their share. This is also what happens when no craft is ready.

What is missing is simply ordered again.

## The story

*App 0.10.0 – 0.10.3.* The story runs by itself as the days pass; you meet it as messages that
stop the clock, choices, offered missions and, finally, an ending.

- **Messages and news.** A story message stops the clock and shows in a report (a chapter of
  the default campaign's serial, an alien transmission in `interceptor`); news lines go into the
  log without stopping it.
- **Choices.** *A decision* lists the options as buttons; click one (or press its number). The
  clock does not run until you choose. What you choose changes money, relations, flags and
  sometimes offers a mission.
- **Offered missions.** *Mission offered* names it: **Fly it** flies it at once with the first
  ready craft that has a pilot; its result counts like a raid's battle
  (score, money, salvage, and the story flags it sets). With no ready craft it says so, and the
  offer is gone.
- **The rival's plan.** A rival builds something from shipments that cross the map as contacts:
  intercept them and win, and the shipment is lost; let one reach its destination and the plan
  grows. Messages warn you as it nears completion, and the Archive list shows how far it is.
  If it is completed, you lose.
- **Relations** (default campaign). Every faction has a standing with you from −100 to 100 and
  a stance — hostile, wary, neutral, friendly or allied. Story events and, since app 0.12.1, the
  missions you fly move them (winning the nebula-pocket ambush lowers the raiders' standing), and
  they drift back towards neutral each month. A change of stance from a story event stops the
  clock; one from a mission is only logged. A faction that is no
  longer **hostile** stops raiding you and does not attack you in battle; one that turns hostile
  does. The Senate's stipend grows with its standing.
- **The rival's mood.** Sit back and the rival grows bolder — its raids come more often; win
  battles every month and it grows more careful.
- **The end.** When an ending is reached a **Victory** or **Defeat** prompt shows its text;
  **Leave** saves and goes to the main menu. In the default campaign you win by destroying the Eclipse Engine and
  lose if it is lit, if you lose your last base, or if the Senate turns against you; in
  `interceptor` you win by destroying the Doomsday weapon and lose if it is completed or your
  last base falls.

## Films, cutscenes and music

*App 0.11.0 – 0.13.0.*

- **Title cards.** Each chapter of the default campaign's serial opens with a full-screen card —
  the chapter's title and its hook — and the clock stops. **Continue** (or Enter, Esc or a
  click) dismisses it; then the chapter's mission is offered as before.
- **Films.** In `interceptor`, each of the five alien transmissions plays the original's
  transmission film from your data, the victory plays the closing credits, and a new campaign
  opens with the original's opening film. The film fills the screen (letterboxed); **Enter**,
  **Esc** or a click skips it, and the map comes back where you left it.
- **Cutscenes.** The default campaign opens with a short scene played by the engine itself —
  a Guard patrol meets a saucer over the Gatehouse — with captions and black bars. **Esc** or
  **Enter** skips it.
- **Music.** Battles play the campaign's music: calm while no enemy is near, tense when one
  is within radar range, driving when one is close, changing at the end of a musical phrase.
  A win plays a victory sting, any other result a darker one. The strategy screen has its own
  music; the launcher and the other testbed screens are silent. Set the music and effects
  volumes in the launcher with **M** and **V** ([Settings](settings.md#volumes-in-the-launcher)).
- **Who is speaking.** In any battle, when someone speaks on the radio a small panel at the top
  left shows their portrait and name for a moment (app 0.13.1) — in the default campaign the
  Guard Wing and Colony Control; voices without a portrait show moving signal bars.
- **The end.** An ending with a film plays it first, then the **Victory** or **Defeat** prompt
  with its text. Continuing a finished campaign shows its ending again (app 0.12.1).

## Money and the monthly report

Incomes (the default campaign's Senate stipend grows with your bases, your month's score and,
since app 0.10.1, the Senate's standing with you)
and expenses are paid on their own schedules and logged. Every 30 days the **monthly report**
charges module maintenance, craft upkeep and pilot salaries, shows the month's score, the total
and your funds, and a new set of recruits is offered.

## Saves

- **Save** (top bar) or **F5** saves; **Menu** or **Esc** saves and leaves to the main menu. The save is `<user data folder>/saves/<campaign>.yaml`
  (for example `saves/default.yaml`); the user data folder is described on
  [Settings](settings.md).
- A save records the packs it was played with. Continuing refuses a save that needs a pack that
  is not active, or one whose rules no longer fit it (a mod was changed), with a message.

## Testbed cheats

The testbed keeps the original game's own cheat list as debug keys on the strategy screen (app
0.8.1). Each says so in the yellow feedback line.

| Key | Cheat | Does |
|---|---|---|
| Ctrl+M | *payday* | A million more in funds. |
| Ctrl+K | *knowitall* | Every research topic known; bases stop researching. |
| Ctrl+B | *quickbase* | Every base and module under construction finished now (not supply facilities). |
| Ctrl+J | *jump to battle* | With a contact selected on the map (or its row selected in the Contacts list): a ready craft with a pilot is put at it at once, and the **Interception** prompt appears. Otherwise it says *Select a contact first*, or *no ready craft with a pilot*. |

## Limits in app 0.16.0

- Fitting takes the best items in store; you cannot choose them one by one yet.
- Facilities have no fighters of their own, and the Supply list is plain text.
- The info panel shows at most four action buttons; a system where more could be built shows
  the first four (the Bases and Supply lists offer them all).
- There is no separate screen per list, no 3D base view and no report screen yet.
- Several chapters of the default campaign's serial fly an existing mission rather than their
  own; allied factions do not yet fly at your side; two of the original's seven side missions
  are in `interceptor`. Neither campaign has been played through from start to finish yet.
- The ending is a prompt with an optional film, not a screen of its own (Stage 6).
- Films and cutscenes cannot be skipped from the tablet console or the gamepad yet. (A film's
  sound follows the effects volume since app 0.14.1.)
- The `interceptor` campaign's strategy layer has not been play-tested; its research is slow
  with one downlink.
- Fixed in app 0.9.0: manufacturing now costs money as each unit starts, and the default
  campaign's salvage topics (*Saucer hull analysis*, *Robot brains*) can be started after
  salvaging the right raid.
