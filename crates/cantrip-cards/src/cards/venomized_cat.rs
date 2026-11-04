use cantrip_api::*;
use cantrip_mechanics::triggers;
use cantrip_oracle as oracle;

/// "Mill two cards" (CR 701.17a): the top two cards of the controller's library go to their graveyard.
fn mill_two(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.mill(item.controller, 2);
    Ok(())
}

// Deathtouch is the core's, read from the card's JSON.
pub static CARD: CardDef = CardDef {
    abilities: &[triggered(triggers::THIS_ENTERS, mill_two, "When this creature enters, mill two cards.")],
    ..oracle::VENOMIZED_CAT
};
