//! A spell or ability whose targets are chosen slot by slot (more than one) and whose cost reads them, over test cards: the
//! total cost is determined once the targets are announced (CR 601.2c, 601.2f, 602.2b), so it may be begun if some legal
//! choice of them could be paid for, and a choice that leaves the cost unpayable takes it back (CR 601.2h, 733.1).

use crate::actions::priority_options;
use crate::{Cx, Game, Setup};
use cantrip_api::*;
use cantrip_state::{Frame, FrameKind};

const YOU: PlayerId = PlayerId(0);
const THEM: PlayerId = PlayerId(1);

static FILLER: CardDef = CardDef { name: "Slot Test Filler", has_cost: false, types: TypeSet::of(&[CardType::Land]), ..CardDef::BLANK };

static BEAR: CardDef = CardDef {
    name: "Slot Test Bear",
    types: TypeSet::of(&[CardType::Creature]),
    power: Some(2),
    toughness: Some(2),
    ..CardDef::BLANK
};

fn permanent(q: &dyn Query, _: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if q.obj(id).zone.is(ZoneKind::Battlefield))
}

/// "This spell costs {3} less to cast if it targets a permanent you control" (This Town Ain't Big Enough's), and the same for
/// an ability of the card.
fn less_if_yours(q: &dyn Query, _: ObjectId, player: PlayerId, _: Costed, targets: &[Target]) -> CostDelta {
    let yours = targets.iter().any(|&t| matches!(t, Target::Object(id) if q.obj(id).zone.is(ZoneKind::Battlefield) && q.obj(id).controller == player));
    if yours { CostDelta::less(3) } else { CostDelta::NONE }
}

fn creature(q: &dyn Query, _: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if q.obj(id).zone.is(ZoneKind::Battlefield) && q.chars(id).is(CardType::Creature))
}

fn gain_three(g: &mut dyn cantrip_api::Game, item: &StackItem) -> Result<(), Suspend> {
    g.gain_life(item.controller, 3);
    Ok(())
}

const FOUR: ManaCost = ManaCost { generic: 4, ..ManaCost::ZERO };

/// {4} instant: "This spell costs {3} less to cast if it targets a permanent you control. Up to two target permanents: you
/// gain 3 life."
static TOWN: CardDef = CardDef {
    name: "Slot Test Town",
    cost: FOUR,
    has_cost: true,
    types: TypeSet::of(&[CardType::Instant]),
    spell: spell(&[up_to(2, permanent)], gain_three),
    own_cost_changes: &[less_if_yours],
    ..CardDef::BLANK
};

/// The same, with two slots of one target each: "target permanent and target permanent".
static PAIR: CardDef = CardDef {
    name: "Slot Test Pair",
    cost: FOUR,
    has_cost: true,
    types: TypeSet::of(&[CardType::Instant]),
    spell: spell(&[target(permanent), target(permanent)], gain_three),
    own_cost_changes: &[less_if_yours],
    ..CardDef::BLANK
};

/// The same, with its one target announced with the cast: "target permanent".
static SINGLE: CardDef = CardDef {
    name: "Slot Test Single",
    cost: FOUR,
    has_cost: true,
    types: TypeSet::of(&[CardType::Instant]),
    spell: spell(&[target(permanent)], gain_three),
    own_cost_changes: &[less_if_yours],
    ..CardDef::BLANK
};

/// "{4}: Up to two target creatures: you gain 3 life. This ability costs {3} less to activate if it targets a permanent you
/// control."
static ENGINE: CardDef = CardDef {
    name: "Slot Test Engine",
    types: TypeSet::of(&[CardType::Artifact]),
    abilities: &[ability(cost("{4}"), gain_three, "{4}: Up to two target creatures: you gain 3 life.").targets(&[up_to(2, creature)])],
    own_cost_changes: &[less_if_yours],
    ..CardDef::BLANK
};

/// Your precombat main phase with priority, nothing anywhere.
fn game() -> Game {
    let table: &'static CardTable = Box::leak(Box::new(CardTable::new([&FILLER, &BEAR, &TOWN, &PAIR, &SINGLE, &ENGINE])));
    let deck = || vec![table.id_of(&FILLER); 20];
    let mut g = Game::new(table, &Setup { decks: vec![deck(), deck()], ..Default::default() });
    while !g.state.agenda.is_empty() {
        g.state.frame_remove(0);
    }
    g.state.update_turn(|t| {
        t.active = YOU;
        t.priority = YOU;
        t.set_step(Step::PrecombatMain);
    });
    g.state.frame_push(Frame::new(FrameKind::Priority));
    g.state.update_player(YOU, |p| p.turn_began = 1);
    g
}

fn put(g: &mut Game, def: &'static CardDef, owner: PlayerId, zone: Zone) -> ObjectId {
    let id = g.state.push_obj(Obj::new(g.cards.id_of(def), owner, zone));
    g.state.zone_push(zone, id);
    id
}

fn cx(game: &mut Game) -> Cx<'_> {
    Cx { g: game, frame: usize::MAX, cursor: 0, mana_source: None, entering: None, lineage: Vec::new(), asked_at: None, doer: None, producing: None }
}

fn add_mana(g: &mut Game, n: u32) {
    g.state.add_mana(YOU, Mana::C, n, ObjectId(0), None, false);
}

fn next(g: &mut Game) -> Request {
    g.run();
    g.pending().cloned().expect("a question")
}

fn answer(g: &mut Game, r: &Request, a: Answer) {
    g.answer(r.id, r.player, a).expect("a legal answer");
}

fn offered(g: &mut Game, source: ObjectId) -> Option<ActionOption> {
    priority_options(&cx(g), YOU).into_iter().find(|o| o.source == Some(source))
}

/// Your Bear and theirs, and `def` in your hand (or on your battlefield) with one mana in your pool.
fn board(def: &'static CardDef, mine: bool) -> (Game, ObjectId, ObjectId, ObjectId) {
    let mut g = game();
    let theirs = put(&mut g, &BEAR, THEM, Zone::BATTLEFIELD);
    let yours = if mine { put(&mut g, &BEAR, YOU, Zone::BATTLEFIELD) } else { theirs };
    let zone = if def.types.has(CardType::Artifact) { Zone::BATTLEFIELD } else { Zone::hand(YOU) };
    let card = put(&mut g, def, YOU, zone);
    add_mana(&mut g, 1);
    (g, card, yours, theirs)
}

/// Begins the action, answers its targets with `targets` slot by slot, and gives back the question after them.
fn begin(g: &mut Game, option: &ActionOption, slots: &[Vec<Target>]) -> Request {
    let r = next(g);
    answer(g, &r, Answer::Act(Action::of(option)));
    let mut r = next(g);
    for chosen in slots {
        assert!(matches!(r.kind, RequestKind::ChooseTarget { .. }), "expected a target slot, got {:?}", r.kind);
        answer(g, &r, Answer::Targets(chosen.clone()));
        r = next(g);
    }
    r
}

/// Pays the pool's one way and passes until the stack is empty.
fn pay_and_resolve(g: &mut Game, mut r: Request) {
    if let RequestKind::PayCosts { options, .. } = &r.kind {
        let confirm = options.iter().find(|o| o.kind == ActionKind::ConfirmCasting).expect("the pool pays it");
        answer(g, &r, Answer::Act(Action::of(confirm)));
        r = next(g);
    }
    while !g.state.stack.is_empty() {
        assert!(matches!(r.kind, RequestKind::Priority { .. }), "expected priority, got {:?}", r.kind);
        answer(g, &r, Answer::Pass);
        r = next(g);
    }
}

#[test]
fn a_spell_of_up_to_two_targets_is_offered_for_what_a_target_of_its_casters_makes_it_cost() {
    let (mut g, town, yours, theirs) = board(&TOWN, true);
    let option = offered(&mut g, town).expect("{1} casts it at a permanent its caster controls (CR 601.2c, 601.2f)");
    let life = g.state.player(YOU).life;
    let r = begin(&mut g, &option, &[vec![yours, theirs].into_iter().map(Target::Object).collect()]);
    pay_and_resolve(&mut g, r);
    assert_eq!(g.state.player(YOU).life, life + 3);
    assert_eq!(g.state.player(YOU).pool.total(), 0, "{{1}} was paid");
}

#[test]
fn a_spell_of_up_to_two_targets_is_not_offered_where_no_target_makes_it_affordable() {
    let (mut g, town, _, _) = board(&TOWN, false);
    assert!(offered(&mut g, town).is_none(), "only the opponent's Bear: it costs {{4}} at any choice of targets");
    add_mana(&mut g, 3);
    assert!(offered(&mut g, town).is_some(), "{{4}} pays for it at full cost");
}

#[test]
fn targets_that_leave_the_cost_unpayable_take_the_cast_back() {
    let (mut g, town, _, theirs) = board(&TOWN, true);
    let option = offered(&mut g, town).expect("offered");
    let life = g.state.player(YOU).life;
    let r = begin(&mut g, &option, &[vec![Target::Object(theirs)]]);
    // {4} with one mana: nothing pays it, and the cast is given up (CR 601.2h, 733.1).
    let RequestKind::PayCosts { options, .. } = &r.kind else { panic!("expected the payment, got {:?}", r.kind) };
    assert!(!options.iter().any(|o| o.kind == ActionKind::ConfirmCasting));
    let cancel = options.iter().find(|o| o.kind == ActionKind::CancelCasting).expect("the cast may be taken back");
    answer(&mut g, &r, Answer::Act(Action::of(cancel)));
    let r = next(&mut g);
    assert!(matches!(r.kind, RequestKind::Priority { .. }), "back to priority, got {:?}", r.kind);
    assert!(g.state.stack.is_empty());
    assert!(g.state.obj(town).zone.is(ZoneKind::Hand), "the spell returns to the hand");
    assert_eq!(g.state.player(YOU).pool.total(), 1, "no payment was made");
    assert_eq!(g.state.player(YOU).life, life);
    // And it may be cast again, at the Bear its caster controls.
    assert!(offered(&mut g, town).is_some());
}

#[test]
fn a_spell_of_two_slots_is_offered_for_what_a_target_of_its_casters_makes_it_cost() {
    let (mut g, pair, yours, theirs) = board(&PAIR, true);
    let option = offered(&mut g, pair).expect("{1} casts it with a slot at a permanent its caster controls");
    let life = g.state.player(YOU).life;
    let r = begin(&mut g, &option, &[vec![Target::Object(theirs)], vec![Target::Object(yours)]]);
    pay_and_resolve(&mut g, r);
    assert_eq!(g.state.player(YOU).life, life + 3);
    let (mut g, pair, _, _) = board(&PAIR, false);
    assert!(offered(&mut g, pair).is_none(), "both slots at the opponent's Bear cost {{4}}");
}

#[test]
fn a_spell_of_one_target_announced_with_the_cast_keeps_the_targets_it_could_be_paid_for() {
    let (mut g, single, yours, _) = board(&SINGLE, true);
    let option = offered(&mut g, single).expect("offered at the caster's Bear");
    assert_eq!(option.target_candidates, vec![Target::Object(yours)]);
    let (mut g, single, _, _) = board(&SINGLE, false);
    assert!(offered(&mut g, single).is_none());
}

#[test]
fn an_ability_of_up_to_two_targets_is_offered_for_what_a_target_of_its_controllers_makes_it_cost() {
    let (mut g, engine, yours, _) = board(&ENGINE, true);
    let option = offered(&mut g, engine).expect("{1} activates it at a permanent its controller controls (CR 602.2b)");
    let life = g.state.player(YOU).life;
    let r = begin(&mut g, &option, &[vec![Target::Object(yours)]]);
    pay_and_resolve(&mut g, r);
    assert_eq!(g.state.player(YOU).life, life + 3);
    let (mut g, engine, _, _) = board(&ENGINE, false);
    assert!(offered(&mut g, engine).is_none(), "only the opponent's Bear: it costs {{4}} at any choice of targets");
}
