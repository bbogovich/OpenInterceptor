---
spec_version: 1.16.0
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
| [F-CPN-006 Story, rivals and relations](#f-cpn-006-story-rivals-and-relations) | In progress |
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

- **Status:** in-progress — rules, scenarios, nine missions (five using environments, hazards and ground installations, app 0.6.4), radio voices (app 0.6.2); the strategy layer and logistics (app 0.7.1 – 0.9.0); flown convoy battles (0.9.2); the 12-chapter serial with the Eclipse Engine plan, relations, endings and the finale `eclipse_assault` (0.10.1) — chapters 1 – 11 still fly existing missions; not played through.
- **Source:** Design page *The default campaign*; plan items 2.14, 3.12, 4.13, 5.9, 6.8, 7.8.
- **Specified by:** [R-STR-017](../../specification/requirements/strategy-logistics.md#r-str-017-supply-vessel-model), [R-CPN-003](../../specification/requirements/campaigns.md#r-cpn-003-default-campaign-plays-without-the-cd), [R-CPN-004](../../specification/requirements/campaigns.md#r-cpn-004-default-campaign-content), [R-TST-012](../../specification/requirements/testbed-tools.md#r-tst-012-asset-generators)

### F-CPN-004 The *Interceptor* campaign

**What.** The full *X-COM: Interceptor* campaign, played from the player's CD.

**Why.** Compatibility, and the primary test that the formats can express every system of the original.

- **Status:** in-progress — 26 mission types (app 0.6.5) and the strategy layer from the player's data, derived and not yet play-tested (app 0.8.0); the Doomsday arc and plan, MacArthur and the final mission, two of seven side missions (app 0.10.2, derived gates); the `Think.ini` strategist remains; not played through.
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

- **Status:** in-progress — the story layer is built (app 0.10.0 – 0.10.3): events with choices and mission offers, arcs, rival plans built from interceptable shipments, the player's dynamic relations (raids stop, battles follow the stances), the strategist's aggression, endings, the encyclopaedia; both campaigns' stories. Remaining: rival logistics and third-party convoys (5.11), allies in battle, relation rules and trade effects, the full strategist, a play-through.
- **Source:** Plan items 5.1–5.7 and 5.11.
- **Specified by:** [R-STR-027](../../specification/requirements/strategy-logistics.md#r-str-027-third-party-convoys), [R-CPN-007](../../specification/requirements/campaigns.md#r-cpn-007-events-and-arcs), [R-CPN-008](../../specification/requirements/campaigns.md#r-cpn-008-rival-plans-and-strategist), [R-CPN-009](../../specification/requirements/campaigns.md#r-cpn-009-dynamic-relations), [R-CPN-010](../../specification/requirements/campaigns.md#r-cpn-010-victory-defeat-and-endings), [R-TST-011](../../specification/requirements/testbed-tools.md#r-tst-011-campaign-soak)

### F-CPN-007 Presentation

**What.** Skinned screens and menus, cut-scene video, interactive music and voices.

**Why.** The campaigns look and sound finished.

- **Status:** in-progress — radio voices and the computer voice (R-MSN-007, app 0.6.2); interactive music, skins on the launcher and strategy screen, title cards, the original's Smacker films and the comms panel with speakers' faces (app 0.11.0 – 0.13.1). Remaining: screen layouts, menus, the X-COM wrapper screens and talking-head mapping, video for mods
- **Source:** Plan items 6.1–6.6.
- **Specified by:** [R-MSN-007](../../specification/requirements/missions.md#r-msn-007-voices), [R-CPN-011](../../specification/requirements/campaigns.md#r-cpn-011-screens-and-skins), [R-CPN-012](../../specification/requirements/campaigns.md#r-cpn-012-video-and-music)

## Revision history

| Version | Date | Change |
|---|---|---|
| 1.0.0 | 2026-11-11 | Bootstrap: F-CPN-001 – F-CPN-007 (7 features) |
| 1.12.0 | 2026-11-11 | F-CPN-003: nine default missions and radio voices noted. F-CPN-007: radio and computer voices built (R-MSN-007); the rest planned |
| 1.14.0 | 2026-11-11 | F-CPN-004: the strategy layer from the player's data (app 0.8.0) noted |
| 1.15.0 | 2026-11-11 | F-CPN-006 in progress (app 0.10.0 – 0.10.3: events, arcs, plans, relations, the strategist's aggression, endings, the encyclopaedia). F-CPN-003: convoy battles and the serial noted. F-CPN-004: the Doomsday arc noted |
| 1.16.0 | 2026-11-11 | F-CPN-007 in progress (app 0.11.0 – 0.13.1: music, skins, title cards, Smacker films, endings' films, the comms panel) |
