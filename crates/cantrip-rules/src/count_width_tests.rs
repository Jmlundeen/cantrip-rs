//! Counts of things that happen, played past the widths they were once kept in (255): more than 255 additional combat
//! phases in a turn (CR 500.8), and more than 255 spells cast by one player in a turn, each recorded as the nth it is
//! (`Event::SpellCast::nth`), and a discard to hand size of more than 255 cards (CR 514.1). Over test cards.

use crate::{Game, Setup};
use cantrip_api::*;

const P0: PlayerId = PlayerId(0);
const N: usize = 300;

fn nothing(_: &mut dyn cantrip_api::Game, _: &StackItem) -> Result<(), Suspend> {
    Ok(())
}

fn combats(g: &mut dyn cantrip_api::Game, _: &StackItem) -> Result<(), Suspend> {
    for _ in 0..N {
        g.add_combat_phase();
    }
    Ok(())
}

static FILLER: CardDef = CardDef { name: "Count Test Filler", has_cost: false, types: TypeSet::of(&[CardType::Land]), ..CardDef::BLANK };
static FREE: CardDef = CardDef { name: "Count Test Free", types: TypeSet::of(&[CardType::Sorcery]), spell: spell(&[], nothing), ..CardDef::BLANK };
static COMBATS: CardDef = CardDef {
    name: "Count Test Combats",
    types: TypeSet::of(&[CardType::Sorcery]),
    spell: spell(&[], combats),
    ..CardDef::BLANK
};

/// A duel on turn 1, P0 holding `hand` beside the opening hand of fillers.
fn game(hand: &'static CardDef, copies: usize) -> Game {
    let table: &'static CardTable = Box::leak(Box::new(CardTable::new([&FILLER, &FREE, &COMBATS])));
    let deck = || vec![table.id_of(&FILLER); 30];
    let mut g = Game::new(table, &Setup { decks: vec![deck(), deck()], ..Default::default() });
    for _ in 0..copies {
        let id = g.state.push_obj(Obj::new(table.id_of(hand), P0, Zone::hand(P0)));
        g.state.zone_push(Zone::hand(P0), id);
    }
    g
}

/// Plays turn 1: P0 casts every `def` it is offered in its precombat main phase while the stack is empty, and everything else
/// is passed or declared empty. Stops as turn 2 begins.
fn play_turn_one(g: &mut Game, def: &'static CardDef) {
    for _ in 0..100_000 {
        g.run();
        let request = g.pending().cloned().expect("a question");
        if g.state.turn.number > 1 {
            return;
        }
        let answer = match &request.kind {
            RequestKind::Priority { options } if request.player == P0 && g.state.turn.step() == Step::PrecombatMain && g.state.stack.is_empty() => options
                .iter()
                .find(|o| o.kind == ActionKind::CastSpell && o.source.is_some_and(|s| g.state.obj(s).card == g.cards.id_of(def)))
                .map_or(Answer::Pass, |o| Answer::Act(Action::of(o))),
            RequestKind::Priority { .. } => Answer::Pass,
            RequestKind::DeclareAttackers { .. } => Answer::Attack(Vec::new()),
            RequestKind::DeclareBlockers { .. } => Answer::Block(Vec::new()),
            RequestKind::DiscardToHandSize { count, hand } => Answer::Cards(hand.iter().copied().take(*count as usize).collect()),
            other => panic!("unexpected question {other:?}"),
        };
        g.answer(request.id, request.player, answer).unwrap();
    }
    panic!("turn 1 never ended");
}

/// 300 additional combat phases (CR 500.8): the turn has 301 combat phases, each beginning with its beginning of combat step.
#[test]
fn more_than_255_additional_combat_phases_are_all_taken() {
    let mut g = game(&COMBATS, 1);
    play_turn_one(&mut g, &COMBATS);
    let began = g.log.iter().filter(|r| r.turn == 1 && r.event == Event::StepBegan { step: Step::BeginCombat }).count();
    assert_eq!(began, N + 1);
}

/// 300 spells cast by one player in a turn: the last is recorded as the 300th.
#[test]
fn the_300th_spell_cast_in_a_turn_is_recorded_as_the_300th() {
    let mut g = game(&FREE, N);
    play_turn_one(&mut g, &FREE);
    let nth: Vec<u32> = g.log.iter().filter_map(|r| match r.event { Event::SpellCast { player: P0, nth, .. } => Some(nth), _ => None }).collect();
    assert_eq!(nth.len(), N);
    assert_eq!(nth, (1..=N as u32).collect::<Vec<_>>());
}

/// A hand of 270 at the cleanup step (CR 514.1): 263 cards are discarded, and seven are left.
#[test]
fn a_discard_to_hand_size_of_263_cards_is_asked_and_made() {
    let mut g = game(&FILLER, 263);
    let mut asked = None;
    for _ in 0..1000 {
        g.run();
        let request = g.pending().cloned().expect("a question");
        if g.state.turn.number > 1 {
            break;
        }
        let answer = match &request.kind {
            RequestKind::DiscardToHandSize { count, hand } => {
                asked = Some((*count, hand.len()));
                Answer::Cards(hand.iter().copied().take(*count as usize).collect())
            }
            RequestKind::Priority { .. } => Answer::Pass,
            RequestKind::DeclareAttackers { .. } => Answer::Attack(Vec::new()),
            RequestKind::DeclareBlockers { .. } => Answer::Block(Vec::new()),
            other => panic!("unexpected question {other:?}"),
        };
        g.answer(request.id, request.player, answer).unwrap();
    }
    assert_eq!(asked, Some((263, 270)));
    assert_eq!(g.state.zone(Zone::hand(P0)).len(), 7);
}

/// An answer is judged at its whole length: 300 triggers are put in order by an order of 300 (CR 603.3b), and 257 colors
/// are no answer to a choice of one, as an order of 256 is none of 300.
#[test]
fn an_answer_past_255_is_judged_at_its_whole_length() {
    let g = game(&FILLER, 0);
    let ask = |kind: RequestKind| Request { id: 1, player: P0, kind, may_cancel: false };
    let sources: Vec<ObjectId> = (0..300).map(ObjectId).collect();
    let abilities = vec![AbilityRef { card: CardId(0), index: 0 }; 300];
    let order = ask(RequestKind::OrderTriggers { sources, abilities });
    assert!(crate::check_answer(&g, &order, &Answer::Order((0..300).rev().collect())).is_ok());
    assert!(crate::check_answer(&g, &order, &Answer::Order((0..256).collect())).is_err());
    let colors = ask(RequestKind::ChooseColors { source: ObjectId(0), choices: ColorSet::NONE.with(Color::Red), count: 1, purpose: ColorUse::AddMana });
    assert!(crate::check_answer(&g, &colors, &Answer::Colors(vec![Color::Red])).is_ok());
    assert!(crate::check_answer(&g, &colors, &Answer::Colors(vec![Color::Red; 257])).is_err());
}
