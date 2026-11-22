//! # The schemas of every mod file type
//!
//! | File | Schema |
//! |---|---|
//! | `pack.yaml` | [`pack`] — the manifest |
//! | `rules/*.yaml`, `campaign/*.yaml` | [`rules`] — definitions (ships, beams, behaviours…) and global settings |
//! | `scenarios/*.yaml` | [`scenario`] — a testbed arena |
//!
//! Each schema is built from [`super::schema`] types; the field `doc` strings appear in
//! editors (through the JSON Schema files) and on the docs site.

use super::schema::{f, Field, Ty, Unit};

fn name() -> Ty {
    Ty::Name
}
fn names() -> Ty {
    Ty::List(Box::new(Ty::Name))
}
fn dist() -> Ty {
    Ty::Qty(Unit::Distance)
}
fn time() -> Ty {
    Ty::Qty(Unit::Time)
}
fn rgb() -> Ty {
    Ty::Numbers(3)
}

/// `pack.yaml`.
pub fn pack() -> Ty {
    Ty::Struct(vec![
        f("id", name(), "unique id: a-z, 0-9, - and _"),
        f("name", Ty::Text, "shown in the launcher"),
        f("version", Ty::Text, "the pack's own version"),
        f("description", Ty::Text, "one or two sentences for the launcher"),
        f("kind", Ty::Enum(&["campaign", "addon", "library"]), "campaign, addon (default) or library"),
        f("parent", name(), "campaigns: the campaign this one extends"),
        f("targets", names(), "addons: campaigns it applies to (* = all)"),
        f("depends", names(), "packs that must load first"),
        f("requires_original", Ty::Bool, "true when the pack reads the player's Interceptor CD"),
        f("format", Ty::Num, "mod-format version the pack was written for"),
        f("authors", Ty::Text, "informational"),
        f("licence", Ty::Text, "informational"),
    ])
}

fn faction() -> Ty {
    Ty::Def(vec![
        f("name", Ty::Text, "display name"),
        f("color", rgb(), "radar and HUD colour, 0..1"),
        f("hostile", names(), "faction ids this faction fights"),
    ])
}

fn beam() -> Ty {
    Ty::Def(vec![
        f("name", Ty::Text, "display name"),
        f("damage", Ty::Num, "damage per hit"),
        f("energy_cost", Ty::Num, "energy per shot"),
        f("rate_of_fire", Ty::Num, "shots per second per mount"),
        f("speed", Ty::Num, "bolt speed, m/s"),
        f("range", dist(), "reach (2.4km, 2400m; a plain number is metres)"),
        f("shield_factor", Ty::Num, "shield points lost per point of damage they stop (1)"),
        f("color", rgb(), "bolt colour"),
        f("sound", Ty::Asset, "firing sound"),
    ])
}

fn missile() -> Ty {
    Ty::Def(vec![
        f("name", Ty::Text, "display name"),
        f("speed", Ty::Num, "top speed, m/s"),
        f("duration", time(), "flight time before self-destruct"),
        f("impact_damage", Ty::Num, "damage on a direct hit"),
        f("proximity_damage", Ty::Num, "damage from the proximity fuse"),
        f("proximity_radius", dist(), "proximity fuse radius"),
        f("mass", Ty::Num, "kg per round carried (flight.drive)"),
        f("lead_speed", Ty::Num, "homing leads the target as if it flew at this speed, m/s (0 = pure pursuit)"),
        f("turn_rate", Ty::Num, "degrees per second while homing"),
        f("lock_time", time(), "seconds in the lock cone to lock on"),
        f("color", rgb(), "trail colour"),
        f("sound", Ty::Asset, "launch sound"),
        f("explosion", name(), "effect id of its detonation"),
        f(
            "guidance",
            Ty::Enum(&["homing", "dumbfire", "sidestep", "parallax", "cloaked_wait", "cluster", "doppelganger"]),
            "how it steers (default homing)",
        ),
        f("damage", Ty::Enum(&["normal", "bypass_shields", "enveloping"]), "how the warhead's damage lands"),
        f("decoys", Ty::Num, "parallax: decoys released at launch (default 2)"),
        f(
            "cluster",
            Ty::Struct(vec![
                f("missile", name(), "submunition missile id"),
                f("count", Ty::Num, "how many"),
                f("at", dist(), "split this close to the target"),
            ]),
            "cluster: what it splits into",
        ),
        f("trigger_range", dist(), "cloaked_wait: wakes when a hostile comes this close"),
    ])
}

fn swaps() -> Ty {
    Ty::Dict(Box::new(Ty::Asset))
}

fn modelset() -> Ty {
    Ty::Def(vec![
        f("intact", Ty::Asset, "the normal model (all its LODs)"),
        f("scale", Ty::Num, "model units to metres"),
        f("textures", swaps(), "texture name: replacement, always applied"),
        f(
            "damage",
            Ty::List(Box::new(Ty::Struct(vec![
                f("below", Ty::Qty(Unit::Fraction), "applies while the hull is below this fraction"),
                f("textures", swaps(), "texture name: replacement"),
            ]))),
            "damage levels as the hull falls (cumulative)",
        ),
        f(
            "states",
            Ty::Dict(Box::new(Ty::Struct(vec![
                f("model", Ty::Asset, "a mesh replacing the intact one in this state (construction stages)"),
                f("debris", name(), "debris set used when destroyed in this state"),
                f("textures", swaps(), "texture name: replacement"),
            ]))),
            "named conditions: build_25, destroyed … (scenario objects pick one with state:)",
        ),
        f("debris", name(), "debris set id used when destroyed"),
        f("destroyed_effect", name(), "effect id of the explosion"),
    ])
}

fn debris() -> Ty {
    let pair = || Ty::List(Box::new(Ty::Num));
    Ty::Def(vec![
        f("pieces", Ty::List(Box::new(Ty::Asset)), "piece models, in the object's own model space"),
        f("pick", pair(), "[min, max] pieces thrown, chosen at random"),
        f("speed", pair(), "[min, max] outward speed, m/s"),
        f("spin", pair(), "[min, max] tumble, degrees per second"),
        f("life", time(), "how long the pieces last"),
        f(
            "chunks",
            Ty::Struct(vec![
                f("models", Ty::List(Box::new(Ty::Asset)), "generic rubble models"),
                f("count", Ty::Num, "how many"),
                f("speed", pair(), "[min, max] m/s"),
            ]),
            "generic rubble",
        ),
    ])
}

fn effect() -> Ty {
    Ty::Def(vec![
        f("size", Ty::Num, "fireball size: × the object's radius (missiles: metres)"),
        f("duration", time(), "how long it lasts"),
        f("color", rgb(), "fireball colour"),
        f("shockwave", Ty::Bool, "an expanding ring of light"),
        f("sound", Ty::Text, "sound event name or asset"),
    ])
}

/// Fields shared by `ships` and `objects`.
fn ship() -> Ty {
    Ty::Def(vec![
        f("name", Ty::Text, "display name"),
        f("description", Ty::Text, "one line for the viewer and encyclopaedia"),
        f(
            "kind",
            Ty::Enum(&["fighter", "freighter", "capital", "drone", "station", "probe", "mine", "debris", "ground"]),
            "what sort of object it is",
        ),
        f("faction", name(), "owning faction id"),
        f("model_set", name(), "the look: a model set id (instead of model and debris)"),
        f("model", Ty::Asset, "model reference: models/x.obj or original:MESH (shorthand for a model set)"),
        f("model_scale", Ty::Num, "model units to metres"),
        f("playable", Ty::Bool, "the player may fly it"),
        f("cockpit", name(), "cockpit id for the player's view"),
        f("voice", name(), "the radio voice set its pilot speaks with (voices.<id>)"),
        f("hit_points", Ty::Num, "hull points"),
        f("armor", Ty::Num, "armour points (absorbed after shields)"),
        f("shields", Ty::Num, "shield points per facing (fore and aft)"),
        f(
            "fitted",
            Ty::Struct(vec![
                f("shield", Ty::Asset, "the shield it carries: protection from that definition"),
                f("armor", Ty::Asset, "the armour it carries"),
                f("engine", Ty::Asset, "the engine it carries: its special is the thrust in kN"),
            ]),
            "default equipment whose protection sets shields/armor (e.g. an original: section)",
        ),
        f(
            "shield_recharge",
            Ty::Num,
            "shield points per second at full charge (per facing, or shared with power.balance: share)",
        ),
        f(
            "protection_scale",
            Ty::Struct(vec![
                f("shields", Ty::Num, "multiplies each shield bank (1)"),
                f("armor", Ty::Num, "multiplies each armour bank (1)"),
            ]),
            "a class that holds more than its fitted equipment gives",
        ),
        f("mass", Ty::Num, "hull mass in kg, before shields, missiles and systems (flight.drive)"),
        f("thrust", Ty::Num, "engine thrust in kN (flight.drive; or fitted: {engine: …})"),
        f(
            "ai_speed",
            Ty::Either(
                Box::new(Ty::Num),
                Box::new(Ty::Struct(vec![
                    f("min", Ty::Num, "lowest factor"),
                    f("max", Ty::Num, "highest factor (each ship rolls one in between)"),
                    f("level_step", Ty::Num, "added per difficulty level"),
                ])),
            ),
            "AI drive factor S (flight.drive): acceleration S × throttle, top speed S / drag",
        ),
        f(
            "ai_turn",
            Ty::Struct(vec![
                f("pitch", Ty::Num, "radians per second"),
                f("yaw", Ty::Num, "radians per second"),
                f("roll", Ty::Num, "radians per second"),
            ]),
            "AI pilots of this class turn at these fixed rates, with no damping",
        ),
        f("max_speed", Ty::Num, "top speed, m/s"),
        f("acceleration", Ty::Num, "m/s²"),
        f("roll_rate", Ty::Num, "degrees per second"),
        f("pitch_rate", Ty::Num, "degrees per second at rest"),
        f("yaw_rate", Ty::Num, "degrees per second at rest"),
        f("angular_damp", Ty::Num, "0 tight … 10 loose"),
        f("lock_speed", Ty::Num, "speed at which pitch and yaw fall to zero, m/s"),
        f("vct", Ty::Bool, "fitted with vector control thrusters"),
        f("atmosphere_max_speed", Ty::Num, "top speed in air of density 1 (default 60 % of max_speed)"),
        f("lift", Ty::Num, "forward speed at which the wings carry the craft's weight; 0 = no wings"),
        f("hover", Ty::Bool, "hovers: vertical thrust cancels gravity"),
        f("atmospheric", Ty::Bool, "can enter an atmosphere (default true)"),
        f("energy", Ty::Num, "energy pool for beams, afterburner and cloak (default power.energy)"),
        f("energy_recharge", Ty::Num, "pool recharge per second at full beam charge (default power.energy_recharge)"),
        f("beams", names(), "one beam id per mount"),
        f("missiles", Ty::Dict(Box::new(Ty::Num)), "missile id: count, one rack each"),
        f("systems", names(), "fitted systems (replaces the default list)"),
        f("repair_rate", Ty::Num, "hit points per second the crew repairs"),
        f("capturable", Ty::Bool, "can be captured with a tractor beam when disabled"),
        f(
            "towable",
            Ty::Bool,
            "can be towed away even when not disabled (cargo containers); a mounted one is torn off",
        ),
        f("cargo", Ty::Bool, "a cargo container: counted by the cargo(role) mission query"),
        f("behaviour", name(), "AI behaviour id"),
        f("turret_arc", Ty::Qty(Unit::Angle), "beams can aim this far off the nose (turrets)"),
        f("debris", Ty::List(Box::new(Ty::Asset)), "wreckage models"),
        f("engine_color", rgb(), "engine glow colour"),
        f(
            "modules",
            Ty::List(Box::new(Ty::Struct(vec![
                f("slot", Ty::Text, "attachment point: its index (original meshes) or slot name (OBJ slot_<name>)"),
                f("class", name(), "the module's object id"),
            ]))),
            "stations: modules mounted at the model's attachment points",
        ),
        f("shield_generator", Ty::Bool, "modules: while one survives, its station's shields work"),
        f(
            "shield_capacity",
            Ty::Num,
            "generator modules: the hub's shield_pool maximum while it is the first standing generator",
        ),
        f(
            "shield_pool",
            Ty::Struct(vec![
                f("recharge", Ty::Num, "points per second while a generator stands (default 2)"),
                f(
                    "on_generator_lost",
                    Ty::Enum(&["keep", "drop"]),
                    "keep the points without recharging, or drop to 0 (default drop)",
                ),
            ]),
            "stations: one shield pool instead of fore and aft banks",
        ),
        f("shielded_by_hub", Ty::Bool, "modules: hits drain the hub's shield_pool while it holds points"),
        f(
            "damage_model",
            Ty::Enum(&["ship", "station"]),
            "station: hits past the shields go straight to the hull (no armour, difficulty factor or system hits)",
        ),
        f(
            "point_defence",
            Ty::Struct(vec![
                f("range", dist(), "reach (default 800m)"),
                f("rate", Ty::Num, "shots per second (default 3)"),
                f("chance", Ty::Qty(Unit::Fraction), "chance a shot destroys the missile (default 35%)"),
                f("energy", Ty::Num, "energy per shot (default 4)"),
            ]),
            "guns that shoot down hostile missiles",
        ),
        f(
            "probe",
            Ty::Struct(vec![
                f("sensors", dist(), "its side's radar sees every contact this close to the probe"),
                f("jamming", dist(), "hostile ships this close have half radar range and lock missiles at half speed"),
            ]),
            "sensor and jamming probes",
        ),
    ])
}

fn cockpit() -> Ty {
    Ty::Def(vec![
        f("image", Ty::Asset, "2D cockpit art (or none)"),
        f("design", Ty::Numbers(2), "width and height of the instrument coordinate space"),
        f("model", Ty::Asset, "3D cockpit model around the pilot's eye"),
        f("model_offset", Ty::Numbers(3), "metres from the eye: x right, y up, -z ahead"),
        f("model_scale", Ty::Num, "model units to metres"),
        f(
            "instruments",
            Ty::Dict(Box::new(Ty::Numbers(4))),
            "instrument name: [x, y, width, height] in the design space",
        ),
        f(
            "anchors",
            Ty::Dict(Box::new(Ty::Struct(vec![
                f("at", Ty::Numbers(3), "anchor point on the cockpit model, metres"),
                f("normal", Ty::Numbers(3), "the surface's outward normal"),
                f("size", Ty::Numbers(2), "width and height on the surface, metres"),
            ]))),
            "3D cockpits: instruments drawn in perspective on the model's surface",
        ),
        f(
            "wide",
            Ty::Struct(vec![
                f("blackout", Ty::Bool, "cover the bands beside the cockpit art (hiding the 3D view there)"),
                f("color", rgb(), "the bands' colour (default black)"),
                f("from", Ty::Num, "design-space y where the bands start: 0 = full height"),
                f("left", Ty::Asset, "extension art stretched over the left band (your own image)"),
                f("right", Ty::Asset, "extension art stretched over the right band (your own image)"),
            ]),
            "wide screens: the side bands beside a narrower cockpit (R-HUD-013)",
        ),
        f(
            "ship_status",
            Ty::Struct(vec![
                f(
                    "rings",
                    Ty::Numbers(3),
                    "radius of each shield band's centre, in lighting order (fractions of the half-size)",
                ),
                f("width", Ty::Num, "band thickness (fraction of the half-size)"),
                f("tilt", Ty::Num, "counter-clockwise rotation of the display in degrees"),
                f("arc", Ty::Num, "degrees each half (fore, aft) spans; 180 = no side gaps"),
                f("craft", Ty::Num, "size of the craft silhouette (fraction of the half-size)"),
            ]),
            "where the ship_status instrument's shield bands lie on the cockpit art",
        ),
    ])
}

fn system() -> Ty {
    Ty::Def(vec![
        f(
            "kind",
            Ty::Enum(&[
                "engines",
                "manoeuvring",
                "sensors",
                "shields",
                "hyperdrive",
                "vct",
                "tractor",
                "cloak",
                "ecm",
                "damage_control",
            ]),
            "the built-in effect (defaults to the id)",
        ),
        f("name", Ty::Text, "display name"),
        f("hit_points", Ty::Num, "damage before it stops working"),
        f("power", Ty::Num, "power steps while switched on"),
        f("range", dist(), "tractor hold range, ECM radius, cloak detection range"),
        f("strength", Ty::Num, "tractor pull m/s², ECM/cloak break chance % per second"),
        f("repair_rate", Ty::Num, "damage control: repair speed multiplier"),
        f("mass", Ty::Num, "kg it adds to the ship (flight.drive)"),
        f("lock_time", time(), "sensors: missile lock time (0 = each missile's lock_time)"),
        f("lock_angle", Ty::Qty(Unit::Angle), "sensors: lock cone half-angle (0 = flight.lock_angle)"),
        f("lock_range", dist(), "sensors: lock range (0 = the missile's reach)"),
    ])
}

fn mission() -> Ty {
    let at = || {
        Ty::Struct(vec![
            f("distance", dist(), "from the reference point"),
            f(
                "bearing",
                Ty::Either(Box::new(Ty::Numbers(2)), Box::new(Ty::Enum(&["random"]))),
                "[yaw, pitch] in degrees from straight ahead, or random",
            ),
            f("from", name(), "a role to place it relative to (default: the player's start)"),
            f("spread", dist(), "how far apart the members are (default 120m)"),
        ])
    };
    let count = || Ty::Either(Box::new(Ty::Num), Box::new(Ty::Numbers(2)));
    let role = Ty::Struct(vec![
        f(
            "class",
            Ty::Either(Box::new(name()), Box::new(names())),
            "a class id, or a list: each member picks one at random",
        ),
        f("count", count(), "how many: 3, or [min, max]"),
        f("faction", name(), "faction id (default: the class's)"),
        f("ai", Ty::Enum(&["enemy", "wingman", "target", "idle"]), "how it is flown (default enemy)"),
        f("behaviour", name(), "behaviour id instead of the class's"),
        f("protect", Ty::Bool, "an objective the player must keep alive"),
        f("state", name(), "model-set state (a construction stage)"),
        f("hull", Ty::Qty(Unit::Fraction), "starting hull"),
        f("destroyed_systems", names(), "systems that start destroyed"),
        f("formation", name(), "wingmen and escorts: the formation they fly in (formations.<id>)"),
        f("escort", name(), "escorts: the role whose newest member they fly in formation on (a tug)"),
        f("speed", Ty::Num, "starting speed, m/s"),
        f("at", at(), "where it enters"),
        f("arrive", Ty::Condition, "when it enters (default: at the start)"),
    ]);
    Ty::Def(vec![
        f("name", Ty::Text, "shown in the launcher (a @key)"),
        f("briefing", Ty::Text, "the briefing text (a @key)"),
        f("order", Ty::Num, "sort key in the launcher"),
        f("cargo", name(), "the role whose cargo containers count: the debrief reports the share still aboard"),
        f("starfield_seed", Ty::Num, "the star field"),
        f("environment", Ty::Name, "the battle environment (environments.<id>; default open space)"),
        f("hazards", super::hazards::schema(), "asteroids, a minefield, a black hole, an ion storm"),
        f("sun", Ty::Numbers(3), "direction of the sun"),
        f(
            "player",
            Ty::Struct(vec![
                f("class", name(), "the player's ship class, or choose"),
                f("position", Ty::Numbers(3), "the player's start"),
            ]),
            "the player's craft",
        ),
        f("roles", Ty::Dict(Box::new(role)), "role name: a group of craft"),
        f(
            "waves",
            Ty::List(Box::new(Ty::Struct(vec![
                f("role", name(), "the role that gets more craft"),
                f("when", Ty::Condition, "when (once, unless every:)"),
                f("count", count(), "how many (default: the role's count)"),
                f("at", at(), "where (default: the role's arrival)"),
                f("every", time(), "comes again this long after it last came, while when holds"),
            ]))),
            "reinforcements",
        ),
        f(
            "objectives",
            Ty::Struct(vec![
                f("win", Ty::Condition, "the player wins when this holds"),
                f("lose", Ty::Condition, "the player loses when this holds (checked first)"),
                f("draw", Ty::Condition, "a draw"),
            ]),
            "how the battle ends",
        ),
        f("optional", Ty::Dict(Box::new(Ty::Condition)), "optional objectives, reported on the debrief"),
        f(
            "timer",
            Ty::Struct(vec![
                f("after", time(), "battle time"),
                f("then", Ty::Enum(&["win", "lose", "draw", "retreat"]), "the result when it runs out"),
            ]),
            "a time limit",
        ),
        f(
            "outcomes",
            Ty::Dict(Box::new(Ty::List(Box::new(Ty::Text)))),
            "result: actions (score(x), salvage, relation(f, +n) …)",
        ),
        f(
            "chatter",
            Ty::List(Box::new(Ty::Struct(vec![
                f("when", Ty::Condition, "said once when this first holds"),
                f("line", Ty::Text, "the line (a @key)"),
                f("voice", Ty::Asset, "a radio clip spoken with the line"),
                f("speaker", name(), "who says it (voices.<id>): its name and face on the comms panel"),
            ]))),
            "radio lines",
        ),
        f("debrief", Ty::Dict(Box::new(Ty::Text)), "result: debrief text (a @key)"),
    ])
}

fn behaviour() -> Ty {
    const MANOEUVRES: &[&str] =
        &["idle", "circle", "patrol", "attack", "evade", "retreat", "turret", "tow", "escort", "travel", "mine"];
    const DO: &[&str] = &[
        "idle", "circle", "patrol", "attack", "evade", "retreat", "turret", "tow", "escort", "travel", "mine", "script",
    ];
    let state = Ty::Struct(vec![
        f("do", Ty::Enum(DO), "the manoeuvre, or script (a Tier 2 script decides; see call:)"),
        f("call", Ty::Text, "do: script: the function to run, scripts/<file>.ois:<function> (object context)"),
        f("every", time(), "do: script: seconds between runs (0.5 s)"),
        f("fly", Ty::Enum(MANOEUVRES), "do: script: the manoeuvre flown meanwhile (attack)"),
        f("range", dist(), "attack: open fire inside; patrol: radius; turret: reach; mine: trigger distance"),
        f("fire_cone", Ty::Qty(Unit::Angle), "fire when the aim point is within this angle of the nose"),
        f("burst", time(), "seconds of firing"),
        f("pause", time(), "seconds holding fire between bursts"),
        f("duration", time(), "evade: how long"),
        f("speed", Ty::Qty(Unit::Fraction), "fraction of top speed"),
        f("missiles", time(), "seconds between missile launches (0 = none)"),
        f("missile_range", dist(), "launch missiles only this close (default 2.5 × range, at least 2 km)"),
        f("jump", Ty::Bool, "retreat: start the hyperdrive"),
        f("escape", dist(), "tow: haul the prize this far before jumping out (2.5 km)"),
    ]);
    Ty::Def(vec![
        f("start", name(), "the first state"),
        f("vars", names(), "object variables (at most 8): read with var(name), set by scripts with set_var"),
        f("states", Ty::Dict(Box::new(state)), "state name: manoeuvre and parameters"),
        f(
            "transitions",
            Ty::List(Box::new(Ty::Struct(vec![
                f("from", Ty::Text, "state name, or * for any"),
                f("to", name(), "state name"),
                f("when", Ty::Condition, "condition (object context)"),
            ]))),
            "tried in order; the first whose condition holds wins",
        ),
        f(
            "systems",
            Ty::Struct(vec![
                f("cloak", Ty::Condition, "cloak on while this holds, off otherwise"),
                f("ecm", Ty::Condition, "ECM on while this holds, off otherwise"),
                f("shields_first", Ty::Condition, "while this holds, move power to shields"),
                f("beams_first", Ty::Condition, "while this holds (and shields_first does not), move power to beams"),
            ]),
            "when the pilot uses its ship's systems",
        ),
        f(
            "prefer",
            Ty::Dict(Box::new(Ty::Num)),
            "target preferences: player, a kind (fighter, station…), class:<id> or faction:<id> → weight (2 = as if half as far)",
        ),
        f(
            "skill",
            Ty::Struct(vec![
                f("aim", Ty::Qty(Unit::Fraction), "0 wild … 1 perfect"),
                f("reaction", time(), "seconds between looks at the target"),
                f(
                    "accuracy",
                    Ty::Qty(Unit::Fraction),
                    "the original's aim model instead of aim: errs in the lead time by 1 − accuracy",
                ),
                f("lead_range", dist(), "beyond this, aim straight at the target without lead (0 = always lead)"),
            ]),
            "how well the pilot flies and shoots",
        ),
    ])
}

fn flight() -> Ty {
    Ty::Def(vec![
        f("model", Ty::Enum(&["newtonian", "assisted"]), "flight model"),
        f("arena_radius", dist(), "ships are halted at this distance from the centre (0 = none)"),
        f("boost", Ty::Num, "afterburner speed bonus, m/s"),
        f("cost", Ty::Num, "afterburner energy per second"),
        f("lock_angle", Ty::Qty(Unit::Angle), "missile lock cone half-angle"),
        f("radar_range", dist(), "radar range"),
        f("hyperspace_time", time(), "hyperdrive countdown"),
        f(
            "radar_colors",
            Ty::Struct(vec![
                f("friendly", rgb(), "the player's own faction"),
                f("hostile", rgb(), "factions at war with the player's"),
                f("neutral", rgb(), "everyone else"),
                f("objective", rgb(), "objects the player must protect (scenario protect: true)"),
            ]),
            "radar and target-marker colours by relation to the player",
        ),
        f(
            "bolt_frame",
            Ty::Enum(&["world", "ship"]),
            "world: shots fly straight once fired (physics); ship: they turn with the nose for bolt_carry",
        ),
        f("bolt_carry", time(), "with bolt_frame: ship, how long a shot stays in the ship's frame"),
        f(
            "bolt_inherit",
            Ty::Bool,
            "shots leave with the ship's velocity plus muzzle speed (true) or muzzle speed alone",
        ),
        f("lock_decay", Ty::Bool, "off target, a part-built lock runs down instead of being lost at once (false)"),
        f("lock_hold", Ty::Num, "once locked, the lock cone is this many times wider (1)"),
        f(
            "lighting",
            Ty::Struct(vec![
                f("ambient", Ty::Num, "light everywhere (0.30)"),
                f("sun", Ty::Num, "extra light on faces towards the sun (0.85)"),
                f("rim", Ty::Num, "extra light on edges seen side-on, outlining hulls (0)"),
                f(
                    "distant_dot",
                    Ty::Num,
                    "pixels: smaller ships are drawn as a dot in their radar colour (4; 0 = off)",
                ),
            ]),
            "how ships are lit",
        ),
        f(
            "collision",
            Ty::Struct(vec![
                f("restitution", Ty::Num, "bounciness, 0 (stop together) to 1 (perfect bounce); 0.5"),
                f("spin", Ty::Num, "how strongly the angle of impact spins a ship; 1"),
                f("tumble", time(), "longest time a hard hit leaves the pilot without full control; 2.5s"),
            ]),
            "how collisions feel",
        ),
        f(
            "debrief_delay",
            Ty::Qty(Unit::Time),
            "the battle runs on this long after the result before the debrief; 30s",
        ),
        f("formation", Ty::Name, "the formation wingmen fly in when a role names none (formations.<id>; vee)"),
        f("formation_gap", Ty::Num, "wingman slots are at least this many x the combined radii from the leader; 3"),
        f(
            "drive",
            Ty::Struct(vec![
                f("drag", Ty::Num, "dv/dt = a − drag × v: ships settle at a / drag and coast to rest (2)"),
                f("thrust_scale", Ty::Num, "newtons per kN of thrust at full throttle (1000)"),
                f("power_cost", Ty::Qty(Unit::Fraction), "engine power lost per beam/shield level and system step (0)"),
                f("reverse", Ty::Qty(Unit::Fraction), "reverse thrust as a share of forward (50 %)"),
                f("afterburner", Ty::Qty(Unit::Fraction), "afterburner thrust as a share of full thrust (50 %)"),
                f("shield_mass", Ty::Num, "kilograms a fitted shield adds (0)"),
                f("ai_speed_range", Ty::Numbers(2), "AI drive factors are held to this range ([100, 500])"),
            ]),
            "the original's motion model: thrust, mass and drag for the player, a drive factor for AI ships",
        ),
        f(
            "turning",
            Ty::Struct(vec![
                f("gain", Ty::Num, "stick to turn request per unit of hull rate (0.08)"),
                f("damping", Ty::Num, "response at AngularDamp 0, 1/s (4)"),
                f("damp_step", Ty::Num, "how much each point of AngularDamp slows the response (0.1)"),
                f(
                    "ai_rates",
                    Ty::Struct(vec![
                        f("pitch", Ty::Num, "radians per second (2)"),
                        f("yaw", Ty::Num, "radians per second (2)"),
                        f("roll", Ty::Num, "radians per second (3)"),
                    ]),
                    "AI turn rates for classes without ai_turn",
                ),
            ]),
            "the original's turn model: hull turn rates in its own units, AI at fixed rates",
        ),
    ])
}

fn console_layer() -> Ty {
    Ty::Struct(vec![
        f(
            "colors",
            Ty::Struct(vec![
                f("bg", rgb(), "page background"),
                f("panel", rgb(), "panel background"),
                f("line", rgb(), "borders, and the fill of a pressed button"),
                f("ink", rgb(), "text"),
                f("dim", rgb(), "labels and secondary text"),
                f("hot", rgb(), "values and highlights"),
                f("warn", rgb(), "warnings (amber)"),
                f("danger", rgb(), "danger (red)"),
                f("off", rgb(), "features not fitted"),
            ]),
            "colours (RGB 0..1)",
        ),
        f("font", Ty::Text, "a CSS font list, e.g. \"Futura, 'Avenir Next', sans-serif\""),
        f("corner", Ty::Num, "panel corner radius in pixels"),
    ])
}

fn console_theme() -> Ty {
    Ty::Def(vec![
        f("theme", console_layer(), "the campaign's console theme"),
        f(
            "cockpits",
            Ty::Dict(Box::new(console_layer())),
            "per cockpit id: overrides while flying a craft with that cockpit",
        ),
    ])
}

fn power() -> Ty {
    Ty::Def(vec![
        f("beam_recharge", Ty::List(Box::new(Ty::Num)), "per level A, B, …: fraction of the ship's energy_recharge"),
        f("shield_recharge", Ty::List(Box::new(Ty::Num)), "per level: fraction of the ship's shield_recharge"),
        f("budget", Ty::Num, "power steps shared by both settings and switched-on systems"),
        f("start_beam", Ty::Num, "starting beam level (0 = A)"),
        f("start_shield", Ty::Num, "starting shield level"),
        f("balance_shift", Ty::Qty(Unit::Fraction), "with balance: shift, how much shield balance shifts capacity"),
        f(
            "balance",
            Ty::Enum(&["shift", "share"]),
            "shift: balance moves shield capacity; share: it sends the recharge to one facing first (the original)",
        ),
        f("energy", Ty::Num, "energy pool of classes without their own energy (500)"),
        f(
            "energy_recharge",
            Ty::Num,
            "pool recharge at the top beam level, per second, for classes without their own (40)",
        ),
    ])
}

fn damage() -> Ty {
    Ty::Def(vec![
        f(
            "system_share",
            Ty::Qty(Unit::Fraction),
            "share of the damage reaching the hull that strikes a system (100 %)",
        ),
        f(
            "system_below",
            Ty::Qty(Unit::Fraction),
            "systems are struck only while the hull is below this share (100 %)",
        ),
        f("system_divert", Ty::Bool, "true: the system share is taken off the hull damage; false: dealt as well"),
        f("spread", Ty::Qty(Unit::Fraction), "each hit does between this share and all of its damage (100 %)"),
        f("armor", Ty::Enum(&["pool", "per_side"]), "one armour pool, or a fore and an aft bank each holding it all"),
        f(
            "difficulty",
            Ty::Struct(vec![
                f("side", Ty::Name, "the faction whose ships use the own_* factors"),
                f(
                    "own_shields",
                    Ty::List(Box::new(Ty::Qty(Unit::Fraction))),
                    "per level, easiest first: that side's shields",
                ),
                f("own_armor", Ty::List(Box::new(Ty::Qty(Unit::Fraction))), "per level: that side's armour"),
                f("other_shields", Ty::List(Box::new(Ty::Qty(Unit::Fraction))), "per level: everyone else's shields"),
                f("other_armor", Ty::List(Box::new(Ty::Qty(Unit::Fraction))), "per level: everyone else's armour"),
                f(
                    "player_hits",
                    Ty::List(Box::new(Ty::Qty(Unit::Fraction))),
                    "per level: beam damage the player's craft deals",
                ),
                f(
                    "hits_on_player",
                    Ty::List(Box::new(Ty::Qty(Unit::Fraction))),
                    "per level: beam damage the player's craft takes",
                ),
            ]),
            "protection and damage factors by difficulty level (a short list gives 100 % for the rest)",
        ),
        f("destroy_below", Ty::Qty(Unit::Fraction), "a system this far below zero is destroyed"),
        f("repair_rate", Ty::Num, "default crew repair rate, hit points per second"),
    ])
}

fn campaign() -> Ty {
    Ty::Def(
        vec![
            f("name", Ty::Text, "campaign title"),
            f("difficulty", names(), "difficulty level ids, easiest first"),
            f("default_difficulty", name(), "the level a new game starts at"),
            f("skin", name(), "the skin of the menus and strategy screens (skins.<id>)"),
            f("intro", name(), "the cutscene played before a new campaign (cutscenes/<id>)"),
            f("intro_video", Ty::Asset, "a video played before a new campaign"),
            f(
                "strategy_screen",
                name(),
                "the first of the original-style strategy screens (screens.<id>); none: the testbed layout",
            ),
        ]
        .into_iter()
        .chain(super::front::fields())
        .collect(),
    )
}

/// The top-level sections of a rules file.
pub fn rules_sections() -> Vec<Field> {
    let dict = |t: Ty| Ty::Dict(Box::new(t));
    vec![
        f("factions", dict(faction()), "factions by id"),
        f("beams", dict(beam()), "beam weapons by id"),
        f("missiles", dict(missile()), "missiles by id"),
        f("ships", dict(ship()), "craft by id"),
        f("objects", dict(ship()), "stations and other objects by id (kind defaults to station)"),
        f("systems", dict(system()), "ship systems by id"),
        f("cockpits", dict(cockpit()), "cockpits by id"),
        f("behaviours", dict(behaviour()), "AI behaviours by id"),
        f("missions", dict(mission()), "mission types by id"),
        f(
            "voices",
            dict(Ty::Dict(Box::new(Ty::Either(Box::new(Ty::List(Box::new(Ty::Asset))), Box::new(Ty::Asset))))),
            "radio voice sets: line (acknowledge, status, down, engaging, kill): clip or clips; name: the speaker; face: a picture or Smacker clip for the comms panel",
        ),
        f("environments", dict(super::environment::environment_schema()), "battle environments by id"),
        f("formations", dict(super::formation::formation_schema()), "formations wingmen (and cutscenes) fly in"),
        f("music", dict(super::music::schema()), "interactive music sets"),
        f("skins", dict(super::skin::schema()), "skins for the menus and strategy screens"),
        f("screens", dict(super::screens::schema()), "screen layouts: design space and widget rectangles"),
        f("terrains", dict(super::environment::terrain_schema()), "terrains (heightfields) by id"),
        f("modelsets", dict(modelset()), "looks: models, damage textures, states, debris"),
        f("debris", dict(debris()), "debris sets by id"),
        f("effects", dict(effect()), "explosions and other effects by id"),
        f("flight", flight(), "flight model and sensors"),
        f("console", console_theme(), "the tablet console's look: colours, font, corners; per cockpit"),
        f("power", power(), "beam/shield charge settings"),
        f("damage", damage(), "damage to systems and repair"),
        f("sounds", dict(Ty::Asset), "event name: sound (an asset, or cue:<Name> from the cue table)"),
        f(
            "audio",
            Ty::Def(vec![f(
                "cue_table",
                Ty::Asset,
                "a table of named sound cues: original:<file>/<section> (the original's soundfx.ini)",
            )]),
            "sound cue table",
        ),
        f("campaign", campaign(), "the campaign ruleset"),
    ]
    .into_iter()
    .chain(super::strategy::sections())
    .collect()
}

/// A rules file (`rules/*.yaml`, `campaign/*.yaml`).
pub fn rules() -> Ty {
    Ty::Struct(rules_sections())
}

/// The definition schema of a rules section (`ships` → the ship definition).
pub fn section(name: &str) -> Option<Ty> {
    let ty = rules_sections().into_iter().find(|fd| fd.name == name)?.ty;
    Some(match ty {
        Ty::Dict(inner) => *inner,
        other => other,
    })
}

/// A scenario file (`scenarios/*.yaml`).
pub fn scenario() -> Ty {
    let object = Ty::Struct(vec![
        f("class", name(), "ship or object id"),
        f("faction", name(), "overrides the class's faction"),
        f("position", Ty::Numbers(3), "metres; -z is ahead of the player's start"),
        f("rotation", Ty::Numbers(3), "yaw, pitch, roll in degrees"),
        f("ai", Ty::Enum(&["wingman", "enemy", "idle", "target"]), "how it is flown"),
        f("leader", name(), "wingmen: object to follow (player = the player)"),
        f("offset", Ty::Numbers(3), "wingmen: formation slot relative to the leader"),
        f("speed", Ty::Num, "starting speed, m/s"),
        f("destroyed_systems", names(), "systems (kinds or ids) that start destroyed"),
        f("state", name(), "model-set state to show it in (a construction stage such as build_60)"),
        f(
            "protect",
            Ty::Bool,
            "an objective the player must keep alive: marked on the HUD, skipped by nearest-enemy targeting",
        ),
        f("hull", Ty::Qty(Unit::Fraction), "starting hull fraction"),
        f("behaviour", name(), "behaviour id instead of the class's"),
    ]);
    Ty::Struct(vec![
        f("id", name(), "unique scenario id"),
        f("title", Ty::Text, "launcher title"),
        f("description", Ty::Text, "launcher description"),
        f("order", Ty::Num, "launcher sort key (lower first)"),
        f("player", Ty::Text, "ship id, or choose"),
        f("player_position", Ty::Numbers(3), "metres"),
        f("player_rotation", Ty::Numbers(3), "yaw, pitch, roll in degrees"),
        f("sun", Ty::Numbers(3), "direction towards the sun"),
        f("starfield_seed", Ty::Num, "background stars"),
        f("environment", Ty::Name, "the battle environment (environments.<id>; default open space)"),
        f("hazards", super::hazards::schema(), "asteroids, a minefield, a black hole, an ion storm"),
        f("music", Ty::Asset, "music"),
        f("objects", Ty::Dict(Box::new(object)), "placed objects by name"),
        f("nav", Ty::Dict(Box::new(Ty::Struct(vec![f("position", Ty::Numbers(3), "metres")]))), "navigation points"),
    ])
}

/// A text file (`text/<language>/*.yaml`): strings by key, and original language files
/// imported by reference.
pub fn text() -> Ty {
    Ty::Struct(vec![
        f("text", Ty::Dict(Box::new(Ty::Text)), "text key: the string (\"@key\" in rules refers to it)"),
        f(
            "imports",
            Ty::List(Box::new(Ty::Struct(vec![
                f("file", Ty::Asset, "an original language file: original:LangFile1.txt"),
                f("prefix", name(), "keys are imported as <prefix>.<KEY> (default orig)"),
            ]))),
            "the original's language files, read from the player's CD",
        ),
    ])
}

/// Every file type's JSON Schema: `(file name, contents)`.
pub fn json_schemas() -> Vec<(&'static str, String)> {
    use super::schema::json_schema;
    vec![
        ("pack.schema.json", json_schema("OpenInterceptor pack manifest (pack.yaml)", &pack())),
        ("rules.schema.json", json_schema("OpenInterceptor rules file (rules/*.yaml, campaign/*.yaml)", &rules())),
        ("scenario.schema.json", json_schema("OpenInterceptor scenario (scenarios/*.yaml)", &scenario())),
        (
            "cutscene.schema.json",
            json_schema("OpenInterceptor cutscene (cutscenes/*.yaml)", &super::cutscene::schema()),
        ),
        ("text.schema.json", json_schema("OpenInterceptor text (text/<language>/*.yaml)", &text())),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::schema::json_schema;

    #[test]
    fn every_schema_builds_and_exports() {
        for (name, ty) in [("pack", pack()), ("rules", rules()), ("scenario", scenario()), ("text", text())] {
            let j = json_schema(name, &ty);
            assert_eq!(j.matches('{').count(), j.matches('}').count(), "{name}");
        }
        assert!(section("ships").is_some() && section("flight").is_some() && section("nope").is_none());
    }
}
