//! # Saving and loading a campaign (Stage 4.11)
//!
//! A save is a YAML document: the campaign's identity (which campaign, which packs and
//! mods at which versions), then the whole [`Campaign`] state. Because the state holds
//! only numbers, text and indices (see the module docs of [`super`]), writing it is a
//! direct walk; reading it back checks every index against the loaded rules, so a save
//! from a different version of a mod is refused with a message rather than crashing.
//!
//! ```yaml
//! save: 1
//! campaign: default
//! packs: [{id: default, version: 0.6.7}]
//! hours: 1234.5
//! money: 2150000
//! …
//! ```
//!
//! Loading refuses a save whose packs are not all loaded (R-SAV-002): playing on
//! without a mod the campaign was started with would silently change the game.

use super::*;
use crate::content::schema::Get;
use crate::formats::yaml::{self, Node, Value as Y};

/// The save format version.
pub const FORMAT: u32 = 1;

/// The save's subsystems and their versions (RQ-S-28, after the original's tagged chunks,
/// unknowns §12.6): written as `sections:`; a save whose section has another version is
/// refused with a message naming it, instead of being half read. Raise a section's version
/// when its layout changes in a way older readers would misread.
pub const SECTIONS: [(&str, u32); 10] = [
    ("player", 1),
    ("bases", 2),
    ("craft", 1),
    ("pilots", 1),
    ("contacts", 1),
    ("deliveries", 1),
    ("outposts", 1),
    ("history", 1),
    ("story", 2),
    ("aliens", 1),
];

/// A pack a save was made with.
#[derive(Debug, Clone, PartialEq)]
pub struct PackRef {
    pub id: String,
    pub version: String,
}

// Small builders for the YAML tree.
fn s(t: impl ToString) -> Node {
    Node { value: Y::Scalar { text: t.to_string(), quoted: true }, line: 0 }
}
fn n(v: f64) -> Node {
    Node::text(format!("{v}"), 0)
}
fn seq(items: Vec<Node>) -> Node {
    Node { value: Y::Seq(items), line: 0 }
}
fn map(entries: Vec<(&str, Node)>) -> Node {
    Node { value: Y::Map(entries.into_iter().map(|(k, v)| (k.to_string(), v)).collect()), line: 0 }
}
fn nums(v: &[f64]) -> Node {
    seq(v.iter().map(|x| n(*x)).collect())
}

/// Lane waypoints as a flat list `[x0, y0, x1, y1, …]`.
fn route(r: &[[f32; 2]]) -> Node {
    nums(&r.iter().flat_map(|p| [p[0] as f64, p[1] as f64]).collect::<Vec<_>>())
}

/// A store of goods as `{goods: amount}`.
fn store(st: &[(String, f32)]) -> Node {
    Node { value: Y::Map(st.iter().map(|(g, a)| (g.clone(), n(*a as f64))).collect()), line: 0 }
}

fn dest(d: &logistics::Dest) -> Node {
    match d {
        logistics::Dest::Base(b) => s(format!("base:{b}")),
        logistics::Dest::Facility(f) => s(format!("facility:{f}")),
    }
}

/// The logistics state (facilities, orders, convoys, surveys, reserves, base stores).
fn write_logistics(l: &logistics::Logistics) -> Node {
    map(vec![
        (
            "facilities",
            seq(l
                .facilities
                .iter()
                .map(|f| {
                    map(vec![
                        ("type", n(f.ftype as f64)),
                        ("system", n(f.system as f64)),
                        ("ready_at", n(f.ready_at)),
                        ("store", store(&f.store)),
                        ("lines", seq(f.lines.iter().map(|x| nums(&[x.recipe as f64, x.hours_left as f64])).collect())),
                        ("project", f.project.map_or(Node::null(0), |(t, st)| nums(&[t as f64, st as u8 as f64]))),
                        ("attacks", n(f.attacks as f64)),
                    ])
                })
                .collect()),
        ),
        (
            "orders",
            seq(l
                .orders
                .iter()
                .map(|o| {
                    map(vec![
                        ("goods", s(&o.goods)),
                        ("amount", n(o.amount as f64)),
                        ("to", dest(&o.to)),
                        ("source", n(o.source as f64)),
                    ])
                })
                .collect()),
        ),
        (
            "convoys",
            seq(l
                .convoys
                .iter()
                .map(|v| {
                    map(vec![
                        ("cargo", store(&v.cargo)),
                        ("to", dest(&v.to)),
                        ("pos", nums(&[v.pos[0] as f64, v.pos[1] as f64])),
                        ("route", route(&v.route)),
                        ("vessels", n(v.vessels as f64)),
                        ("ambush_in", v.ambush_in.map_or(Node::null(0), |a| n(a as f64))),
                        ("held", s(v.held)),
                    ])
                })
                .collect()),
        ),
        (
            "surveys",
            seq(l
                .surveys
                .iter()
                .map(|v| {
                    map(vec![
                        ("system", n(v.system as f64)),
                        ("pos", nums(&[v.pos[0] as f64, v.pos[1] as f64])),
                        ("attacked", s(v.attacked)),
                        ("held", s(v.held)),
                    ])
                })
                .collect()),
        ),
        ("surveyed", seq(l.surveyed.iter().map(s).collect())),
        (
            "reserves",
            seq(l
                .reserves
                .iter()
                .map(|r| seq(r.iter().map(|x| x.map_or(s("unlimited"), |v| n(v as f64))).collect()))
                .collect()),
        ),
        ("base_goods", seq(l.base_goods.iter().map(|g| store(g)).collect())),
        ("stats", {
            let t = &l.stats;
            nums(&[
                t.convoys as f64,
                t.shipped as f64,
                t.delivered as f64,
                t.ambushes as f64,
                t.vessels_lost as f64,
                t.surveys as f64,
                t.surveys_attacked as f64,
                t.surveys_lost as f64,
            ])
        }),
    ])
}

/// The story state: event counts, arcs, plans, standings, the rival's mood, the ending.
fn write_story(st: &story::StoryState) -> Node {
    let f = |v: &[f64]| nums(v);
    map(vec![
        ("fired", f(&st.fired.iter().map(|x| *x as f64).collect::<Vec<_>>())),
        // (Never-fired events keep -inf, written as a very old hour.)
        ("last_fired", f(&st.last_fired.iter().map(|x| x.max(-1e12)).collect::<Vec<_>>())),
        ("arc_step", f(&st.arc_step.iter().map(|x| *x as f64).collect::<Vec<_>>())),
        ("plan_started", seq(st.plan_started.iter().map(s).collect())),
        ("plan_done", f(&st.plan_done.iter().map(|x| *x as f64).collect::<Vec<_>>())),
        ("plan_next", f(&st.plan_next)),
        ("plan_warned", f(&st.plan_warned.iter().map(|x| *x as f64).collect::<Vec<_>>())),
        ("standing", store(&st.standing)),
        ("aggression", n(st.aggression as f64)),
        ("battles_won", n(st.battles_won as f64)),
        ("month_wins", n(st.month_wins as f64)),
        ("battle_pending", s(st.battle_pending)),
        ("research_pending", s(st.research_pending)),
        ("ending", st.ending.map_or(Node::null(0), |e| n(e as f64))),
        // Delayed firings (-1: none), the next side-mission check and the aliens' economy.
        ("pending_at", f(&st.pending_at.iter().map(|p| p.unwrap_or(-1.0)).collect::<Vec<_>>())),
        ("side_next", n(st.side_next)),
        ("alien", f(&st.alien.to_nums())),
    ])
}

fn read_story(node: Option<&Node>, def: &StrategyDef) -> Result<story::StoryState, String> {
    let mut st = story::StoryState::new(def);
    let Some(node) = node.filter(|n| !n.is_null()) else { return Ok(st) };
    let g = Get(node);
    let nums_of = |k: &str| -> Vec<f64> {
        g.node(k).and_then(|n| n.as_seq()).unwrap_or(&[]).iter().filter_map(|x| x.as_str()?.parse().ok()).collect()
    };
    let bad = |what: &str| format!("save: the story's {what} do not match the loaded rules (was a mod changed?)");
    let (fired, last, arcs) = (nums_of("fired"), nums_of("last_fired"), nums_of("arc_step"));
    if fired.len() != st.fired.len() || arcs.len() != st.arc_step.len() {
        return Err(bad("events or arcs"));
    }
    st.fired = fired.iter().map(|x| *x as u32).collect();
    st.last_fired = last.iter().map(|x| if *x <= -1e12 { f64::NEG_INFINITY } else { *x }).collect();
    st.arc_step = arcs.iter().map(|x| *x as usize).collect();
    let (done, next, warned) = (nums_of("plan_done"), nums_of("plan_next"), nums_of("plan_warned"));
    if done.len() != st.plan_done.len() {
        return Err(bad("plans"));
    }
    st.plan_started = g
        .node("plan_started")
        .and_then(|n| n.as_seq())
        .unwrap_or(&[])
        .iter()
        .map(|x| x.as_str() == Some("true"))
        .collect();
    st.plan_done = done.iter().map(|x| *x as u32).collect();
    st.plan_next = next;
    st.plan_warned = warned.iter().map(|x| *x as usize).collect();
    st.standing = g
        .node("standing")
        .and_then(|n| n.as_map())
        .unwrap_or(&[])
        .iter()
        .filter_map(|(k, v)| Some((k.clone(), v.as_str()?.parse().ok()?)))
        .collect();
    let num = |k: &str| g.str(k).and_then(|t| t.parse::<f64>().ok()).unwrap_or(0.0);
    st.aggression = num("aggression") as f32;
    st.battles_won = num("battles_won") as u32;
    st.month_wins = num("month_wins") as u32;
    st.battle_pending = g.bool("battle_pending").unwrap_or(false);
    // (Since app NEXT; older saves keep the new campaign's values.)
    let pending = nums_of("pending_at");
    if pending.len() == st.pending_at.len() {
        st.pending_at = pending.iter().map(|p| (*p >= 0.0).then_some(*p)).collect();
    }
    if g.node("side_next").is_some() {
        st.side_next = num("side_next");
    }
    st.alien = story::AlienState::from_nums(&nums_of("alien"), st.alien);
    st.research_pending = g.bool("research_pending").unwrap_or(false);
    st.ending = g.str("ending").and_then(|e| e.parse().ok()).filter(|e| *e < def.story.endings.len());
    Ok(st)
}

/// Reads the logistics state back (indices checked against the loaded rules).
fn read_logistics(
    node: Option<&Node>,
    def: &StrategyDef,
    bases: usize,
    systems: usize,
) -> Result<logistics::Logistics, String> {
    let Some(node) = node.filter(|n| !n.is_null()) else {
        return Ok(def.logistics.as_ref().map(|l| logistics::Logistics::new(l, systems)).unwrap_or_default());
    };
    let Some(l) = def.logistics.as_ref() else {
        // A campaign without logistics saves an empty block; anything in it is a mismatch.
        let empty = Get(node).node("facilities").and_then(|n| n.as_seq()).is_none_or(|s| s.is_empty());
        return if empty {
            Ok(Default::default())
        } else {
            Err("save: this save has logistics but the loaded campaign has none".into())
        };
    };
    let bad = |what: &str| format!("save: {what} does not match the loaded rules (was a mod changed?)");
    let g = Get(node);
    let list = |k: &str| g.node(k).and_then(|n| n.as_seq()).unwrap_or(&[]).to_vec();
    let num = |g: Get, k: &str| g.str(k).and_then(|t| t.parse::<f64>().ok()).unwrap_or(0.0);
    let nums_of =
        |n: &Node| -> Vec<f64> { n.as_seq().unwrap_or(&[]).iter().filter_map(|x| x.as_str()?.parse().ok()).collect() };
    let store_of = |n: Option<&Node>| -> Vec<(String, f32)> {
        n.and_then(|n| n.as_map())
            .unwrap_or(&[])
            .iter()
            .filter_map(|(k, v)| Some((k.clone(), v.as_str()?.parse().ok()?)))
            .collect()
    };
    let dest_of = |t: &str| -> Result<logistics::Dest, String> {
        match t.split_once(':') {
            Some(("base", b)) => b.parse().ok().filter(|b| *b < bases).map(logistics::Dest::Base),
            Some(("facility", f)) => f.parse().ok().map(logistics::Dest::Facility),
            _ => None,
        }
        .ok_or_else(|| bad("a delivery destination"))
    };
    let mut out = logistics::Logistics::default();
    for f in list("facilities") {
        let f = Get(&f);
        let ftype = num(f, "type") as usize;
        if ftype >= l.facility_types.len() {
            return Err(bad("a facility type"));
        }
        let lines = f
            .node("lines")
            .and_then(|n| n.as_seq())
            .unwrap_or(&[])
            .iter()
            .map(|x| {
                let v = nums_of(x);
                logistics::Line {
                    recipe: v.first().copied().unwrap_or(0.0) as usize,
                    hours_left: v.get(1).copied().unwrap_or(0.0) as f32,
                }
            })
            .collect::<Vec<_>>();
        if lines.iter().any(|x| x.recipe >= l.recipes.len()) {
            return Err(bad("a recipe"));
        }
        let project = f
            .node("project")
            .filter(|n| !n.is_null())
            .map(nums_of)
            .map(|v| (v[0] as usize, v.get(1).copied().unwrap_or(0.0) > 0.5));
        out.facilities.push(logistics::Facility {
            ftype,
            system: num(f, "system") as usize,
            ready_at: num(f, "ready_at"),
            store: store_of(f.node("store")),
            lines,
            project,
            // Saves from before 0.16.3 have no count: none.
            attacks: f.num("attacks").unwrap_or(0.0) as u32,
        });
    }
    for o in list("orders") {
        let o = Get(&o);
        out.orders.push(logistics::Order {
            goods: o.str("goods").unwrap_or("").into(),
            amount: num(o, "amount") as f32,
            to: dest_of(o.str("to").unwrap_or(""))?,
            source: num(o, "source") as usize,
        });
    }
    for v in list("convoys") {
        let v = Get(&v);
        let p = v.node("pos").map(nums_of).unwrap_or_default();
        out.convoys.push(logistics::Convoy {
            cargo: store_of(v.node("cargo")),
            to: dest_of(v.str("to").unwrap_or(""))?,
            pos: [p.first().copied().unwrap_or(0.0) as f32, p.get(1).copied().unwrap_or(0.0) as f32],
            route: v
                .node("route")
                .map(nums_of)
                .unwrap_or_default()
                .chunks_exact(2)
                .map(|q| [q[0] as f32, q[1] as f32])
                .collect(),
            vessels: num(v, "vessels") as u32,
            ambush_in: v.str("ambush_in").and_then(|a| a.parse().ok()),
            held: v.bool("held").unwrap_or(false),
        });
    }
    for v in list("surveys") {
        let v = Get(&v);
        let p = v.node("pos").map(nums_of).unwrap_or_default();
        out.surveys.push(logistics::Survey {
            system: num(v, "system") as usize,
            pos: [p.first().copied().unwrap_or(0.0) as f32, p.get(1).copied().unwrap_or(0.0) as f32],
            attacked: v.bool("attacked").unwrap_or(false),
            held: v.bool("held").unwrap_or(false),
        });
    }
    out.surveyed = list("surveyed").iter().map(|x| x.as_str() == Some("true")).collect();
    out.reserves = list("reserves")
        .iter()
        .map(|r| r.as_seq().unwrap_or(&[]).iter().map(|x| x.as_str().and_then(|t| t.parse().ok())).collect())
        .collect();
    out.base_goods = list("base_goods").iter().map(|g| store_of(Some(g))).collect();
    // The supply totals, in a fixed order (saves from before 0.20.0 have none: zeros).
    let t = g.node("stats").map(nums_of).unwrap_or_default();
    let at = |i: usize| t.get(i).copied().unwrap_or(0.0);
    out.stats = logistics::LogisticsStats {
        convoys: at(0) as u32,
        shipped: at(1) as f32,
        delivered: at(2) as f32,
        ambushes: at(3) as u32,
        vessels_lost: at(4) as u32,
        surveys: at(5) as u32,
        surveys_attacked: at(6) as u32,
        surveys_lost: at(7) as u32,
    };
    if out.surveyed.len() != systems || out.facilities.iter().any(|f| f.system >= systems) {
        return Err(bad("the map"));
    }
    Ok(out)
}

/// Writes a campaign as a save document.
pub fn write(c: &Campaign, campaign_id: &str, packs: &[PackRef]) -> String {
    let product = |p: &Product| match p {
        Product::Item(i) => format!("item:{i}"),
        Product::Hull(h) => format!("hull:{h}"),
    };
    let pilot = |p: &Pilot| {
        let state = match p.state {
            PilotState::Ready => s("ready"),
            PilotState::Injured(h) => s(format!("injured:{h}")),
            PilotState::Dead => s("dead"),
        };
        map(vec![
            ("name", s(&p.name)),
            ("stats", nums(&p.stats.iter().map(|x| *x as f64).collect::<Vec<_>>())),
            ("experience", n(p.experience as f64)),
            ("kills", n(p.kills as f64)),
            ("missions", n(p.missions as f64)),
            ("state", state),
            ("craft", p.craft.map_or(Node::null(0), |k| n(k as f64))),
            ("training", s(if p.training { "yes" } else { "no" })),
            ("health", s(p.health)),
            ("max_health", s(p.max_health)),
            ("trained", s(p.trained)),
        ])
    };
    let doc = map(vec![
        ("save", n(FORMAT as f64)),
        ("sections", map(SECTIONS.iter().map(|(k, v)| (*k, n(*v as f64))).collect())),
        ("campaign", s(campaign_id)),
        ("packs", seq(packs.iter().map(|p| map(vec![("id", s(&p.id)), ("version", s(&p.version))])).collect())),
        ("hours", n(c.hours)),
        ("money", n(c.money)),
        ("difficulty", n(c.difficulty as f64)),
        ("rng", s(c.rng)),
        (
            "systems",
            seq(c
                .systems
                .iter()
                .map(|x| {
                    map(vec![
                        ("id", s(&x.id)),
                        ("name", s(&x.name)),
                        ("pos", nums(&[x.pos[0] as f64, x.pos[1] as f64])),
                        ("class", s(&x.class)),
                        ("hidden", s(x.hidden)),
                        ("planets", s(x.planets)),
                    ])
                })
                .collect()),
        ),
        (
            "bases",
            seq(c
                .bases
                .iter()
                .map(|b| {
                    map(vec![
                        ("name", s(&b.name)),
                        ("type", n(b.base_type as f64)),
                        ("system", n(b.system as f64)),
                        ("ready_at", n(b.ready_at)),
                        ("hull", n(b.hull as f64)),
                        // `[module, ready_at]`, with a third number 1 for a wreck.
                        (
                            "modules",
                            seq(b
                                .modules
                                .iter()
                                .map(|m| {
                                    if m.wrecked {
                                        nums(&[m.module as f64, m.ready_at, 1.0])
                                    } else {
                                        nums(&[m.module as f64, m.ready_at])
                                    }
                                })
                                .collect()),
                        ),
                        ("research", b.research.map_or(Node::null(0), |t| n(t as f64))),
                        (
                            "jobs",
                            seq(b
                                .jobs
                                .iter()
                                .map(|j| {
                                    map(vec![
                                        ("product", s(product(&j.product))),
                                        ("left", n(j.left as f64)),
                                        ("count", n(j.count as f64)),
                                        ("started", s(j.started)),
                                    ])
                                })
                                .collect()),
                        ),
                        ("stock", seq(b.stock.iter().map(|(i, k)| nums(&[*i as f64, *k as f64])).collect())),
                    ])
                })
                .collect()),
        ),
        ("known", seq(c.known.iter().map(s).collect())),
        ("progress", nums(&c.progress.iter().map(|x| *x as f64).collect::<Vec<_>>())),
        (
            "craft",
            seq(c
                .craft
                .iter()
                .map(|k| {
                    map(vec![
                        ("name", s(&k.name)),
                        ("hull", n(k.hull as f64)),
                        ("base", n(k.base as f64)),
                        (
                            "fit",
                            seq(k
                                .fit
                                .iter()
                                .map(|(slot, i)| map(vec![("slot", s(slot)), ("item", n(*i as f64))]))
                                .collect()),
                        ),
                        ("away", k.away.map_or(Node::null(0), |(id, p)| nums(&[id as f64, p[0] as f64, p[1] as f64]))),
                        ("route", route(&k.route)),
                        (
                            "condition",
                            k.condition
                                .map_or(Node::null(0), |v| nums(&v.iter().map(|x| *x as f64).collect::<Vec<_>>())),
                        ),
                    ])
                })
                .collect()),
        ),
        ("pilots", seq(c.pilots.iter().map(pilot).collect())),
        ("recruits", seq(c.recruits.iter().map(pilot).collect())),
        ("offscreen", seq(c.offscreen.iter().map(write_offscreen).collect())),
        ("aliens", write_aliens(&c.aliens)),
        (
            "contacts",
            seq(c
                .contacts
                .iter()
                .map(|k| {
                    map(vec![
                        ("id", n(k.id as f64)),
                        ("raid", n(k.raid as f64)),
                        ("pos", nums(&[k.pos[0] as f64, k.pos[1] as f64])),
                        ("dest", nums(&[k.dest[0] as f64, k.dest[1] as f64])),
                        ("target_base", k.target_base.map_or(Node::null(0), |b| n(b as f64))),
                        ("detected", s(k.detected)),
                        ("route", route(&k.route)),
                        ("plan", k.plan.map_or(Node::null(0), |p| n(p as f64))),
                    ])
                })
                .collect()),
        ),
        (
            "deliveries",
            seq(c
                .deliveries
                .iter()
                .map(|d| {
                    map(vec![
                        ("product", s(product(&d.product))),
                        ("base", n(d.base as f64)),
                        ("count", n(d.count as f64)),
                        ("arrive_at", n(d.arrive_at)),
                        ("held", s(d.held)),
                    ])
                })
                .collect()),
        ),
        ("next_flow", nums(&c.next_flow)),
        ("next_raid", nums(&c.next_raid)),
        ("next_month", n(c.next_month)),
        ("score", n(c.score)),
        ("month_score", n(c.month_score)),
        ("flags", seq(c.flags.iter().map(s).collect())),
        ("salvaged", seq(c.salvaged.iter().map(s).collect())),
        ("messages", seq(c.messages.iter().map(|m| map(vec![("hour", n(m.hour)), ("text", s(&m.text))])).collect())),
        ("next_contact", n(c.next_contact as f64)),
        ("logistics", write_logistics(&c.logistics)),
        ("story", write_story(&c.story)),
        ("history", seq(c.history.iter().map(|m| nums(&m.to_nums())).collect())),
        ("this_month", nums(&c.this_month.to_nums())),
        (
            "outposts",
            seq(c
                .outposts
                .iter()
                .map(|o| {
                    map(vec![
                        ("system", n(o.system as f64)),
                        ("funding", n(o.funding as f64)),
                        ("founded", n(o.founded)),
                        ("freighter_free", n(o.freighter_free)),
                        ("opps", seq(o.opps.iter().map(|(k, r)| nums(&[*k as f64, *r as f64])).collect())),
                    ])
                })
                .collect()),
        ),
        ("system_scores", seq(c.system_scores.iter().map(|s| nums(&[s[0], s[1]])).collect())),
        ("pool_scores", nums(&[c.pool_scores[0], c.pool_scores[1]])),
        ("raid_wins", nums(&c.raid_wins.iter().map(|n| *n as f64).collect::<Vec<_>>())),
        (
            "activity_grid",
            seq(c
                .activity
                .iter()
                .map(|a| nums(&[a.month as f64, a.cell as f64, a.who as f64, a.count as f64]))
                .collect()),
        ),
    ]);
    format!("# OpenInterceptor campaign save\n{}", doc.to_yaml())
}

/// Reads a save's header: the campaign id and the packs it was made with (to choose and
/// check what to load before reading the rest).
/// The difficulty level index a save was played at (its `difficulty`), read without
/// loading it: the content must be loaded at that level before [`read`], because
/// `per_level` values are resolved when the content loads.
pub fn difficulty_of(text: &str) -> Option<usize> {
    let doc = yaml::parse(text).ok()?;
    Get(&doc).num("difficulty").map(|d| d.max(0.0) as usize)
}

pub fn header(text: &str) -> Result<(String, Vec<PackRef>), String> {
    let doc = yaml::parse(text).map_err(|e| format!("save: {e}"))?;
    let g = Get(&doc);
    if g.num("save").map(|v| v as u32) != Some(FORMAT) {
        return Err("not a campaign save this version can read".into());
    }
    let packs = g
        .node("packs")
        .and_then(|p| p.as_seq())
        .unwrap_or(&[])
        .iter()
        .map(|p| PackRef {
            id: Get(p).str("id").unwrap_or("").to_string(),
            version: Get(p).str("version").unwrap_or("").to_string(),
        })
        .collect();
    Ok((g.str("campaign").unwrap_or("").to_string(), packs))
}

/// Reads a save against the loaded strategy data. `loaded` are the packs loaded now;
/// a save naming a pack that is not loaded is refused.
pub fn read(text: &str, def: &StrategyDef, loaded: &[PackRef]) -> Result<Campaign, String> {
    let (_, packs) = header(text)?;
    let missing: Vec<&str> =
        packs.iter().filter(|p| !loaded.iter().any(|l| l.id == p.id)).map(|p| p.id.as_str()).collect();
    if !missing.is_empty() {
        return Err(format!("this save needs packs that are not loaded: {}", missing.join(", ")));
    }
    // A pack updated since the save still loads, with a notice: its rules may differ.
    let changed: Vec<String> = packs
        .iter()
        .filter_map(|p| {
            let now = loaded.iter().find(|l| l.id == p.id)?;
            (now.version != p.version).then(|| {
                format!("Pack {} is now version {} (this save was made with {})", p.id, now.version, p.version)
            })
        })
        .collect();
    let doc = yaml::parse(text).map_err(|e| format!("save: {e}"))?;
    let g = Get(&doc);
    // Each subsystem's version must be the one this app reads (saves made before the
    // sections were versioned have none and load as before).
    if let Some(sec) = g.node("sections").map(Get) {
        for (name, version) in SECTIONS {
            if let Some(v) = sec.num(name) {
                if v as u32 != version {
                    return Err(format!("save: its {name} section is version {v}; this app reads version {version}"));
                }
            }
        }
    }
    // Field readers with bounds checks against the rules now loaded.
    let bad = |what: &str| format!("save: {what} does not match the loaded rules (was a mod changed?)");
    let f = |g: Get, k: &str| g.str(k).and_then(|t| t.parse::<f64>().ok()).unwrap_or(0.0);
    let idx = |g: Get, k: &str, len: usize, what: &str| -> Result<usize, String> {
        let i = f(g, k) as usize;
        if i < len {
            Ok(i)
        } else {
            Err(bad(what))
        }
    };
    let list = |key: &str| -> &[Node] { g.node(key).and_then(|n| n.as_seq()).unwrap_or(&[]) };
    let numbers = |node: &Node| -> Vec<f64> {
        node.as_seq().unwrap_or(&[]).iter().filter_map(|x| x.as_str()?.parse().ok()).collect()
    };
    let waypoints =
        |node: &Node| -> Vec<[f32; 2]> { numbers(node).chunks_exact(2).map(|p| [p[0] as f32, p[1] as f32]).collect() };
    let product = |t: &str| -> Result<Product, String> {
        match t.split_once(':') {
            Some(("item", i)) => i.parse::<usize>().ok().filter(|i| *i < def.items.len()).map(Product::Item),
            Some(("hull", h)) => h.parse::<usize>().ok().filter(|h| *h < def.hulls.len()).map(Product::Hull),
            _ => None,
        }
        .ok_or_else(|| bad("a product"))
    };
    let opt = |g: Get, k: &str| g.str(k).and_then(|t| t.parse::<usize>().ok());
    let pilot = |p: &Node| -> Pilot {
        let p = Get(p);
        Pilot {
            name: p.str("name").unwrap_or("").to_string(),
            stats: p.node("stats").map(numbers).unwrap_or_default().into_iter().map(|x| x as f32).collect(),
            experience: f(p, "experience") as f32,
            kills: f(p, "kills") as u32,
            missions: f(p, "missions") as u32,
            state: match p.str("state").unwrap_or("ready") {
                "dead" => PilotState::Dead,
                t => match t.strip_prefix("injured:").and_then(|h| h.parse().ok()) {
                    Some(h) => PilotState::Injured(h),
                    None => PilotState::Ready,
                },
            },
            craft: opt(p, "craft"),
            // Saves from before 0.16.7 have no flag: not training.
            training: p.str("training") == Some("yes"),
            health: f(p, "health") as f32,
            max_health: f(p, "max_health") as f32,
            trained: f(p, "trained") as f32,
        }
    };
    let systems: Vec<StarSystem> = list("systems")
        .iter()
        .map(|x| {
            let x = Get(x);
            let p = x.node("pos").map(numbers).unwrap_or_default();
            StarSystem {
                id: x.str("id").unwrap_or("").into(),
                name: x.str("name").unwrap_or("").into(),
                pos: [p.first().copied().unwrap_or(0.0) as f32, p.get(1).copied().unwrap_or(0.0) as f32],
                class: x.str("class").unwrap_or("").into(),
                hidden: x.bool("hidden").unwrap_or(false),
                planets: x.num("planets").map_or(0, |n| n.clamp(0.0, 255.0) as u8),
            }
        })
        .collect();
    let systems_len = systems.len();
    let mut bases = Vec::new();
    for b in list("bases") {
        let b = Get(b);
        let mut modules = Vec::new();
        for m in b.node("modules").and_then(|n| n.as_seq()).unwrap_or(&[]) {
            let v = numbers(m);
            let module = v.first().copied().unwrap_or(f64::MAX) as usize;
            if module >= def.modules.len() {
                return Err(bad("a module"));
            }
            modules.push(ModuleInst {
                module,
                ready_at: v.get(1).copied().unwrap_or(0.0),
                wrecked: v.get(2).is_some_and(|w| *w != 0.0),
            });
        }
        let mut jobs = Vec::new();
        for j in b.node("jobs").and_then(|n| n.as_seq()).unwrap_or(&[]) {
            let j = Get(j);
            jobs.push(Job {
                product: product(j.str("product").unwrap_or(""))?,
                left: f(j, "left") as f32,
                count: f(j, "count") as u32,
                started: j.bool("started").unwrap_or(false),
            });
        }
        let mut stock = Vec::new();
        for st in b.node("stock").and_then(|n| n.as_seq()).unwrap_or(&[]) {
            let v = numbers(st);
            let item = v.first().copied().unwrap_or(f64::MAX) as usize;
            if item >= def.items.len() {
                return Err(bad("an item in store"));
            }
            stock.push((item, v.get(1).copied().unwrap_or(0.0) as u32));
        }
        let research = opt(b, "research");
        if research.is_some_and(|t| t >= def.research.len()) {
            return Err(bad("a research topic"));
        }
        bases.push(Base {
            name: b.str("name").unwrap_or("").into(),
            base_type: idx(b, "type", def.base_types.len(), "a base type")?,
            system: idx(b, "system", systems.len(), "a system")?,
            modules,
            research,
            jobs,
            stock,
            ready_at: f(b, "ready_at"),
            // The hub's hull (app NEXT; older saves: whole).
            hull: b.num("hull").unwrap_or_else(|| def.base_types.get(f(b, "type") as usize).map_or(0.0, |t| t.hull)),
        });
    }
    let known: Vec<bool> = list("known").iter().map(|k| k.as_str() == Some("true")).collect();
    let progress: Vec<f32> =
        g.node("progress").map(numbers).unwrap_or_default().into_iter().map(|x| x as f32).collect();
    if known.len() != def.research.len() || progress.len() != def.research.len() {
        return Err(bad("the research list"));
    }
    let mut craft = Vec::new();
    for k in list("craft") {
        let k = Get(k);
        let mut fit = Vec::new();
        for x in k.node("fit").and_then(|n| n.as_seq()).unwrap_or(&[]) {
            fit.push((
                Get(x).str("slot").unwrap_or("").to_string(),
                idx(Get(x), "item", def.items.len(), "a fitted item")?,
            ));
        }
        let away = k.node("away").filter(|n| !n.is_null()).map(numbers).map(|v| {
            (
                v.first().copied().unwrap_or(0.0) as u32,
                [v.get(1).copied().unwrap_or(0.0) as f32, v.get(2).copied().unwrap_or(0.0) as f32],
            )
        });
        craft.push(Craft {
            name: k.str("name").unwrap_or("").into(),
            hull: idx(k, "hull", def.hulls.len(), "a hull")?,
            base: idx(k, "base", bases.len(), "a craft's base")?,
            fit,
            away,
            route: k.node("route").map(waypoints).unwrap_or_default(),
            condition: k
                .node("condition")
                .filter(|n| !n.is_null())
                .map(numbers)
                .filter(|v| v.len() == 5)
                .map(|v| [v[0] as f32, v[1] as f32, v[2] as f32, v[3] as f32, v[4] as f32]),
        });
    }
    let mut contacts = Vec::new();
    for k in list("contacts") {
        let k = Get(k);
        let p = k.node("pos").map(numbers).unwrap_or_default();
        let d = k.node("dest").map(numbers).unwrap_or_default();
        contacts.push(Contact {
            id: f(k, "id") as u32,
            raid: idx(k, "raid", def.raids.len(), "a raid")?,
            pos: [p.first().copied().unwrap_or(0.0) as f32, p.get(1).copied().unwrap_or(0.0) as f32],
            dest: [d.first().copied().unwrap_or(0.0) as f32, d.get(1).copied().unwrap_or(0.0) as f32],
            target_base: opt(k, "target_base"),
            detected: k.bool("detected").unwrap_or(false),
            route: k.node("route").map(waypoints).unwrap_or_default(),
            plan: opt(k, "plan").filter(|p| *p < def.story.plans.len()),
        });
    }
    let mut deliveries = Vec::new();
    for d in list("deliveries") {
        let d = Get(d);
        deliveries.push(Delivery {
            product: product(d.str("product").unwrap_or(""))?,
            base: idx(d, "base", bases.len(), "a delivery's base")?,
            count: f(d, "count") as u32,
            arrive_at: f(d, "arrive_at"),
            held: d.bool("held").unwrap_or(false),
        });
    }
    let strings =
        |key: &str| -> Vec<String> { list(key).iter().filter_map(|x| x.as_str().map(str::to_string)).collect() };
    let next_flow = g.node("next_flow").map(numbers).unwrap_or_default();
    let next_raid = g.node("next_raid").map(numbers).unwrap_or_default();
    if next_flow.len() != def.incomes.len() + def.expenses.len() || next_raid.len() != def.raids.len() {
        return Err(bad("the incomes, expenses or raids"));
    }
    let logistics = read_logistics(g.node("logistics"), def, bases.len(), systems.len())?;
    let system_count = systems.len();
    Ok(Campaign {
        notices: changed,
        logistics,
        story: read_story(g.node("story"), def)?,
        hours: f(g, "hours"),
        money: f(g, "money"),
        difficulty: f(g, "difficulty") as usize,
        rng: g.str("rng").and_then(|t| t.parse().ok()).unwrap_or(1),
        systems,
        bases,
        known,
        progress,
        craft,
        pilots: list("pilots").iter().map(pilot).collect(),
        recruits: list("recruits").iter().map(pilot).collect(),
        contacts,
        offscreen: list("offscreen").iter().map(read_offscreen).collect(),
        aliens: g.node("aliens").map(|n| read_aliens(n, system_count)).unwrap_or_default(),
        deliveries,
        next_flow,
        next_raid,
        next_month: f(g, "next_month"),
        score: f(g, "score"),
        month_score: f(g, "month_score"),
        flags: strings("flags"),
        salvaged: strings("salvaged"),
        messages: list("messages")
            .iter()
            .map(|m| Message { hour: f(Get(m), "hour"), text: Get(m).str("text").unwrap_or("").to_string() })
            .collect(),
        next_contact: f(g, "next_contact") as u32,
        // The reports' history (since app NEXT; older saves have none).
        history: list("history").iter().map(|m| MonthRecord::from_nums(&numbers(m))).collect(),
        this_month: g.node("this_month").map(|m| MonthRecord::from_nums(&numbers(m))).unwrap_or_default(),
        // The outposts and the month's system scores (app NEXT; older saves have none).
        outposts: list("outposts")
            .iter()
            .map(|o| {
                let g = Get(o);
                super::outposts::Outpost {
                    system: (f(g, "system") as usize).min(systems_len.saturating_sub(1)),
                    funding: f(g, "funding") as f32,
                    founded: f(g, "founded"),
                    freighter_free: f(g, "freighter_free"),
                    opps: g
                        .node("opps")
                        .and_then(|n| n.as_seq())
                        .unwrap_or(&[])
                        .iter()
                        .filter_map(|p| match numbers(p).as_slice() {
                            [k, r] => Some((*k as u8, *r as u8)),
                            _ => None,
                        })
                        .collect(),
                }
            })
            .collect(),
        system_scores: list("system_scores")
            .iter()
            .map(|s| match numbers(s).as_slice() {
                [a, b] => [*a, *b],
                _ => [0.0, 0.0],
            })
            .collect(),
        raid_wins: g.node("raid_wins").map_or(Vec::new(), |n| numbers(n).iter().map(|v| *v as u32).collect()),
        pool_scores: g.node("pool_scores").map_or([0.0, 0.0], |p| match numbers(p).as_slice() {
            [a, b] => [*a, *b],
            _ => [0.0, 0.0],
        }),
        // The activity grid (app NEXT; 0.22.21's per-system `activity` is not read).
        activity: list("activity_grid")
            .iter()
            .filter_map(|a| match numbers(a).as_slice() {
                [m, cell, who, n] => {
                    Some(ActivityCell { month: *m as u32, cell: *cell as u16, who: *who as u8, count: *n as u32 })
                }
                _ => None,
            })
            .collect(),
    })
}

/// An off-screen battle under way (`Campaign::offscreen`), as a mapping: its clock, its
/// units' numbers, the station and the outcomes still to apply.
fn write_offscreen(o: &super::autoresolve::Offscreen) -> Node {
    let unit = |u: &super::autoresolve::Unit| {
        map(vec![
            ("side", n(u.side as f64)),
            ("craft", u.craft.map_or(Node::null(0), |k| n(k as f64))),
            ("class", s(&u.class)),
            (
                "v",
                nums(
                    &[
                        u.hull_max,
                        u.hull,
                        u.shield_max,
                        u.shields[0],
                        u.shields[1],
                        u.armor_max,
                        u.armor[0],
                        u.armor[1],
                        u.beam,
                        u.missile,
                        u.rounds as f32,
                        u.offence,
                        u.attack,
                        if u.alive { 1.0 } else { 0.0 },
                    ]
                    .map(|x| x as f64),
                ),
            ),
        ])
    };
    let station = o.station.as_ref().map_or(Node::null(0), |st| {
        map(vec![
            ("base", n(st.base as f64)),
            (
                "v",
                nums(
                    &[
                        st.hull,
                        st.hull_max,
                        st.pool,
                        st.attack,
                        st.defence,
                        st.offence,
                        st.evasion,
                        st.return_fire[0],
                        st.return_fire[1],
                        st.stage as f32,
                        if st.alive { 1.0 } else { 0.0 },
                    ]
                    .map(|x| x as f64),
                ),
            ),
            ("modules", nums(&st.modules.iter().flat_map(|(m, hp)| [*m as f64, *hp as f64]).collect::<Vec<_>>())),
        ])
    });
    map(vec![
        ("contact", n(o.contact as f64)),
        ("began", n(o.began)),
        ("next_round", n(o.next_round)),
        ("last_round", n(o.last_round)),
        ("units", seq(o.units.iter().map(unit).collect())),
        ("station", station),
        ("destroyed", seq(o.destroyed.iter().map(s).collect())),
        ("wrecked", nums(&o.wrecked.iter().map(|m| *m as f64).collect::<Vec<_>>())),
        (
            "outcomes",
            seq(o
                .outcomes
                .iter()
                .map(|(r, a)| map(vec![("result", s(r)), ("actions", seq(a.iter().map(s).collect()))]))
                .collect()),
        ),
    ])
}

/// Reads [`write_offscreen`]'s mapping back.
fn read_offscreen(node: &Node) -> super::autoresolve::Offscreen {
    let g = Get(node);
    let v = |n: Option<&Node>| -> Vec<f32> {
        n.and_then(|n| n.as_seq()).unwrap_or(&[]).iter().filter_map(|x| x.as_str()?.parse().ok()).collect()
    };
    let at = |v: &[f32], k: usize| v.get(k).copied().unwrap_or(0.0);
    let units = g
        .node("units")
        .and_then(|n| n.as_seq())
        .unwrap_or(&[])
        .iter()
        .map(|u| {
            let u = Get(u);
            let x = v(u.node("v"));
            super::autoresolve::Unit {
                side: u.num("side").unwrap_or(0.0) as u8,
                craft: u.num("craft").map(|k| k as usize),
                class: u.str("class").unwrap_or("").into(),
                hull_max: at(&x, 0),
                hull: at(&x, 1),
                shield_max: at(&x, 2),
                shields: [at(&x, 3), at(&x, 4)],
                armor_max: at(&x, 5),
                armor: [at(&x, 6), at(&x, 7)],
                beam: at(&x, 8),
                missile: at(&x, 9),
                rounds: at(&x, 10) as u32,
                offence: at(&x, 11),
                attack: at(&x, 12),
                alive: at(&x, 13) > 0.5,
            }
        })
        .collect();
    let station = g.node("station").filter(|n| !n.is_null()).map(|n| {
        let st = Get(n);
        let x = v(st.node("v"));
        let m = v(st.node("modules"));
        super::autoresolve::Station {
            base: st.num("base").unwrap_or(0.0) as usize,
            hull: at(&x, 0),
            hull_max: at(&x, 1),
            pool: at(&x, 2),
            attack: at(&x, 3),
            defence: at(&x, 4),
            offence: at(&x, 5),
            evasion: at(&x, 6),
            return_fire: [at(&x, 7), at(&x, 8)],
            stage: at(&x, 9) as u32,
            alive: at(&x, 10) > 0.5,
            modules: m.chunks(2).filter(|c| c.len() == 2).map(|c| (c[0] as usize, c[1])).collect(),
        }
    });
    let strings = |k: &str| -> Vec<String> {
        g.node(k)
            .and_then(|n| n.as_seq())
            .unwrap_or(&[])
            .iter()
            .filter_map(|x| x.as_str().map(str::to_string))
            .collect()
    };
    super::autoresolve::Offscreen {
        contact: g.num("contact").unwrap_or(0.0) as u32,
        units,
        station,
        began: g.num("began").map_or(0.0, |x| x as f64),
        next_round: g.num("next_round").map_or(0.0, |x| x as f64),
        last_round: g.num("last_round").map_or(0.0, |x| x as f64),
        destroyed: strings("destroyed"),
        wrecked: v(g.node("wrecked")).into_iter().map(|m| m as usize).collect(),
        outcomes: g
            .node("outcomes")
            .and_then(|n| n.as_seq())
            .unwrap_or(&[])
            .iter()
            .map(|o| {
                let o = Get(o);
                let acts = o
                    .node("actions")
                    .and_then(|n| n.as_seq())
                    .unwrap_or(&[])
                    .iter()
                    .filter_map(|x| x.as_str().map(str::to_string))
                    .collect();
                (o.str("result").unwrap_or("").to_string(), acts)
            })
            .collect(),
    }
}

// --- The rival's bases and outposts (section `aliens`, #139) -----------------------------

/// The rival's places and clocks (`Campaign::aliens`).
fn write_aliens(a: &super::aliens::AlienState) -> Node {
    map(vec![
        ("started", s(a.started)),
        (
            "bases",
            seq(a
                .bases
                .iter()
                .map(|b| nums(&[b.system as f64, b.variant as f64, b.queued as f64, b.next_queue, b.founded]))
                .collect()),
        ),
        ("outposts", seq(a.outposts.iter().map(|o| nums(&[o.system as f64, o.founded, o.opps as f64])).collect())),
        (
            "clocks",
            nums(&[
                a.next_expand,
                a.next_refound,
                a.next_growth,
                a.next_raid,
                a.next_harass,
                a.last_founded,
                a.piece_clock,
            ]),
        ),
        ("pieces", seq(a.piece_work.iter().map(|(w, b)| nums(&[*w, *b as f64])).collect())),
    ])
}

/// Reads [`write_aliens`] back; places in systems the map no longer has are dropped.
fn read_aliens(node: &Node, systems: usize) -> super::aliens::AlienState {
    let g = Get(node);
    let row =
        |n: &Node| -> Vec<f64> { n.as_seq().unwrap_or(&[]).iter().filter_map(|x| x.as_str()?.parse().ok()).collect() };
    let rows =
        |k: &str| -> Vec<Vec<f64>> { g.node(k).and_then(|n| n.as_seq()).unwrap_or(&[]).iter().map(row).collect() };
    let at = |v: &[f64], k: usize| v.get(k).copied().unwrap_or(0.0);
    let clocks = g.node("clocks").map(row).unwrap_or_default();
    super::aliens::AlienState {
        started: g.bool("started").unwrap_or(false),
        bases: rows("bases")
            .iter()
            .filter(|v| (at(v, 0) as usize) < systems)
            .map(|v| super::aliens::AlienBase {
                system: at(v, 0) as usize,
                variant: at(v, 1) as u8,
                queued: at(v, 2) as u32,
                next_queue: at(v, 3),
                founded: at(v, 4),
            })
            .collect(),
        outposts: rows("outposts")
            .iter()
            .filter(|v| (at(v, 0) as usize) < systems)
            .map(|v| super::aliens::AlienOutpost {
                system: at(v, 0) as usize,
                founded: at(v, 1),
                opps: at(v, 2) as u32,
            })
            .collect(),
        next_expand: at(&clocks, 0),
        next_refound: at(&clocks, 1),
        next_growth: at(&clocks, 2),
        next_raid: at(&clocks, 3),
        next_harass: at(&clocks, 4),
        last_founded: at(&clocks, 5),
        piece_clock: at(&clocks, 6),
        piece_work: rows("pieces").iter().map(|v| (at(v, 0), at(v, 1) as u32)).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::strategy::tests::{build, SAMPLE};

    #[test]
    fn a_save_round_trips_and_refuses_missing_packs() {
        let (d, _) = build(SAMPLE);
        let d = d.unwrap();
        let mut c = Campaign::new(&d, 1, 7);
        c.start_research(&d, 0, d.topic("plasma").unwrap()).unwrap();
        c.buy(&d, 0, Product::Item(0), 2).unwrap();
        c.manufacture(&d, 0, Product::Hull(0), 1).unwrap();
        c.flags.push("met_the_senate".into());
        for _ in 0..40 {
            c.advance(&d, 24.0);
        }
        c.pilots[0].state = PilotState::Injured(c.hours + 10.5);
        // The reports' history and activity (app NEXT) are saved too.
        assert!(!c.history.is_empty(), "40 days close a month");
        c.count_activity(&d, [100.0, 100.0], 1);
        // A wreck and a delivery waiting for room (app 0.19.0) are saved too.
        c.bases[0].modules[0].wrecked = true;
        c.deliveries.push(Delivery {
            product: Product::Item(0),
            base: 0,
            count: 3,
            arrive_at: c.hours + 1.0,
            held: true,
        });
        let packs = [
            PackRef { id: "default".into(), version: "0.6.7".into() },
            PackRef { id: "mymod".into(), version: "1".into() },
        ];
        let text = write(&c, "default", &packs);
        assert_eq!(header(&text).unwrap().0, "default");
        let back = read(&text, &d, &packs).unwrap();
        assert_eq!(back, c, "everything round-trips");
        // A section of another version is refused, naming it (RQ-S-28).
        let newer = text.replace("story: 2", "story: 3");
        assert!(read(&newer, &d, &packs).unwrap_err().contains("story section is version 3"));
        // And the game continues identically from the save.
        let (mut a, mut b) = (c.clone(), back);
        a.advance(&d, 300.0);
        b.advance(&d, 300.0);
        assert_eq!(a, b);
        let err = read(&text, &d, &packs[..1]).unwrap_err();
        assert!(err.contains("mymod"), "{err}");
        // A pack updated since: loads, with a notice.
        let mut newer = packs.to_vec();
        newer[0].version = "9.9.9".into();
        let updated = read(&text, &d, &newer).unwrap();
        assert!(updated.notices.iter().any(|n| n.contains("9.9.9")), "{:?}", updated.notices);
        // A save from other rules (an index out of range) is refused, not a crash.
        let broken = text.replacen("type: 0", "type: 9", 1);
        assert!(read(&broken, &d, &packs).unwrap_err().contains("loaded rules"));
    }
}
