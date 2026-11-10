# Scenarios and missions

A scenario places objects in a space arena. In Stage 1 scenarios are the testbed's live test
environments; in Stage 3 the same files gain objectives and triggers and become missions.

```ini
[scenario]
id = flight-test
title = Flight test: station patrol
description = Fly around the X-COM base with two wingmen; one of each enemy type nearby.
environment = deep-space        ; backdrop: starfield, sun direction, nebula
player = choose                 ; a ship id, or "choose" to pick in the launcher
requires = interceptor          ; packs needed

[object.player]
class = choose
side = xcom
position = 0, 0, 0
heading = 0, 0              ; yaw, pitch in degrees
ai = player

[object.alpha2]
class = ship:lightning
side = xcom
position = -60, 0, 40
ai = wingman
leader = player
formation = left

[object.base]
class = object:xcom-base
side = xcom
position = 0, -200, -3000
ai = idle

[object.wraith]
class = ship:wraith
side = alien
position = 800, 100, -2500
ai = enemy

[nav.1]                     ; optional nav point (engine extension)
name = Patrol point
position = 0, 0, -8000
```

| `ai` | Behaviour (Stage 1) |
|---|---|
| `player` | Controlled by the player |
| `wingman` | Holds formation on `leader`; attacks the leader's target when ordered |
| `enemy` | Basic pursuit and attack (full per-class AI in Stage 2) |
| `idle` | Drifts or holds position (targets, stations) |
| `turret` | Fixed; turns to fire (Stage 2) |

Positions are metres; −Z is "ahead" of the default heading.
