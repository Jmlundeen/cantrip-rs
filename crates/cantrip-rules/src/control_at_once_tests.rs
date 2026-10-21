//! Control settled at once (CR 613.1b, 611.2b), not only as the state-based actions are checked: an Aura that gave control
//! leaving, and a "for as long as" condition turning false, give the permanent back before anything reads it next, over
//! test cards.

use crate::{Cx, Game, Setup};
use cantrip_api::*;

fn enchanted(q: &dyn LayerQuery, source: ObjectId, affected: ObjectId, _: &Chars) -> bool {
    q.state().obj(source).attached_to == Some(affected)
}

/// "You control enchanted creature."
static MIND: CardDef = CardDef {
    name: "Control Test Mind",
    types: TypeSet::of(&[CardType::Enchantment]),
    subtypes: SubtypeSet::of(&[Subtype::Aura]),
    statics: &[control_changing(enchanted, "")],
    ..CardDef::BLANK
};

/// "... for as long as that creature's power remains 2 or less."
fn small(q: &dyn Query, e: &ContinuousEffect) -> bool {
    q.chars(e.affected).power.is_some_and(|p| p <= 2)
}

static OLD_MAN: CardDef = CardDef {
    name: "Control Test Old Man",
    types: TypeSet::of(&[CardType::Creature]),
    power: Some(2),
    toughness: Some(3),
    lasting: &[Lasting { text: "", holds: small }],
    ..CardDef::BLANK
};

static BEAR: CardDef = CardDef {
    name: "Control Test Bear",
    types: TypeSet::of(&[CardType::Creature]),
    power: Some(2),
    toughness: Some(2),
    ..CardDef::BLANK
};

const YOU: PlayerId = PlayerId(0);
const THEM: PlayerId = PlayerId(1);

fn cx(game: &mut Game) -> Cx<'_> {
    Cx { g: game, frame: usize::MAX, cursor: 0, mana_source: None, entering: None, lineage: Vec::new(), asked_at: None, doer: None }
}

fn game() -> (Game, &'static CardTable) {
    let table: &'static CardTable = Box::leak(Box::new(CardTable::new([&MIND, &OLD_MAN, &BEAR])));
    (Game::new(table, &Setup { decks: vec![Vec::new(), Vec::new()], ..Default::default() }), table)
}

fn put(game: &mut Game, table: &CardTable, def: &'static CardDef, owner: PlayerId) -> ObjectId {
    let id = game.state.push_obj(Obj::new(table.id_of(def), owner, Zone::BATTLEFIELD));
    let timestamp = game.state.timestamp();
    game.state.update_obj(id, |o| o.timestamp = timestamp);
    game.state.zone_push(Zone::BATTLEFIELD, id);
    game.state.see_statics();
    id
}

/// Your Aura gives you their Bear; as it is put into the graveyard, the Bear is theirs again at once, before the
/// state-based actions are checked.
#[test]
fn an_aura_that_gave_control_leaving_gives_the_creature_back_at_once() {
    let (mut game, table) = game();
    let bear = put(&mut game, table, &BEAR, THEM);
    let mind = put(&mut game, table, &MIND, YOU);
    game.state.update_obj(mind, |o| o.attached_to = Some(bear));
    let mut cx = cx(&mut game);
    crate::ops::settle_control(&mut cx);
    assert_eq!(cx.s().obj(bear).controller, YOU);
    crate::ops::move_to(&mut cx, mind, Zone::graveyard(YOU));
    assert_eq!(cx.s().obj(bear).controller, THEM);
}

/// The Old Man takes their Bear for as long as its power is 2 or less; the moment an effect makes it 3, the Bear is
/// theirs again (CR 611.2b), not when the state-based actions are next checked.
#[test]
fn a_for_as_long_as_condition_turning_false_ends_the_control_at_once() {
    let (mut game, table) = game();
    let old_man = put(&mut game, table, &OLD_MAN, YOU);
    let bear = put(&mut game, table, &BEAR, THEM);
    let mut cx = cx(&mut game);
    let lasting = Duration::AsLongAs(table.id_of(&OLD_MAN), 0);
    cantrip_api::Game::add_effect(&mut cx, old_man, bear, EffectKind::Control(YOU), lasting);
    assert_eq!(cx.s().obj(bear).controller, YOU);
    cantrip_api::Game::add_effect(&mut cx, old_man, bear, EffectKind::Pump(1, 0), Duration::EndOfTurn);
    assert_eq!(cx.s().obj(bear).controller, THEM);
}

/// "... for as long as you control an artifact."
fn you_control_an_artifact(q: &dyn Query, e: &ContinuousEffect) -> bool {
    q.state().battlefield().iter().any(|&id| q.state().obj(id).controller == e.controller && q.chars(id).is(CardType::Artifact))
}

static KEEPER: CardDef = CardDef {
    name: "Control Test Keeper",
    types: TypeSet::of(&[CardType::Enchantment]),
    lasting: &[Lasting { text: "", holds: you_control_an_artifact }],
    ..CardDef::BLANK
};

static RELIC: CardDef = CardDef { name: "Control Test Relic", types: TypeSet::of(&[CardType::Artifact]), ..CardDef::BLANK };

/// You hold their Bear for as long as you control an artifact. An exchange of your Relic for theirs is one instruction
/// (CR 701.12a): there is no moment between its halves in which you control no artifact, so the Bear stays yours.
#[test]
fn an_exchange_of_control_leaves_no_moment_between_its_halves() {
    let table: &'static CardTable = Box::leak(Box::new(CardTable::new([&KEEPER, &RELIC, &BEAR])));
    let mut game = Game::new(table, &Setup { decks: vec![Vec::new(), Vec::new()], ..Default::default() });
    let keeper = put(&mut game, table, &KEEPER, YOU);
    let yours = put(&mut game, table, &RELIC, YOU);
    let theirs = put(&mut game, table, &RELIC, THEM);
    let bear = put(&mut game, table, &BEAR, THEM);
    let mut cx = cx(&mut game);
    cantrip_api::Game::add_effect(&mut cx, keeper, bear, EffectKind::Control(YOU), Duration::AsLongAs(table.id_of(&KEEPER), 0));
    assert_eq!(cx.s().obj(bear).controller, YOU);
    assert!(cantrip_api::Game::exchange_control(&mut cx, keeper, yours, theirs));
    assert_eq!((cx.s().obj(yours).controller, cx.s().obj(theirs).controller), (THEM, YOU));
    assert_eq!(cx.s().obj(bear).controller, YOU);
}

/// An instruction that panics leaves settling as it found it: a host that catches the panic goes on with the game.
#[test]
fn a_panicking_instruction_does_not_leave_settling_held() {
    let (mut game, _) = game();
    let mut cx = cx(&mut game);
    let caught = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        crate::ops::as_one_instruction(&mut cx, |_| panic!("the instruction fails"));
    }));
    assert!(caught.is_err());
    assert_eq!(cx.s().settle_held.get(), 0);
}
