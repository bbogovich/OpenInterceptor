---
spec_version: 1.0.0
status: current
updated: 2026-11-11
---

# Campaigns

The campaigns that ship with the engine, how one is chosen and played, and what turns battles into a story. Background on each campaign is in the pages beside this one.

Requirements for these features: [Campaigns requirements](../../specification/requirements/campaigns.md).

| Feature | Status |
|---|---|
| [F-CPN-001 Choose a campaign](#f-cpn-001-choose-a-campaign) | Implemented |
| [F-CPN-002 Difficulty levels](#f-cpn-002-difficulty-levels) | Implemented |
| [F-CPN-003 The default campaign, *Perils of the Void*](#f-cpn-003-the-default-campaign-perils-of-the-void) | In progress |
| [F-CPN-004 The *Interceptor* campaign](#f-cpn-004-the-interceptor-campaign) | In progress |
| [F-CPN-005 *Interceptor Remastered*](#f-cpn-005-interceptor-remastered) | Planned |
| [F-CPN-006 Story, rivals and relations](#f-cpn-006-story-rivals-and-relations) | Planned |
| [F-CPN-007 Presentation](#f-cpn-007-presentation) | Planned |

## Features

### F-CPN-001 Choose a campaign

**What.** One campaign is played at a time, chosen on the command line, in the settings or in the launcher; its addons come with it.

**Why.** Modding first: a campaign is the unit a modder ships.

- **Status:** implemented
- **Source:** Plan item 2.1.
- **Specified by:** [R-CPN-001](../../specification/requirements/campaigns.md#r-cpn-001-one-campaign-at-a-time)

### F-CPN-002 Difficulty levels

**What.** Each campaign defines its difficulty levels; the player picks one, and it is remembered per campaign.

**Why.** The original has five levels; the default campaign has four.

- **Status:** implemented
- **Source:** Plan items 2.8 and 2.23.
- **Specified by:** [R-CPN-002](../../specification/requirements/campaigns.md#r-cpn-002-difficulty-levels)

### F-CPN-003 The default campaign, *Perils of the Void*

**What.** A complete, smaller campaign made only from content made for this project, playable with no CD: rockets against saucers in a 1950s film serial.

**Why.** Modding first: proves the engine stands alone and is the worked example for modders.

- **Status:** in-progress
- **Source:** Design page *The default campaign*; plan items 2.14, 3.12, 4.13, 5.9, 6.8, 7.8.
- **Specified by:** [R-STR-017](../../specification/requirements/strategy-logistics.md#r-str-017-supply-vessel-model), [R-CPN-003](../../specification/requirements/campaigns.md#r-cpn-003-default-campaign-plays-without-the-cd), [R-CPN-004](../../specification/requirements/campaigns.md#r-cpn-004-default-campaign-content), [R-TST-012](../../specification/requirements/testbed-tools.md#r-tst-012-asset-generators)

### F-CPN-004 The *Interceptor* campaign

**What.** The full *X-COM: Interceptor* campaign, played from the player's CD.

**Why.** Compatibility, and the primary test that the formats can express every system of the original.

- **Status:** in-progress
- **Source:** Design page *The Interceptor campaign mod*; plan items 2.13, 3.11, 4.12, 5.8, 6.7.
- **Specified by:** [R-CPN-005](../../specification/requirements/campaigns.md#r-cpn-005-interceptor-campaign-content)

### F-CPN-005 *Interceptor Remastered*

**What.** A campaign that extends *Interceptor*, with sharper textures and an upgraded fighter model generated from the player's data.

**Why.** Proves a mod can extend another campaign and override individual assets.

- **Status:** planned
- **Source:** Design page *Interceptor Remastered*; plan item 7.7.
- **Specified by:** [R-CPN-006](../../specification/requirements/campaigns.md#r-cpn-006-interceptor-remastered)

### F-CPN-006 Story, rivals and relations

**What.** Events, story arcs, a rival with its own plans and strategist, factions whose relations change, and endings.

**Why.** What turns the strategy layer into a campaign with a beginning, middle and end.

- **Status:** planned
- **Source:** Plan items 5.1–5.7 and 5.11.
- **Specified by:** [R-STR-027](../../specification/requirements/strategy-logistics.md#r-str-027-third-party-convoys), [R-CPN-007](../../specification/requirements/campaigns.md#r-cpn-007-events-and-arcs), [R-CPN-008](../../specification/requirements/campaigns.md#r-cpn-008-rival-plans-and-strategist), [R-CPN-009](../../specification/requirements/campaigns.md#r-cpn-009-dynamic-relations), [R-CPN-010](../../specification/requirements/campaigns.md#r-cpn-010-victory-defeat-and-endings), [R-TST-011](../../specification/requirements/testbed-tools.md#r-tst-011-campaign-soak)

### F-CPN-007 Presentation

**What.** Skinned screens and menus, cut-scene video, interactive music and voices.

**Why.** The campaigns look and sound finished.

- **Status:** planned
- **Source:** Plan items 6.1–6.6.
- **Specified by:** [R-MSN-007](../../specification/requirements/missions.md#r-msn-007-voices), [R-CPN-011](../../specification/requirements/campaigns.md#r-cpn-011-screens-and-skins), [R-CPN-012](../../specification/requirements/campaigns.md#r-cpn-012-video-and-music)

## Revision history

| Version | Date | Change |
|---|---|---|
| 1.0.0 | 2026-11-11 | Bootstrap: F-CPN-001 – F-CPN-007 (7 features) |
