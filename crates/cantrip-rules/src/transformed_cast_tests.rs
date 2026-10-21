//! A card cast transformed as an effect resolves (CR 608.2g, 712.11a) is judged as its back face alone, a "cast this spell
//! only ..." restriction of that face among it (CR 601.3), over test cards: no printed double-faced card in the engine has
//! a back face with such a restriction.

use crate::actions::{could_cast_transformed_during_resolution, stack_item};
use crate::{Cx, Game, Setup};
use cantrip_api::*;

const YOU: PlayerId = PlayerId(0);

/// "Cast this spell only if you have more than 10 life."
fn more_than_ten_life(q: &dyn Query, asker: &Asker) -> bool {
    q.state().player(asker.controller).life > 10
}

/// The back face: no mana cost of its own (CR 712.11c-d), and a restriction on when it may be cast.
static BACK: CardDef = CardDef {
    name: "Transformed Cast Test Back",
    has_cost: false,
    types: TypeSet::of(&[CardType::Sorcery]),
    cast_only_if: Some(more_than_ten_life),
    ..CardDef::BLANK
};
static FRONT: CardDef = CardDef {
    name: "Transformed Cast Test Front",
    types: TypeSet::of(&[CardType::Creature]),
    power: Some(1),
    toughness: Some(1),
    back: Some(&BACK),
    ..CardDef::BLANK
};
/// What resolves and would cast it.
static CASTER: CardDef = CardDef { name: "Transformed Cast Test Caster", types: TypeSet::of(&[CardType::Sorcery]), ..CardDef::BLANK };

/// Your precombat main phase: the double-faced card in your exile, and a spell of yours resolving on the stack.
fn game() -> (Game, ObjectId) {
    let table: &'static CardTable = Box::leak(Box::new(CardTable::new([&FRONT, &BACK, &CASTER])));
    let mut game = Game::new(table, &Setup { decks: vec![Vec::new(), Vec::new()], ..Default::default() });
    let card = game.state.push_obj(Obj::new(table.id_of(&FRONT), YOU, Zone::exile(YOU)));
    game.state.zone_push(Zone::exile(YOU), card);
    let caster = game.state.push_obj(Obj::new(table.id_of(&CASTER), YOU, Zone::STACK));
    game.state.zone_push(Zone::STACK, caster);
    let item = stack_item(StackKind::Spell, caster, AbilityRef { card: table.id_of(&CASTER), index: 0 }, YOU, &[]);
    game.state.stack_push(item);
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

#[test]
fn a_back_face_cast_transformed_is_held_to_its_own_cast_only_restriction() {
    let (mut game, card) = game();
    assert!(could_cast_transformed_during_resolution(&cx(&mut game), card), "at 20 life the back face may be cast");
    game.state.update_player(YOU, |p| p.life = 5);
    assert!(!could_cast_transformed_during_resolution(&cx(&mut game), card), "at 5 life its restriction forbids the cast");
}

/// "Spells with mana value 3 or greater can't be cast" (as Gaddock Teeg's, for every spell).
fn below_three(q: &dyn Query, _: &Asker, _: PlayerId, spell: ObjectId) -> bool {
    q.mana_value(spell) < 3
}
static WARDEN: CardDef = CardDef { name: "Transformed Cast Test Warden", types: TypeSet::of(&[CardType::Enchantment]), spell_rules: &[below_three], ..CardDef::BLANK };
/// The back face of a front face of mana value 4: a creature with no mana cost of its own.
static HEAVY_BACK: CardDef = CardDef {
    name: "Transformed Cast Test Colossus",
    has_cost: false,
    types: TypeSet::of(&[CardType::Creature]),
    power: Some(6),
    toughness: Some(6),
    ..CardDef::BLANK
};
static HEAVY_FRONT: CardDef = CardDef {
    name: "Transformed Cast Test Squire",
    cost: ManaCost { generic: 4, ..ManaCost::ZERO },
    types: TypeSet::of(&[CardType::Creature]),
    power: Some(1),
    toughness: Some(1),
    back: Some(&HEAVY_BACK),
    ..CardDef::BLANK
};

/// CR 601.3e, 601.6, 202.3b: before the cast the card is judged as its back face alone, mana value 0, which the Warden allows;
/// once its proposal is complete the spell on the stack has its front face's mana value, 4, which the Warden forbids, so the
/// cast is taken back and the card stays in exile.
#[test]
fn a_back_face_cast_transformed_is_taken_back_when_its_spell_has_a_forbidden_mana_value() {
    let table: &'static CardTable = Box::leak(Box::new(CardTable::new([&HEAVY_FRONT, &HEAVY_BACK, &WARDEN, &CASTER])));
    let mut game = Game::new(table, &Setup { decks: vec![Vec::new(), Vec::new()], ..Default::default() });
    let card = game.state.push_obj(Obj::new(table.id_of(&HEAVY_FRONT), YOU, Zone::exile(YOU)));
    game.state.zone_push(Zone::exile(YOU), card);
    let warden = game.state.push_obj(Obj::new(table.id_of(&WARDEN), PlayerId(1), Zone::BATTLEFIELD));
    game.state.zone_push(Zone::BATTLEFIELD, warden);
    let caster = game.state.push_obj(Obj::new(table.id_of(&CASTER), YOU, Zone::STACK));
    game.state.zone_push(Zone::STACK, caster);
    game.state.stack_push(stack_item(StackKind::Spell, caster, AbilityRef { card: table.id_of(&CASTER), index: 0 }, YOU, &[]));
    game.state.update_turn(|t| {
        t.active = YOU;
        t.priority = YOU;
        t.set_step(Step::PrecombatMain);
    });
    assert!(could_cast_transformed_during_resolution(&cx(&mut game), card), "the back face alone is allowed");
    let cast = crate::actions::cast_transformed_during_resolution(&mut cx(&mut game), card).ok();
    assert_eq!(cast, Some(false), "the spell of mana value 4 was cast");
    assert!(game.state.obj(card).zone.is(ZoneKind::Exile), "the card left exile");
    assert_eq!(game.state.stack.len(), 1);
}
