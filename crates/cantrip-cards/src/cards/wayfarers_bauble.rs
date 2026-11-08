use cantrip_api::*;
use cantrip_mechanics::lands;
use cantrip_oracle as oracle;

/// "{2}, {T}, Sacrifice this artifact: Search your library for a basic land card, put that card onto the battlefield
/// tapped, then shuffle." (CR 701.23 search, 701.24 shuffle): only the ability's controller's library, and the land
/// arrives tapped. The search and the shuffle are the shared call's; the embedded land comes in tapped.
fn fetch_a_basic_land(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    lands::search_basic_land_to_battlefield(g, item.controller, item.object, true)
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(
        cost("{2}, {T}").and(SACRIFICE_THIS),
        fetch_a_basic_land,
        "{2}, {T}, Sacrifice this artifact: Search your library for a basic land card, put that card onto the battlefield tapped, then shuffle.",
    )],
    ..oracle::WAYFARERS_BAUBLE
};
