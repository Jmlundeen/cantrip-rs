//! A choice asked of a player who has left the game (CR 800.4f-h), over a test card: a cost is not paid (800.4f); a choice
//! an object asks is made by another player its controller chooses, another opponent where an opponent was to choose
//! (800.4g); a choice a rule asks is made by the next player in turn order (800.4h). Both for a run that replays its answers
//! (`Cx::ask_core`) and for one written in stages (`ask_then`).

use crate::{Cx, Game, Setup};
use cantrip_api::*;
use cantrip_state::{Frame, FrameKind};

const A: PlayerId = PlayerId(0);
const B: PlayerId = PlayerId(1);
const C: PlayerId = PlayerId(2);
const D: PlayerId = PlayerId(3);

static TRINKET: CardDef = CardDef { name: "Left Choice Test Trinket", types: TypeSet::of(&[CardType::Artifact]), ..CardDef::BLANK };

/// A game of `players` in which A controls a Trinket, B has left, and a frame is running: the game, the Trinket and the frame.
fn game(players: usize) -> (Game, ObjectId, usize) {
    let table: &'static CardTable = Box::leak(Box::new(CardTable::new([&TRINKET])));
    let mut g = Game::new(table, &Setup { decks: vec![Vec::new(); players], ..Default::default() });
    let trinket = g.state.push_obj(Obj::new(table.id_of(&TRINKET), A, Zone::BATTLEFIELD));
    g.state.zone_push(Zone::BATTLEFIELD, trinket);
    g.state.update_player(B, |p| p.lost = true);
    g.state.frame_push(Frame::new(FrameKind::Settle));
    let at = g.state.agenda.len() - 1;
    (g, trinket, at)
}

fn cx(game: &mut Game, frame: usize) -> Cx<'_> {
    Cx { g: game, frame, cursor: 0, mana_source: None, entering: None, lineage: Vec::new(), asked_at: None, doer: None }
}

/// "Choose a card": what the Trinket asks.
fn choose(trinket: ObjectId) -> RequestKind {
    RequestKind::ChooseCards { source: trinket, purpose: Purpose::Vote, cards: vec![trinket], eligible: vec![trinket], count: 1, up_to: false, any_number: false, kind: "", total: None }
}

fn pending(g: &Game, at: usize) -> (PlayerId, RequestKind) {
    let r = g.state.agenda[at].pending.clone().expect("a question");
    (r.player, r.kind)
}

/// CR 800.4f: a cost the player who left would pay, or choose whether to pay, is not paid.
#[test]
fn a_cost_asked_of_a_player_who_left_is_not_paid() {
    let (mut g, trinket, at) = game(3);
    let craft = RequestKind::Craft { source: trinket, cards: Vec::new(), minimum: 0, maximum: 0, quality: "" };
    assert_eq!(cx(&mut g, at).ask_core(B, craft, false).ok(), Some(Answer::Pass));
    assert!(g.state.agenda[at].pending.is_none(), "someone was asked to pay it");
}

/// CR 800.4g, three players: the Trinket's controller A chooses another opponent to make B's choice, and C is the only
/// one, so C is asked.
#[test]
fn an_objects_choice_of_a_player_who_left_is_made_by_the_only_other_opponent() {
    let (mut g, trinket, at) = game(3);
    assert!(cx(&mut g, at).ask_core(B, choose(trinket), false).is_err());
    assert_eq!(pending(&g, at), (C, choose(trinket)));
}

/// CR 800.4g, four players: A chooses which of the other opponents, C or D, makes B's choice; then D is asked.
#[test]
fn an_objects_choice_of_a_player_who_left_is_made_by_the_opponent_its_controller_chooses() {
    let (mut g, trinket, at) = game(4);
    assert!(cx(&mut g, at).ask_core(B, choose(trinket), false).is_err());
    let (asked, kind) = pending(&g, at);
    assert_eq!(asked, A, "the controller does not choose");
    assert!(matches!(kind, RequestKind::ChoosePlayer { ref candidates, .. } if *candidates == vec![C, D]), "{kind:?}");
    // The run is replayed with A's answer.
    g.state.update_frame(at, |f| {
        f.pending = None;
        f.answers = vec![(Some(A), Answer::Player(D))];
    });
    assert!(cx(&mut g, at).ask_core(B, choose(trinket), false).is_err());
    assert_eq!(pending(&g, at), (D, choose(trinket)));
}

/// CR 800.4h: a choice a rule asks of the player who left (the order of their triggers) is made by the next player in turn
/// order.
#[test]
fn a_rules_choice_of_a_player_who_left_is_made_by_the_next_player_in_turn_order() {
    let (mut g, trinket, at) = game(3);
    let order = RequestKind::OrderTriggers { sources: vec![trinket, trinket], abilities: Vec::new() };
    assert!(cx(&mut g, at).ask_core(B, order.clone(), false).is_err());
    assert_eq!(pending(&g, at), (C, order));
}

/// A run written in stages asks the player who left (Duress's `ask_then`): with four players the controller chooses who makes
/// it, and the question then goes to them, continuing at the stage it would have; a cost is not paid, and the run goes on.
#[test]
fn a_staged_question_of_a_player_who_left_follows_the_same_rules() {
    let (mut g, trinket, at) = game(4);
    let _ = cantrip_api::Game::ask_then(&mut cx(&mut g, at), 7, B, choose(trinket));
    let request = g.state.agenda[at].pending.clone().expect("a question");
    assert_eq!(request.player, A, "the controller does not choose");
    g.answer(request.id, A, Answer::Player(C)).unwrap();
    assert_eq!(pending(&g, at), (C, choose(trinket)));
    assert_eq!(g.state.agenda[at].next_stage, 7, "the question does not continue where it would have");

    let (mut g, trinket, at) = game(3);
    let craft = RequestKind::Craft { source: trinket, cards: Vec::new(), minimum: 0, maximum: 0, quality: "" };
    let _ = cantrip_api::Game::ask_then(&mut cx(&mut g, at), 5, B, craft);
    let f = &g.state.agenda[at];
    assert!(f.pending.is_none() && f.stage == 5 && f.answers == vec![(None, Answer::Pass)], "the cost was not left unpaid");
}
