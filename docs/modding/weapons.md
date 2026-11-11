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
| `homing` (default) | Turns towards its locked target at `turn_rate`. |
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
`proximity_radius` and moving away) or when its fuel (`duration`) runs out, with a small fixed
fireball (the `explosion` key naming an effect is accepted but not used yet).

## Damage modes

| `damage:` | How the warhead's damage lands |
|---|---|
| `normal` (default) | On the shield facing that was hit, then armour, then hull. |
| `bypass_shields` | Ignores shields: straight to armour and hull (the original's Permeator). |
| `enveloping` | Spread over both shield facings, as if the whole surface were hit. |

## Mines

An object with `kind: mine` and a behaviour whose state does `do: mine` waits until a hostile is
within the state's `range`, then detonates its **first missile rack** as the warhead: an **area
blast** reaching four times the warhead's `proximity_radius` (at least 100 m), dealing its
`proximity_damage` falling off linearly with distance, `enveloping` — and the mine is gone. See
[Stations and modules › Mines](stations.md#mines) for a complete example.

## In the two campaigns

| Campaign | Missiles |
|---|---|
| `interceptor` | X-Winder (homing), Sidestep, Permeator (`bypass_shields`), Fusion, Enveloping torpedo, Parallax torpedo (2 decoys), Passive Cloaking torpedo (`cloaked_wait`), Doppelganger — the special behaviours follow the original's descriptions (derived); the Super Avenger's testbed load carries one of each. |
| `default` | Seeker rockets, atomic torpedoes, cluster rockets (Thunderbolt), decoy rockets (Meteor), Dominion space-mine warheads (`dominion-raid`). |
