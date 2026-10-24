//! Cycling is discarding a card to pay a cycling ability's cost (CR 702.29c), typecycling's included (CR 702.29f), and only
//! that: a channel ability's "Discard this card" (Boseiju, Who Endures) is a discard from the hand, not a cycle. The core
//! announces a cycle as `Event::Cycled` in the discard's batch, right after its `Discarded`, naming the card where the
//! discard put it (CR 400.7), so "when you cycle this card" triggers from that zone (CR 702.29c). Test-only cards: one with
//! a cycling ability (`ActivatedDef::cycling`), one with a channel-style ability of the same cost.

use crate::{Game, Setup};
use cantrip_api::*;

const YOU: PlayerId = PlayerId(0);

fn nothing(_: &mut dyn cantrip_api::Game, _: &StackItem) -> Result<(), Suspend> {
    Ok(())
}

static CYCLER: CardDef = CardDef {
    name: "Cycled Test Cycler",
    types: TypeSet::of(&[CardType::Sorcery]),
    abilities: &[ability(DISCARD_THIS, nothing, "Cycling {0} ({0}, Discard this card: Draw a card.)").from(ZoneKind::Hand).cycling()],
    ..CardDef::BLANK
};
static CHANNELER: CardDef = CardDef {
    name: "Cycled Test Channeler",
    types: TypeSet::of(&[CardType::Sorcery]),
    abilities: &[ability(DISCARD_THIS, nothing, "Channel \u{2014} Discard this card: Nothing happens.").from(ZoneKind::Hand)],
    ..CardDef::BLANK
};
static LAND: CardDef = CardDef {
    name: "Cycled Test Land",
    has_cost: false,
    types: TypeSet::of(&[CardType::Land]),
    ..CardDef::BLANK
};

/// A game at your first main phase with `def` in your hand, and that card.
fn game(def: &'static CardDef) -> (Game, ObjectId) {
    let table = Box::leak(Box::new(CardTable::new([&CYCLER, &CHANNELER, &LAND])));
    let filler = table.id_of(&LAND);
    let mut game = Game::new(table, &Setup { decks: vec![vec![filler; 30], vec![filler; 30]], ..Default::default() });
    let card = game.state.push_obj(Obj::new(table.id_of(def), YOU, Zone::hand(YOU)));
    game.state.zone_push(Zone::hand(YOU), card);
    for _ in 0..150 {
        game.run();
        let r = game.pending().cloned().expect("a question");
        if r.player == YOU && game.state.turn.active == YOU && game.state.turn.step() == Step::PrecombatMain && matches!(r.kind, RequestKind::Priority { .. }) {
            return (game, card);
        }
        let answer = match &r.kind {
            RequestKind::Priority { .. } => Answer::Pass,
            RequestKind::DeclareAttackers { .. } => Answer::Attack(Vec::new()),
            RequestKind::DeclareBlockers { .. } => Answer::Block(Vec::new()),
            RequestKind::DiscardToHandSize { count, hand } => Answer::Cards(hand.iter().copied().filter(|&c| c != card).take(*count as usize).collect()),
            other => panic!("unexpected {other:?}"),
        };
        game.answer(r.id, r.player, answer).unwrap();
    }
    panic!("main phase never reached");
}

/// Activates the card's ability from the hand, and returns the events it recorded.
fn activate(game: &mut Game, card: ObjectId) -> Vec<Event> {
    let start = game.log.len();
    let r = game.pending().cloned().expect("a question");
    let RequestKind::Priority { options } = &r.kind else { panic!("unexpected {:?}", r.kind) };
    let option = options.iter().find(|o| o.kind == ActionKind::ActivateAbility && o.source == Some(card)).expect("the ability is offered");
    game.answer(r.id, r.player, Answer::Act(Action::of(option))).unwrap();
    game.run();
    game.log[start..].iter().map(|r| r.event).collect()
}

#[test]
fn a_cycling_cost_cycles_the_card_where_the_discard_put_it() {
    let (mut g, card) = game(&CYCLER);
    let events = activate(&mut g, card);
    let discarded = events.iter().position(|e| matches!(e, Event::Discarded { .. })).expect("the card is discarded");
    let Event::Discarded { card: in_graveyard, .. } = events[discarded] else { unreachable!() };
    assert!(g.state.obj(in_graveyard).zone.is(ZoneKind::Graveyard));
    assert_eq!(g.state.obj(card).became, Some(in_graveyard));
    assert!(
        matches!(events.get(discarded + 1), Some(Event::Cycled { card: c, player: YOU }) if *c == in_graveyard),
        "the cycle is announced right after the discard, naming the card in the graveyard: {events:?}"
    );
    assert_eq!(events.iter().filter(|e| matches!(e, Event::Cycled { .. })).count(), 1);
}

#[test]
fn a_discard_this_card_cost_of_no_cycling_ability_is_no_cycle() {
    let (mut g, card) = game(&CHANNELER);
    let events = activate(&mut g, card);
    assert!(events.iter().any(|e| matches!(e, Event::Discarded { .. })), "the card is discarded: {events:?}");
    assert!(!events.iter().any(|e| matches!(e, Event::Cycled { .. })), "a channel cost is not cycling (CR 702.29c): {events:?}");
}
