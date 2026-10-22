use cantrip_api::*;
use cantrip_mechanics::{library, select};
use cantrip_oracle as oracle;

/// Search your library for a land card, reveal it, put it into your hand, then shuffle.
fn search(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    library::find_to_hand(g, item.controller, item.object, select::LAND_CARD, true)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(cost("{2}, {T}").and(SACRIFICE_THIS), search, "{2}, {T}, Sacrifice Expedition Map: Search your library for a land card, reveal it, put it into your hand, then shuffle.")],
    ..oracle::EXPEDITION_MAP
};
