use cantrip_api::*;
use cantrip_mechanics::{library, select};
use cantrip_oracle as oracle;

/// "Search your library for a basic land card, reveal it, put it into your hand, then shuffle" (CR 701.23).
fn search_basic_land(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    library::find_to_hand(g, item.controller, item.object, select::BASIC_LAND_CARD, true)?;
    Ok(())
}

/// "Search your library for a land card, reveal it, put it into your hand, then shuffle" (CR 701.23).
fn search_land(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    library::find_to_hand(g, item.controller, item.object, select::LAND_CARD, true)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(cost("{1}, {T}").and(SACRIFICE_THIS), search_basic_land, "{1}, {T}, Sacrifice this artifact: Search your library for a basic land card, reveal it, put it into your hand, then shuffle."),
        ability(cost("{3}, {T}").and(SACRIFICE_THIS), search_land, "{3}, {T}, Sacrifice this artifact: Search your library for a land card, reveal it, put it into your hand, then shuffle."),
    ],
    ..oracle::WORLD_MAP
};
