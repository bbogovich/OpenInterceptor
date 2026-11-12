---
spec_version: 1.14.0
status: current
updated: 2026-11-11
---

# Testbed & tools

The testbed that every stage ends in, the viewers, headless runs and the asset generators.

Requirements for these features: [Testbed & tools requirements](../specification/requirements/testbed-tools.md).

| Feature | Status |
|---|---|
| [F-TST-001 Testbed](#f-tst-001-testbed) | Implemented |
| [F-TST-002 Model viewer and asset browser](#f-tst-002-model-viewer-and-asset-browser) | Implemented |
| [F-TST-003 Headless runs](#f-tst-003-headless-runs) | Implemented |
| [F-TST-004 Debugging tools for missions and campaigns](#f-tst-004-debugging-tools-for-missions-and-campaigns) | In progress |
| [F-TST-005 Original-free asset generators](#f-tst-005-original-free-asset-generators) | Implemented |

## Features

### F-TST-001 Testbed

**What.** A launcher lists live test scenarios and missions for each campaign, so each component can be playtested on its own; every stage adds scenarios.

**Why.** Every stage ends in a playable build.

- **Status:** implemented
- **Source:** Plan item 1.11; project rules.
- **Specified by:** [R-TST-001](../specification/requirements/testbed-tools.md#r-tst-001-launcher), [R-TST-002](../specification/requirements/testbed-tools.md#r-tst-002-command-line), [R-TST-003](../specification/requirements/testbed-tools.md#r-tst-003-pause-menu), [R-TST-008](../specification/requirements/testbed-tools.md#r-tst-008-scenarios-per-stage)

### F-TST-002 Model viewer and asset browser

**What.** Every model, look, texture, image and sound can be inspected, with pack overrides marked.

**Why.** Modders and developers check assets without flying.

- **Status:** implemented
- **Source:** Plan items 1.11 and 1.18.
- **Specified by:** [R-TST-004](../specification/requirements/testbed-tools.md#r-tst-004-model-viewer), [R-TST-005](../specification/requirements/testbed-tools.md#r-tst-005-asset-browser)

### F-TST-003 Headless runs

**What.** Validation, balance soak runs and screenshots run without a player.

**Why.** Repeatable balance and regression checks.

- **Status:** implemented
- **Source:** Plan items 1.13 and 2.23; `--screenshot` (2.18).
- **Specified by:** [R-FLT-010](../specification/requirements/flight.md#r-flt-010-fixed-deterministic-tick), [R-TST-002](../specification/requirements/testbed-tools.md#r-tst-002-command-line), [R-TST-006](../specification/requirements/testbed-tools.md#r-tst-006-soak-runs), [R-TST-007](../specification/requirements/testbed-tools.md#r-tst-007-screenshots), [R-TST-011](../specification/requirements/testbed-tools.md#r-tst-011-campaign-soak)

### F-TST-004 Debugging tools for missions and campaigns

**What.** A mission browser, a terrain sandbox, a condition console, a strategy sandbox with a debug clock and cheats, and an event and arc debugger.

**Why.** Later stages need their own playtest tools.

- **Status:** in-progress — the mission browser is the simulator (app 0.6.3), and the simulator's environments with the surface scenarios serve as the terrain sandbox; the strategy screen has the original's cheats and jump to battle (app 0.8.1) and `--strategy-soak` plays a strategy layer headless (app 0.8.0); the condition console (Stage 3) and the event and arc tools (Stage 5) remain.
- **Source:** Testbed adds of Stages 3–5.
- **Specified by:** [R-TST-009](../specification/requirements/testbed-tools.md#r-tst-009-mission-and-condition-tools), [R-MSN-010](../specification/requirements/missions.md#r-msn-010-simulator-missions), [R-TST-010](../specification/requirements/testbed-tools.md#r-tst-010-strategy-and-campaign-tools)

### F-TST-005 Original-free asset generators

**What.** Tools generate the default campaign's models, textures and sounds.

**Why.** The default campaign must exist without original content.

- **Status:** implemented
- **Source:** Plan item 2.14.
- **Specified by:** [R-TST-012](../specification/requirements/testbed-tools.md#r-tst-012-asset-generators)

## Revision history

| Version | Date | Change |
|---|---|---|
| 1.0.0 | 2026-11-11 | Bootstrap: F-TST-001 – F-TST-005 (5 features) |
| 1.12.0 | 2026-11-11 | F-TST-004 in progress: the mission browser (the simulator, app 0.6.3) and the terrain sandbox; condition console and strategy tools remain |
| 1.14.0 | 2026-11-11 | F-TST-004 progress: strategy cheats and jump to battle (app 0.8.1); `--strategy-soak` (app 0.8.0) |
