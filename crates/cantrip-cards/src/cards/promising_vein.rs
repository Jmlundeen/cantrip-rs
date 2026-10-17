use cantrip_api::*;
use cantrip_mechanics::lands::search_basic_land_to_battlefield;
use cantrip_oracle as oracle;

/// "Search your library for a basic land card, put it onto the battlefield tapped, then shuffle" (CR 701.23): the player
/// finds one or none, and the library is shuffled either way.
fn search(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    search_basic_land_to_battlefield(g, item.controller, item.object, true)
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        ability(
            cost("{1}, {T}").and(SACRIFICE_THIS),
            search,
            "{1}, {T}, Sacrifice this land: Search your library for a basic land card, put it onto the battlefield tapped, then shuffle.",
        ),
    ],
    ..oracle::PROMISING_VEIN
};
