# Scripting (design)

Behaviour and logic are designed in **[Behaviours and scripting](../design/format-behaviour.md)**:

- **Tier 1** (Stages 2–3): behaviours as state machines, missions and events as definitions,
  and a small built-in condition and action language — enough for both shipped campaigns.
  **Implemented in Stage 2:** behaviours and the condition language in the object context —
  see [Behaviours and conditions](behaviours.md). Actions, and the mission and event contexts,
  come with Stage 3.
- **Tier 2** (decided at the start of Stage 3): optional scripts called through hooks, with Lua
  5.4 via `mlua` recommended, subject to the [dependency policy](../best-practices/dependencies.md).
