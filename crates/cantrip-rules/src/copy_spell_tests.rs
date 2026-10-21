//! A copy of a spell was paid for by no one (CR 707.10): what the mana spent on the original did to it (CR 106.6: Cavern of
//! Souls' "can't be countered", Arena of Glory's haste) is not the copy's. Over test cards: no printed card in the engine
//! copies a creature or legendary spell, the only spells such mana pays for.

use crate::actions::stack_item;
use crate::{Cx, Game, Setup, ops};
use cantrip_api::*;

const YOU: PlayerId = PlayerId(0);

static BEAST: CardDef = CardDef {
    name: "Copy Test Beast",
    types: TypeSet::of(&[CardType::Creature]),
    power: Some(3),
    toughness: Some(3),
    ..CardDef::BLANK
};

fn cx(game: &mut Game) -> Cx<'_> {
    Cx { g: game, frame: usize::MAX, cursor: 0, mana_source: None, entering: None, lineage: Vec::new(), asked_at: None, doer: None }
}

#[test]
fn a_copy_of_a_spell_made_uncounterable_and_hasty_by_its_mana_is_neither() {
    let table: &'static CardTable = Box::leak(Box::new(CardTable::new([&BEAST])));
    let mut game = Game::new(table, &Setup { decks: vec![Vec::new(), Vec::new()], ..Default::default() });
    let spell = game.state.push_obj(Obj::new(table.id_of(&BEAST), YOU, Zone::STACK));
    game.state.zone_push(Zone::STACK, spell);
    game.state.update_obj(spell, |o| {
        o.marks.cant_be_countered = true;
        o.marks.gains_haste = true;
    });
    game.state.stack_push(stack_item(StackKind::Spell, spell, AbilityRef { card: table.id_of(&BEAST), index: 0 }, YOU, &[]));
    game.state.update_turn(|t| {
        t.active = YOU;
        t.priority = YOU;
        t.set_step(Step::PrecombatMain);
    });
    let copy = ops::copy_spell(&mut cx(&mut game), spell, YOU, false).ok().flatten().expect("the spell is copied");
    assert_ne!(copy, spell);
    let (original, copied) = (game.state.obj(spell).marks, game.state.obj(copy).marks);
    assert!(original.cant_be_countered && original.gains_haste, "the original keeps what its mana gave it");
    assert!(!copied.cant_be_countered && !copied.gains_haste, "the copy keeps what only the original's mana gave it");
    assert_eq!(game.state.stack.len(), 2);
}
