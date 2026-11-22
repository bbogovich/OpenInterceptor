# Requirements: stations, pilot stats, specials and the auto-resolve (from the exe, 2026-11-22)

This list turns the RE facts in `README.md` §8–§14 into engine requirements: one behaviour per item.
Section references (§n) point to `README.md`, and addresses there are for the CD build.

**Rules for every item.**

* All numbers are **pack data**. The `interceptor` pack sets them to the original's values.
* A key that is absent keeps today's engine behaviour, so the `default` campaign keeps working
  unchanged. Where a default campaign value is suggested, that is a separate balance decision (#56).
* Every new key is documented on the docs site's *Modding* pages in the same change.

The key names below are proposals; the schema reviewer can rename them.

Priority: **P1** = changes how often battles are won or lost, **P2** = visible in play, **P3** = fidelity detail.

---

## A. Stations (Track A4, DEV-035)

### RQ-C-01 Station hull is HP1 only (P1)

* **Original.** A station's maximum hull is the `HP1` of the chart chosen by side and subtype
  (§8.1). `HP2`, `HP3`, `HP4` and `SD` are loaded and never read. The Outpost and OPP charts are
  never used.
* **Engine now.** `rules.rs` sets hull = HP1 + HP2 + HP3 + HP4 for `inherit: original:…/*HP`. The
  X-COM base gets 11,500 instead of 6,000, and the alien base 7,700 instead of 5,000.
* **Gap.** Stations are 1.5–1.9× too tough.
* **Where.** `content::rules` (inherit mapping for `*HP` sections); `packs/interceptor/rules/stations.yaml`
  comments.
* **Pack data.** None new: the inherit reads `HP1` into `hit_points`. A pack may still write
  `hit_points:` directly.
* **Test.** Update `rules.rs` test `station … hull = HP1 + …` to expect HP1 (10.0 in the
  synthetic chart).

### RQ-C-02 One shield pool, sized by the shield-generator module type (P1)

* **Original.** A station has one shield pool, with no fore and aft (§8.2).
  * Its maximum is set by the type of the module in its shield slot: X-COM types 1/2/3 give
    2,000/3,000/4,000, alien types 2/3 give 3,000/4,000. Only one generator counts.
  * The pool recharges **2 points per second**, up to the maximum. It starts full (derived).
  * When the generator is destroyed, the maximum becomes 0: the pool **keeps its current points but
    no longer recharges**.
  * While the station is cloaked, the pool is skipped.
* **Engine now.**
  * The hub has `shields: 1000` (X-COM) or `1500` (alien) per bank, with fore and aft banks and the
    ship recharge rule.
  * `update_modules` zeroes both banks when the last generator is gone. Generators carry only
    `shield_generator: true`.
* **Gap.**
  * Capacity: 2,000–4,000 in the original against the engine's 1,000–1,500.
  * There is no single pool.
  * The engine drops the shields instantly; the original lets them drain.
* **Where.** `sim::mod` (`update_modules`), `sim::combat::apply_hit` (station branch), `sim::systems`
  (recharge).
* **Pack data.**
  * Station object: `shield_pool: {recharge: 2/s, on_generator_lost: keep | drop}`; absent = today's
    banks.
  * Generator module: `shield_capacity: 3000`. The station's maximum is the capacity of the first
    live generator; `stack: false` is the default.
  * Interceptor values: `xcom_shield_generator` 2,000 (Type1), alien pod 3,000 (Type2), and
    `on_generator_lost: keep`. The default campaign keeps `drop`.
* **Test.**
  * `sim` unit test: a station with a 3,000-capacity generator starts at 3,000.
  * After 10 s at 2,990 it reaches 3,000 and no more.
  * With the generator destroyed, the pool keeps 1,500 and stays there with `keep`, and goes to 0
    with `drop`.
  * Hits from fore and aft drain the same pool.

### RQ-C-03 Module hits are taken by the hub's pool while it is up (P1)

* **Original.** A hit on a station module (rt type 3) or tendril (type 0x20) goes to the **hub's**
  shield pool while that pool is above 0. Only when the pool is down does the module's own HP
  (from `M_*` `HitPoints`) take damage (§8.3).
* **Engine now.** Modules are separate ships, and their own hull takes the hit at once.
* **Gap.** Players can strip guns and generators through intact shields.
* **Where.** `sim::combat::apply_hit`: redirect when the target has `mount = Some(hub)` and the hub
  has a pool.
* **Pack data.** Module object: `shielded_by_hub: true` (interceptor modules); absent = today.
* **Test.**
  * A hit of 100 on a module of a hub with pool 500 leaves the module's HP unchanged and the pool
    at 400.
  * With the pool at 0, the module loses 100.
  * A hit of 600 at pool 500: the original sends the overflow to the **hub's hull**, not the module
    (`0x454854` on the parent). The test asserts that overflow goes to the hub hull.

### RQ-C-04 Station hull hits: no armour, no difficulty factor, no system hits (P2)

* **Original.** The hub's chain is pool → hull (`0x4205d9`), with no armour bank, no difficulty
  multiplier, no 30 % system share and no pilot roll (§8.3).
* **Engine now.** The generic `apply_hit` applies, including `beam_factor` on player hits, any
  armour, and `damage_system`.
* **Gap.** Easy levels kill stations 15 % faster than the original, and system rolls land on
  stations.
* **Where.** `sim::combat::apply_hit`.
* **Pack data.** Station object: `damage_model: station` (= pool → hull only); absent = today.
* **Test.** A station hit on Beginner by the player loses exactly the rolled damage, and no system
  event is emitted.

### RQ-C-05 Alien base hard points (P1)

* **Original.** The alien base has a **connection point** (1,400 HP, a 124 m sphere at model
  (0, −217, 0)) and **three tendrils** (550 HP each, sub-objects 1–3 of the model) (§8.4).
  * A *direct* beam or missile hit (not the Permeator/Enveloping half-paths) that lands within the
    sphere while the pool is down damages the connection point instead of the hull.
  * Tendrils take hits only while the pool is down.
  * When all four are at 0, the base is destroyed, whatever its hull.
  * Messages: "Connection Point Destroyed" and "Tendril Destroyed".
* **Engine now.** Nothing like it. The alien base dies only to hull damage, and its modules are
  turrets and pods (DEV-035).
* **Gap.** This is the alien base's intended weak point, and it is missing.
* **Where.**
  * `sim::combat`: hard-point table on the target; the zone test uses the impact point, which
    `apply_hit` already has as `at`.
  * `sim::mod`: tendrils as mounted parts.
  * `content::rules`: parsing.
* **Pack data.** Station object:
  * `hard_points: [{name, hit_points, at: [x,y,z], radius, part: <attachment or sub-object>,
    message}]`;
  * `destroyed_when: hull | hard_points | either` (default `hull`).

  Interceptor: `alien_base` with the four entries above and `either`. The text keys stay in the
  player's language file, referenced by id.
* **Test.**
  * A direct hit inside the radius with the pool down lowers the point's HP and not the hull.
  * A Permeator half-hit inside the radius goes to the hull.
  * Destroying all four ends the base with hull > 0.
  * With the pool up, the hit drains the pool only.

### RQ-C-06 Station beam turrets: the original's fire control (P1)

* **Original** (§8.6). A turret fires the module's beam with **no energy cost**, under these rules:
  * **Range** 5,000 m, and an **arc** (85° on the alien base) around its facing.
  * **Aim.** It leads fully with the beam's `Speed`. When `chance(accuracy)` fails and the target
    is over 200 m away, it adds a uniform ±20 m lateral offset along the target's x or y axis.
  * **Line of sight.** With the LOS flag, it fires only when a 5,000 m line of sight hits nothing,
    the target, or another side's object.
  * **Interval.** One shot per interval:
    * X-COM base: interval = `1000 / RateOfFire` ms, accuracy rand[60 − 5d, 100 − 5d).
    * Alien base: interval = `1000 − 25d + rand[0,50)` ms, accuracy rand[40 + 5d, 75 + 5d).
  * **Beam type** follows the module type, and no module fires TracerCannon or PsiBlaster.
* **Engine now.** Behaviour `station_turret`: range 2.5 km, burst/pause per level, `aim` per level
  (angle error model), and the beams pay energy from the module's power.
* **Gap.**
  * Range is half the original's.
  * The aim model differs: the original's error is lateral metres, the engine's is an angle.
  * Turrets pay energy where the original's do not.
  * The fire pattern differs: a steady interval in the original, bursts in the engine.
* **Where.** `sim::ai` (the `turret` manoeuvre), `content::behaviour::Skill`.
* **Pack data.** Behaviour state `do: turret` gains:
  * `interval: {base, per_level, jitter}` or `interval: rate_of_fire` (1000 / beam RateOfFire);
  * `accuracy: {from, to, per_level}`, a per-turret uniform roll at spawn, in percent;
  * `miss_offset: 20m`, `miss_beyond: 200m`, `arc`, `range`, `line_of_sight: true`, and
    `free_fire: true` (no energy).

  Interceptor: `station_turret_xcom` and `station_turret_alien` with the values above.
* **Test.**
  * With accuracy 100 %, the aim point equals target + v × d / speed.
  * With accuracy 0 % at 300 m, the aim point is offset by 20 m or less along one target axis.
  * At 150 m it is not offset.
  * Shots are spaced by the interval.
  * No energy is drawn.
  * A blocked line of sight prevents fire.

### RQ-C-07 Station layouts, pirate turret tables and alien base mines (P2)

* **Original** (§8.6).
  * The X-COM and alien bases get one turret per beam- or missile-defence module they own.
  * The pirate arms dealer has 5 fixed Laser turrets and the pirate raider/base 10. Each has a
    position, facing, range (2,000–3,000 m), arc (75–125°), accuracy `a + b·d` and interval
    `c + e·d + rand[0,50)`. Some pirate-base turrets exist only with `chance(25 + 5d)`.
  * Science stations have no turrets.
  * The alien base also lays two lines of `d/2 + 2` missile mines 4,000 m apart (d > 3,
    `chance(50 + 5d)`) and two lines of `d + 5` type-1 mines 2,000 m apart (d > 0,
    `chance(25 + 5d)`).
* **Engine now.** Module placement is derived (DEV-035), the pirate base has no turret list, and
  there are no base mines.
* **Gap.** Pirate stations are toothless, and alien bases on Genius/Superhuman lack their minefields.
* **Where.** `content::rules` (station `modules`), `sim::mod::spawn_modules`, `sim::hazards` (mines).
* **Pack data.**
  * `modules: [{slot | at: [x,y,z], facing, class, chance: {base, per_level}}]`;
  * a turret class per table row;
  * station `minefields: [{mine, count: {base, per_level}, lines: 2, spacing, min_level, chance:
    {base, per_level}}]`.

  The values are facts (positions are model-space numbers, not art).
* **Test.**
  * A pirate base spawns 10 turrets at chance 100 %, and fewer with a seeded chance.
  * The alien base on level 4 with forced chances spawns `2 × (4/2 + 2) = 8` missile mines and
    `2 × 9 = 18` type-1 mines; on level 0 it spawns none.

### RQ-C-08 Missile-defence modules (P2)

* **Original.** A missile turret has range 5,000 m and arc 85°. A 2,000 ms value is kept (lock
  time, derived), and it fires every `5000 − 25d + rand[0,50)` ms. Its rounds are 5 on the alien
  base and come from the module on the X-COM base (derived). The missile type comes from the
  module (§8.6).
* **Engine now.** There are no missile turrets. Stations have a generic `point_defence` (DEV-038).
* **Gap.** Bases do not fire missiles at all.
* **Where.** `sim::ai` (turret), `sim::guidance` (launch), `content::rules`.
* **Pack data.** Behaviour `do: missile_turret` with `range`, `arc`, `lock_time`,
  `interval: {base, per_level, jitter}` and `rounds`. Module class `missiles: [<missile id>]`.
* **Test.**
  * A missile turret with a target in its arc launches after `lock_time`, then every interval,
    until its rounds are spent.
  * Outside the arc it does not launch.

### RQ-C-09 Turrets on capital ships and freighters (P2)

* **Original** (§8.8).
  * AlienType3 carries Phase, Plasma, Psi and Plasma turrets, with accuracy from the pilot's
    Accuracy (`S − rand[5,15)` etc.).
  * AlienType4 has one Phase turret, AlienType5 two Tachyon turrets, and the AlienFreighter two
    Plasma turrets.
  * The X-COM, Outpost and Pirate freighters have two Laser turrets (2,000 m, LOS), and the Patton
    a table of turrets.
* **Engine now.** Freighters and capital ships use the class's own beams and the ship AI.
* **Gap.** Turret arcs and ranges and pilot-based accuracy are missing.
* **Where.** `content::rules` (ship `turrets:`), `sim::ai`.
* **Pack data.** Ship object: `turrets: [{beam, at, facing, range, arc, accuracy: {pilot_stat:
  accuracy, minus: [5, 15]} | {base, per_level}, interval: {base, per_level}, line_of_sight}]`.
* **Test.** A freighter turret engages a fighter at 1,900 m and not at 2,100 m. Its accuracy
  tracks a pilot stat that the test sets.

---

## B. Pilot stats in battle (Track A8, #28)

### RQ-C-10 Pilot records drive the AI's skill (P1)

* **Original** (§5.1, §9).
  * **Accuracy** sets the lead-time error and the offset aim. With d < 2 it is clamped:
    [50, 100] for X-COM, [25 + 10d, 50 + 10d] for others.
  * **Intelligence and Accuracy.** The fire mode (all mounts of the type) follows
    `chance((Int + Acc)/2)`.
  * **Intelligence and Flying.** The weapon choice follows `W = 0.75 Int + 0.25 Fly`.
  * **No pilot:** 50, or 25 for the weapon choice.
  * **Aggressiveness** is not read in flight.
* **Engine now.** `Skill.accuracy` comes from the behaviour (per level). Campaign pilot stats are
  not passed to the sim. Wingmen and enemies do not differ by pilot.
* **Gap.** A veteran wingman flies like a rookie.
* **Where.**
  * `content::behaviour::Skill`: new `accuracy_from`, `intelligence_from`, `flying_from`.
  * `sim` (Pilot::Ai carries a stat vector).
  * `app::strategy::fly`: hand the craft's pilot stats to the scenario.
* **Pack data.**
  * `strategy/campaign.yaml` `pilots.battle: {accuracy: accuracy, intelligence: intelligence,
    flying: flying, bravery: bravery, stamina: stamina}` maps the campaign's stat names to sim
    roles.
  * Behaviour `skill: {accuracy: pilot, clamp_low_levels: {own: [50, 100], others: [25, 50],
    per_level: 10}}`.

  Without the mapping, today's per-level values apply.
* **Test.**
  * A pilot with Accuracy 100 aims with lead factor 1.
  * With 50, the factor is 0.5 or 1.5.
  * The fire-mode roll uses `(Int + Acc)/2`, with a seeded generator.
  * The no-pilot defaults are 50 and 25.

### RQ-C-11 AI retreat when disarmed (P2)

* **Original** (§9). The roll happens at most every 5 s, at least 5 s after the last state change,
  and only for combat hulls. When no beam mount is left, the AI jumps out with `chance(p)`, where
  `b = 100 − Bravery` and p is clamped to [5, 85]:

  | Rank | p |
  |---|---|
  | < 2 | 0.7b + 40 |
  | 2 | 0.6b + 25 |
  | ≥ 3 | 0.5b |
  | no pilot | rand[50, 75) |

  There is **no retreat on hull damage**.
* **Engine now.** Behaviours retreat on script conditions such as `hull < 20%`; there is no
  disarmed rule.
* **Gap.** Interceptor AI flees on damage where the original does not, and does not flee when
  disarmed.
* **Where.** `content::behaviour` (condition functions), `sim::ai`.
* **Pack data.**
  * New condition queries: `beams_left`, `pilot(<role>)`, `rank`.
  * An expression for the chance, e.g. `random_pct(clamp(rank_table(...), 5, 85))` built from a
    table key `retreat_when_disarmed: {interval: 5s, ranks: [[0,1,0.7,40],[2,2,0.6,25],[3,9,0.5,0]],
    clamp: [5, 85], no_pilot: [50, 75]}`.

  The interceptor `behaviours.yaml` drops its hull-based flee.
* **Test.**
  * A ship with all beam mounts destroyed and rank 0 and Bravery 100 retreats with p = 40 (seeded).
  * Rank 3 and Bravery 100 give p = 5 (clamp).
  * A ship with one live mount never rolls.

### RQ-C-12 AI retreat from a Nova detonation (P3)

* **Original** (§9, §12). Once per ship, when a Nova bomb detonates, the AI rolls, with
  `b = 100 − Brav`, `i = 100 − Int` and `s = 100 − Sta`, clamped to [5, 95]:

  | Rank | p |
  |---|---|
  | > 3 | 0.5b + 0.25i + 0.25s − 25 |
  | 2–3 | 0.6b + 0.1i + 0.05s + 25 |
  | ≤ 1 | 0.25b + 0.1i + 0.05s + 50 |
  | no pilot | rand[50, 75) |

  It retreats with `chance(p)`.
* **Engine now.** None.
* **Where.** `sim::ai`, an event hook on the Nova detonation (RQ-C-14).
* **Pack data.** Behaviour `on_event: {nova_detonated: {retreat_chance: <table as RQ-C-11>}}`.
* **Test.** Forced detonation: a rank-0 pilot with all stats 100 gets p = 50, and one with all
  stats 0 gets p = 95 (clamped).

### RQ-C-13 Break-off and re-engage timing (P3)

* **Original** (§10).
  * Under 250 m the AI breaks off (state 3). It re-engages when the manoeuvre ends and more than
    `5000 − 500d` ms have passed (derived).
  * On break-off, with `chance(25)` and d > 1, it makes a radio call.
  * States: 1 search, 2 attack, 3 break off, 4 re-evaluate, 5 jump out.
* **Engine now.** The interceptor behaviour has `evade → engage when time_in_state > 2.5s` (or
  similar).
* **Where.** `packs/interceptor/rules/behaviours.yaml` only. The condition language already has
  `time_in_state` and `difficulty`.
* **Pack data.** `time_in_state > per_level([5s, 4.5s, 4s, 3.5s, 3s])`; break-off trigger
  `target_distance < 250m`.
* **Test.** A behaviour test with level 0 stays in evade for 5 s, and level 4 for 3 s.

---

## C. Systems and special missiles

### RQ-C-14 Nova bomb timeline (P2)

* **Original** (§12). The Nova bomb fuse is **60 s**, with growth and messages at 20/30/55 s. At
  60 s it detonates and the battle ends 60 s later (messages 32 s and 13 s before the end).
* **Engine now.** DEV-036 models specials from descriptions, and the Nova has no timeline.
* **Where.** `sim::guidance`, `sim::mod` (end of battle), mission outcome.
* **Pack data.** Missile `nova_bomb`: `fuse: 60s`, `ends_battle_after: 60s`,
  `warnings: [{at: 30s, text: <id>}, …]`.
* **Test.** Simulated time: no detonation event at 59 s, one at 60 s, and a battle-end event at
  120 s.

### RQ-C-15 PassiveCloaking, Parallax, Doppelganger behaviour (P3)

* **Original** (§12).
  * **PassiveCloaking** cloaks 1 s after launch, de-cloaks and homes when near, and **expires**
    after `min(Duration, 10 s)` as a dud.
  * **Parallax** spawns two images ±20 m sideways 1 s after launch.
  * **Doppelganger** pulls AI agents onto its decoy every 5–6 s while both live.
* **Engine now** (DEV-036). Parallax lures at 80 %/s within 2.5 km, PassiveCloaking waits within
  800 m with no expiry, and Doppelganger copies are fragile.
* **Where.** `sim::guidance`.
* **Pack data.** Missile keys: `cloak_after: 1s`, `max_life: 10s`, `images: {count: 2, offset:
  20m, after: 1s}`, `retarget_every: [5s, 6s]`. The values in interceptor, the default campaign
  unchanged.
* **Test.**
  * A PassiveCloaking missile with Duration 30 s is removed at 10 s without a blast.
  * A Parallax missile has two image objects at t = 1 s, 40 m apart.

### RQ-C-16 Engine failure below 15 % (P2)

* **Original** (§11). When an engine hit leaves the engine below 15 % of its `HitPoints`, or
  destroys it, the ship becomes **disabled** (rt flag 0x8):
  * thrust is zeroed and the fire selection and lock are reset;
  * it de-cloaks;
  * repair then puts the hull first;
  * a dead pilot's ship does not tumble.
* **Engine now.** `disabled` needs both engines and manoeuvring at 0 efficiency (`systems.rs` 457).
* **Gap.** A badly hit engine leaves the engine's ship flying.
* **Where.** `sim::systems` (disabled check, repair order), `sim::combat`.
* **Pack data.** `rules/systems.yaml` `engine: {disable_below: 15%}`; absent = today's rule.
* **Test.**
  * Engine HP 14 % after a hit → disabled, zero thrust, cloak off.
  * 16 % → not disabled.
  * Repair on a disabled ship fixes the hull first.

### RQ-C-17 Damaged sensors and hyperdrives change nothing (P3)

* **Original** (§11). Sensor HP and hyperdrive HP are only damaged, repaired and displayed.
  Missile lock and the jump never read them.
* **Engine now.** A damaged hyperdrive takes twice as long (`systems.rs` 300). Sensor condition may
  scale lock (DEV-029).
* **Gap.** The engine is harsher than the original.
* **Where.** `sim::systems`, `sim::targeting`.
* **Pack data.** `systems.yaml` `hyperdrive: {damaged_factor: 1.0}`, `sensor: {damage_affects_lock:
  false}` in interceptor; the default keeps today's.
* **Test.** With the factors at 1, a 10 % sensor and hyperdrive give the same lock time and jump
  time as 100 %.

---

## D. Off-screen battles and the campaign (Track A10, DEV-057)

### RQ-C-18 Auto-resolve as the original's hourly simulation (P1)

* **Original** (§13).
  * **Timing.** The first round comes 5 game-hours after the start, then one every game-hour.
  * **Who wins the hour.** Each side scores `rand(50) + 50 · N_side / N_total`, where N is the
    *ship count*. Only the hour's winner attacks.
  * **Ratings.** Each ship has:
    * offence `A = (0.6 Fly + 0.7 Brav + 0.5 Int + 0.35 Sta + 0.75 Agg + 0.95 Acc) / 385`;
    * evasion `B = (0.4 Fly + 0.6 Brav + 0.75 Int + 0.55 Sta + 0.05 Acc − k·Agg) / 235`;
    * attack `C` = hull-class base + sensor and VCT bonuses;
    * defence `D` = base + 0.75 hull + 0.85 armour + 0.95 shields + equipment bonuses.
  * **Duels.** A winning ship duels a random enemy for `clamp(hours, 1, 10)` exchanges. The shot
    hits with `chance(A_att)` and does **full damage** (the frac computed from `B_tgt`, C and D is
    discarded). The damage is:
    * 1/50 of the time a missile's `ImpactDamage`, one round spent;
    * else the first beam's `Damage`;
    * else 25–34.
  * **Damage chain.** A random shield bank, then a random armour bank, then the hull. A destroyed
    craft loses its pilot, always.
  * **End.** When one side is left.
* **Engine now.** One roll, `p = P² / (P² + E²)`, from √(effective HP × DPS). The player's craft
  takes no damage and pilots are untouched (DEV-057).
* **Gap.**
  * The original's odds come from ship counts and pilots, not firepower.
  * Its battles take hours of game time.
  * Its losses are real.
* **Where.**
  * A new `campaign::autoresolve` module: pure, seeded and unit-testable.
  * `app::strategy`: the prompt and the clock ticking the sim.
  * `balance` stays for `--balance`.
* **Pack data.** `strategy/campaign.yaml` `auto_resolve:`:
  * `{model: hourly | odds, first_round: 5h, round: 1h, score_random: 50, score_share: 50,
    max_exchanges: 10, missile_chance: 2%, fallback_damage: [25, 35]}`;
  * pilot weights `offence: {flying: 0.6, …, divisor: 385}`,
    `evasion: {…, aggression_penalty: [[85, 0.75], [70, 0.5]], divisor: 235}`;
  * per hull `auto_resolve: {attack: 25, defence: 10}`;
  * equipment bonuses `{sensor: [50,100,150], vct: {attack: 25, defence: 50}, cloak: 75,
    damage_control: 75, ecm: 60}`;
  * defence weights `{hull: 0.75, armour: 0.85, shields: 0.95}`.

  `model: odds` (or the section absent) = today's roll, which the default campaign keeps.
* **Test.** `campaign::autoresolve` unit tests:
  * Rating formulas against hand-computed values: all stats 100 gives A = 1.0 and B = 1.0.
  * The side score with a fixed generator.
  * One exchange with chance forced: damage = beam `Damage` through shield → armour → hull.
  * The pilot is deleted with the craft.
  * Rounds happen at 5 h, 6 h, 7 h.
  * The odds model is unchanged when `model: odds`.

### RQ-C-19 Auto-resolve against stations (P1)

* **Original** (§13.2–13.4).
  * **Who attacks the station.** A winning ship attacks the station when the share
    `f ∈ {0, 0.9, 1}` says so: f = 1 if one enemy side is empty, else `max(N_enemy)/N_winner` by
    integer division, capped at 0.9.
  * **Station ratings.** Base: A, B ∈ [0.65, 0.95); C = Σ module weights (beams 10–35, missiles
    5–35); D = 1,000 + 100 × ECM-1 + shield pool. The outpost and OPP use random ranges.
  * **Shots at the station** do damage × frac (clamped to [0.25, 1.5]). The pool absorbs them,
    then:
    * 75 %: a random one of 33 slots. A module there takes it, and is destroyed (aliens +250)
      when damage ≥ its HP. The damage never reaches the hull.
    * Else the hull: at 0 the station is destroyed (aliens +2,500). Otherwise a damage stage is
      added each time the hull crosses a quarter of HP1.
  * **Return fire.** The station shoots back for 25–124 (base), 25–74 (outpost) or 10–29 (OPP).
* **Engine now.** Base defence missions resolve with the same odds roll, and stations are not
  damaged.
* **Where.** `campaign::autoresolve`, `campaign` base records (hull, shield, modules per base).
* **Pack data.**
  * `auto_resolve.stations: {module_hit: 75%, slots: 33, stage_quarters: true, return_fire:
    {base: [25, 125], outpost: [25, 75], opp: [10, 30]}, rating: {…}}`;
  * per module class `auto_resolve: {attack: 30}`;
  * `scores: {module_lost: 250, base_lost: 2500}` (alien tally).
* **Test.**
  * A forced module hit destroys a module of HP 100 with damage 100 and leaves the hull untouched.
  * A forced hull hit at HP1/4 boundaries raises the stage once per quarter.
  * A base whose hull reaches 0 is removed and the alien tally gains 2,500.

### RQ-C-20 Battle results persist on craft and stations (P1)

* **Original** (§14). Flight and the auto-resolve write to the same ship records. Hull, shield,
  armour and system damage stay on the craft after either mode. Station hull, shields and modules
  stay damaged in the base record.
* **Engine now.** `campaign::Craft` has no condition. A flown battle's damage is lost on return,
  and a lost auto-resolve does nothing to craft.
* **Gap.** There is no attrition.
* **Where.** `campaign::Craft` (a `condition` struct: hull, banks, system HP), `app::strategy`
  (copy in at launch, copy out at debrief), save format (versioned), `campaign` repair per day.
* **Pack data.** `strategy/campaign.yaml` `repair: {hull_per_day, systems_per_day}`. The original's
  rate was not traced, so the interceptor value is derived and marked so. `persist_damage: true`;
  absent = today.
* **Test.**
  * Round-trip: launch a craft at 60 % hull and it spawns at 60 %.
  * Return at 30 % and the campaign craft reads 30 %.
  * Save, load and compare.
  * A day of repair adds `hull_per_day`.

### RQ-C-21 Off-screen battle joins real time (P3)

* **Original** (§13.1). An off-screen sim ends ("exited due to realtime resolution") when one of
  its ships enters real-time combat. The fight then continues in flight, with the damage taken so
  far.
* **Engine now.** Auto-resolve is instant, so there is nothing to join.
* **Where.** `campaign::autoresolve` (cancel), `app::strategy`.
* **Pack data.** None.
* **Test.** Starting a flown battle with a craft that is in a running sim removes the sim and
  launches with the craft's current condition.

---

### RQ-C-22 Gimballed turrets aim perfectly (P2)

* **Original:** gimballed station turrets (kinds 1–3) lead the target at a fixed 2,000 m/s, slew up to
  90° per frame, fire only when aligned and only at targets in front of the mount, and apply **no
  accuracy error**; fixed turrets keep their accuracy roll (`../unknowns-2026-11.md` §6).
* **Pack key:** `turret.aim: {lead_speed: 2000, slew_deg_per_frame: 90, accuracy: none|roll}`.
* **Default:** same for the default campaign's turrets.
* **Test:** U — aim point for a moving synthetic target; no error for gimballed kinds.

### RQ-C-23 Pilot grade (P2)

* **Original:** pilot +0x28 is a grade 0–4 (X-COM 0, pirates 1, outpost freighter crews 2, aliens 1–4 by
  hull and month); it adds a generation bonus to stats and sets the AI look-ahead (225/100/250 m) and
  sprint (`../unknowns-2026-11.md` §5).
* **Pack key:** `pilots.grades: {<n>: {stat_bonus, lookahead_m, sprint}}`, hull `pilot_grade`.
* **Default:** same.
* **Test:** U — look-ahead per grade; generated stats include the bonus.

### RQ-C-24 Station shield pool on generator loss; outpost/OPP pools (P1) — #113

* **Original:** destroying the shield-generator module sets the station's pool to 0; a new generator
  starts full; outposts and OPPs recharge +2/s up to their chart's SD with HP max = HP1 (§8).
* **Pack key:** `stations.shield_on_generator_loss: drop`, outpost/OPP `hp`, `shield` from the charts.
* **Default:** same.
* **Test:** U — pool after generator destruction; outpost pool recharge cap.

## Order by player impact

1. RQ-C-01 (station hull).
2. RQ-C-02 and RQ-C-03 (shield pool and module protection).
3. RQ-C-05 (alien base hard points).
4. RQ-C-06 (turret fire control).
5. RQ-C-18, RQ-C-19 and RQ-C-20 (auto-resolve and attrition).
6. RQ-C-10 (pilot stats).
7. RQ-C-16 (engine failure).
8. RQ-C-11 (retreat when disarmed).

The rest follows.
