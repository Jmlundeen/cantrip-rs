use cantrip_api::*;
use cantrip_mechanics::library::{Find, search_to_battlefield};
use cantrip_oracle as oracle;

/// A creature of `green`, the color word the text has now (CR 612).
fn is_green_creature(q: &dyn Query, id: ObjectId, green: Color) -> bool {
    let c = q.chars(id);
    c.is(CardType::Creature) && c.colors.has(green)
}

fn green_creature(q: &dyn Query, asker: &Asker, id: ObjectId) -> bool {
    is_green_creature(q, id, q.words(asker.source).color(Color::Green))
}

/// Search your library for a green creature card, put it onto the battlefield, then shuffle.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let player = item.controller;
    let green = g.words(item.object).color(Color::Green);
    search_to_battlefield(g, player, item.object, Find::ONE, false, |q, c| is_green_creature(q, c, green))?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], effect),
    // As an additional cost to cast this spell, sacrifice a green creature.
    additional_cost: Some(Cost::sacrifice(1, green_creature)),
    ..oracle::NATURAL_ORDER
};
