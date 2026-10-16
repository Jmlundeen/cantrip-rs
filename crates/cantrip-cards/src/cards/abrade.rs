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

static MODAL: ModalDef = modes(&[
    mode(damage_creature, "Abrade deals 3 damage to target creature.")
        .targets(&[targets::CREATURE]),
    mode(destroy_artifact, "Destroy target artifact.")
        .targets(&[targets::ARTIFACT]),
]);

pub static CARD: CardDef = CardDef { spell: Some(SpellDef::MODAL), modal: Some(&MODAL), ..oracle::ABRADE };
