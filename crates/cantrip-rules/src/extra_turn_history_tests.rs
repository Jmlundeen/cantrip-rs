//! Extra turns are recorded separately from ordinary turn beginnings (CR 500.7), after skip-turn replacement effects.

use crate::{Cx, Flow, Game, Setup};
use cantrip_api::*;

const PLAYER: PlayerId = PlayerId(1);

static CARD: CardDef = CardDef { name: "Extra Turn History Filler", has_cost: false, types: TypeSet::of(&[CardType::Land]), ..CardDef::BLANK };

fn game() -> Game {
    let cards: &'static CardTable = Box::leak(Box::new(CardTable::new([&CARD])));
    let filler = cards.id_of(&CARD);
    Game::new(cards, &Setup { decks: vec![vec![filler; 20], vec![filler; 20]], ..Setup::default() })
}

fn begin_untap(game: &mut Game) {
    let mut cx = Cx { g: game, frame: usize::MAX, cursor: 0, mana_source: None, entering: None, lineage: Vec::new(), asked_at: None, doer: None, producing: None };
    assert!(matches!(crate::turn::begin_step(&mut cx), Ok(Flow::Done)));
}

fn set_turn(game: &mut Game, extra: u32) {
    game.state.update_turn(|turn| {
        turn.number = 2;
        turn.active = PLAYER;
        turn.extra = extra;
        turn.set_step(Step::Untap);
    });
}

#[test]
fn an_extra_turn_records_both_the_turn_beginning_and_its_extra_classification() {
    let mut game = game();
    set_turn(&mut game, 9);
    begin_untap(&mut game);

    let history = game.history();
    assert_eq!(history.count(EventQuery::kind(EventMask::TURN_BEGAN).player(PLAYER)), 1);
    assert_eq!(history.count(EventQuery::kind(EventMask::EXTRA_TURN_BEGAN).player(PLAYER)), 1);
}

#[test]
fn an_extra_turn_skipped_at_untap_is_not_recorded_as_begun() {
    let mut game = game();
    set_turn(&mut game, 9);
    let source = game.state.push_obj(Obj::new(game.cards.id_of(&CARD), PLAYER, Zone::BATTLEFIELD));
    game.state.zone_push(Zone::BATTLEFIELD, source);
    let timestamp = game.state.timestamp();
    game.state.effect_push(ContinuousEffect {
        source,
        controller: PLAYER,
        timestamp,
        affected: source,
        kind: EffectKind::SkipNextTurn(PLAYER),
        duration: Duration::Indefinite,
    });

    begin_untap(&mut game);

    let history = game.history();
    assert_eq!(history.count(EventQuery::kind(EventMask::TURN_BEGAN).player(PLAYER)), 0);
    assert_eq!(history.count(EventQuery::kind(EventMask::EXTRA_TURN_BEGAN).player(PLAYER)), 0);
    assert_eq!(game.state.turn.step(), Step::Cleanup);
}

#[test]
fn an_ordinary_turn_is_not_classified_as_an_extra_turn() {
    let mut game = game();
    set_turn(&mut game, 0);
    begin_untap(&mut game);

    let history = game.history();
    assert_eq!(history.count(EventQuery::kind(EventMask::TURN_BEGAN).player(PLAYER)), 1);
    assert_eq!(history.count(EventQuery::kind(EventMask::EXTRA_TURN_BEGAN).player(PLAYER)), 0);
}
