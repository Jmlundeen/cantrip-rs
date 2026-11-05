use cantrip_api::*;
use cantrip_mechanics::lands;
use cantrip_oracle as oracle;

/// "{1}, {T}, Sacrifice this land: Search your library for a basic land card, put it onto the battlefield tapped, then
/// shuffle." (CR 701.23): the search of a Ghost Quarter, with {1} in the cost instead of no cost at all.
fn fetch_basic(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    lands::search_basic_land_to_battlefield(g, item.controller, item.object, true)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        ability(
            cost("{1}").and(TAP).and(SACRIFICE_THIS),
            fetch_basic,
            "{1}, {T}, Sacrifice this land: Search your library for a basic land card, put it onto the battlefield tapped, then shuffle.",
        ),
    ],
    ..oracle::UNEXPLORED_TERRITORY
};
