use cantrip_api::*;
use cantrip_mechanics::{damage_target, targets, destroy_target};
use cantrip_oracle as oracle;

/// Abrade deals 3 damage to target creature.
fn damage_creature(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    damage_target(g, item, 0, 3);
    Ok(())
}

/// Destroy target artifact.
fn destroy_artifact(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    destroy_target(g, item, 0);
    Ok(())
}

static MODAL: ModalDef = ModalDef {
    modes: &[
        ModeDef {
            text: "Abrade deals 3 damage to target creature.",
            targets: &[targets::CREATURE],
            effect: damage_creature,
            cost: ManaCost::ZERO,
        },
        ModeDef {
            text: "Destroy target artifact.",
            targets: &[targets::ARTIFACT],
            effect: destroy_artifact,
            cost: ManaCost::ZERO,
        },
    ],
    min: 1,
    max: 1,
    escalate: None,
    repeat: false, allowed: None,
};

pub static CARD: CardDef = CardDef { spell: Some(SpellDef::MODAL), modal: Some(&MODAL), ..oracle::ABRADE };
