use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

/// "When this creature enters, mill three cards" (CR 701.17).
fn mill_three(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.mill(item.controller, 3);
    Ok(())
}

/// "Return all land cards from your graveyard to the battlefield tapped": every land card in the graveyard as the ability
/// resolves, entering together, tapped, under the control of the ability's controller (CR 110.2a). Only
/// the controller's graveyard is looked at.
fn return_lands(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let lands: Vec<(ObjectId, PlayerId, bool)> = g
        .state()
        .zone(Zone::graveyard(you))
        .iter()
        .copied()
        .filter(|&card| g.chars(card).is(CardType::Land))
        .map(|card| (card, you, true))
        .collect();
    g.put_all_onto_battlefield_as(&lands)?;
    Ok(())
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
