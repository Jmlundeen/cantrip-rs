use cantrip_api::*;
use cantrip_mechanics::{library, targets};
use cantrip_oracle as oracle;

/// "When this creature enters, mill four cards. When you do, return target land card from your graveyard to the
/// battlefield tapped." The reflexive trigger exists only if four cards were milled (CR 603.12), and chooses its target
/// after milling.
fn mill_and_reflex(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let this = g.card_id(&CARD);
    library::mill_then_reflexive(g, item, 4, AbilityRef::delayed(this, 0));
    Ok(())
}

/// "Return target land card from your graveyard to the battlefield tapped": it enters under its owner's control, tapped
/// (CR 110.2a, 614.1c). A target gone by resolution is left alone (CR 608.2b).
fn return_land(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(card) = targets::object(g, item, 0) else { return Ok(()) };
    let owner = g.obj(card).owner;
    g.put_onto_battlefield(card, owner, true)?;
    Ok(())
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
