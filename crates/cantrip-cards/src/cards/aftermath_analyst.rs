use cantrip_api::*;
use cantrip_mechanics::lands;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

/// "When this creature enters, mill three cards" (CR 701.17).
fn mill_three(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.mill(item.controller, 3);
    Ok(())
}

/// "Return all land cards from your graveyard to the battlefield tapped."
fn return_lands(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    lands::return_all_land_cards_from_graveyard_tapped(g, item.controller)
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, mill_three, "When this creature enters, mill three cards."),
        ability(
            cost("{3}{G}").and(SACRIFICE_THIS),
            return_lands,
            "{3}{G}, Sacrifice this creature: Return all land cards from your graveyard to the battlefield tapped.",
        ),
    ],
    ..oracle::AFTERMATH_ANALYST
};
