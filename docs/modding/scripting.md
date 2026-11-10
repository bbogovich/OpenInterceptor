# Scripting (design)

Behaviour and logic are designed in **[Behaviours and scripting](../design/format-behaviour.md)**:

- **Tier 1** (Stages 2–3): behaviours as state machines, missions and events as definitions,
  and a small built-in condition and action language — enough for both shipped campaigns.
- **Tier 2** (decided at the start of Stage 3): optional scripts called through hooks, with Lua
  5.4 via `mlua` recommended, subject to the [dependency policy](../best-practices/dependencies.md).
