# Missiles and mines

Every missile class chooses how it **steers** (`guidance`) and how its warhead's **damage
lands** (`damage`). Both are fixed engine behaviours (`src/sim/guidance.rs`); everything else —
speeds, turn rates, damage, decoy counts, split distances — is data. Stage 2 item 2.10. Every
key: [Mod file reference › missiles](reference.md#missiles).

```yaml
missiles:
  cluster_rocket:
    name: Cluster rocket
    guidance: cluster          # splits into six seekers near the target
    cluster: {missile: seeker, count: 6, at: 700m}
    impact_damage: 120
    proximity_damage: 50
  decoy_rocket:
    guidance: parallax         # two flares that draw enemy rockets
    decoys: 2
  permeator:
    damage: bypass_shields     # straight to armour and hull
```

## Guidance

| `guidance:` | Behaviour |
|---|---|
| `homing` (default) | Turns towards its locked target at `turn_rate`; with `lead_speed` (app 0.22.0) it aims ahead of the target as if it flew at that speed. |
| `dumbfire` | Never steers. |
| `sidestep` | Its own thrusters: turns twice as hard inside 600 m of the target and weaves, so it is hard to dodge. |
| `parallax` | Homing; at launch releases `decoys` (default 2) that enemy missiles may chase instead of the launcher. |
| `cloaked_wait` | Dropped behind the launcher; drifts to a stop and waits, hidden, until a hostile comes within `trigger_range` (default 800 m), then strikes. |
| `cluster` | Homing; near the target (`cluster.at`, default 600 m — or when half its fuel is spent) it splits into `cluster.count` (default 6) submunitions of `cluster.missile`, which carry on to the target. |
| `doppelganger` | Launches a **decoy copy of the launcher's ship**: a short-lived, fragile double that enemy ships and missiles may chase instead. |

**Decoys.** A decoy is a harmless missile flying away from its launcher. An enemy missile homing
on the launcher, within 2.5 km of one of its decoys, may become **lured** (80 % chance per second)
and from then on chases the nearest decoy; when the decoys are gone it returns to its target.

Every missile still detonates on impact, on a near miss (the proximity fuse: inside
`proximity_radius` and moving away) or when its fuel (`duration`) runs out. Its blast is the
`explosion` effect it names ([Looks](looks.md#effects)) — size in metres, with the effect's
sound — or a 25 m, 0.8 s fireball.

## Damage modes

| `damage:` | How the warhead's damage lands |
|---|---|
| `normal` (default) | On the shield facing that was hit, then armour, then hull. |
| `bypass_shields` | Ignores shields: straight to armour and hull. |
| `enveloping` | Half the damage runs the fore chain (fore shield, fore armour, hull) and half the aft chain, whatever side was hit (the original's Enveloping torpedo, **verified**; until app 0.22.14 both halves met the struck side's armour, which is the same with one armour pool). |
| `permeator` | Half runs the struck side's normal chain, half skips the shield and goes armour → hull (app 0.22.14; the original's Permeator, **verified**). |

## Point defence

A ship or object with `point_defence` carries automatic guns that shoot at the **nearest hostile
missile in range** — decoys included, since the guns cannot tell them apart:

```yaml
objects:
  guard_station:
    point_defence: {range: 1.2km, rate: 4, chance: 40%, energy: 3}
```

| Key | Default | Meaning |
|---|---|---|
| `range` | 800 m | Reach. |
| `rate` | 3 | Shots per second. |
| `chance` | 35 % | Chance a shot destroys the missile, rolled on the world's seeded generator (battles replay identically). |
| `energy` | 4 | Energy per shot, from the ship's capacitor; no energy, no shot. |

Each shot draws a tracer and plays the `point_defence` event sound. Point defence is fitted to
the dreadnought and the stations (`default`); the numbers are derived (DEV-038). The original has
no automatic point defence: since app 0.22.19 `interceptor` has none, and instead every beam bolt
can strike a missile (`flight.beams_hit_missiles`, 90 % to apply its damage against the missile's
`hit_points`).

## The original's missile flight

With `flight: original` (app 0.22.19) a missile flies as the original's: straight for 250 ms, then
homing at most 2.5 rad/s, losing throttle while it is off target, so a hard turn slows it. Its
warhead can hit everything round it:

```yaml
missiles:
  xwinder:
    inherit: original:GameSettings.ini/XWinderMissile
    launcher_rate: 4                     # shots a second from its launcher
    flight: original
    pass_fuse: 25m                       # goes off passing within 25 m, > 45 deg off
    blast: {impact: 15m, radius: 200m}   # every object, either side, no falloff
    dud_on_expiry: true
    lose_lock_beyond: 170
```

`images`, `max_life` and `retarget_every` give the Parallax, Passive Cloaking and Doppelganger
torpedoes their original behaviour; see the [reference](reference.md#missiles).

## Mines

An object with `kind: mine` and a behaviour whose state does `do: mine` waits until a hostile is
within the state's `range`, then detonates its **first missile rack** as the warhead: an **area
blast** reaching four times the warhead's `proximity_radius` (at least 100 m), dealing its
`proximity_damage` falling off linearly with distance, `enveloping` — and the mine is gone. See
[Stations and modules › Mines](stations.md#mines) for a complete example.

## In the two campaigns

| Campaign | Missiles |
|---|---|
| `interceptor` | X-Winder (homing), Sidestep, Permeator (`permeator`), Fusion, Enveloping torpedo, Parallax torpedo (two images after 1 s), Passive Cloaking torpedo (`cloaked_wait`, 10 s), Doppelganger (hostiles turn to it every 5 – 6 s), all with the original's flight, fuse, blast, launcher rate and rounds (app 0.22.19, **verified**); the Super Avenger's testbed load carries one of each. |
| `default` | Seeker rockets, atomic torpedoes, cluster rockets (Thunderbolt), decoy rockets (Meteor), Dominion space-mine warheads (`dominion-raid`). |

## The Nova bomb

A missile with a `fuse` is a Nova bomb: it never goes off on contact; it flies on and detonates
`fuse` after launch, doing no damage (the original's detonation damages nothing, **verified**); it
sets the `nova_time` mission query running, sends every pilot with `retreat_on_nova` to its roll,
and `ends_battle_after` later turns `battle_over` true — the battle then ends as a draw unless an
objective decides it (app 0.22.19). Its `warnings` are voice lines of the launcher's wingmen and/or
texts at given times:

```yaml
missiles:
  nova_bomb:
    inherit: original:GameSettings.ini/NovaBomb
    fuse: 60s
    ends_battle_after: 60s
    warnings: [{at: 30s, line: wave33}, {at: 60s, line: wave34}, {at: 88s, line: wave35}, {at: 107s, line: wave36}]
```

`interceptor` has it with the original's timing and messages: voice waves 33 – 36, which its
`mpilot1` … `fpilot5` voice sets map to the original's chatter clips `{M|F}Pilot{v}{33–36}.wav`
(**verified**; there is no subtitle). A campaign craft fitted with the Nova Bomb carries it. The
original's final missions count a fired Nova as a win when an X-COM Patton is still in the battle;
the win and loss only cue the victory and death music stingers (the original's `0x550fd5(5, 3)` and
`(5, 1)`, which the engine plays for any won or lost mission) — nothing is recorded on the strategy
layer.
