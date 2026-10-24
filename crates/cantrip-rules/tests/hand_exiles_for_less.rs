//! "As an additional cost to cast this spell, you may exile any number of [white] cards from your hand. This spell costs {2}
//! less to cast for each card exiled this way" (`Cost::exile_from_hand_for_less`, the Marches). The cards are chosen as the
//! intention to pay is announced (CR 601.2b), before X, and the total cost is locked in knowing how many (CR 601.2f); which
//! cards pay is the payer's choice as the cost is paid (CR 601.2h). These tests are of what no printed card the engine has
//! can reach: a mana ability activated while paying (CR 601.2g) that takes a chosen card out of the hand. The cards here are
//! small probes, not card implementations; an enchantment card stands for "a white card".

use std::sync::OnceLock;

use cantrip_api::*;
use cantrip_rules::{Game as RulesGame, Setup};
use cantrip_state::{Frame, FrameKind};

const ME: PlayerId = PlayerId(0);

/// The quality the probe's cost exiles: an enchantment card, standing for "a white card".
fn enchantment_card(q: &dyn Query, _: &Asker, id: ObjectId) -> bool {
    q.chars(id).is(CardType::Enchantment)
}

fn nothing_happens(_: &mut dyn cantrip_api::Game, _: &StackItem) -> Result<(), Suspend> {
    Ok(())
}

/// {X}{1}, "you may exile any number of enchantment cards from your hand; this spell costs {2} less for each".
static PROBE: CardDef = CardDef {
    name: "Hand Exile Probe",
    oracle: true,
    cost: ManaCost { generic: 1, x: 1, ..ManaCost::ZERO },
    has_cost: true,
    types: TypeSet::of(&[CardType::Instant]),
    additional_cost: Some(Cost { exile_from_hand_for_less: Some(ExileFromHandForLess { filter: enchantment_card, less_each: 2 }), ..Cost::NONE }),
    spell: Some(SpellDef { targets: &[], effect: nothing_happens }),
    ..CardDef::BLANK
};

static FODDER_A: CardDef = CardDef { name: "Hand Exile Fodder A", oracle: true, types: TypeSet::of(&[CardType::Enchantment]), ..CardDef::BLANK };
static FODDER_B: CardDef = CardDef { name: "Hand Exile Fodder B", ..FODDER_A };

/// Adds {C}{C}{C}{C}{C}, then discards the first enchantment card in its controller's hand: a mana ability that changes
/// which cards of the hand could pay while the probe is paid for (CR 601.2g, 605.1a).
fn add_five_and_discard_an_enchantment(g: &mut dyn cantrip_api::Game, asker: &Asker, _: Option<Color>) -> Result<(), Suspend> {
    g.add_mana(asker.controller, Mana::C, 5);
    let first = g.state().zone(Zone::hand(asker.controller)).iter().copied().find(|&c| g.chars(c).is(CardType::Enchantment));
    if let Some(card) = first {
        g.discard_together(&[card]);
    }
    Ok(())
}

static HAND_CHANGER: CardDef = CardDef {
    name: "Hand Exile Changer",
    oracle: true,
    types: TypeSet::of(&[CardType::Artifact]),
    abilities: &[mana_ability(TAP, add_n(Mana::C, 5).by(add_five_and_discard_an_enchantment), "{T}: Add {C}{C}{C}{C}{C}. Discard the first enchantment card in your hand.")],
    ..CardDef::BLANK
};

static FILLER: CardDef = CardDef { name: "Hand Exile Filler", oracle: true, ..CardDef::BLANK };

fn table() -> &'static CardTable {
    static TABLE: OnceLock<CardTable> = OnceLock::new();
    TABLE.get_or_init(|| CardTable::new([&PROBE, &FODDER_A, &FODDER_B, &HAND_CHANGER, &FILLER]))
}

/// A game in the first player's precombat main phase with priority, `hand` as their hand (the rest of it filler), `colorless`
/// mana in their pool, and a Hand Exile Changer on their battlefield. The probe's id, and the fodder cards' in hand order.
fn game(hand: &[&str], colorless: u32) -> (RulesGame, ObjectId, Vec<ObjectId>) {
    let cards = table();
    let filler = cards.find("Hand Exile Filler").unwrap();
    let mut deck: Vec<CardId> = hand.iter().map(|n| cards.find(n).unwrap()).collect();
    deck.resize(7, filler);
    deck.extend([filler; 10]);
    let mut game = RulesGame::new(cards, &Setup { decks: vec![deck, vec![filler; 17]], seed: 1, ..Setup::default() });
    let changer = game.state.push_obj(Obj::new(cards.find("Hand Exile Changer").unwrap(), ME, Zone::BATTLEFIELD));
    game.state.zone_push(Zone::BATTLEFIELD, changer);
    game.state.update_turn(|t| {
        t.number = 1;
        t.active = ME;
        t.priority = ME;
        t.set_step(Step::PrecombatMain);
        t.began = true;
    });
    game.state.update_player(ME, |p| p.pool.0[Mana::C as usize] = colorless);
    game.state.frame_push(Frame::new(FrameKind::Priority));
    let in_hand = game.state.zone(Zone::hand(ME)).to_vec();
    let named = |name: &str| in_hand.iter().copied().filter(|&id| game.cards.get(game.state.obj(id).card).name == name).collect::<Vec<_>>();
    let probe = named("Hand Exile Probe")[0];
    let fodder = [named("Hand Exile Fodder A"), named("Hand Exile Fodder B")].concat();
    game.run();
    (game, probe, fodder)
}

fn request(game: &RulesGame) -> Request {
    game.pending().cloned().expect("a question")
}

fn answer(game: &mut RulesGame, answer: Answer) {
    let request = request(game);
    game.answer(request.id, request.player, answer).unwrap();
    game.run();
}

/// Casts the probe and chooses `chosen` as the cards its cost exiles; X is announced at the value the question starts at.
fn cast_choosing(game: &mut RulesGame, probe: ObjectId, chosen: &[ObjectId]) {
    let RequestKind::Priority { options } = request(game).kind else { panic!("expected priority") };
    let cast = options.iter().find(|o| o.kind == ActionKind::CastSpell && o.source == Some(probe)).expect("the probe is castable").clone();
    answer(game, Answer::Act(Action::of(&cast)));
    match request(game).kind {
        RequestKind::ChooseCards { purpose: Purpose::ExileToPay, .. } => answer(game, Answer::Cards(chosen.to_vec())),
        other => panic!("expected the cards to exile, got {other:?}"),
    }
    match request(game).kind {
        RequestKind::PayCosts { announcing: true, x, .. } => {
            assert_eq!(x, 2 * chosen.len() as u32, "X starts at what the reduction pays for");
            answer(game, Answer::Announce(x));
        }
        other => panic!("expected X announced, got {other:?}"),
    }
}

/// Pays what is owed with the Hand Exile Changer's mana ability (CR 601.2g), then from the pool.
fn pay_with_the_changer(game: &mut RulesGame) {
    let RequestKind::PayCosts { announcing: false, options, .. } = request(game).kind else { panic!("expected the payment") };
    let activate = options.iter().find(|o| o.kind == ActionKind::ActivateAbility).expect("the changer's mana ability").clone();
    answer(game, Answer::Act(Action::of(&activate)));
    pay_from_the_pool(game);
}

fn pay_from_the_pool(game: &mut RulesGame) {
    while let RequestKind::PayCosts { announcing: false, options, .. } = request(game).kind {
        let pay = options
            .iter()
            .find(|o| o.kind == ActionKind::AssignMana)
            .or_else(|| options.iter().find(|o| o.kind == ActionKind::ConfirmCasting))
            .expect("pool mana to pay with")
            .clone();
        answer(game, Answer::Act(Action::of(&pay)));
    }
}

/// Whether a card of that name is in the zone: a card that moves is a new object (CR 400.7), so it is found by name.
fn holds(game: &RulesGame, zone: Zone, name: &str) -> bool {
    game.state.zone(zone).iter().any(|&id| game.cards.get(game.state.obj(id).card).name == name)
}

fn cast_recorded(game: &RulesGame) -> bool {
    game.log.iter().any(|r| matches!(r.event, Event::SpellCast { player: ME, .. }))
}

/// (c) Nothing changed the hand while paying: the cards chosen as the cost was announced are exiled, without a second
/// question (CR 601.2h).
#[test]
fn the_cards_chosen_are_exiled_without_asking_again_when_the_hand_did_not_change() {
    let (mut game, probe, fodder) = game(&["Hand Exile Probe", "Hand Exile Fodder A", "Hand Exile Fodder B"], 1);
    cast_choosing(&mut game, probe, &fodder[..1]);
    // {2}{1} less {2}: {1}, from the pool.
    pay_from_the_pool(&mut game);
    assert!(matches!(request(&game).kind, RequestKind::Priority { .. }), "no second choice of cards: {:?}", request(&game).kind);
    assert!(cast_recorded(&game));
    assert!(holds(&game, Zone::exile(ME), "Hand Exile Fodder A"));
    assert!(holds(&game, Zone::hand(ME), "Hand Exile Fodder B"));
}

/// (a) A mana ability activated while paying discards the card chosen: as many as were announced (one) are chosen again from
/// the cards that could pay now, the other enchantment card, which is exiled (CR 601.2f, 601.2g, 601.2h). The total locked
/// in stays {1}.
#[test]
fn a_chosen_card_a_mana_ability_took_from_the_hand_is_chosen_again_from_the_cards_that_could_pay_now() {
    let (mut game, probe, fodder) = game(&["Hand Exile Probe", "Hand Exile Fodder A", "Hand Exile Fodder B"], 0);
    cast_choosing(&mut game, probe, &fodder[..1]);
    pay_with_the_changer(&mut game);
    match request(&game).kind {
        RequestKind::ChooseCards { purpose: Purpose::ExileToPay, rule, .. } => {
            assert!(rule.listed_card(fodder[1]), "the card that could pay now");
            assert!(!rule.listed_card(fodder[0]), "not the one discarded");
        }
        other => panic!("expected the card chosen again, got {other:?}"),
    }
    answer(&mut game, Answer::Cards(vec![fodder[1]]));
    assert!(cast_recorded(&game));
    assert!(holds(&game, Zone::graveyard(ME), "Hand Exile Fodder A"));
    assert!(holds(&game, Zone::exile(ME), "Hand Exile Fodder B"));
    assert_eq!(game.state.player(ME).pool.0[Mana::C as usize], 4, "five added, {{1}} paid");
}

/// (b) A mana ability activated while paying discards the only card that could pay: the cost announced can't be paid, and the
/// cast is taken back (CR 601.2h, 733.1) - the probe back in hand, the discard undone, nothing cast.
#[test]
fn the_cast_is_taken_back_when_too_few_cards_could_pay_once_the_mana_is_paid() {
    let (mut game, probe, fodder) = game(&["Hand Exile Probe", "Hand Exile Fodder A"], 0);
    cast_choosing(&mut game, probe, &fodder[..1]);
    pay_with_the_changer(&mut game);
    assert!(matches!(request(&game).kind, RequestKind::Priority { .. }), "back to priority: {:?}", request(&game).kind);
    assert!(!cast_recorded(&game));
    assert!(game.state.stack.is_empty());
    assert!(game.state.zone(Zone::hand(ME)).contains(&probe));
    assert!(game.state.zone(Zone::hand(ME)).contains(&fodder[0]));
}
