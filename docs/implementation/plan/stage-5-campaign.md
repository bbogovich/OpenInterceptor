# Stage 5 — Campaigns

**Version 0.6.0. Status: in progress** — the core (events, arcs, rival plans, relations, the
strategist, endings, the encyclopaedia) in app **0.10.0** (main `485d01e`); the default
campaign's serial in **0.10.1** (`66d0b5a`); the X-COM Doomsday arc in **0.10.2** (`8926d11`);
relations in battle and the year-long soak in **0.10.3** (`76bda5c`). Story, rivals and
relations: what turns the strategy layer into a campaign with a beginning, middle and end. At
the end of this stage **both campaigns are playable from start to finish**.

Design: [campaign and game flow](../../specification/design/format-campaign.md),
[events](../../specification/design/format-missions.md#campaign-events-eventid),
[relations and strategist](../../specification/design/format-strategy.md). As built:
[Story layer](../../modding/story.md), [Architecture › Campaign › The story](../architecture/campaign.md#the-story-app-0100).

## Work items

| # | Item | Module | Status |
|---|---|---|---|
| 5.1 | Events: conditions, chances per check, actions, choices, offers (accept/refuse), cooldowns; news | `campaign::events` | **Done** (0.10.0; built as `content::story`, `campaign::story`): `when`, `check` (daily, weekly, monthly, after_battle, on_research), `chance`, `do`, `choices`, `once`, `cooldown`; `mission(x)` offers (Enter flies, Esc declines); `news(…)`. The design's offer expiry and `reward:` are not built (DEV-051). GitHub #38. |
| 5.2 | Arcs: ordered steps, arc state in conditions | `campaign::events` | **Done** (0.10.0): `arcs.<id>.steps`, `arc(x)`, `fired(x)`. #38. |
| 5.3 | Rival plans: stages, budgets, shipments by convoy, visibility, grants, completion, warnings | `campaign::plan` | **Partial** (0.10.0): stages, `start_when`, shipments rolled `every`/`chance` and flown as a raid's contact (one that arrives adds a component; one intercepted adds nothing), warnings, completion, `plan(x)`. Not built: budgets, grants if missed, `plan_stage(x)`; visibility is the shipment's `chance` (DEV-051). #39. |
| 5.4 | The strategist: budgets, aggression, exploration, expansion, outposts, offence ladder, defence priorities, tech levels, cheats, build priorities | `campaign::strategist` | **Partial** (0.10.0): **aggression** only — it scales every raid's chance, rising while the player is passive and falling while the player wins. Not built: budgets, exploration, expansion, outposts, the offence ladder, defence priorities, tech levels, cheats, build priorities, the `Think.ini` values. #39. |
| 5.5 | Factions: patrons, independents, hostiles; **dynamic relations** with relation rules, drift, stances and their battle and strategy effects; stance-change events; relations screen | `campaign::relations`, `app` | **Partial** (0.10.0, 0.10.3): the player's standings, `relation(f, n)`, monthly drift, stances; a faction no longer hostile stops raiding (0.10.0) and campaign battles carry the stances (0.10.3, `flight::apply_stances`); a stance change is a notice; the Archive tab lists standings. Not built: relation rules, standings between other factions, allied factions fighting at the player's side, effects on prices and trade, stance-change events a campaign can react to, mission outcomes' `relation(…)`. #40. |
| 5.6 | Victory, defeat and endings (total/partial victories) | `campaign` | **Done for the engine** (0.10.0): `endings`, `campaign_end` victory and defeat conditions (defeats first), `victory(x)`/`defeat(x)` actions; the ending shown as a banner, saved. Neither campaign defines a partial victory yet; the ending screen is Stage 6. #41. |
| 5.7 | Encyclopaedia (functional) with unlock conditions | `app` | **Done** (0.10.0): `encyclopaedia.<id>` with `unlock`, listed on the Archive tab. #42. |
| 5.8 | `interceptor`: the Doomsday arc (five messages and their gates), the Doomsday plan per difficulty, MacArthur and the final mission, the seven side missions, the alien strategist from `Think.ini` and the guide, news and report triggers | `packs/interceptor` | **Partial** (0.10.2): the arc (five messages with the guide's date, research and plan gates, MacArthur, the final mission), the plan by difficulty with its convoys and three warnings, endings using the original's own text keys, a strategist with `DERIVED` values; **two of the seven side missions**. Not built: the other five, the `Think.ini` strategist, report triggers. The saucer-examination topics in the gates are *derived* (unconfirmed). Not play-tested. #43. |
| 5.9 | `default`: the 12-chapter serial arc with progress-based triggers, the Eclipse Engine plan, the Dominion strategist, dynamic relations among the six factions (Senate, Cartel, Prospectors, raiders), chapter title cards (plain), narrator text | `resources/default` | **Partial** (0.10.1): the twelve chapters as messages with progress-based triggers, the Cartel's choice, side events, the Eclipse Engine plan, dynamic relations (the Senate stipend follows its standing), the strategist, four endings, the encyclopaedia, the finale `eclipse_assault`. **Chapters whose bespoke special mission is not built fly the nearest existing mission.** Relations are the player's with each faction, not among all six. Not play-tested. #44. |
| 5.10 | Fidelity: the [open items](../../specification/design/coverage.md#open-items-data-the-original-keeps-in-code) closed or recorded as *derived* | `packs/interceptor`, docs | **Not started** beyond the story's own `DERIVED` marks. #45. |
| 5.11 | **Rival logistics** (default campaign): the Dominion's mines, cosmium shipments (the Eclipse Engine's convoys) and **supply depots** storing captured cargo (the player seizes a destroyed depot's stock); raids on the Guard's lanes and strikes on facilities under construction; **third-party convoys**: Prospector expeditions (to a system under investigation, pause, home) and Cartel runs to random destinations, with relation effects | `campaign::strategist`, `resources/default` | **Not built.** The Eclipse Engine's shipments exist as plan contacts (5.3) and the Guard's own convoys can be ambushed and escorted (app 0.9.2, Stage 4.15); the Dominion's mines and depots, seizing them, lane raids by the rival's logistics, construction strikes and third-party convoys are not. #46. |

**Testbed adds**: an event and arc debugger (fire events, set flags and plan progress, view
relations), and campaign start-at-chapter for the default campaign — **not built**. Built
instead: `--strategy-soak` reports the story (arcs, plans, aggression, ending; 0.10.1) and the
Archive tab shows plans, standings and aggression. #47.

**Exit**: a full play-through of each campaign at a middle difficulty (manual tests, with a log
of milestone dates); automated soak test running each campaign for a simulated year with the
player's battles auto-resolved, checking invariants (money accounting, no orphan references,
arc progress monotonic).

!!! note "Exit status at app 0.10.3"
    The **automated soak is built for the default campaign**:
    `campaign::tests::the_default_campaign_runs_a_year_with_its_story_and_stays_consistent` plays a
    year a month at a time (`campaign::autoplay_from`), checking every reference, that money and
    score stay finite and that arcs only move forward, and that the serial and the Engine
    progress. Money accounting is checked only as finite. The `interceptor` story needs the
    player's data, so it has no unit test; `--strategy-soak 365 --campaign interceptor` runs it
    by hand. **No play-through of either campaign has been done yet** (MT-5-01 – MT-5-10).
