use cantrip_api::*;
use cantrip_mechanics::{outside, triggers};
use cantrip_oracle as oracle;

/// "You may put a card you own from outside the game into your hand" (CR 400.11b): "you" is the ability's controller.
fn fetch(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    outside::may_put_card_into_hand(g, item.controller, item.object)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::THIS_ENTERS_IF_CAST,
        fetch,
        "When this creature enters, if you cast it, you may put a card you own from outside the game into your hand.",
    )],
    ..oracle::NORTH_WIND_AVATAR
};
