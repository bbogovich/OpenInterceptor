# Manual tests

Playtest cases for behaviour that unit tests cannot show (what is drawn, how it feels, sound,
real devices). Each stage's cases must be run and recorded as passed before the stage is
complete. Record results in the table (date, tester, platform, result, notes); keep earlier
rows when re-running a case.

**How to run**: `cargo build --release`, then `target/release/openinterceptor --data game`
(the folder with your *Interceptor* CD contents). Every case names its launcher entry; to jump
straight in use `--scenario <id> --ship <id>` (and `--view chase` for an outside view). Ship ids:
`lightning`, `firestar`, `avenger`. Keys are the defaults on [Controls](../original-game/controls.md).

## Stage 1 results

| ID | Case | Scenario | Result | Date | Tester / platform | Notes |
|---|---|---|---|---|---|---|
| MT-1-01 | Launcher lists scenarios | Launcher | partial | 2026-11-10 | developer / macOS | Seen with data; `--no-original` run seen in sample arena |
| MT-1-02 | Ship selection: all three hulls | `flight-test` | partial | 2026-11-10 | developer / macOS | Preview seen; Lightning, Firestar, Avenger flown |
| MT-1-03 | Model viewer | Model viewer | partial | 2026-11-10 | developer / macOS | Browsing and zoom checked |
| MT-1-04 | Keyboard flight: attitude | `flight-test` | partial | 2026-11-10 | developer / macOS | Yaw checked |
| MT-1-05 | Keyboard flight: throttle and afterburner | `flight-test` | partial | 2026-11-10 | developer / macOS | Full throttle checked |
| MT-1-06 | Mouse flight | `flight-test` | not run | | | |
| MT-1-07 | Gamepad flight | `flight-test` | not run | | | Needs a physical controller |
| MT-1-08 | Cockpit per hull | `flight-test` | partial | 2026-11-10 | developer / macOS | A and C seen |
| MT-1-09 | Views F1–F8 | `flight-test` | partial | 2026-11-10 | developer / macOS | F1, F7 seen |
| MT-1-10 | Targeting, brackets and target MFD | `gunnery-range` | partial | 2026-11-10 | developer / macOS | R, G, MFD, off-screen arrow seen |
| MT-1-11 | Beams, energy and link modes | `gunnery-range` | partial | 2026-11-10 | developer / macOS | Beams and energy drain seen |
| MT-1-12 | Missile lock and hit | `gunnery-range` | partial | 2026-11-10 | developer / macOS | Lock seen |
| MT-1-13 | Destroying enemies, debris | `flight-test` | not run | | | |
| MT-1-14 | Wingmen formation and orders | `flight-test` | not run | | | |
| MT-1-15 | Station collision | `station-tour` | not run | | | |
| MT-1-16 | Hostile AI | `combat-test` | partial | 2026-11-10 | developer / macOS | Hostile fire seen in cockpit C |
| MT-1-17 | Nav points | `flight-test` | not run | | | |
| MT-1-18 | Sound effects | `gunnery-range` | not run | | | |
| MT-1-19 | Original-free sample pack | `sample-arena` | partial | 2026-11-10 | developer / macOS | Runs without data |
| MT-1-20 | Pause, menu, restart, death | any | not run | | | |
| MT-1-21 | Linux build and run | `flight-test` | not run | | | Type-checks for Linux; not yet run on Linux |

"partial" = seen working during development; the full steps below still need a recorded pass.

## Stage 1 cases

### MT-1-01 — Launcher lists scenarios

| Field | |
|---|---|
| Scenario | Launcher |
| Preconditions | Original data available. |
| Steps | 1. Run `openinterceptor --data game`.<br>2. Look at the list; move with Up/Down; read each description.<br>3. Quit (Esc) and run `openinterceptor --no-original`. |
| Expected | With data: Model viewer, Flight test: station patrol, Combat test: hostile arena, Gunnery range, Station tour, Sample pack arena (no original data). The packs line names both packs. With `--no-original`: Model viewer and the Sample pack arena only, and an orange note that the Interceptor pack was skipped. Clicking a line selects it; clicking it again starts it. |

### MT-1-02 — Ship selection: all three hulls

| Field | |
|---|---|
| Scenario | Flight test: station patrol |
| Steps | 1. Select the scenario.<br>2. Press Left/Right repeatedly.<br>3. For each hull press Enter, fly briefly, Esc → L back. |
| Expected | A turning model, name, description and stats for X-1A Lightning II, S-2 Firestar and M-3 Super Avenger, wrapping round. Each starts the scenario in that hull (the MFD Status page shows its name). Scenarios with a fixed ship (Sample arena) show no choice. |

### MT-1-03 — Model viewer

| Field | |
|---|---|
| Scenario | Model viewer |
| Steps | 1. Open it. 2. PgDn/PgUp (or N/B) through the list. 3. Arrows or mouse drag to orbit; +/− or wheel to zoom. 4. L several times. 5. Space. 6. Esc. |
| Expected | Ship classes first (name and id), then every original mesh (`mesh NAME`). Each model is framed fully and textured; zoom never enters the model and does not run away. L cycles auto → LOD 0 → 1 → 2 → 3 → auto with visibly simpler models and the info line showing the forced LOD. Attachment points show as small RGB axes. Space stops/starts the spin. Esc returns to the launcher. |

### MT-1-04 — Keyboard flight: attitude

| Field | |
|---|---|
| Scenario | `flight-test`, Lightning II |
| Steps | 1. Press Up, then Down. 2. Left, Right. 3. Kp7/Q and Kp9/E. 4. Repeat at full throttle. |
| Expected | Up lowers the nose, Down raises it; Left/Right turn; Kp7/Q roll left, Kp9/E roll right. Response builds up smoothly and stops smoothly. At full throttle pitch and yaw are noticeably slower than at rest (roll is not). The Avenger turns more slowly than the Lightning. |

### MT-1-05 — Keyboard flight: throttle and afterburner

| Field | |
|---|---|
| Scenario | `flight-test`, any hull |
| Steps | 1. Press `=` (or keypad +) three times, `-` once. 2. Backspace. 3. Hold Tab. 4. `\`. 5. Z, then Backspace. 6. Target a moving drone (R) and press M. |
| Expected | THR reads 30 %, 20 %, then 100 %; speed climbs to the hull's MaxSpeed (600 m/s) along the nose. Tab: BURN shown, speed above max, energy gauge falls; released or out of energy, it stops. `\` decelerates to 0. Z shows REV and the ship backs up at half the throttle speed. M sets the throttle to the target's speed. A single key press always changes the throttle exactly once. |

### MT-1-06 — Mouse flight

| Field | |
|---|---|
| Scenario | `flight-test` |
| Steps | 1. Press C. 2. Move the cursor up, down, left, right of centre. 3. Left button, right button. 4. Press C again. |
| Expected | The cursor hides and is confined; the caption says "mouse flight (C)". Cursor **down = nose down**, up = nose up, left/right turn, proportional to the distance from centre. Left button fires beams; right button fires a missile. C again restores the cursor. |

### MT-1-07 — Gamepad flight

| Field | |
|---|---|
| Scenario | `flight-test`; Xbox/PlayStation-style controller connected before starting |
| Steps | Use the left stick, right stick, triggers, shoulders, A/B/X/Y, D-pad, Back, Start; in the launcher use the D-pad and A. |
| Expected | Per [Controls → gamepad](../original-game/controls.md#openinterceptor-gamepad-defaults): left stick forward = nose down, sideways = yaw; right stick = roll and vertical slide (VCT hulls); RT beams, LT missile; RB/LB next/previous target; A afterburner; B nearest enemy; X match speed; Y chase view; D-pad throttle and MFD pages; Back link mode; Start pause. Small stick movements inside the dead zone do nothing. |

### MT-1-08 — Cockpit per hull

| Field | |
|---|---|
| Scenario | `flight-test` |
| Steps | Start once with each hull; stay in F1. |
| Expected | Lightning II → cockpit A, Firestar → B, Avenger → C, centred and scaled to the window height with no cyan fringe. Both radar scopes, the MFD, the four gauges (speed, energy, shields, hull) and the message line sit inside the art's instrument faces. |

### MT-1-09 — Views F1–F8

| Field | |
|---|---|
| Scenario | `flight-test` |
| Steps | Press F1 … F8; in F8 use Insert/Delete, PageUp/PageDown, Home/End, Kp5. |
| Expected | F1 cockpit art; F2 full forward with glass instruments; F3/F4 look left/right; F5/F6 rear-left/rear-right; F7 chase camera behind and above showing the ship with its engine glow; F8 orbit camera that moves with the keys and resets with Kp5. The caption names the view. Target brackets show only in F1, F2 and F7. |

### MT-1-10 — Targeting, brackets and target MFD

| Field | |
|---|---|
| Scenario | `gunnery-range` |
| Steps | 1. T several times, Y. 2. R. 3. Point the nose at a drone, G. 4. Turn away so the target leaves the screen. 5. `.` and `,` to cycle MFD pages. |
| Expected | Each target change shows "TARGET: NAME" and a click; brackets in the faction colour sized to the target, with name and range and a lead circle. R picks the nearest hostile; G picks the one nearest the reticle. Off screen, an arrow on a ring points to it. MFD pages: Target (name, faction, range, speed, shields, armour, hull, lock state), Weapons, Status, Nav. On the radar the target is boxed. |

### MT-1-11 — Beams, energy and link modes

| Field | |
|---|---|
| Scenario | `gunnery-range`, Firestar (four beams) |
| Steps | 1. Hold Enter (or LeftControl) at a drone. 2. Press X three times, firing in each mode. 3. Fire until energy runs out. |
| Expected | Coloured bolts leave the guns with sound and travel to their range. "WEAPONS FULL", "WEAPONS UNLINKED", "WEAPONS LINKED" messages; Linked fires pairs, Full all mounts, Unlinked one at a time. The energy gauge falls while firing and refills when not. At low energy "INSUFFICIENT POWER" and a denied sound; firing resumes once energy recovers. Hits flash the target and show sparks (blue on shields, orange on hull). |

### MT-1-12 — Missile lock and hit

| Field | |
|---|---|
| Scenario | `gunnery-range` |
| Steps | 1. R, keep the drone near the reticle. 2. Watch the lock ring. 3. Space when locked. 4. W to change missile type; fire one without a lock. |
| Expected | A red ring closes in on the target, then a red diamond with "LOCK" and "MISSILE LOCK" plus a lock tone; the MFD shows "locking", then "LOCKED". A locked missile homes in and explodes on the drone. W moves the `>` marker on the Weapons page. An unlocked missile flies straight and expires after its duration. Rack counts go down; an empty rack reports "NO MISSILES". |

### MT-1-13 — Destroying enemies, debris

| Field | |
|---|---|
| Scenario | `flight-test` |
| Steps | Destroy one of each kind of enemy (beams and missiles). |
| Expected | Shields, then armour, then hull fall on the target MFD. At zero hull: an explosion (bigger for bigger ships), a sound, tumbling debris chunks of that ship that fade after about 12 s, "NAME DESTROYED", and the target is cleared. |

### MT-1-14 — Wingmen formation and orders

| Field | |
|---|---|
| Scenario | `flight-test` (two Lightning wingmen, alpha2/alpha3) |
| Steps | 1. Fly turns at several throttles; F7 to watch. 2. Shift+F near the aliens. 3. Shift+I. 4. Target an enemy, Shift+A. |
| Expected | Wingmen hold slots beside and slightly behind the player, match speed and catch up with afterburner when far. Messages "WINGMEN: FIRE AT WILL", "WINGMEN: FORM ON ME", "WINGMEN: ATTACK MY TARGET"; they attack accordingly and return to formation. |

### MT-1-15 — Station collision

| Field | |
|---|---|
| Scenario | `station-tour` |
| Steps | Fly past each station; then fly slowly into the X-COM base's arms and hull. |
| Expected | Every station type renders textured with plausible LOD changes as you approach. Flying into the structure (not just its bounding sphere) bounces the ship back with a collision sound and hull damage; flying through the gaps between arms does not collide. |

### MT-1-16 — Hostile AI

| Field | |
|---|---|
| Scenario | `combat-test` |
| Steps | Fly towards the aliens and pirates; dogfight. |
| Expected | Enemies close in, lead their shots, fire beams and missiles, break off when about to ram, and jink sideways when targeted (VCT ships). The player can be destroyed (MT-1-20). |

### MT-1-17 — Nav points

| Field | |
|---|---|
| Scenario | `flight-test` |
| Steps | Press N three times; MFD Nav page (`,`/`.`). |
| Expected | "NAV BASE", "NAV ALIENS", "NAV PIRATES" cycle; a blue diamond with the name marks the point on screen; the Nav page shows its name and range. |

### MT-1-18 — Sound effects

| Field | |
|---|---|
| Scenario | `gunnery-range` |
| Steps | Fire each weapon, get a lock, hit and destroy a drone, collide gently, run out of energy, change target. |
| Expected | Original sounds play: beam sounds per weapon, missile launch, lock tone, shield/hull hits, explosions, collision, denied, target click. Distant events are quieter. |

### MT-1-19 — Original-free sample pack

| Field | |
|---|---|
| Scenario | Sample pack arena |
| Steps | Run `openinterceptor --no-original` (or without `--data`); start the arena; fight. |
| Expected | The Dart (no cockpit art: glass instruments), two Dart wingmen, the textured ring station and three red raiders that attack. Everything works without any original file. The Model viewer lists only dart, raider and ring station. |

### MT-1-20 — Pause, menu, restart, death

| Field | |
|---|---|
| Scenario | `combat-test` |
| Steps | 1. P, P. 2. Esc, Esc. 3. Esc, R. 4. Get destroyed; press Enter. 5. Esc, L. |
| Expected | P freezes the simulation ("PAUSED (P)"). Esc dims the screen with the menu and a key summary; Esc resumes; R restarts the scenario with the same ship. On death: "SHIP DESTROYED", the camera orbits the wreck, Enter restarts. L returns to the launcher. |

### MT-1-21 — Linux build and run

| Field | |
|---|---|
| Scenario | `flight-test` |
| Preconditions | Linux with `libasound2-dev libx11-dev libxi-dev libgl1-mesa-dev`; an Xbox-style pad at `/dev/input/js0` if available. |
| Steps | `scripts/check.sh`, then run MT-1-01, 04, 07, 08, 18. |
| Expected | The gate passes; the cases behave as on macOS; `bindings.ini` is created under `~/.local/share/openinterceptor/`. |
