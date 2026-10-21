//! A modal double-faced card's back face is cast only if its targets could be announced (CR 601.2c, 712.8a): judged as that face,
//! from a hand with priority and as an effect casts it, over test cards - no printed castable back face in the engine has a
//! target (Jennifer Walters', Gwen Stacy's, King T'Challa's and Norman Osborn's are creatures with none).

use crate::actions::{cast_ways_during_resolution, priority_options, stack_item};
use crate::{Cx, Game, Setup};
use cantrip_api::*;

const YOU: PlayerId = PlayerId(0);

fn artifact(q: &dyn Query, _: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if q.is_on_battlefield(id) && q.chars(id).is(CardType::Artifact))
}

fn nothing(_: &mut dyn cantrip_api::Game, _: &StackItem) -> Result<(), Suspend> {
    Ok(())
}

/// The back face, a sorcery of its own cost: "Destroy target artifact" (what it does does not matter here).
static BACK: CardDef = CardDef {
    name: "Back Face Test Shatter",
    types: TypeSet::of(&[CardType::Sorcery]),
    spell: spell(&[target(artifact)], nothing),
    ..CardDef::BLANK
};
/// The front face, a creature: a modal double-faced card whose faces may each be cast (CR 712.8b).
static FRONT: CardDef = CardDef {
    name: "Back Face Test Bear",
    types: TypeSet::of(&[CardType::Creature]),
    power: Some(2),
    toughness: Some(2),
    back: Some(&BACK),
    ..CardDef::BLANK
};
static TRINKET: CardDef = CardDef { name: "Back Face Test Trinket", types: TypeSet::of(&[CardType::Artifact]), ..CardDef::BLANK };
static CASTER: CardDef = CardDef { name: "Back Face Test Caster", types: TypeSet::of(&[CardType::Sorcery]), ..CardDef::BLANK };

/// Your precombat main phase with the card in your hand, and an artifact on the battlefield if `artifact`.
fn game(artifact: bool) -> (Game, ObjectId) {
    let table: &'static CardTable = Box::leak(Box::new(CardTable::new([&FRONT, &BACK, &TRINKET, &CASTER])));
    let mut game = Game::new(table, &Setup { decks: vec![Vec::new(), Vec::new()], ..Default::default() });
    let put = |game: &mut Game, def: &'static CardDef, zone: Zone| {
        let id = game.state.push_obj(Obj::new(table.id_of(def), YOU, zone));
        game.state.zone_push(zone, id);
        id
    };
    let card = put(&mut game, &FRONT, Zone::hand(YOU));
    if artifact {
        put(&mut game, &TRINKET, Zone::BATTLEFIELD);
    }
    game.state.update_turn(|t| {
        t.active = YOU;
        t.priority = YOU;
        t.set_step(Step::PrecombatMain);
    });
    (game, card)
}

fn cx(game: &mut Game) -> Cx<'_> {
    Cx { g: game, frame: usize::MAX, cursor: 0, mana_source: None, entering: None, lineage: Vec::new(), asked_at: None, doer: None }
}

/// The ways the card is offered to be cast with priority: 0 the front face, `WAY_BACK_FACE` the back.
fn offered(artifact: bool) -> Vec<u8> {
    let (mut game, card) = game(artifact);
    let mut ways: Vec<u8> =
        priority_options(&cx(&mut game), YOU).iter().filter(|o| o.kind == ActionKind::CastSpell && o.source == Some(card)).map(|o| o.index).collect();
    ways.sort_unstable();
    ways
}

/// The ways an effect resolving could cast it.
fn during_resolution(artifact: bool) -> Vec<u8> {
    let (mut game, card) = game(artifact);
    let caster = game.state.push_obj(Obj::new(game.cards.find("Back Face Test Caster").unwrap(), YOU, Zone::STACK));
    game.state.zone_push(Zone::STACK, caster);
    let item = stack_item(StackKind::Spell, caster, AbilityRef { card: game.cards.find("Back Face Test Caster").unwrap(), index: 0 }, YOU, &[]);
    game.state.stack_push(item);
    let mut ways: Vec<u8> = cast_ways_during_resolution(&cx(&mut game), card).iter().map(|w| w.way).collect();
    ways.sort_unstable();
    ways
}

#[test]
fn a_back_face_with_no_legal_target_is_not_offered_from_the_hand() {
    assert_eq!(offered(false), [0], "the back face is offered with no artifact to target");
    assert_eq!(offered(true), [0, WAY_BACK_FACE], "the back face is not offered with an artifact to target");
}

#[test]
fn a_back_face_with_no_legal_target_is_not_among_the_ways_an_effect_casts() {
    assert_eq!(during_resolution(false), [0]);
    assert_eq!(during_resolution(true), [0, WAY_BACK_FACE]);
}
