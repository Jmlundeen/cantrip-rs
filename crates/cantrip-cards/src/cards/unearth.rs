use cantrip_api::*;
use cantrip_mechanics::{targets, triggers::cycling};
use cantrip_oracle as oracle;

/// Return target creature card with mana value 3 or less from your graveyard to the battlefield.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(card) = targets::object(g, item, 0) {
        g.put_onto_battlefield(card, item.controller, false)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE_CARD_WITH_MANA_VALUE_3_OR_LESS_IN_YOUR_GRAVEYARD], effect),
    abilities: &[cycling(mana("{2}"), "Cycling {2} ({2}, Discard this card: Draw a card.)")],
    ..oracle::UNEARTH
};
