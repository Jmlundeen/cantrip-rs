//! "X can't be 0" (CR 107.3a, 601.2b, 602.2b), over test cards: the announcement of an activated ability's X and of a
//! spell's X starts at 1 and refuses 0; neither is offered unless it could be paid with X as 1; and a spell is not cast
//! for a cost where X could only be 0 (CR 107.3b; Mind Grind's ruling).

use crate::actions::priority_options;
use crate::{Cx, Game, Setup};
use cantrip_api::*;
use cantrip_state::{Frame, FrameKind};

const YOU: PlayerId = PlayerId(0);

fn nothing(_: &mut dyn cantrip_api::Game, _: &StackItem) -> Result<(), Suspend> {
    Ok(())
}

static FILLER: CardDef = CardDef { name: "X Test Filler", has_cost: false, types: TypeSet::of(&[CardType::Land]), ..CardDef::BLANK };

/// "{X}{G}: ... X can't be 0." (Lair of the Hydra's cost.)
static LAIR: CardDef = CardDef {
    name: "X Test Lair",
    has_cost: false,
    types: TypeSet::of(&[CardType::Land]),
    abilities: &[ability(of_mana(mana("{X}{G}")), nothing, "{X}{G}: Nothing. X can't be 0.").x_cant_be_zero()],
    ..CardDef::BLANK
};

/// "{X}{G}: ..." with no word on X: X may be 0.
static DEN: CardDef = CardDef {
    name: "X Test Den",
    has_cost: false,
    types: TypeSet::of(&[CardType::Land]),
    abilities: &[ability(of_mana(mana("{X}{G}")), nothing, "{X}{G}: Nothing.")],
    ..CardDef::BLANK
};

/// A {X}{U} sorcery: "X can't be 0." It may also be cast for {U}, a cost with no {X}.
static GRIND: CardDef = CardDef {
    name: "X Test Grind",
    cost: mana("{X}{U}"),
    types: TypeSet::of(&[CardType::Sorcery]),
    spell: spell(&[], nothing),
    ways: &[way(of_mana(mana("{U}")), ZoneKind::Hand, "You may pay {U} rather than pay this spell's mana cost.")],
    x_cant_be_zero: true,
    ..CardDef::BLANK
};

/// The same sorcery with no word on X.
static SIFT: CardDef = CardDef {
    name: "X Test Sift",
    cost: mana("{X}{U}"),
    types: TypeSet::of(&[CardType::Sorcery]),
    spell: spell(&[], nothing),
    ways: &[way(of_mana(mana("{U}")), ZoneKind::Hand, "You may pay {U} rather than pay this spell's mana cost.")],
    ..CardDef::BLANK
};

/// Your precombat main phase with priority, nothing anywhere.
fn game() -> Game {
    let table: &'static CardTable = Box::leak(Box::new(CardTable::new([&FILLER, &LAIR, &DEN, &GRIND, &SIFT])));
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

fn put(g: &mut Game, def: &'static CardDef, zone: Zone) -> ObjectId {
    let id = g.state.push_obj(Obj::new(g.cards.id_of(def), YOU, zone));
    g.state.zone_push(zone, id);
    id
}

fn cx(game: &mut Game) -> Cx<'_> {
    Cx { g: game, frame: usize::MAX, cursor: 0, mana_source: None, entering: None, lineage: Vec::new(), asked_at: None, doer: None }
}

fn add_mana(g: &mut Game, pool: &[Mana]) {
    let source = g.state.battlefield().first().copied().unwrap_or(ObjectId(0));
    for &m in pool {
        g.state.add_mana(YOU, m, 1, source, None, false);
    }
}

fn next(g: &mut Game) -> Request {
    g.run();
    g.pending().cloned().expect("a question")
}

/// The ways `source` is offered at `YOU`'s priority: an ability's index, or a cast's way number.
fn offered(g: &mut Game, source: ObjectId) -> Vec<u8> {
    priority_options(&cx(g), YOU).iter().filter(|o| o.source == Some(source)).map(|o| o.index).collect()
}

/// Takes the first option `source` has, and gives back the announcement it asks.
fn announce(g: &mut Game, source: ObjectId) -> Request {
    let r = next(g);
    let RequestKind::Priority { options } = &r.kind else { panic!("expected priority, got {:?}", r.kind) };
    let option = options.iter().find(|o| o.source == Some(source) && o.index == 0).expect("it is offered").clone();
    g.answer(r.id, r.player, Answer::Act(Action::of(&option))).expect("a legal answer");
    let r = next(g);
    assert!(matches!(r.kind, RequestKind::PayCosts { announcing: true, .. }), "expected the announcement of X, got {:?}", r.kind);
    r
}

fn kinds(r: &Request) -> Vec<ActionKind> {
    let RequestKind::PayCosts { options, .. } = &r.kind else { panic!("expected a payment, got {:?}", r.kind) };
    options.iter().map(|o| o.kind).collect()
}

#[test]
fn an_ability_whose_x_cant_be_0_is_offered_only_where_x_as_1_could_be_paid() {
    let mut g = game();
    let lair = put(&mut g, &LAIR, Zone::BATTLEFIELD);
    let den = put(&mut g, &DEN, Zone::BATTLEFIELD);
    add_mana(&mut g, &[Mana::G]);
    assert!(offered(&mut g, lair).is_empty(), "{{G}} pays X as 0 only, which the Lair's X can't be");
    assert_eq!(offered(&mut g, den), vec![0], "the Den's X may be 0");
    add_mana(&mut g, &[Mana::C]);
    assert_eq!(offered(&mut g, lair), vec![0], "{{1}}{{G}} pays X as 1");
}

#[test]
fn an_ability_whose_x_cant_be_0_announces_from_1_and_refuses_0() {
    let mut g = game();
    let lair = put(&mut g, &LAIR, Zone::BATTLEFIELD);
    add_mana(&mut g, &[Mana::G, Mana::C, Mana::C]);
    let r = announce(&mut g, lair);
    let RequestKind::PayCosts { x, least, total, .. } = &r.kind else { unreachable!() };
    assert_eq!((*x, *least), (1, 1), "the announcement starts at 1");
    assert_eq!(total.generic, 1, "{{X}}{{G}} shown at X 1");
    assert!(!kinds(&r).contains(&ActionKind::DecreaseX), "X can't be lowered to 0");
    assert!(g.answer(r.id, r.player, Answer::Announce(0)).is_err(), "0 is refused (CR 107.3a)");
    g.answer(r.id, r.player, Answer::Announce(2)).expect("2 may be announced");
    let r = next(&mut g);
    if let RequestKind::PayCosts { options, .. } = &r.kind {
        let confirm = options.iter().find(|o| o.kind == ActionKind::ConfirmCasting).expect("the pool pays it").clone();
        g.answer(r.id, r.player, Answer::Act(Action::of(&confirm))).expect("a legal answer");
    }
    next(&mut g);
    assert_eq!(g.state.stack.last().map(|i| (i.object, i.x)), Some((lair, 2)));
    assert_eq!(g.state.player(YOU).pool.total(), 0, "{{2}}{{G}} paid");
}

#[test]
fn an_ability_whose_x_may_be_0_still_announces_from_0() {
    let mut g = game();
    let den = put(&mut g, &DEN, Zone::BATTLEFIELD);
    add_mana(&mut g, &[Mana::G]);
    let r = announce(&mut g, den);
    let RequestKind::PayCosts { x, least, .. } = &r.kind else { unreachable!() };
    assert_eq!((*x, *least), (0, 0));
    g.answer(r.id, r.player, Answer::Announce(0)).expect("0 may be announced");
}

#[test]
fn a_spell_whose_x_cant_be_0_is_cast_with_x_1_or_more_and_never_for_a_cost_without_x() {
    let mut g = game();
    let grind = put(&mut g, &GRIND, Zone::hand(YOU));
    let sift = put(&mut g, &SIFT, Zone::hand(YOU));
    add_mana(&mut g, &[Mana::U]);
    // {U} pays the mana cost with X as 0, which the Grind's X can't be, and its {U} way has no {X}, so X could only be 0
    // (CR 107.3b): neither is offered. The Sift is offered both ways.
    assert!(offered(&mut g, grind).is_empty());
    assert_eq!(offered(&mut g, sift), vec![0, 1]);
    add_mana(&mut g, &[Mana::C]);
    assert_eq!(offered(&mut g, grind), vec![0], "{{1}}{{U}} pays X as 1; the way with no {{X}} is still not offered");
    let r = announce(&mut g, grind);
    let RequestKind::PayCosts { x, least, total, .. } = &r.kind else { unreachable!() };
    assert_eq!((*x, *least, total.generic), (1, 1, 1));
    assert!(!kinds(&r).contains(&ActionKind::DecreaseX));
    assert!(g.answer(r.id, r.player, Answer::Announce(0)).is_err(), "0 is refused (CR 107.3a)");
    g.answer(r.id, r.player, Answer::Announce(1)).expect("1 may be announced");
}
