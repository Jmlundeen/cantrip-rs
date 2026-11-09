//! Actual Promise of Loyalty: independent semantic probes through deterministic games and the public API.
use cantrip_api::Game as Rules;
use cantrip_engine::*;
use cantrip_mechanics::targets;
use cantrip_script::{GameScript, run, run_on};
use std::sync::OnceLock;
const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const BASIC: &str = include_str!("../../../scenarios/promise-of-loyalty-both-players-keep-one-and-new-creatures-can-attack.cantrip");
fn target(item: &StackItem, n: usize) -> ObjectId {
    item.target(n).object().unwrap()
}
fn remove_refill(g: &mut dyn Rules, item: &StackItem) -> Result<(), Suspend> {
    let id = target(item, 0);
    cantrip_api::instruction(g, |g| {
        g.remove_counters(id, CounterKind::Vow, 1);
        g.add_counters(id, CounterKind::Vow, 1);
    });
    Ok(())
}
fn move_vow(g: &mut dyn Rules, item: &StackItem) -> Result<(), Suspend> {
    g.move_counters(target(item, 0), target(item, 1), CounterKind::Vow, 1);
    Ok(())
}
fn blink_vow(g: &mut dyn Rules, item: &StackItem) -> Result<(), Suspend> {
    let id = target(item, 0);
    let controller = g.controller(id);
    let card = g.exile(id);
    let new = g.put_onto_battlefield(card, controller, false)?;
    g.add_counters(new, CounterKind::Vow, 1);
    Ok(())
}
fn mute(g: &mut dyn Rules, item: &StackItem) -> Result<(), Suspend> {
    g.add_effect(item.object, target(item, 0), EffectKind::LoseAllAbilities, Duration::Indefinite);
    Ok(())
}
const ONE: [TargetSpec; 1] = [targets::CREATURE];
const TWO: [TargetSpec; 2] = [targets::CREATURE, targets::CREATURE];
const fn probe(name: &'static str, ts: &'static [TargetSpec], effect: fn(&mut dyn Rules, &StackItem) -> Result<(), Suspend>) -> CardDef {
    CardDef { name, oracle: true, types: TypeSet::of(&[CardType::Instant]), spell: spell(ts, effect), ..CardDef::BLANK }
}
static REFILL: CardDef = probe("Probe Remove Refill", &ONE, remove_refill);
static MOVE: CardDef = probe("Probe Move Vow", &TWO, move_vow);
static BLINK: CardDef = probe("Probe Blink Vow", &ONE, blink_vow);
static MUTE: CardDef = probe("Probe Lose Abilities", &ONE, mute);
fn table() -> &'static CardTable {
    static TABLE: OnceLock<CardTable> = OnceLock::new();
    TABLE.get_or_init(|| CardTable::new(cantrip_cards::all().into_iter().chain([&REFILL, &MOVE, &BLINK, &MUTE, &WITNESS])))
}
fn play(text: &str) -> Game {
    let script = GameScript::parse(text, "promise-semantic-probe").unwrap();
    run_on(&script, table()).unwrap_or_else(|e| panic!("{}", e.report)).game
}
fn setup(probe: &str) -> String {
    let basic = BASIC.replace("\r", "");
    basic[..basic.find("[T4 Bot2.Main1]").unwrap()].replace("Memnite\nPlains\nPlains x16", &format!("{probe}\nPlains\nPlains x16"))
}
fn creature(g: &Game, name: &str, p: PlayerId) -> ObjectId {
    g.state.battlefield().iter().copied().find(|&id| g.name(id) == name && g.state.obj(id).controller == p).unwrap()
}
#[test]
fn removing_and_readding_during_one_resolution_does_not_restore_the_promise() {
    let mut s = setup("Probe Remove Refill");
    s.push_str("[T4 Bot2.Main1]\nBot2.Hand.Probe Remove Refill.Cast(Bot2.Battlefield.Grizzly Bears)\nBot2.Pass\nBot1.Pass\n? Bot2.Battlefield.Grizzly Bears.Counters.Vow = 1\nBot2.Attack(Grizzly Bears > Bot1)\n[T4 Bot2.Main2]\n[Expect]\nBot1.Life = 18\n");
    let g = play(&s);
    let id = creature(&g, "Grizzly Bears", P1);
    assert!(
        !g.state.effects.iter().any(|e| e.affected == id && matches!(e.kind, EffectKind::CantAttackPlayer { .. })),
        "expired Promise effect was retained"
    );
}
#[test]
fn a_moved_vow_does_not_bind_its_recipient() {
    let mut s = setup("Probe Move Vow");
    s = s.replace("Plains\nPlains x16", "Ornithopter\nPlains x16");
    s.push_str("[T4 Bot2.Main1]\nBot2.Hand.Ornithopter.Cast\nBot2.Pass\nBot1.Pass\nBot2.Hand.Probe Move Vow.Cast\nBot2.Target(Bot2.Battlefield.Grizzly Bears)\nBot2.Target(Bot2.Battlefield.Ornithopter)\nBot2.Pass\nBot1.Pass\n? Bot2.Battlefield.Grizzly Bears.Counters.Vow = (absent)\n? Bot2.Battlefield.Ornithopter.Counters.Vow = 1\n[T6 Bot2.Main1]\nBot2.Attack(Grizzly Bears > Bot1; Ornithopter > Bot1)\n[T6 Bot2.Main2]\n[Expect]\nBot1.Life = 18\n");
    play(&s);
}
#[test]
fn a_flickered_object_with_another_vow_is_free_of_the_old_promise() {
    let mut s = setup("Probe Blink Vow");
    s.push_str("[T4 Bot2.Main1]\nBot2.Hand.Probe Blink Vow.Cast(Bot2.Battlefield.Grizzly Bears)\nBot2.Pass\nBot1.Pass\n? Bot2.Battlefield.Grizzly Bears.Counters.Vow = 1\n[T6 Bot2.Main1]\nBot2.Attack(Grizzly Bears > Bot1)\n[T6 Bot2.Main2]\n[Expect]\nBot1.Life = 18\n");
    play(&s);
}
#[test]
fn losing_all_abilities_cannot_remove_a_resolved_restriction() {
    let mut s = setup("Probe Lose Abilities");
    s = s.replace("Plains\nPlains x16", "Memnite\nPlains x16");
    s.push_str("[T4 Bot2.Main1]\nBot2.Hand.Memnite.Cast\nBot2.Hand.Probe Lose Abilities.Cast(Bot2.Battlefield.Grizzly Bears)\nBot2.Pass\nBot1.Pass\n[T6 Bot2.Main1]\nBot2.Pass\n! Bot2.Attack(Grizzly Bears > Bot1)\nBot2.Attack(Memnite > Bot1)\n[T6 Bot2.Main2]\n[Expect]\nBot1.Life = 19\n");
    play(&s);
}
#[test]
fn mandatory_own_creature_choices_and_exact_tree_undo_redo() {
    let before_choice = &BASIC[..BASIC.find("Bot1.Choose(Grizzly Bears)").unwrap()];
    let text = format!("{before_choice}[Handover Bot1.Choose]\n");
    let mut g = run(&GameScript::parse(&text, "promise-choice-probe").unwrap()).unwrap_or_else(|e| panic!("{}", e.report)).game;
    let first = g.pending().cloned().unwrap();
    assert_eq!(first.player, P0);
    let mine = creature(&g, "Grizzly Bears", P0);
    let theirs = creature(&g, "Grizzly Bears", P1);
    let state = g.state.clone();
    let log = g.log.clone();
    assert!(g.answer(first.id, P0, Answer::Cards(Vec::new())).is_err());
    assert!(g.state == state && g.log == log, "refused empty choice changed the state");
    assert!(g.answer(first.id, P0, Answer::Cards(vec![theirs])).is_err());
    g.answer(first.id, P0, Answer::Cards(vec![mine])).unwrap();
    let first_done = g.at;
    let next = g.pending().cloned().unwrap();
    assert_eq!(next.player, P1);
    assert_eq!(g.state.obj(mine).counters(CounterKind::Vow), 0);
    assert_eq!(g.state.obj(theirs).counters(CounterKind::Vow), 0);
    assert_eq!(g.state.battlefield().iter().filter(|&&id| g.chars(id).is(CardType::Creature)).count(), 5, "no sacrifice before all choices");
    g.answer(next.id, P1, Answer::Cards(vec![theirs])).unwrap();
    let done = g.at;
    let state = g.state.clone();
    let log = g.log.clone();
    assert_eq!(g.state.obj(mine).counters(CounterKind::Vow), 1);
    assert_eq!(g.state.obj(theirs).counters(CounterKind::Vow), 1);
    g.goto(first_done);
    assert_eq!(g.state.obj(mine).counters(CounterKind::Vow), 0);
    assert_eq!(g.state.obj(theirs).counters(CounterKind::Vow), 0);
    g.goto(done);
    assert!(g.state == state && g.log == log, "choice/counter/sacrifice/effect exact tree restoration");
}

// These predicates are observers: they assert the board the simultaneous event is judged on.
fn counter_batch(q: &dyn Query, _: ObjectId, event: &Event) -> Option<Fired> {
    if let Event::CountersPut { object, kind: CounterKind::Vow, by, .. } = *event {
        assert_eq!(by, q.controller(object), "each player puts their own vow");
        assert_eq!(
            q.state().battlefield().iter().filter(|&&id| q.state().obj(id).counters(CounterKind::Vow) > 0).count(),
            2,
            "all vows are placed before the first counter event"
        );
        assert_eq!(
            q.state().battlefield().iter().filter(|&&id| q.chars(id).is(CardType::Creature)).count(),
            5,
            "counter placement precedes every sacrifice"
        );
        Some(Fired { subject: Target::Object(object), data: [0, 0] })
    } else {
        None
    }
}
fn sacrifice_batch(q: &dyn Query, _: ObjectId, event: &Event) -> Option<Fired> {
    if let Event::ZoneChange { old, reason: ZoneReason::Sacrifice, .. } = *event {
        if !q.chars(old).is(CardType::Creature) {
            return None;
        }
        assert_eq!(
            q.permanents_of(P0).into_iter().chain(q.permanents_of(P1)).filter(|&id| q.chars(id).is(CardType::Creature)).count(),
            5,
            "every sacrifice looks back to the same five-creature board"
        );
        Some(Fired { subject: Target::Object(old), data: [0, 0] })
    } else {
        None
    }
}
fn gain_one(g: &mut dyn Rules, item: &StackItem) -> Result<(), Suspend> {
    g.gain_life(item.controller, 1);
    Ok(())
}
static WITNESS: CardDef = CardDef {
    name: "Probe Promise Witness",
    oracle: true,
    types: TypeSet::of(&[CardType::Enchantment]),
    abilities: &[
        triggered(when(EventMask::COUNTERS, counter_batch), gain_one, "Whenever one or more vow counters are put, gain 1 life.").once_per_batch(),
        triggered(when(EventMask::ZONE_CHANGE, sacrifice_batch), gain_one, "Whenever one or more creatures are sacrificed, gain 1 life.")
            .once_per_batch(),
    ],
    ..CardDef::BLANK
};
#[test]
fn counters_and_sacrifices_are_separate_simultaneous_events() {
    let mut text = BASIC.replace("\r", "");
    text = text[..text.find("[T4 Bot2.Main1]").unwrap()].to_string();
    text = text.replace("Plains x2\nPlains x16", "Plains x2\nProbe Promise Witness\nPlains x16");
    text = text.replace("Bot1.Hand.Promise of Loyalty.Cast", "Bot1.Hand.Probe Promise Witness.Cast\nBot1.Hand.Promise of Loyalty.Cast");
    text.push_str("[Expect]\nBot1.Life = 22\n");
    play(&text);
}
#[test]
fn two_controllers_resolve_independent_promises_in_rotated_apnap_order() {
    let g = play(include_str!("../../../scenarios/promise-of-loyalty-two-controllers-and-rotated-apnap.cantrip"));
    for p in [P0, P1] {
        let id = creature(&g, "Grizzly Bears", p);
        assert_eq!(g.state.obj(id).counters(CounterKind::Vow), 2);
        let protected: Vec<_> = g
            .state
            .effects
            .iter()
            .filter_map(|e| {
                if e.affected == id { if let EffectKind::CantAttackPlayer { player, .. } = e.kind { Some(player) } else { None } } else { None }
            })
            .collect();
        assert_eq!(protected, vec![P0, P1]);
    }
}
#[test]
fn the_casters_own_chosen_creature_keeps_the_vow_after_being_stolen() {
    let g = play(include_str!("../../../scenarios/promise-of-loyalty-stolen-casters-creature-remembers.cantrip"));
    let id = g.state.battlefield().iter().copied().find(|&id| g.name(id) == "Grizzly Bears" && g.state.obj(id).owner == P0).unwrap();
    assert_eq!(g.state.obj(id).controller, P1);
    assert!(g.state.effects.iter().any(|e| e.affected == id && e.kind == EffectKind::CantAttackPlayer { player: P0, planeswalkers: true }));
}
