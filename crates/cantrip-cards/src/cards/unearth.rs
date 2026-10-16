use cantrip_api::*;
use cantrip_mechanics::{targets, triggers::cycling};
use cantrip_oracle as oracle;

/// Target creature card with mana value 3 or less from your graveyard.
fn small_creature_in_your_graveyard(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    targets::card_in_your_graveyard_with_mana_value_at_most(q, asker, t, &[CardType::Creature], 3)
}

/// Return target creature card with mana value 3 or less from your graveyard to the battlefield.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(card) = targets::object(g, item, 0) {
        g.put_onto_battlefield(card, item.controller, false)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[target(small_creature_in_your_graveyard)], effect),
    abilities: &[cycling(mana("{2}"), "Cycling {2} ({2}, Discard this card: Draw a card.)")],
    ..oracle::UNEARTH
};
