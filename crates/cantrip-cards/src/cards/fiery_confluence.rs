use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// Fiery Confluence deals 1 damage to each creature.
fn creatures(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    // One event (CR 120.2, 603.2c): "whenever a creature is dealt damage" sees them all at once.
    cantrip_mechanics::effects::damage_each_creature(g, item, 1);
    Ok(())
}

/// Fiery Confluence deals 2 damage to each opponent.
fn opponents(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    cantrip_mechanics::effects::damage_each_opponent(g, item, 2);
    Ok(())
}

/// Destroy target artifact.
fn destroy_artifact(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    cantrip_mechanics::destroy_target(g, item, 0);
    Ok(())
}

/// Choose three; you may choose the same mode more than once (CR 700.2d).
static MODAL: ModalDef = modes(&[
    mode(creatures, "Fiery Confluence deals 1 damage to each creature."),
    mode(opponents, "Fiery Confluence deals 2 damage to each opponent."),
    mode(destroy_artifact, "Destroy target artifact.")
        .targets(&[targets::ARTIFACT]),
])
    .choose(3, 3)
    .repeat();

pub static CARD: CardDef = CardDef { spell: Some(SpellDef::MODAL), modal: Some(&MODAL), ..oracle::FIERY_CONFLUENCE };
