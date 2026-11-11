# Campaigns (design)

Campaigns — the star map, factions and relations, economy, research, bases, pilots, the story
and the rival's plan — are designed in full in **[Game design and mod format](../definition/campaigns/index.md)**:

- [Mod structure](../specification/design/mod-structure.md): campaign, addon and library mods; `parent`.
- [Campaign and game flow](../specification/design/format-campaign.md): start state, difficulty, arcs, rival plans, score, endings.
- [Strategy layer](../specification/design/format-strategy.md): map, relations, economy, research, bases, ships, pilots, strategist.
- [Missions, environments and events](../specification/design/format-missions.md).

!!! note "Built so far"
    The strategy layer's first part is built (app 0.7.0 – 0.7.3): every key a mod can write today
    is on **[Strategy layer](strategy.md)**, with the default campaign as the example.

They are implemented in Stages 4 and 5 ([plan](../implementation/plan/index.md)). The
[default campaign](../definition/campaigns/default-campaign.md) will be the worked example for modders.
