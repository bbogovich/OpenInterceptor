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
        ])
    };
    let doc = map(vec![
        ("save", n(FORMAT as f64)),
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
                        ("modules", seq(b.modules.iter().map(|m| nums(&[m.module as f64, m.ready_at])).collect())),
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
                    ])
                })
                .collect()),
        ),
        ("pilots", seq(c.pilots.iter().map(pilot).collect())),
        ("recruits", seq(c.recruits.iter().map(pilot).collect())),
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
    ]);
    format!("# OpenInterceptor campaign save\n{}", doc.to_yaml())
}

/// Reads a save's header: the campaign id and the packs it was made with (to choose and
/// check what to load before reading the rest).
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
    let doc = yaml::parse(text).map_err(|e| format!("save: {e}"))?;
    let g = Get(&doc);
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
            }
        })
        .collect();
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
            modules.push(ModuleInst { module, ready_at: v.get(1).copied().unwrap_or(0.0) });
        }
        let mut jobs = Vec::new();
        for j in b.node("jobs").and_then(|n| n.as_seq()).unwrap_or(&[]) {
            let j = Get(j);
            jobs.push(Job {
                product: product(j.str("product").unwrap_or(""))?,
                left: f(j, "left") as f32,
                count: f(j, "count") as u32,
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
        });
    }
    let strings =
        |key: &str| -> Vec<String> { list(key).iter().filter_map(|x| x.as_str().map(str::to_string)).collect() };
    let next_flow = g.node("next_flow").map(numbers).unwrap_or_default();
    let next_raid = g.node("next_raid").map(numbers).unwrap_or_default();
    if next_flow.len() != def.incomes.len() + def.expenses.len() || next_raid.len() != def.raids.len() {
        return Err(bad("the incomes, expenses or raids"));
    }
    Ok(Campaign {
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
    })
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
        let packs = [
            PackRef { id: "default".into(), version: "0.6.7".into() },
            PackRef { id: "mymod".into(), version: "1".into() },
        ];
        let text = write(&c, "default", &packs);
        assert_eq!(header(&text).unwrap().0, "default");
        let back = read(&text, &d, &packs).unwrap();
        assert_eq!(back, c, "everything round-trips");
        // And the game continues identically from the save.
        let (mut a, mut b) = (c.clone(), back);
        a.advance(&d, 300.0);
        b.advance(&d, 300.0);
        assert_eq!(a, b);
        let err = read(&text, &d, &packs[..1]).unwrap_err();
        assert!(err.contains("mymod"), "{err}");
        // A save from other rules (an index out of range) is refused, not a crash.
        let broken = text.replacen("type: 0", "type: 9", 1);
        assert!(read(&broken, &d, &packs).unwrap_err().contains("loaded rules"));
    }
}
