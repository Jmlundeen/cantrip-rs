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

static MODAL: ModalDef = ModalDef {
    modes: &[
        ModeDef { text: "Put two +1/+1 counters on this creature.", targets: &[], effect: counters, cost: ManaCost::ZERO },
        ModeDef {
            text: "Destroy target artifact or enchantment.",
            targets: &[targets::ARTIFACT_OR_ENCHANTMENT],
            effect: destroy,
            cost: ManaCost::ZERO,
        },
        ModeDef { text: "You gain 4 life.", targets: &[], effect: cantrip_mechanics::effects::gain_life::<4>, cost: ManaCost::ZERO },
    ],
    min: 1,
    max: 1,
    escalate: None,
    repeat: false, allowed: None,
};

pub static CARD: CardDef = CardDef {
    abilities: &[modal_triggered(THIS_ENTERS, &MODAL, "When this creature enters, choose one \u{2014} Put two +1/+1 counters on this creature; or destroy target artifact or enchantment; or you gain 4 life.")],
    ..oracle::KNIGHT_OF_AUTUMN
};
