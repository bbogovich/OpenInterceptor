# Scripting (design)

Behaviour and logic are designed in **[Behaviours and scripting](../specification/design/format-behaviour.md)**:

- **Tier 1** (Stages 2–3): behaviours as state machines, missions and events as definitions,
  and a small built-in condition and action language — enough for both shipped campaigns.
  **Implemented in Stage 2:** behaviours and the condition language in the object context —
  see [Behaviours and conditions](behaviours.md). Actions, and the mission and event contexts,
  come with Stage 3.
- **Tier 2** (Stage 3.10, **awaiting the user's decision**): optional scripts called through
  hooks. The [evaluation](../learning/best-practices/dependencies.md#tier-2-scripting-evaluated-stage-310-awaiting-the-users-decision)
  compares Lua 5.4 via `mlua`, `rhai`, and growing the condition and action language into a small
  statement language (variables, `if`/`else`, bounded loops, functions); it recommends the last
  as the default, with Lua as the fallback. Until the decision, write Tier 1.
