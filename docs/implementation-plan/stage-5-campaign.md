# Stage 5 — Campaigns

**Version 0.6.0. Status: planned.** Story, rivals and relations: what turns the strategy layer
into a campaign with a beginning, middle and end. At the end of this stage **both campaigns are
playable from start to finish**.

Design: [campaign and game flow](../design/format-campaign.md),
[events](../design/format-missions.md#campaign-events-eventid),
[relations and strategist](../design/format-strategy.md).

## Work items

| # | Item | Module |
|---|---|---|
| 5.1 | Events: conditions, chances per check, actions, choices, offers (accept/refuse), cooldowns; news | `campaign::events` |
| 5.2 | Arcs: ordered steps, arc state in conditions | `campaign::events` |
| 5.3 | Rival plans: stages, budgets, shipments by convoy, visibility, grants, completion, warnings | `campaign::plan` |
| 5.4 | The strategist: budgets, aggression, exploration, expansion, outposts, offence ladder, defence priorities, tech levels, cheats, build priorities | `campaign::strategist` |
| 5.5 | Factions: patrons, independents, hostiles; **dynamic relations** with relation rules, drift, stances and their battle and strategy effects; stance-change events; relations screen | `campaign::relations`, `app` |
| 5.6 | Victory, defeat and endings (total/partial victories) | `campaign` |
| 5.7 | Encyclopaedia (functional) with unlock conditions | `app` |
| 5.8 | `interceptor`: the Doomsday arc (five messages and their gates), the Doomsday plan per difficulty, MacArthur and the final mission, the seven side missions, the alien strategist from `Think.ini` and the guide, news and report triggers | `packs/interceptor` |
| 5.9 | `default`: the 12-chapter serial arc with progress-based triggers, the Eclipse Engine plan, the Dominion strategist, dynamic relations among the six factions (Senate, Cartel, Prospectors, raiders), chapter title cards (plain), narrator text | `packs/default` |
| 5.10 | Fidelity: the [open items](../design/coverage.md#open-items-data-the-original-keeps-in-code) closed or recorded as *derived* | `packs/interceptor`, docs |
| 5.11 | **Rival logistics** (default campaign): the Dominion strategist builds mines and ships cosmium by convoy (the Eclipse Engine shipments use the same convoys), raids the Guard's supply lanes and strikes facilities under construction; third-party convoys (Prospectors, Cartel) with relation effects | `campaign::strategist`, `packs/default` |

**Testbed adds**: an event and arc debugger (fire events, set flags and plan progress, view
relations), and campaign start-at-chapter for the default campaign.

**Exit**: a full play-through of each campaign at a middle difficulty (manual tests, with a log
of milestone dates); automated soak test running each campaign for a simulated year with the
player's battles auto-resolved, checking invariants (money accounting, no orphan references,
arc progress monotonic).
