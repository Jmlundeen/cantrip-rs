use cantrip_api::*;
use cantrip_mechanics::{library, select};
use cantrip_oracle as oracle;

/// Search your library for a creature card, reveal it, then shuffle and put that card on top. The card is revealed
/// while it is still in the library; it never changes zones, and the search may find nothing (CR 701.23b).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    library::find_to_top(g, item.controller, item.object, select::CREATURE_CARD, true)?;
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[], effect), ..oracle::WORLDLY_TUTOR };
