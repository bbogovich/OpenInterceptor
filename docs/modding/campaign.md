# Tech tree and campaign (planned)

Stage 4 turns the strategy layer into data:

- **Tech tree**: research topics as `[research:<id>]` rules with prerequisites, cost and what
  they unlock — the original's `R_*` sections imported the same way ships are.
- **Star map**: systems, jump lanes, sensor ranges.
- **Bases and modules**: module rules (the original's `M_*` sections) with effects.
- **Economy**: income sources, maintenance, manufacturing.
- **Enemy strategy**: the alien planner's budgets (`Think.ini`) as rules plus scripts.
- **Campaign script**: start state, story triggers, victory and defeat.

A custom campaign is then a pack with its own map, tree, factions and missions, and no original
assets.
