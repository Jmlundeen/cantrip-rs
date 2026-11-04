use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "Return target nonland permanent to its owner's hand. If this spell was kicked, draw a card." (CR 702.33e, 608.2c.) A spell
/// whose only target has become illegal does nothing at all, so there is no draw then (CR 608.2b); otherwise the draw follows
/// whether or not the permanent really moved.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::return_first_target_to_hand(g, item)?;
    if item.kicked > 0 {
        effects::draw(g, item, 1)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    kicker: kicker(mana("{1}{U}")),
    spell: spell(&[targets::NONLAND_PERMANENT], effect),
    ..oracle::WHOOSH
};
