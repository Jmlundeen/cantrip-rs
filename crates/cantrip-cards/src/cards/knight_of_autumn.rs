use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

/// Mode 0: put two +1/+1 counters on this creature.
fn counters(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.add_counters(item.object, CounterKind::PlusOne, 2);
    Ok(())
}

/// Mode 1: destroy target artifact or enchantment.
fn destroy(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    cantrip_mechanics::destroy_target(g, item, 0);
    Ok(())
}

static MODAL: ModalDef = modes(&[
    mode(counters, "Put two +1/+1 counters on this creature."),
    mode(destroy, "Destroy target artifact or enchantment.")
        .targets(&[targets::ARTIFACT_OR_ENCHANTMENT]),
    mode(|g, item| cantrip_mechanics::effects::gain_life(g, item, 4), "You gain 4 life."),
]);

pub static CARD: CardDef = CardDef {
    abilities: &[modal_triggered(THIS_ENTERS, &MODAL, "When this creature enters, choose one \u{2014} Put two +1/+1 counters on this creature; or destroy target artifact or enchantment; or you gain 4 life.")],
    ..oracle::KNIGHT_OF_AUTUMN
};
