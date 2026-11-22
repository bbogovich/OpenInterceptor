# X-COM campaign parity: the consolidated work plan (2026-11-22)

This merges four analyses of the player's own `Interceptor.exe` (facts only, no original content):

| Source | What it holds |
|---|---|
| [`audit-2026-11-22.md`](audit-2026-11-22.md) | D-01…D-43: where the engine's `interceptor` campaign differs from the RE notes and the INI today |
| [`flight/requirements.md`](flight/requirements.md) | RQ-C-01…21: stations, turrets, pilot stats in battle, special missiles, auto-resolve (facts in `flight/README.md` §8–§14) |
| [`strategy/requirements.md`](strategy/requirements.md) | RQ-S-01…20: research, economy, outposts, bases, pilots, aliens, side missions (facts in `strategy/README.md`) |
| [`wrappers/requirements.md`](wrappers/requirements.md) | RQ-UI-01…17: sprites, bitmap fonts, text anchoring, widgets, lists, dialogs, navigation (facts in `wrappers/ui-framework.md`) |

Rules for every item: values are pack data (absent keys keep today's behaviour), the `default` campaign gets a
counterpart or stays unaffected, unit tests with synthetic data, modding docs in the same change.

## Done

| Item | App |
|---|---|
| Release bundle finds the player's data in the user data folder; clearer message when it doesn't | 0.22.1 |
| Cockpit ship-status bands lie on the art's painted rings (playtest) | 0.22.2 |
| D-01 / RQ-S-01: tech ids from `GameSettings.ini`'s numbered topic list (was a databank table, off by one) | 0.22.3 |
| RQ-S-02 (part): research progress belongs to the topic (`switching: keep_progress`) | 0.22.3 |
| #113 RQ-C-01 = D-16 (station hull is HP1), RQ-C-02/03 (one shield pool by generator type, covering the modules), RQ-C-04 (`damage_model: station`) | 0.22.4 |
| #116 D-02 (repair order, hull, 2/4 per s, 0 is final), D-05 (system table, mounts and launchers), D-06 (per-system efficiency), RQ-C-16 (engine fails below 15 %), RQ-C-17 (sensor/hyperdrive damage inert); D-08 deferred to #133 | 0.22.6 |
| #113 follow-up: the pool drops to 0 when the last generator falls; outposts/OPPs pooled by their chart's SD (unknowns-2026-11 §8) | 0.22.6 |
| #118 D-19 (AI speed in m/s), D-20 (no AI afterburner), D-21 (original steering), D-22 (AI's own missile lock), D-23 (sprint, 250 m break-off; weapon choice and wobble open), RQ-C-10 (pilot Accuracy/Intelligence via records; campaign hand-off → #133), RQ-C-23 (grades: look-ahead, sprint) | NEXT |

## Issues

Filed 2026-11-22 as GitHub issues (labels `parity-m1`, `parity-m2`, `parity-m3`, `parity-re`):

| Track | Issues |
|---|---|
| M1 combat | #113 station hull/shields, #114 alien base hard points and layouts, #115 turret fire control, #116 repair and system damage, #117 shield types and special weapons, #118 AI piloting, #119 AI retreat and Nova, #120 missiles/cloak/PD/hulls, #121 balance report and re-measure |
| M2 screens | #122 sheet fonts and anchors, #123 CSD readouts and captions, #124 compact lists, #125 sprites and buttons, #126 sounds/tooltips/dial/lamps, #127 dialogs/history/clock, #128 screen content |
| M3 strategy | #129 research, #130 sensors, #131 outposts, #132 bases, #133 pilots, #134 auto-resolve, #135 aliens, #137 deliveries and saves |
| RE | #136 remaining unknowns |

## Order of work

Ordered by what a player meets first and what blocks other work. Track A (combat) and Track B (screens) touch
different layers and can run in parallel.

### M1: combat feel (0.23.x)

1. **Stations** RQ-C-01 = D-16 (hull is HP1 only), RQ-C-02/03 (one shield pool by generator type, protects
   modules), RQ-C-04, RQ-C-05 (alien base hard points), RQ-C-06 (turret fire control: 5 km, free fire, full lead).
2. **Repair and damage** D-02 (hull repair 2/s, 4 with a damage control computer; systems never return from 0),
   D-05/D-06 (system damage table and thresholds), D-03 (accumulator shields), D-04 (Tachyon Pulser), D-07,
   D-11, RQ-C-16 (engine fails below 15 %).
3. **AI** D-19 (speed requests), D-20 (afterburner energy without thrust), D-21 (steering law), D-22 (missile
   lock window), RQ-C-10 (pilot records drive skill), RQ-C-11 (retreat when disarmed).
4. **Ships and missiles** D-17 (pirate/corporate hulls from their own sections), D-13 (launcher rounds and rate),
   D-12, D-09 (no automatic point defence), D-10 (cloak), D-15, D-18.
5. Re-measure with `--balance` (fix D-31 first so it honours difficulty and energy); D-42 should close.

### M2: text and widgets on the original screens (0.24.x), the playtest's screen-text finding

1. RQ-UI-03/04: bitmap sheet fonts from SpritesDLL and the exe's anchor rules (glyph top = y − 5/7, single lines
   centred in their box); a generated sheet font for `default`.
2. RQ-UI-05/06: CSD readouts exactly (font 23010, centred on x 561, y 370/390/410/430: time, weekday, date,
   funds); captions placed from their control's rect. Replaces `draw_panel`'s fixed layout (D-32…D-41).
3. RQ-UI-13: compact list style (row pitch = line height + 2, visible rows, "...", pulsing bar).
4. RQ-UI-01/02 sprites and button states, RQ-UI-07…12 widget behaviour, RQ-UI-14 dialog frames,
   RQ-UI-15/16 history and clock rules.
5. D-28 star names, D-29 negative zero, D-26 UFOpedia, D-27 six reports, D-30 Bases/Equip Ships art areas.

### M3: strategy logic (0.25.x)

1. RQ-S-03 artefact-gated topics (127), research score by Level (RQ-S-02 rest).
2. RQ-S-07 sensor ranges (⅓, ⅔, 1 light year; the engine is 3-4× too far).
3. RQ-S-04/05/06 outposts as objects: monthly income, funding drift, withdrawal; scores by system.
4. RQ-S-08 base price/time/placement, RQ-S-09 technician manufacturing, RQ-S-10/11 pilots.
5. RQ-C-18/19/20 auto-resolve as the hourly simulation, with damage persisting on craft (D-24's craft losses).
6. RQ-S-12/13/15 alien messages, side missions, MacArthur; RQ-S-14 alien treasury; RQ-S-17/18 reports, dialogs.

### Still unknown (more RE or reference play)

Settled on 2026-11-22 (#136) in [`unknowns-2026-11.md`](unknowns-2026-11.md): the CSD map (RQ-UI-18…22),
deliveries and strategic speed (RQ-S-21…23), the alien raid planner and base growth (RQ-S-24…26), the
mission rating (RQ-S-27), the pilot "temperament" = pilot grade (RQ-C-23), gimballed turret aim (RQ-C-22),
the save layout (RQ-S-28), station/outpost shield pools (RQ-C-24, for #113).

What truly remains: `MissionScores` counters 36–90 and 95–104 (objective events) and the debrief field
+0x30c; raid fleet sizes per target type and difficulty (two jump tables in `0x573998`); the alien base
variant A/B flag; alien ship build time; Doomsday piece costs; the two small ints per outpost/OPP chart;
the byte layout inside each save chunk (not needed for our own format).
