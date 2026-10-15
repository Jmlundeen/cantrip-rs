use cantrip_api::*;
use cantrip_mechanics::library::{Find, search_to_battlefield};
use cantrip_oracle as oracle;

fn is_green_creature(q: &dyn Query, id: ObjectId) -> bool {
    let c = q.chars(id);
    c.is(CardType::Creature) && c.colors.has(Color::Green)
}

fn green_creature(q: &dyn Query, _: &Asker, id: ObjectId) -> bool {
    is_green_creature(q, id)
}

/// Search your library for a green creature card, put it onto the battlefield, then shuffle.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let player = item.controller;
    search_to_battlefield(g, player, item.object, Find::ONE, false, is_green_creature)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef { targets: &[], effect }),
    // As an additional cost to cast this spell, sacrifice a green creature.
    additional_cost: Some(Cost::sacrifice(1, green_creature)),
    ..oracle::NATURAL_ORDER
};
