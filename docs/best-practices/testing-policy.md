# Testing policy

1. **All code has unit tests before its stage is complete.** Readers, rules, flight maths,
   weapons, targeting, AI decisions, bindings parsing — anything that can run without a window.
2. **Behaviour only a playtest can show** (how flight *feels*, whether the cockpit lines up,
   whether a gamepad works) gets **documented manual test cases** in
   [Manual tests](../testing/manual-tests.md), run and recorded before the stage is complete.
3. **Original data is never a test fixture in the repository.** Unit tests build their own
   inputs; checks against real data run through `--check` when the data is present.
4. **A bug fix comes with a test** that fails before the fix (or a manual case if it is
   playtest-only).
5. **The gate passes on every commit**: `scripts/check.sh`.

Design for testability: pure functions for maths, `World::step` independent of the window,
`Controls` as plain data so AI and input can be tested the same way.
