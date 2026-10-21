//! A fragmented loop (CR 732.3) on a table of test-only cards, the rules' own example with tapping for flying: the active
//! player taps an artifact, the other untaps it, and the game is back where it was. No printed card the engine has makes a
//! position come back exactly through two players' free actions.

use std::sync::OnceLock;

use cantrip_api::Game as Rules;
use cantrip_engine::*;

fn artifact(q: &dyn Query, _: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if q.state().battlefield().contains(&id) && q.chars(id).is(CardType::Artifact))
}

fn tap_it(g: &mut dyn Rules, item: &StackItem) -> Result<(), Suspend> {
    if let Target::Object(id) = item.target(0) {
        g.tap(id);
    }
    Ok(())
}

fn untap_it(g: &mut dyn Rules, item: &StackItem) -> Result<(), Suspend> {
    if let Target::Object(id) = item.target(0) {
        g.untap(id);
    }
    Ok(())
}

static TAPPER: CardDef = CardDef {
    name: "Probe Tapper",
    types: TypeSet::of(&[CardType::Enchantment]),
    abilities: &[ability(Cost::NONE, tap_it, "{0}: Tap target artifact.").targets(&[target(artifact)])],
    ..CardDef::BLANK
};

static UNTAPPER: CardDef = CardDef {
    name: "Probe Untapper",
    types: TypeSet::of(&[CardType::Enchantment]),
    abilities: &[ability(Cost::NONE, untap_it, "{0}: Untap target artifact.").targets(&[target(artifact)])],
    ..CardDef::BLANK
};

/// An artifact that does nothing.
static BAUBLE: CardDef = CardDef { name: "Probe Bauble", types: TypeSet::of(&[CardType::Artifact]), ..CardDef::BLANK };

/// A land to fill libraries.
static FILLER: CardDef = CardDef { name: "Filler", has_cost: false, types: TypeSet::of(&[CardType::Land]), ..CardDef::BLANK };

fn table() -> &'static CardTable {
    static TABLE: OnceLock<CardTable> = OnceLock::new();
    TABLE.get_or_init(|| CardTable::new([&TAPPER, &UNTAPPER, &BAUBLE, &FILLER]))
}

/// The Tapper's or Untapper's activation at the Bauble, if `options` has it.
fn activation(game: &Game, options: &[ActionOption], name: &str, bauble: ObjectId) -> Option<Answer> {
    let option = options.iter().find(|o| o.kind == ActionKind::ActivateAbility && o.source.is_some_and(|s| game.name(s) == name))?;
    let mut action = Action::of(option);
    action.targets = vec![Target::Object(bauble)];
    Some(Answer::Act(action))
}

/// CR 732.3: Bot1 taps the Bauble and Bot2 untaps it, round after round, each by their own choice, and the game comes back
/// to the same position: a fragmented loop, which the active player must break by making a different choice. Bot1 is then
/// no longer offered the Tapper's activation there and passes; the game is no draw, and the Bauble stays untapped - Bot2,
/// acting last, has the final say (the rule's example). In the next step, with nothing repeated in it, Bot1 may tap again.
#[test]
fn a_fragmented_loop_makes_the_active_player_choose_otherwise() {
    let filler = table().find("Filler").unwrap();
    let mut game = Game::new(table(), &Setup { decks: vec![vec![filler; 30], vec![filler; 30]], seed: 1, shuffle: false, opening_chooser: None, ..Default::default() });
    let mut bauble = ObjectId::NONE;
    for (name, seat) in [("Probe Tapper", 0), ("Probe Bauble", 0), ("Probe Untapper", 1)] {
        let id = game.state.push_obj(Obj::new(table().find(name).unwrap(), PlayerId(seat), Zone::BATTLEFIELD));
        game.state.zone_push(Zone::BATTLEFIELD, id);
        if name == "Probe Bauble" {
            bauble = id;
        }
    }
    game.verify = true;
    let mut rounds = 0;
    let mut refused = None;
    let mut tapped_again = false;
    for _ in 0..100_000 {
        game.run();
        assert_eq!(game.state.outcome, Outcome::InProgress, "a loop the players keep going by choice is no draw");
        let Some(request) = game.pending().cloned() else { panic!("nothing asked") };
        let step = game.state.turn.step();
        let tapped = game.state.obj(bauble).tapped;
        let answer = match &request.kind {
            RequestKind::Priority { options } if game.state.stack.is_empty() && request.player == PlayerId(0) && !tapped => {
                match (activation(&game, options, "Probe Tapper", bauble), refused) {
                    (Some(tap), None) => {
                        rounds += 1;
                        tap
                    }
                    (Some(tap), Some((_, at))) if step != at => {
                        tapped_again = true;
                        tap
                    }
                    (None, None) => {
                        refused = Some((rounds, step));
                        Answer::Pass
                    }
                    _ => Answer::Pass,
                }
            }
            RequestKind::Priority { options } if game.state.stack.is_empty() && request.player == PlayerId(1) && tapped && refused.is_none() => {
                activation(&game, options, "Probe Untapper", bauble).expect("the Untapper's activation")
            }
            RequestKind::Priority { .. } => Answer::Pass,
            other => panic!("unexpected question {other:?}"),
        };
        game.answer(request.id, request.player, answer).unwrap();
        if tapped_again {
            break;
        }
    }
    let (rounds, _) = refused.expect("the active player was made to choose otherwise");
    assert!(rounds > 10, "only once the step is watched for loops: {rounds} rounds");
    assert!(tapped_again, "a step with no loop in it offers the Tapper again");
}

/// RX-1: a game that reaches the most events the engine records is a draw, between two frames. Here, the same two players
/// tap and untap the Bauble in turn, with a limit small enough to reach.
#[test]
fn a_game_that_reaches_the_event_limit_is_a_draw() {
    let filler = table().find("Filler").unwrap();
    let mut game = Game::new(table(), &Setup { decks: vec![vec![filler; 30], vec![filler; 30]], seed: 1, shuffle: false, opening_chooser: None, ..Default::default() });
    let mut bauble = ObjectId::NONE;
    for (name, seat) in [("Probe Tapper", 0), ("Probe Bauble", 0), ("Probe Untapper", 1)] {
        let id = game.state.push_obj(Obj::new(table().find(name).unwrap(), PlayerId(seat), Zone::BATTLEFIELD));
        game.state.zone_push(Zone::BATTLEFIELD, id);
        if name == "Probe Bauble" {
            bauble = id;
        }
    }
    game.max_events = 200;
    for _ in 0..10_000 {
        game.run();
        if game.is_over() {
            break;
        }
        let Some(request) = game.pending().cloned() else { panic!("nothing asked") };
        let tapped = game.state.obj(bauble).tapped;
        let answer = match &request.kind {
            RequestKind::Priority { options } if game.state.stack.is_empty() => {
                let name = if request.player == PlayerId(0) { "Probe Tapper" } else { "Probe Untapper" };
                match activation(&game, options, name, bauble) {
                    Some(act) if tapped == (request.player == PlayerId(1)) => act,
                    _ => Answer::Pass,
                }
            }
            RequestKind::Priority { .. } => Answer::Pass,
            other => panic!("unexpected question {other:?}"),
        };
        game.answer(request.id, request.player, answer).unwrap();
    }
    assert_eq!(game.state.outcome, Outcome::Draw);
    assert!(game.log.iter().any(|r| r.event == Event::GameDrawn { reason: DrawReason::EventLimit }));
    assert!(game.log.len() >= 200 && game.log.len() < 260, "drawn between the frames as it reached the limit: {}", game.log.len());
}
