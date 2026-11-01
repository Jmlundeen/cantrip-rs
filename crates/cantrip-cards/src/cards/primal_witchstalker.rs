use cantrip_api::*;
use cantrip_mechanics::{lands, library, targets};
use cantrip_oracle as oracle;

/// "When this creature enters, mill four cards. When you do, return target land card from your graveyard to the
/// battlefield tapped." The reflexive trigger exists only if four cards were milled (CR 603.12), and chooses its target
/// after milling.
fn mill_and_reflex(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let this = g.card_id(&CARD);
    library::mill_then_reflexive(g, item, 4, AbilityRef::delayed(this, 0));
    Ok(())
}

fn return_land(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    lands::return_target_land_card_from_graveyard_tapped(g, item, 0)
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        cantrip_mechanics::triggers::THIS_ENTERS,
        mill_and_reflex,
        "When this creature enters, mill four cards. When you do, return target land card from your graveyard to the battlefield tapped.",
    )],
    delayed: &[delayed_trigger(
        return_land,
        "When you do, return target land card from your graveyard to the battlefield tapped.",
    )
    .targets(&[targets::LAND_CARD_IN_YOUR_GRAVEYARD])],
    ..oracle::PRIMAL_WITCHSTALKER
};
