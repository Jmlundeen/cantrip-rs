use cantrip_api::*;
use cantrip_mechanics::library::{look_at_top, Rest};
use cantrip_oracle as oracle;

/// Look at the top five cards of your library. Put two of them into your hand and the rest on the bottom of your
/// library in any order.
///
/// A library with fewer cards shows what it has (CR 609.3), and a requirement shrinks with it: the two for the hand
/// are as many as there are to meet them. Cards going to the hand move first; the rest are put on the bottom in the
/// order chosen, the first highest and the last lowest.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    look_at_top(g, item.controller, item.object, 5, (2, 2), false, |_| true, Rest::BottomChosen)?;
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[], effect), ..oracle::STOCK_UP };
