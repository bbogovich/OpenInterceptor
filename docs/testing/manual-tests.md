# Manual tests

Playtest cases for behaviour unit tests cannot show. Each stage's cases must be run and recorded
as passed before the stage is complete. Record results in the table (date, tester, platform,
result, notes); keep earlier rows when re-running.

**How to run**: build with `cargo build --release`, then `target/release/openinterceptor --data game`.
Every case names its testbed scenario; `--scenario <id> --ship <id>` jumps straight to it.

## Stage 1 results

| ID | Case | Scenario | Result | Date | Tester / platform | Notes |
|---|---|---|---|---|---|---|
| MT-1-01 | Launcher lists scenarios | Launcher | not run | | | |
| MT-1-02 | Ship selection: all three hulls | Flight test | not run | | | |
| MT-1-03 | Model viewer browses all meshes | Model viewer | not run | | | |
| MT-1-04 | Keyboard flight: attitude | Flight school | not run | | | |
| MT-1-05 | Keyboard flight: throttle | Flight school | not run | | | |
| MT-1-06 | Mouse flight | Flight school | not run | | | |
| MT-1-07 | Gamepad flight | Flight school | not run | | | |
| MT-1-08 | Cockpit per hull | Flight test | not run | | | |
| MT-1-09 | Views | Flight test | not run | | | |
| MT-1-10 | Targeting and target MFD | Flight test | not run | | | |
| MT-1-11 | Beams, energy and link modes | Gunnery range | not run | | | |
| MT-1-12 | Missile lock and hit | Gunnery range | not run | | | |
| MT-1-13 | Enemy destruction | Flight test | not run | | | |
| MT-1-14 | Wingmen hold formation | Flight test | not run | | | |
| MT-1-15 | Station visible and collidable | Flight test | not run | | | |
| MT-1-16 | Sample pack runs without original data | Sample pack arena | not run | | | |
| MT-1-17 | Pause and quit | Any | not run | | | |

## Stage 1 cases

### MT-1-01 — Launcher lists scenarios

| Field | |
|---|---|
| Scenario | Launcher |
| Preconditions | Original data in `game/` (or `--data`). |
| Steps | 1. Run `openinterceptor --data game`.<br>2. Look at the scenario list.<br>3. Run again with `--no-original`. |
| Expected | With data: Model viewer, Flight test, Flight school, Gunnery range and Sample pack arena are listed, each with a title and description. With `--no-original`: only the Sample pack arena (and Model viewer showing sample models); a note says the Interceptor pack was skipped. |
| Pass / fail | |

### MT-1-02 — Ship selection: all three hulls

| Field | |
|---|---|
| Scenario | Flight test |
| Preconditions | Launcher open. |
| Steps | 1. Choose Flight test.<br>2. Select X-1A Lightning II; start; note the ship and cockpit; Esc → Back.<br>3. Repeat with S-2 Firestar and M-3 Super Avenger. |
| Expected | Each hull starts the scenario as the player's ship. The F7 chase view shows the chosen hull's model. Handling differs: Lightning turns fastest, Avenger slowest. |
| Pass / fail | |

### MT-1-03 — Model viewer browses all meshes

| Field | |
|---|---|
| Scenario | Model viewer |
| Preconditions | Original data present. |
| Steps | 1. Open Model viewer.<br>2. Step through models with Left/Right (hold to scroll).<br>3. Orbit with the mouse; zoom with the wheel.<br>4. Toggle LOD with `L` and textures with `T`. |
| Expected | Every original mesh (297) and every sample-pack model can be shown. Textures appear correctly oriented (no mirrored text). LOD toggle visibly reduces detail. No crash on any model. |
| Pass / fail | |

### MT-1-04 — Keyboard flight: attitude

| Field | |
|---|---|
| Scenario | Flight school |
| Preconditions | Any hull, cockpit view (F1). |
| Steps | 1. Press Up arrow, then Down arrow.<br>2. Press Left / Right arrow.<br>3. Press NumPad 7 / NumPad 9. |
| Expected | Up = nose down, Down = nose up; Left/Right yaw; NumPad 7/9 roll left/right. Rates fall noticeably at high speed and recover when slowing. |
| Pass / fail | |

### MT-1-05 — Keyboard flight: throttle

| Field | |
|---|---|
| Scenario | Flight school |
| Preconditions | Any hull, speed gauge visible. |
| Steps | 1. Press `+` five times, then `-` twice.<br>2. Press Backspace.<br>3. Press `\`.<br>4. Press `Z`, then `+` a few times; press `Z` again.<br>5. Hold Tab for 3 seconds at full throttle, release.<br>6. Target a ship and press `M`. |
| Expected | `+`/`-` change set speed in 10 % steps of MaxSpeed (50 %, then 30 %). Backspace = 100 %; `\` = 0 and the ship slows to a stop. Reverse moves backwards. Tab raises speed above MaxSpeed and drains energy (gauge falls), which recovers after release. `M` matches the target's speed. |
| Pass / fail | |

### MT-1-06 — Mouse flight

| Field | |
|---|---|
| Scenario | Flight school |
| Preconditions | Mouse connected. |
| Steps | 1. Move the mouse forward/back/left/right.<br>2. Click left and right buttons. |
| Expected | Forward = nose down, back = nose up, left/right turn. Left button fires beams; right button fires a missile. |
| Pass / fail | |

### MT-1-07 — Gamepad flight

| Field | |
|---|---|
| Scenario | Flight school |
| Preconditions | A controller connected (Xbox/PlayStation style) before starting. |
| Steps | 1. Use the left stick, right stick, triggers and bumpers.<br>2. Hold A; press B, X, Y; D-pad up/down.<br>3. Disconnect and reconnect the controller. |
| Expected | Left stick pitch/yaw, right stick roll; RT beams, LT missile; bumpers cycle targets; A afterburner, B closest enemy, X match speed, Y cycles view; D-pad changes throttle. Reconnecting resumes control without restarting. (Run on macOS and on Linux.) |
| Pass / fail | |

### MT-1-08 — Cockpit per hull

| Field | |
|---|---|
| Scenario | Flight test |
| Preconditions | — |
| Steps | 1. Start with each hull in turn in the F1 view. |
| Expected | Each hull shows its own cockpit frame (A, B, C). The space view shows through the window area with no cyan pixels. On a wide window the frame is centred and the 3D view fills the sides. |
| Pass / fail | |

### MT-1-09 — Views

| Field | |
|---|---|
| Scenario | Flight test |
| Preconditions | — |
| Steps | 1. Press F1, F2, F3–F6, F7, F8.<br>2. In F8, use Ins/Del, PgUp/PgDn, Home/End, NumPad 5. |
| Expected | F1 cockpit forward; F2 full forward without frame (minimal HUD); F3–F6 look left/right/rear; F7 chase camera follows; F8 chase camera moves and zooms with the keys and resets with NumPad 5. |
| Pass / fail | |

### MT-1-10 — Targeting and target MFD

| Field | |
|---|---|
| Scenario | Flight test |
| Preconditions | — |
| Steps | 1. Press `T` repeatedly, then `Y`.<br>2. Press `R`.<br>3. Point the nose at a ship and press `G`. |
| Expected | `T`/`Y` cycle through targets forwards/backwards; `R` selects the closest enemy; `G` selects the ship nearest the reticle. The MFD shows the target's name, distance, hull state and model; a bracket marks it in space; an off-screen arrow points to it. |
| Pass / fail | |

### MT-1-11 — Beams, energy and link modes

| Field | |
|---|---|
| Scenario | Gunnery range |
| Preconditions | Hull with ≥ 2 beams. |
| Steps | 1. Hold Enter; watch the energy gauge.<br>2. Press `X` and fire in each mode.<br>3. Fire at a drone until it is destroyed. |
| Expected | Bolts leave the beam mounts at the weapon's rate and speed; energy falls per shot and recharges when not firing; firing stops when energy is too low. `X` cycles Link (alternate guns) / Full (all together) / Unlink (one gun), shown on the message line. Hits flash; the drone is destroyed when its hit points are gone. |
| Pass / fail | |

### MT-1-12 — Missile lock and hit

| Field | |
|---|---|
| Scenario | Gunnery range |
| Preconditions | A drone targeted. |
| Steps | 1. Keep the target inside the reticle cone.<br>2. Watch the lock indicator; press Space when locked.<br>3. Try firing before lock. |
| Expected | A lock tone/indicator completes after the sensor's lock time (2.5 s with the standard sensor). The missile homes and hits, damaging the target. Breaking the cone resets the lock. Unlocked missiles fly straight. |
| Pass / fail | |

### MT-1-13 — Enemy destruction

| Field | |
|---|---|
| Scenario | Flight test |
| Preconditions | — |
| Steps | 1. Destroy one enemy of each type with beams and missiles. |
| Expected | Each enemy takes damage and is destroyed with an explosion; it is removed from the target list; a message names it. |
| Pass / fail | |

### MT-1-14 — Wingmen hold formation

| Field | |
|---|---|
| Scenario | Flight test |
| Preconditions | — |
| Steps | 1. Fly straight, then turn hard, change speed and use the afterburner.<br>2. Look with F7. |
| Expected | Two wingmen keep their slots on either side, catching up after turns and speed changes, without colliding with the player or each other. |
| Pass / fail | |

### MT-1-15 — Station visible and collidable

| Field | |
|---|---|
| Scenario | Flight test |
| Preconditions | — |
| Steps | 1. Fly toward the X-COM base from far away.<br>2. Fly into it slowly. |
| Expected | The station is visible from far (lower LOD) and gains detail as you approach. Flying into it stops or bounces the ship and causes damage; you cannot pass through it. |
| Pass / fail | |

### MT-1-16 — Sample pack runs without original data

| Field | |
|---|---|
| Scenario | Sample pack arena |
| Preconditions | Run with `--no-original` (or no `game/`). |
| Steps | 1. Start the Sample pack arena with each sample ship. |
| Expected | The scenario runs entirely on sample-pack models, textures and rules; flight, weapons and HUD work; no original file is read. |
| Pass / fail | |

### MT-1-17 — Pause and quit

| Field | |
|---|---|
| Scenario | Any |
| Preconditions | — |
| Steps | 1. Press `P`, wait, press `P`.<br>2. Press Esc and choose Back to launcher.<br>3. Press Alt+Q. |
| Expected | `P` freezes the simulation (HUD shows PAUSED) and resumes exactly. Esc menu returns to the launcher. Alt+Q quits the application. |
| Pass / fail | |
