use cantrip_api::*;
use cantrip_mechanics::{statics, targets};
use cantrip_oracle as oracle;

/// "{2}: Return target land card from your graveyard to your hand." (CR 400.7: it is a new object in hand.)
fn return_land(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(card) = targets::object(g, item, 0) {
        g.return_to_hand(&[card]);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(
        cost("{2}"),
        return_land,
        "{2}: Return target land card from your graveyard to your hand.",
    )
    .targets(&[targets::LAND_CARD_IN_YOUR_GRAVEYARD])],
    statics: &[static_ability(
        Layer::Ability,
        statics::your_lands,
        statics::have_hexproof,
        "Lands you control have hexproof.",
    )],
    ..oracle::MARWYN_THE_PRESERVER
};
