//! `cantrip_mechanics::enlist` and `cantrip_mechanics::exert` on a table of test-only cards, played as a game is: no printed
//! card the engine has prints enlist or exert as it attacks yet (Guardian of New Benalia and Glorybringer will, with their
//! scenarios). The core's rules of optional costs to attack (CR 508.1g-j) are `cantrip-rules`' `attack_option_tests`.

use std::sync::OnceLock;

use cantrip_api::Game as Rules;
use cantrip_engine::*;
use cantrip_mechanics::{effects, enlist, exert};

fn gain_one(g: &mut dyn Rules, item: &StackItem) -> Result<(), Suspend> {
    effects::gain_life(g, item, 1)
}

/// A free 2/2 with enlist and "Whenever this creature enlists a creature, you gain 1 life."
static ENLISTER: CardDef = CardDef {
    name: "Probe Enlister",
    types: TypeSet::of(&[CardType::Creature]),
    power: Some(2),
    toughness: Some(2),
    abilities: &[enlist::ENLIST, triggered(enlist::THIS_ENLISTS, gain_one, "Whenever this creature enlists a creature, you gain 1 life.")],
    ..CardDef::BLANK
};

/// A free 3/3.
static BEAR: CardDef = CardDef { name: "Probe Bear", types: TypeSet::of(&[CardType::Creature]), power: Some(3), toughness: Some(3), ..CardDef::BLANK };

/// A free 2/2: "You may exert this creature as it attacks. When you do, you gain 1 life."
static EXERTER: CardDef = CardDef {
    name: "Probe Exerter",
    types: TypeSet::of(&[CardType::Creature]),
    power: Some(2),
    toughness: Some(2),
    abilities: &[exert::as_it_attacks(gain_one, "You may exert this creature as it attacks. When you do, you gain 1 life.")],
    ..CardDef::BLANK
};

static FILLER: CardDef = CardDef { name: "Filler", has_cost: false, types: TypeSet::of(&[CardType::Land]), ..CardDef::BLANK };

fn table() -> &'static CardTable {
    static TABLE: OnceLock<CardTable> = OnceLock::new();
    TABLE.get_or_init(|| CardTable::new([&ENLISTER, &BEAR, &EXERTER, &FILLER]))
}

/// Seat 0 casts the named free creatures on turn 1 and attacks with `attackers` on turn 3, answering an enlist choice with
/// `enlisted` (by name; none for "") and an exert question with `exert`; everything else passes. Plays until turn `to`
/// begins its first main phase.
fn play(hand: &[&str], attackers: &[&str], enlisted: &str, exert: bool, to: u32) -> Game {
    let mut deck: Vec<CardId> = hand.iter().map(|n| table().find(n).unwrap()).collect();
    deck.extend(std::iter::repeat_n(table().find("Filler").unwrap(), 30 - deck.len()));
    let filler = vec![table().find("Filler").unwrap(); 30];
    let mut game = Game::new(table(), &Setup { decks: vec![deck, filler], seed: 1, shuffle: false, opening_chooser: None, ..Default::default() });
    game.verify = true;
    for _ in 0..2000 {
        game.run();
        if game.state.turn.number == to && game.state.turn.step() == Step::PrecombatMain {
            return game;
        }
        let request = game.pending().cloned().expect("the game goes on");
        let answer = match &request.kind {
            RequestKind::Priority { options } => options
                .iter()
                .find(|o| request.player == PlayerId(0) && o.kind == ActionKind::CastSpell)
                .map_or(Answer::Pass, |o| Answer::Act(Action::of(o))),
            RequestKind::DeclareAttackers { eligible, .. } if game.state.turn.number == 3 => Answer::Attack(
                eligible.iter().copied().filter(|&a| attackers.contains(&game.name(a))).map(|a| (a, Target::Player(PlayerId(1)))).collect(),
            ),
            RequestKind::DeclareAttackers { .. } => Answer::Attack(Vec::new()),
            RequestKind::DeclareBlockers { .. } => Answer::Block(Vec::new()),
            RequestKind::ChooseCards { purpose: Purpose::Enlist, cards, .. } => {
                Answer::Cards(cards.iter().copied().filter(|&c| game.name(c) == enlisted).take(1).collect())
            }
            RequestKind::May { purpose: Purpose::Ask(q), .. } if q.verb == "Exert" => {
                if exert {
                    Answer::Yes
                } else {
                    Answer::No
                }
            }
            RequestKind::OrderTriggers { sources, .. } => Answer::Order((0..sources.len() as u32).collect()),
            RequestKind::DiscardToHandSize { count, hand } => Answer::Cards(hand.iter().copied().take(*count as usize).collect()),
            other => panic!("unexpected question {other:?}"),
        };
        game.answer(request.id, request.player, answer).unwrap();
    }
    panic!("turn {to} never came");
}

fn named(game: &Game, name: &str) -> ObjectId {
    game.state.battlefield().iter().copied().find(|&id| game.name(id) == name).unwrap_or_else(|| panic!("no {name}"))
}

/// The enlister taps the bear as it attacks and deals 2 + 3; "whenever this creature enlists a creature" gains 1 life.
#[test]
fn an_enlisting_attacker_deals_the_enlisted_creatures_power_too() {
    let game = play(&["Probe Enlister", "Probe Bear"], &["Probe Enlister"], "Probe Bear", false, 4);
    assert_eq!(game.state.player(PlayerId(1)).life, 15);
    assert_eq!(game.state.player(PlayerId(0)).life, 21);
}

/// Declining to enlist taps nothing and adds nothing.
#[test]
fn an_attacker_that_enlists_nothing_deals_its_own_power() {
    let game = play(&["Probe Enlister", "Probe Bear"], &["Probe Enlister"], "", false, 4);
    assert_eq!(game.state.player(PlayerId(1)).life, 18);
    assert_eq!(game.state.player(PlayerId(0)).life, 20);
    assert!(!game.state.obj(named(&game, "Probe Bear")).tapped);
}

/// Exerted as it attacks, the creature gains its controller 1 life and stays tapped through their next untap step (CR
/// 701.43a); not exerted, it untaps.
#[test]
fn an_exerted_attacker_misses_its_next_untap_step() {
    for exert in [true, false] {
        let game = play(&["Probe Exerter"], &["Probe Exerter"], "", exert, 5);
        assert_eq!(game.state.player(PlayerId(0)).life, if exert { 21 } else { 20 });
        assert_eq!(game.state.obj(named(&game, "Probe Exerter")).tapped, exert);
    }
}
