use cantrip_api::*;
use cantrip_mechanics::{library, targets};
use cantrip_oracle as oracle;

/// "Mill two cards. You may put a permanent card from among the milled cards into your hand. You gain 2 life."
/// (CR 701.17, 110.4): the life is gained whatever was milled or taken.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    library::mill_then_may_take_into_hand(g, item.controller, item.object, 2, "permanent", targets::is_permanent_card)?;
    g.gain_life(item.controller, 2);
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[], effect), ..oracle::RAPID_RESCUE };
