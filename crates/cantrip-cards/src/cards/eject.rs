use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// Return target nonland permanent to its owner's hand. Draw a card.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(permanent) = targets::object(g, item, 0) {
        g.return_to_hand(&[permanent]);
    }
    g.draw(item.controller, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[targets::NONLAND_PERMANENT], effect), ..oracle::EJECT };
