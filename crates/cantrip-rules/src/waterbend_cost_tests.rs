//! Waterbend as a cost that must be paid (CR 701.67a-b), over test cards: "as an additional cost to cast this spell, waterbend
//! {N}" and "waterbend {X}" are mana paid with the rest of the total cost, an untapped artifact or creature tapped for each
//! generic mana of the waterbend cost and for no other mana; a cast is offered only where the mana and the taps could pay it;
//! and "unless you waterbend {N}" / "you may waterbend {N}" paid as a spell resolves taps the same way (CR 118.12).

use crate::{Game, Setup};
use cantrip_api::*;

const YOU: PlayerId = PlayerId(0);

fn gain_three(g: &mut dyn cantrip_api::Game, item: &StackItem) -> Result<(), Suspend> {
    g.gain_life(item.controller, 3);
    Ok(())
}

/// {1} sorcery, "As an additional cost to cast this spell, waterbend {2}. You gain 3 life."
static BENT: CardDef = CardDef {
    name: "Waterbend Test Bent",
    cost: ManaCost { generic: 1, ..ManaCost::ZERO },
    has_cost: true,
    types: TypeSet::of(&[CardType::Sorcery]),
    spell: spell(&[], gain_three),
    additional_cost: Some(Cost::waterbend(2, 0)),
    ..CardDef::BLANK
};

/// {1} sorcery, "As an additional cost to cast this spell, waterbend {X}. You gain 3 life."
static BENT_X: CardDef = CardDef {
    name: "Waterbend Test Bent X",
    cost: ManaCost { generic: 1, ..ManaCost::ZERO },
    has_cost: true,
    types: TypeSet::of(&[CardType::Sorcery]),
    spell: spell(&[], gain_three),
    additional_cost: Some(Cost::waterbend(0, 1)),
    ..CardDef::BLANK
};

const PAY: Question = Question::new("Waterbend", "waterbend {cost}?", "", "Waterbend", "Don't");

fn gain_five_if_bent(g: &mut dyn cantrip_api::Game, item: &StackItem) -> Result<(), Suspend> {
    if g.may_pay(item.controller, item.object, item.object, Cost::waterbend(2, 0), &PAY)? {
        g.gain_life(item.controller, 5);
    }
    Ok(())
}

/// {0} sorcery, "You may waterbend {2}. If you do, you gain 5 life."
static MAY_BEND: CardDef = CardDef {
    name: "Waterbend Test May Bend",
    cost: ManaCost::ZERO,
    has_cost: true,
    types: TypeSet::of(&[CardType::Sorcery]),
    spell: spell(&[], gain_five_if_bent),
    ..CardDef::BLANK
};

static BEAR: CardDef = CardDef {
    name: "Waterbend Test Bear",
    types: TypeSet::of(&[CardType::Creature]),
    power: Some(1),
    toughness: Some(1),
    ..CardDef::BLANK
};
static LAND: CardDef = CardDef { name: "Waterbend Test Land", has_cost: false, types: TypeSet::of(&[CardType::Land]), ..CardDef::BLANK };
/// A land with "{T}: Add {C}".
static WASTES: CardDef = CardDef { name: "Waterbend Test Wastes", abilities: &[mana_ability(TAP, add(Mana::C), "{T}: Add {C}.")], ..LAND };

fn put(game: &mut Game, def: &'static CardDef, zone: Zone) -> ObjectId {
    let id = game.state.push_obj(Obj::new(game.cards.id_of(def), YOU, zone));
    game.state.zone_push(zone, id);
    id
}

/// A game at your first main phase with the three spells in your hand, `bears` Bears and `wastes` Wastes on the battlefield.
fn game(bears: usize, wastes: usize) -> (Game, [ObjectId; 3], Vec<ObjectId>, Vec<ObjectId>) {
    let table = Box::leak(Box::new(CardTable::new([&BENT, &BENT_X, &MAY_BEND, &BEAR, &LAND, &WASTES])));
    let filler = table.id_of(&LAND);
    let mut game = Game::new(table, &Setup { decks: vec![vec![filler; 30], vec![filler; 30]], ..Default::default() });
    let spells = [put(&mut game, &BENT, Zone::hand(YOU)), put(&mut game, &BENT_X, Zone::hand(YOU)), put(&mut game, &MAY_BEND, Zone::hand(YOU))];
    let bears: Vec<ObjectId> = (0..bears).map(|_| put(&mut game, &BEAR, Zone::BATTLEFIELD)).collect();
    let wastes: Vec<ObjectId> = (0..wastes).map(|_| put(&mut game, &WASTES, Zone::BATTLEFIELD)).collect();
    for _ in 0..150 {
        game.run();
        let r = game.pending().cloned().expect("a question");
        if r.player == YOU && game.state.turn.active == YOU && game.state.turn.step() == Step::PrecombatMain && matches!(r.kind, RequestKind::Priority { .. }) {
            return (game, spells, bears, wastes);
        }
        let answer = match &r.kind {
            RequestKind::Priority { .. } => Answer::Pass,
            RequestKind::DeclareAttackers { .. } => Answer::Attack(Vec::new()),
            RequestKind::DeclareBlockers { .. } => Answer::Block(Vec::new()),
            RequestKind::DiscardToHandSize { count, hand } => Answer::Cards(hand.iter().copied().filter(|c| !spells.contains(c)).take(*count as usize).collect()),
            other => panic!("unexpected {other:?}"),
        };
        game.answer(r.id, r.player, answer).unwrap();
    }
    panic!("main phase never reached");
}

fn offered(game: &mut Game, card: ObjectId) -> bool {
    game.run();
    match game.pending().map(|r| &r.kind) {
        Some(RequestKind::Priority { options }) => options.iter().any(|o| o.kind == ActionKind::CastSpell && o.source == Some(card)),
        other => panic!("expected priority, got {other:?}"),
    }
}

/// Answers the pending question with the first option `pick` accepts.
fn act(game: &mut Game, pick: impl Fn(&ActionOption) -> bool) {
    let r = game.pending().cloned().expect("a question");
    let options = match &r.kind {
        RequestKind::Priority { options } | RequestKind::PayCosts { options, .. } | RequestKind::UnlessPays { options, .. } => options.clone(),
        other => panic!("unexpected {other:?}"),
    };
    let option = options.iter().find(|o| pick(o)).unwrap_or_else(|| panic!("not offered among {options:?}")).clone();
    game.answer(r.id, r.player, Answer::Act(Action::of(&option))).unwrap();
    game.run();
}

fn tapped(game: &Game, ids: &[ObjectId]) -> usize {
    ids.iter().filter(|&&id| game.state.obj(id).tapped).count()
}

/// Resolves what is on the stack: every player passes.
fn resolve(game: &mut Game) {
    while !game.state.stack.is_empty() {
        let r = game.pending().cloned().expect("a question");
        assert!(matches!(r.kind, RequestKind::Priority { .. }), "expected priority, got {:?}", r.kind);
        game.answer(r.id, r.player, Answer::Pass).unwrap();
        game.run();
    }
}

/// CR 701.67b: the taps pay only the waterbend cost's generic mana: with three Bears and no mana the {1} of the spell itself is
/// unpaid, so a cast that tapping alone could not pay is not offered; with a mana source for it and one Bear short it is not
/// either, and with two it is.
#[test]
fn a_spell_with_a_waterbend_cost_is_offered_where_the_mana_and_the_taps_pay_it() {
    let (mut g, [bent, ..], _, _) = game(3, 0);
    assert!(!offered(&mut g, bent), "{{1}} has to be paid with mana: the taps pay only the waterbend {{2}}");
    let (mut g, [bent, ..], _, _) = game(1, 1);
    assert!(!offered(&mut g, bent), "one Bear and a Wastes pay {{1}} and one of the waterbend {{2}}");
    let (mut g, [bent, ..], _, _) = game(2, 1);
    assert!(offered(&mut g, bent), "the Wastes pay {{1}}, two Bears the waterbend {{2}}");
}

/// CR 701.67a-b, 601.2h: the Bears are tapped as the cost is paid, one for each generic mana of the waterbend cost, and the
/// spell is cast and resolves.
#[test]
fn the_taps_pay_a_waterbend_cost_as_the_spell_is_cast() {
    let (mut g, [bent, ..], bears, wastes) = game(3, 1);
    let life = g.state.player(YOU).life;
    act(&mut g, |o| o.kind == ActionKind::CastSpell && o.source == Some(bent));
    act(&mut g, |o| o.kind == ActionKind::ActivateAbility && o.source == Some(wastes[0]));
    for &bear in &bears[..2] {
        act(&mut g, |o| o.kind == ActionKind::TapToPay && o.index == PAY_WATERBEND && o.source == Some(bear));
    }
    assert!(
        !matches!(g.pending().map(|r| &r.kind), Some(RequestKind::PayCosts { options, .. }) if options.iter().any(|o| o.source == Some(bears[2]) && o.kind == ActionKind::TapToPay)),
        "a third Bear pays no more of a waterbend {{2}}"
    );
    while let Some(RequestKind::PayCosts { .. }) = g.pending().map(|r| &r.kind) {
        act(&mut g, |o| matches!(o.kind, ActionKind::ConfirmCasting | ActionKind::AssignMana));
    }
    assert_eq!(tapped(&g, &bears), 2);
    assert!(g.state.obj(wastes[0]).tapped);
    resolve(&mut g);
    assert_eq!(g.state.player(YOU).life, life + 3);
}

/// CR 107.3a, 701.67a: "waterbend {X}": X is announced with the cast, and each of the X generic mana is paid by a Bear.
#[test]
fn waterbend_x_is_announced_with_the_cast_and_paid_by_taps() {
    let (mut g, [_, bent_x, _], bears, wastes) = game(3, 1);
    assert!(offered(&mut g, bent_x));
    act(&mut g, |o| o.kind == ActionKind::CastSpell && o.source == Some(bent_x));
    let r = g.pending().cloned().expect("a question");
    let RequestKind::PayCosts { announcing: true, x: 0, .. } = r.kind else { panic!("X is announced first: {:?}", r.kind) };
    g.answer(r.id, r.player, Answer::Announce(3)).unwrap();
    g.run();
    act(&mut g, |o| o.kind == ActionKind::ActivateAbility && o.source == Some(wastes[0]));
    for &bear in &bears {
        act(&mut g, |o| o.kind == ActionKind::TapToPay && o.index == PAY_WATERBEND && o.source == Some(bear));
    }
    while let Some(RequestKind::PayCosts { .. }) = g.pending().map(|r| &r.kind) {
        act(&mut g, |o| matches!(o.kind, ActionKind::ConfirmCasting | ActionKind::AssignMana));
    }
    assert_eq!(tapped(&g, &bears), 3);
    assert_eq!(g.state.stack.last().expect("the spell").x, 3);
}

/// CR 107.3a, 701.67b: an X that the Bears there are cannot pay is announced all the same, as any X is; paying it with the
/// one Bear and the pool leaves {2} of it that nothing may pay, so the payment cannot be finished and is taken back.
#[test]
fn a_waterbend_x_the_taps_cannot_pay_cannot_be_finished() {
    let (mut g, [_, bent_x, _], bears, wastes) = game(1, 1);
    let hand = g.state.zone(Zone::hand(YOU)).len();
    act(&mut g, |o| o.kind == ActionKind::CastSpell && o.source == Some(bent_x));
    let r = g.pending().cloned().expect("a question");
    g.answer(r.id, r.player, Answer::Announce(3)).unwrap();
    g.run();
    act(&mut g, |o| o.kind == ActionKind::ActivateAbility && o.source == Some(wastes[0]));
    act(&mut g, |o| o.kind == ActionKind::TapToPay && o.index == PAY_WATERBEND && o.source == Some(bears[0]));
    act(&mut g, |o| o.kind == ActionKind::AssignMana);
    let r = g.pending().cloned().expect("a question");
    let RequestKind::PayCosts { options, .. } = &r.kind else { panic!("expected a payment, got {:?}", r.kind) };
    assert!(
        options.iter().all(|o| o.kind == ActionKind::CancelCasting),
        "{{2}} of the waterbend {{3}} is left, and no Bear to tap: {options:?}"
    );
    act(&mut g, |o| o.kind == ActionKind::CancelCasting);
    assert!(!g.state.obj(bears[0]).tapped, "taking the cast back untaps what was tapped");
    assert_eq!(g.state.zone(Zone::hand(YOU)).len(), hand);
}

/// CR 118.12, 701.67a: "you may waterbend {2}" as a spell resolves: offered to pay once two Bears could, they are tapped, and
/// the payment is made.
#[test]
fn a_waterbend_cost_paid_as_a_spell_resolves_is_paid_by_taps() {
    let (mut g, [.., may_bend], bears, _) = game(2, 0);
    let life = g.state.player(YOU).life;
    act(&mut g, |o| o.kind == ActionKind::CastSpell && o.source == Some(may_bend));
    resolve_until_asked(&mut g);
    act(&mut g, |o| o.kind == ActionKind::PayCost);
    for &bear in &bears {
        act(&mut g, |o| o.kind == ActionKind::TapToPay && o.index == PAY_WATERBEND && o.source == Some(bear));
    }
    while let Some(RequestKind::PayCosts { .. }) = g.pending().map(|r| &r.kind) {
        act(&mut g, |o| matches!(o.kind, ActionKind::ConfirmCasting | ActionKind::AssignMana));
    }
    assert_eq!(tapped(&g, &bears), 2);
    resolve(&mut g);
    assert_eq!(g.state.player(YOU).life, life + 5);
}

/// CR 118.3, 701.67a: with one Bear a waterbend {2} can't be paid, so paying is not offered: only declining is.
#[test]
fn a_waterbend_cost_the_taps_cannot_pay_as_a_spell_resolves_is_not_offered() {
    let (mut g, [.., may_bend], bears, _) = game(1, 0);
    let life = g.state.player(YOU).life;
    act(&mut g, |o| o.kind == ActionKind::CastSpell && o.source == Some(may_bend));
    resolve_until_asked(&mut g);
    let r = g.pending().cloned().expect("a question");
    let RequestKind::UnlessPays { options, .. } = &r.kind else { panic!("expected the payment question, got {:?}", r.kind) };
    assert!(!options.iter().any(|o| o.kind == ActionKind::PayCost), "{options:?}");
    act(&mut g, |o| o.kind == ActionKind::DeclineCost);
    assert_eq!(tapped(&g, &bears), 0);
    resolve(&mut g);
    assert_eq!(g.state.player(YOU).life, life);
}

/// Passes until the resolving spell asks whether to pay.
fn resolve_until_asked(game: &mut Game) {
    for _ in 0..20 {
        let r = game.pending().cloned().expect("a question");
        if matches!(r.kind, RequestKind::UnlessPays { .. }) {
            return;
        }
        game.answer(r.id, r.player, Answer::Pass).unwrap();
        game.run();
    }
    panic!("never asked to pay");
}
