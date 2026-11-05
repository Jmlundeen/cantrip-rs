use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_mechanics::{effects, exile_target, targets};
use cantrip_oracle as oracle;

/// Mode 2: "Surveil 1." (CR 701.25a).
fn surveil_one(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::surveil(g, item, 1)
}

/// Mode 1: "Exile target card from a graveyard." Nothing happens if the target is no longer legal (CR 608.2b).
fn exile_card(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    exile_target(g, item, 0);
    Ok(())
}

/// "When this creature enters, choose one —" (CR 603.6a, 700.2b). Vigilance is a core keyword.
static MODAL: ModalDef = modes(&[
    mode(exile_card, "Exile target card from a graveyard.").targets(&[targets::CARD_IN_A_GRAVEYARD]),
    mode(surveil_one, "Surveil 1."),
]);

pub static CARD: CardDef = CardDef {
    abilities: &[modal_triggered(
        THIS_ENTERS,
        &MODAL,
        "When this creature enters, choose one — exile target card from a graveyard; or surveil 1.",
    )],
    ..oracle::DOMINION_ASSAULT_FORCE
};
