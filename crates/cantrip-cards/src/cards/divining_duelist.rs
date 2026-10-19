use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

/// Mode 0: tap target creature (CR 701.26a).
fn tap(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = targets::object(g, item, 0) {
        g.tap(creature);
    }
    Ok(())
}

/// Mode 1: untap target creature (CR 701.26b).
fn untap(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = targets::object(g, item, 0) {
        g.untap(creature);
    }
    Ok(())
}

static MODAL: ModalDef = modes(&[
    mode(tap, "Tap target creature.").targets(&[targets::CREATURE]),
    mode(untap, "Untap target creature.").targets(&[targets::CREATURE]),
    mode(|g, item| cantrip_mechanics::effects::loot(g, item), "Draw a card, then discard a card."),
]);

pub static CARD: CardDef = CardDef {
    abilities: &[modal_triggered(THIS_ENTERS, &MODAL, "When this creature enters, choose one \u{2014} Tap target creature; or untap target creature; or draw a card, then discard a card.")],
    ..oracle::DIVINING_DUELIST
};
