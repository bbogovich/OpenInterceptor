//! # The story in play: events, arcs, plans, relations, the strategist, endings (Stage 5)
//!
//! The runtime of [`crate::content::story`]. Each hour of campaign time, [`step`]:
//!
//! 1. checks the events whose period has come round (a day, a week, a month, or a battle
//!    or a finished topic since the last check): an event fires when its condition holds,
//!    its chance comes up, it is not spent (`once`) or cooling down, and — for an arc's
//!    step — it is the arc's next step;
//! 2. works on the rival's plans: once started, every so often a **shipment** leaves as a
//!    raid contact; one that reaches its destination adds a component (one the player
//!    destroys does not); warnings and completion run their actions;
//! 3. once a month, lets relations drift and moves the rival's aggression (raids are more
//!    likely while the player sits back, less while the player is winning);
//! 4. checks the victory and defeat conditions; the first that holds ends the campaign.
//!
//! Actions ([`Campaign::apply_actions`]) are the one place the story changes the
//! campaign: messages, flags, money, score, relations, research, offered missions,
//! raids, plan progress, aggression and endings.

use super::{Campaign, Contact, Event, Message};
use crate::content::story::Check;
use crate::content::strategy::StrategyDef;
use crate::script::action::Action;

/// The story's state in a campaign.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct StoryState {
    /// Per event: times fired, and the hour it last fired.
    pub fired: Vec<u32>,
    pub last_fired: Vec<f64>,
    /// Per arc: steps done.
    pub arc_step: Vec<usize>,
    /// Per plan: started?, components done, the hour of the next shipment roll, warnings given.
    pub plan_started: Vec<bool>,
    pub plan_done: Vec<u32>,
    pub plan_next: Vec<f64>,
    pub plan_warned: Vec<usize>,
    /// Faction id → standing with the player.
    pub standing: Vec<(String, f32)>,
    /// The rival's aggression (multiplies raid chances).
    pub aggression: f32,
    pub battles_won: u32,
    /// Battles won since the last monthly review.
    pub month_wins: u32,
    /// A battle was fought / a topic finished since the last check (after_battle, on_research).
    pub battle_pending: bool,
    pub research_pending: bool,
    /// The ending reached (index into `story.endings`).
    pub ending: Option<usize>,
}

impl StoryState {
    pub fn new(def: &StrategyDef) -> StoryState {
        let s = &def.story;
        StoryState {
            fired: vec![0; s.events.len()],
            last_fired: vec![f64::NEG_INFINITY; s.events.len()],
            arc_step: vec![0; s.arcs.len()],
            plan_started: vec![false; s.plans.len()],
            plan_done: vec![0; s.plans.len()],
            plan_next: vec![0.0; s.plans.len()],
            plan_warned: vec![0; s.plans.len()],
            standing: s.relations.as_ref().map(|r| r.start.clone()).unwrap_or_default(),
            aggression: s.strategist.start,
            ..Default::default()
        }
    }

    /// A faction's standing (0 when not listed).
    pub fn standing(&self, faction: &str) -> f32 {
        self.standing.iter().find(|(f, _)| f == faction).map(|(_, v)| *v).unwrap_or(0.0)
    }

    /// A plan's progress, 0..1.
    pub fn plan_progress(&self, def: &StrategyDef, plan: usize) -> f32 {
        (self.plan_done[plan] as f32 / def.story.plans[plan].total() as f32).min(1.0)
    }
}

impl Campaign {
    /// Whether raids of `faction` may fly: with dynamic relations, only while it is
    /// hostile (a bribed or befriended faction stops raiding).
    pub fn raids_allowed(&self, def: &StrategyDef, faction: &str) -> bool {
        match &def.story.relations {
            Some(r) if r.dynamic && self.story.standing.iter().any(|(f, _)| f == faction) => {
                r.stance(self.story.standing(faction)) == "hostile"
            }
            _ => true,
        }
    }

    /// Applies a list of story actions.
    pub fn apply_actions(&mut self, def: &StrategyDef, actions: &[Action], events: &mut Vec<Event>) {
        for a in actions {
            let arg = |i: usize| a.args.get(i).map(String::as_str).unwrap_or("");
            let num = |i: usize| arg(i).trim_start_matches('+').parse::<f32>().unwrap_or(0.0);
            match a.name {
                "message" => {
                    let t = arg(0).to_string();
                    self.log(t.clone());
                    events.push(Event::Notice(t));
                }
                "news" => self.log(format!("News: {}", arg(0))),
                "video" => events.push(Event::Video { reference: arg(0).to_string() }),
                "card" => {
                    let (title, text) = arg(0).split_once('|').unwrap_or((arg(0), ""));
                    self.log(title.trim().to_string());
                    events.push(Event::Card { title: title.trim().to_string(), text: text.trim().to_string() });
                }
                "set_flag" => {
                    if !self.flags.iter().any(|f| f == arg(0)) {
                        self.flags.push(arg(0).to_string());
                    }
                }
                "clear_flag" => self.flags.retain(|f| f != arg(0)),
                "money" => self.money += num(0) as f64,
                "score" => {
                    let v = def.score(arg(0)) as f64;
                    self.score += v;
                    self.month_score += v;
                }
                "relation" => self.change_standing(def, arg(0), num(1), events),
                "research" => {
                    if let Some(t) = def.topic(arg(0)) {
                        self.known[t] = true;
                    }
                }
                "mission" => events.push(Event::Mission { mission: arg(0).to_string() }),
                "raid" => {
                    if let Some(r) = def.raids.iter().position(|x| x.id == arg(0)) {
                        if !self.bases.is_empty() {
                            self.launch(def, r);
                        }
                    }
                }
                "plan" => {
                    if let Some(p) = def.story.plan(arg(0)) {
                        self.add_plan_progress(def, p, num(1).max(0.0) as u32, events);
                    }
                }
                "aggression" => {
                    let s = def.story.strategist;
                    self.story.aggression =
                        (self.story.aggression + num(0)).clamp(s.min.min(s.start), s.max.max(s.start));
                }
                "victory" | "defeat" => {
                    if let (None, Some(e)) = (self.story.ending, def.story.ending(arg(0))) {
                        self.story.ending = Some(e);
                        events.push(Event::Ending { ending: e });
                    }
                }
                _ => {}
            }
        }
    }

    /// Changes a faction's standing; announces a change of stance.
    fn change_standing(&mut self, def: &StrategyDef, faction: &str, by: f32, events: &mut Vec<Event>) {
        let Some(r) = def.story.relations.as_ref().filter(|r| r.dynamic) else { return };
        let before = self.story.standing(faction);
        let after = (before + by).clamp(-100.0, 100.0);
        match self.story.standing.iter_mut().find(|(f, _)| f == faction) {
            Some((_, v)) => *v = after,
            None => self.story.standing.push((faction.to_string(), after)),
        }
        let (old, new) = (r.stance(before).to_string(), r.stance(after).to_string());
        if old != new {
            let t = format!("Relations: {faction} are now {new} (were {old})");
            self.log(t.clone());
            events.push(Event::Notice(t));
        }
    }

    /// Components added to a plan: warnings as it passes them, completion at the end.
    fn add_plan_progress(&mut self, def: &StrategyDef, plan: usize, n: u32, events: &mut Vec<Event>) {
        let p = &def.story.plans[plan];
        let total = p.total();
        if self.story.plan_done[plan] >= total {
            return;
        }
        self.story.plan_done[plan] = (self.story.plan_done[plan] + n).min(total);
        let progress = self.story.plan_progress(def, plan);
        while self.story.plan_warned[plan] < p.warnings.len() && progress >= p.warnings[self.story.plan_warned[plan]].0
        {
            let actions = p.warnings[self.story.plan_warned[plan]].1.clone();
            self.story.plan_warned[plan] += 1;
            self.apply_actions(def, &actions, events);
        }
        if self.story.plan_done[plan] >= total {
            let actions = p.complete.clone();
            self.apply_actions(def, &actions, events);
        }
    }

    /// A shipment of a plan reached its destination.
    pub(crate) fn shipment_arrived(&mut self, def: &StrategyDef, contact: &Contact, events: &mut Vec<Event>) {
        if let Some(p) = contact.plan {
            let name = def.story.plans[p].id.clone();
            self.log(format!("A shipment for the rival's {name} got through"));
            self.add_plan_progress(def, p, 1, events);
        }
    }

    /// The player won a battle (called from `battle_result`).
    pub(crate) fn battle_won(&mut self) {
        self.story.battles_won += 1;
        self.story.month_wins += 1;
    }

    /// A story mission (offered by an event) has been flown: its outcome's campaign
    /// actions apply, and a win counts.
    pub fn mission_result(&mut self, def: &StrategyDef, result: &str, actions: &[String], kills: u32) {
        // No contact is involved: `battle_result` with an id no contact has applies the
        // outcome's actions (score, money, flags, salvage) and the win.
        self.battle_result(def, u32::MAX - 1, result, actions, kills);
    }

    /// The player chose option `choice` of event `event`.
    pub fn choose(&mut self, def: &StrategyDef, event: usize, choice: usize) -> Vec<Event> {
        let mut events = Vec::new();
        if let Some(c) = def.story.events.get(event).and_then(|e| e.choices.get(choice)) {
            let actions = c.actions.clone();
            self.log(format!("Chose: {}", c.text));
            self.apply_actions(def, &actions, &mut events);
        }
        events
    }

    /// The encyclopaedia entries unlocked now: `(title, text)`.
    pub fn encyclopaedia(&mut self, def: &StrategyDef) -> Vec<(String, String)> {
        (0..def.story.encyclopaedia.len())
            .filter(|&k| self.holds(def, def.story.encyclopaedia[k].unlock.as_ref()))
            .map(|k| (def.story.encyclopaedia[k].title.clone(), def.story.encyclopaedia[k].text.clone()))
            .collect()
    }
}

/// One slice of story time (`dt` hours that ended at `c.hours`).
pub fn step(c: &mut Campaign, def: &StrategyDef, dt: f64, events: &mut Vec<Event>) {
    let s = &def.story;
    if c.story.ending.is_some() {
        return;
    }
    let (now, before) = (c.hours, c.hours - dt);
    let crossed = |period: f64| (now / period).floor() > (before / period).floor();
    let due = |k: Check, c: &Campaign| match k {
        Check::Daily => crossed(24.0),
        Check::Weekly => crossed(168.0),
        Check::Monthly => crossed(720.0),
        Check::AfterBattle => c.story.battle_pending,
        Check::OnResearch => c.story.research_pending,
    };
    // 1. Events.
    for e in 0..s.events.len() {
        let ev = &s.events[e];
        if !due(ev.check, c) || (ev.once && c.story.fired[e] > 0) || now - c.story.last_fired[e] < ev.cooldown as f64 {
            continue;
        }
        if let Some(a) = ev.arc {
            if s.arcs[a].steps.get(c.story.arc_step[a]) != Some(&e) {
                continue;
            }
        }
        if !c.holds(def, ev.when.as_ref()) || c.roll() >= ev.chance {
            continue;
        }
        c.story.fired[e] += 1;
        c.story.last_fired[e] = now;
        if let Some(a) = ev.arc {
            c.story.arc_step[a] += 1;
        }
        let actions = ev.actions.clone();
        c.apply_actions(def, &actions, events);
        if !ev.choices.is_empty() {
            events.push(Event::Choice { event: e });
        }
    }
    c.story.battle_pending = false;
    c.story.research_pending = false;
    // 2. Rival plans.
    for p in 0..s.plans.len() {
        let plan = &s.plans[p];
        if !c.story.plan_started[p] {
            if c.holds(def, plan.start_when.as_ref()) {
                c.story.plan_started[p] = true;
                c.story.plan_next[p] = now + plan.every as f64;
                c.log(format!("Intelligence: the rival's {} has begun", plan.id));
            }
            continue;
        }
        if c.story.plan_done[p] >= plan.total() || now < c.story.plan_next[p] {
            continue;
        }
        c.story.plan_next[p] = now + plan.every as f64;
        let Some(r) = plan.raid else { continue };
        if c.bases.is_empty() || c.roll() >= plan.chance {
            continue;
        }
        c.launch(def, r);
        if let Some(k) = c.contacts.last_mut() {
            k.plan = Some(p);
        }
    }
    // 3. The month: relations drift; the rival's mood.
    if crossed(720.0) {
        if let Some(r) = s.relations.as_ref().filter(|r| r.dynamic && r.drift > 0.0) {
            for (_, v) in &mut c.story.standing {
                *v = if *v > 0.0 { (*v - r.drift).max(0.0) } else { (*v + r.drift).min(0.0) };
            }
        }
        let st = s.strategist;
        c.story.aggression = if c.story.month_wins >= st.calm_after_wins && st.calm_after_wins > 0 {
            (c.story.aggression - st.down).max(st.min)
        } else {
            (c.story.aggression + st.up).min(st.max)
        };
        c.story.month_wins = 0;
    }
    // 4. Victory and defeat.
    if crossed(24.0) || events.iter().any(|e| matches!(e, Event::Ending { .. })) {
        check_end(c, def, events);
    }
}

/// Checks the campaign's victory and defeat conditions (defeats first).
pub fn check_end(c: &mut Campaign, def: &StrategyDef, events: &mut Vec<Event>) {
    if c.story.ending.is_some() {
        return;
    }
    let s = &def.story;
    for (cond, ending) in s.defeat.iter().chain(&s.victory) {
        if c.holds(def, Some(cond)) {
            c.story.ending = Some(*ending);
            let t = s.endings[*ending].text.clone();
            c.messages.push(Message { hour: c.hours, text: t });
            events.push(Event::Ending { ending: *ending });
            return;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::campaign::Event;
    use crate::content::strategy::tests::{build, SAMPLE};

    /// The strategy sample plus a story: a two-step arc, a choice, a repeating event, a
    /// plan built by the sample's raid, relations, a strategist and endings.
    fn story_sample() -> String {
        SAMPLE.replace("flags: [raids]", "flags: [raids, act1, paid, built]")
            + "\
events:
  act_one: {when: day >= 2, do: [\"message(Act one)\", set_flag(act1)]}
  act_two: {when: day >= 1, do: [\"mission(m1)\"]}
  bribe: {when: flag(act1), choices: [{text: Pay, do: [\"money(-1000)\", \"relation(enemy, 80)\", set_flag(paid)]}, {text: Refuse, do: [\"relation(enemy, -10)\"]}]}
  tick: {when: day >= 0, once: false, cooldown: 3d, check: daily, do: [\"news(Tick)\"]}
arcs: {main: {steps: [act_one, act_two]}}
plans:
  engine: {owner: enemy, stages: [{name: Core, count: 2}], start_when: flag(built), every: 1d, raid: raid, warnings: [{at: 50%, do: [\"message(Half built)\"]}], complete: [\"defeat(lit)\"]}
relations: {dynamic: true, start: {enemy: -60}, drift: 5}
strategist: {aggression: {start: 1, min: 0.5, max: 2, up: 0.25, down: 0.25}, calm_after_wins: 1}
endings: {lit: {text: The engine is lit}, saved: {text: Saved, victory: true}}
campaign_end: {victory: [{when: \"flag(saved_flag_never) or battles_won >= 99\", ending: saved}]}
encyclopaedia: {e1: {title: Act one, text: It began, unlock: arc(main) >= 1}}
"
    }

    fn run(c: &mut Campaign, d: &StrategyDef, hours: f64) -> Vec<Event> {
        let end = c.hours + hours;
        let mut all = Vec::new();
        while c.hours < end && c.story.ending.is_none() {
            all.extend(c.advance(d, end - c.hours));
        }
        all
    }

    #[test]
    fn arcs_fire_in_order_with_choices_missions_and_cooldowns() {
        let (d, errors) = build(&story_sample().replace("saved_flag_never", "paid_never"));
        assert!(errors.iter().all(|e| !e.contains("events") && !e.contains("arcs")), "{errors:?}");
        let d = d.unwrap();
        let mut c = Campaign::new(&d, 0, 4);
        let events = run(&mut c, &d, 30.0);
        // act_two's condition holds from day 1, but it is the arc's second step.
        assert!(!events.iter().any(|e| matches!(e, Event::Mission { .. })), "not before act one");
        let events = run(&mut c, &d, 48.0);
        assert!(events.iter().any(|e| matches!(e, Event::Notice(t) if t == "Act one")));
        assert!(events.iter().any(|e| matches!(e, Event::Mission { mission } if mission == "m1")), "{events:?}");
        assert_eq!(c.story.arc_step[0], 2);
        let bribe = d.story.event("bribe").unwrap();
        assert!(events.contains(&Event::Choice { event: bribe }));
        // Choosing to pay: money, standing (enemy no longer hostile: no raids), flag.
        let money = c.money;
        c.choose(&d, bribe, 0);
        assert_eq!(c.money, money - 1000.0);
        assert_eq!(c.story.standing("enemy"), 20.0);
        assert!(!c.raids_allowed(&d, "enemy"));
        // The repeating event: every 3 days, not every day.
        let ticks = c.messages.iter().filter(|m| m.text == "News: Tick").count();
        run(&mut c, &d, 24.0 * 6.0);
        let more = c.messages.iter().filter(|m| m.text == "News: Tick").count() - ticks;
        assert!((1..=3).contains(&more), "{more}");
        assert_eq!(c.encyclopaedia(&d).len(), 1);
    }

    #[test]
    fn plans_grow_from_shipments_that_get_through_and_can_end_the_campaign() {
        let (d, _) = build(&story_sample());
        let d = d.unwrap();
        let mut c = Campaign::new(&d, 0, 4);
        c.flags.push("built".into()); // starts the plan (the sample's own raids stay off)
        let events = run(&mut c, &d, 24.0 * 20.0);
        assert!(c.story.plan_started[0]);
        assert!(events.iter().any(|e| matches!(e, Event::Notice(t) if t == "Half built")), "{events:?}");
        assert_eq!(c.story.ending, d.story.ending("lit"), "two shipments got through: the engine is lit");
        assert!(events.iter().any(|e| matches!(e, Event::Ending { .. })));
    }

    #[test]
    fn relations_drift_and_the_strategist_answers_the_player() {
        let (d, _) = build(&story_sample());
        let d = d.unwrap();
        let mut c = Campaign::new(&d, 0, 4);
        run(&mut c, &d, 721.0);
        assert_eq!(c.story.standing("enemy"), -55.0, "drifts 5 towards 0 a month");
        assert_eq!(c.story.aggression, 1.25, "no wins: the rival grows bolder");
        c.battle_won();
        run(&mut c, &d, 720.0);
        assert_eq!(c.story.aggression, 1.0, "a win calms it");
        // Saved and loaded with the story.
        let packs = [crate::campaign::save::PackRef { id: "p".into(), version: "1".into() }];
        let text = crate::campaign::save::write(&c, "x", &packs);
        assert_eq!(crate::campaign::save::read(&text, &d, &packs).unwrap(), c);
    }
}
