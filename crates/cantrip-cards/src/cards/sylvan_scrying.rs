use cantrip_api::*;
use cantrip_mechanics::{library, select};
use cantrip_oracle as oracle;

/// "Search your library for a land card, reveal it, put it into your hand, then shuffle." (CR 701.23a, 701.20a, 701.24a).
fn search_for_land(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    library::find_to_hand(g, item.controller, item.object, select::LAND_CARD, true)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], search_for_land),
    ..oracle::SYLVAN_SCRYING
};
