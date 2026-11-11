# Mathematics

!!! note "Planned"
    This section is **planned** and not yet written. It will teach mathematics using
    OpenInterceptor's own code and data as the worked example. One short worked example is
    already here, because two features of app 0.3.1 depend on it.

## Worked example: turn velocity and changing frames

Two playtest fixes rest on the same two ideas: the velocity of a point on a turning body, and
moving a vector between the world's axes and the ship's own axes.

### A point on a turning body moves at ω × r

A ship turning at angular velocity **ω** (a vector along the turn axis, its length the rate in
radians per second) carries every point on it around that axis. A point at offset **r** from
the centre moves at

> **v** = **ω** × **r**   (the cross product)

On a ship yawing left (ω points up, +Y), a gun on the right wing (r points right, +X) has
ω × r = (0, 1, 0) × (1, 0, 0) = (0, 0, −1): it swings *forwards* (the engine's forward is −Z).
A gun 3 m out on a ship yawing at 1 rad/s moves at 3 m/s from the turn alone.

A shot that leaves the gun physically starts with the gun's whole velocity: the ship's own
velocity, plus ω × r, plus its muzzle speed along the barrel. That is what
`sim::combat::fire_beams` computes:

```rust
let spin = rot * s.angular_velocity;           // ω in world axes (see below)
let offset = rot * mount_offset(k, n, radius); // r, in world axes
let velocity = vel + spin.cross(offset) + dir * beam.speed;
```

### Body axes and world axes

The ship's orientation is a unit quaternion `rot`. Multiplying a vector by it, `rot * v`, turns
a vector written in the **ship's axes** (x right, y up, −z forward) into **world axes**. The
reverse turn is the **conjugate**, `rot.conjugate() * v`: for a unit quaternion the conjugate is
the inverse, and it is cheaper to compute (negate three components) than a general inverse.

- `angular_velocity` is stored in body axes ("yawing left" is the same vector whichever way the
  ship faces), so the code turns it into world axes with `rot *` before the cross product.
- The HUD's drift readout needs the opposite: the world velocity seen from the ship.
  `Ship::body_velocity` is `rot.conjugate() * velocity`, with z negated so forward reads
  positive. A ship flying at 300 m/s that yaws 90° keeps its motion (newtonian flight), so in
  its own axes the same velocity is now 300 m/s *sideways*: the readout shows `X-300` or
  `X+300`, depending on the direction of the turn.

The same pair appears in the arcade option `bolt_frame: ship`: a shot's offset and velocity are
stored in its owner's axes (`rot.conjugate() *`) and placed back into the world each tick with
the owner's *current* orientation (`owner.rotation *`), so the shot turns with the nose.

**Where to look:** `src/sim/combat.rs` (`fire_beams`, `move_bolts`), `src/sim/mod.rs`
(`Ship::body_velocity`), and the tests `a_bolt_leaves_with_the_muzzles_velocity_then_flies_straight`
and `body_velocity_reads_drift_in_the_ships_own_axes`, which check these formulas with numbers.
