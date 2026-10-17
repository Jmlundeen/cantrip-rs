use cantrip_api::*;
use cantrip_mechanics::library::{look_at_top, Rest};
use cantrip_oracle as oracle;

/// "Look at the top two cards of your library. Put one of them into your hand and the other on the bottom of your
/// library." (CR 401.2, 609.3): exactly one goes to the hand, the other to the bottom; a library of one card puts that
/// card into the hand and one of none does nothing.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    look_at_top(g, item.controller, item.object, 2, (1, 1), false, |_| true, Rest::BottomChosen)?;
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[], effect), ..oracle::SLEIGHT_OF_HAND };
