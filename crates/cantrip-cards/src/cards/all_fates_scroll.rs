use cantrip_api::*;
use cantrip_mechanics::lands;
use cantrip_oracle as oracle;

/// "Draw X cards, where X is the number of differently named lands you control." X is found as the ability resolves
/// (CR 608.2h); the Scroll is already sacrificed, and it was never a land, so it adds no name.
fn draw_for_differently_named_lands(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let player = item.controller;
    let count = lands::differently_named_lands(g, player);
    g.draw(player, count as u32);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add_any_color(1), "{T}: Add one mana of any color."),
        ability(
            cost("{7}, {T}").and(SACRIFICE_THIS),
            draw_for_differently_named_lands,
            "{7}, {T}, Sacrifice this artifact: Draw X cards, where X is the number of differently named lands you control.",
        ),
    ],
    ..oracle::ALL_FATES_SCROLL
};
