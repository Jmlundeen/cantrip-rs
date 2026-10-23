use cantrip_api::*;
use cantrip_mechanics::lands;
use cantrip_oracle as oracle;

fn search(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    lands::search_basic_land_to_battlefield(g, item.controller, item.object, true)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        ability(
            cost("{2}, {T}").and(SACRIFICE_THIS),
            search,
            "{2}, {T}, Sacrifice this land: Search your library for a basic land card, put it onto the battlefield tapped, then shuffle.",
        ),
    ],
    ..oracle::WARPED_LANDSCAPE
};
