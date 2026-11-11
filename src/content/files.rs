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
            Ty::Dict(Box::new(Ty::Struct(vec![f("textures", swaps(), "texture name: replacement")]))),
            "named conditions: building, destroyed …",
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
            Ty::Enum(&["fighter", "freighter", "capital", "drone", "station", "probe", "mine", "debris"]),
            "what sort of object it is",
        ),
        f("faction", name(), "owning faction id"),
        f("model_set", name(), "the look: a model set id (instead of model and debris)"),
        f("model", Ty::Asset, "model reference: models/x.obj or original:MESH (shorthand for a model set)"),
        f("model_scale", Ty::Num, "model units to metres"),
        f("playable", Ty::Bool, "the player may fly it"),
        f("cockpit", name(), "cockpit id for the player's view"),
        f("hit_points", Ty::Num, "hull points"),
        f("armor", Ty::Num, "armour points (absorbed after shields)"),
        f("shields", Ty::Num, "shield points per facing (fore and aft)"),
        f("shield_recharge", Ty::Num, "shield points per second per facing at full charge"),
        f("max_speed", Ty::Num, "top speed, m/s"),
        f("acceleration", Ty::Num, "m/s²"),
        f("roll_rate", Ty::Num, "degrees per second"),
        f("pitch_rate", Ty::Num, "degrees per second at rest"),
        f("yaw_rate", Ty::Num, "degrees per second at rest"),
        f("angular_damp", Ty::Num, "0 tight … 10 loose"),
        f("lock_speed", Ty::Num, "speed at which pitch and yaw fall to zero, m/s"),
        f("vct", Ty::Bool, "fitted with vector control thrusters"),
        f("energy", Ty::Num, "beam capacitor"),
        f("energy_recharge", Ty::Num, "capacitor recharge per second at full beam charge"),
        f("beams", names(), "one beam id per mount"),
        f("missiles", Ty::Dict(Box::new(Ty::Num)), "missile id: count, one rack each"),
        f("systems", names(), "fitted systems (replaces the default list)"),
        f("repair_rate", Ty::Num, "hit points per second the crew repairs"),
        f("capturable", Ty::Bool, "can be captured with a tractor beam when disabled"),
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
            "point_defence",
            Ty::Struct(vec![
                f("range", dist(), "reach (default 800m)"),
                f("rate", Ty::Num, "shots per second (default 3)"),
                f("chance", Ty::Qty(Unit::Fraction), "chance a shot destroys the missile (default 35%)"),
                f("energy", Ty::Num, "energy per shot (default 4)"),
            ]),
            "guns that shoot down hostile missiles",
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
    ])
}

fn behaviour() -> Ty {
    let state = Ty::Struct(vec![
        f(
            "do",
            Ty::Enum(&["idle", "circle", "patrol", "attack", "evade", "retreat", "turret", "tow", "mine"]),
            "the manoeuvre",
        ),
        f("range", dist(), "attack: open fire inside; patrol: radius; turret: reach; mine: trigger distance"),
        f("fire_cone", Ty::Qty(Unit::Angle), "fire when the aim point is within this angle of the nose"),
        f("burst", time(), "seconds of firing"),
        f("pause", time(), "seconds holding fire between bursts"),
        f("duration", time(), "evade: how long"),
        f("speed", Ty::Qty(Unit::Fraction), "fraction of top speed"),
        f("missiles", time(), "seconds between missile launches (0 = none)"),
        f("jump", Ty::Bool, "retreat: start the hyperdrive"),
    ]);
    Ty::Def(vec![
        f("start", name(), "the first state"),
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
            "prefer",
            Ty::Dict(Box::new(Ty::Num)),
            "target preferences: player, a kind (fighter, station…), class:<id> or faction:<id> → weight (2 = as if half as far)",
        ),
        f(
            "skill",
            Ty::Struct(vec![
                f("aim", Ty::Qty(Unit::Fraction), "0 wild … 1 perfect"),
                f("reaction", time(), "seconds between looks at the target"),
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
    ])
}

fn power() -> Ty {
    Ty::Def(vec![
        f("beam_recharge", Ty::List(Box::new(Ty::Num)), "per level A, B, …: fraction of the ship's energy_recharge"),
        f("shield_recharge", Ty::List(Box::new(Ty::Num)), "per level: fraction of the ship's shield_recharge"),
        f("budget", Ty::Num, "power steps shared by both settings and switched-on systems"),
        f("start_beam", Ty::Num, "starting beam level (0 = A)"),
        f("start_shield", Ty::Num, "starting shield level"),
        f("balance_shift", Ty::Qty(Unit::Fraction), "how much shield balance shifts capacity"),
    ])
}

fn damage() -> Ty {
    Ty::Def(vec![
        f("system_share", Ty::Qty(Unit::Fraction), "share of hull damage that also strikes a system"),
        f("destroy_below", Ty::Qty(Unit::Fraction), "a system this far below zero is destroyed"),
        f("repair_rate", Ty::Num, "default crew repair rate, hit points per second"),
    ])
}

fn campaign() -> Ty {
    Ty::Def(vec![
        f("name", Ty::Text, "campaign title"),
        f("difficulty", names(), "difficulty level ids, easiest first"),
        f("default_difficulty", name(), "the level a new game starts at"),
    ])
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
        f("modelsets", dict(modelset()), "looks: models, damage textures, states, debris"),
        f("debris", dict(debris()), "debris sets by id"),
        f("effects", dict(effect()), "explosions and other effects by id"),
        f("flight", flight(), "flight model and sensors"),
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
        f("music", Ty::Asset, "music"),
        f("objects", Ty::Dict(Box::new(object)), "placed objects by name"),
        f("nav", Ty::Dict(Box::new(Ty::Struct(vec![f("position", Ty::Numbers(3), "metres")]))), "navigation points"),
    ])
}

/// Every file type's JSON Schema: `(file name, contents)`.
pub fn json_schemas() -> Vec<(&'static str, String)> {
    use super::schema::json_schema;
    vec![
        ("pack.schema.json", json_schema("OpenInterceptor pack manifest (pack.yaml)", &pack())),
        ("rules.schema.json", json_schema("OpenInterceptor rules file (rules/*.yaml, campaign/*.yaml)", &rules())),
        ("scenario.schema.json", json_schema("OpenInterceptor scenario (scenarios/*.yaml)", &scenario())),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::schema::json_schema;

    #[test]
    fn every_schema_builds_and_exports() {
        for (name, ty) in [("pack", pack()), ("rules", rules()), ("scenario", scenario())] {
            let j = json_schema(name, &ty);
            assert_eq!(j.matches('{').count(), j.matches('}').count(), "{name}");
        }
        assert!(section("ships").is_some() && section("flight").is_some() && section("nope").is_none());
    }
}
