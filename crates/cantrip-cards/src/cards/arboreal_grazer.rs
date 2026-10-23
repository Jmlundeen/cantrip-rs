use cantrip_api::*;
use cantrip_mechanics::{hand, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

/// "When this creature enters, you may put a land card from your hand onto the battlefield tapped." (CR 608.2d, 614.1d): any land
/// card of the hand may be chosen, or none; nothing is asked with no land card. Putting a land onto the battlefield is not
/// playing it, so it uses no land play and works after the turn's land drop (CR 305.2).
fn put_a_land(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    hand::may_put_onto_battlefield_tapped(g, item.controller, item.object, |g, c| g.chars(c).is(CardType::Land))?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        THIS_ENTERS,
        put_a_land,
        "When this creature enters, you may put a land card from your hand onto the battlefield tapped.",
    )],
    ..oracle::ARBOREAL_GRAZER
};
