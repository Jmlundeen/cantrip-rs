use cantrip_api::*;
use cantrip_mechanics::replacements::{EXILE_INSTEAD_OF_DYING, exile_if_it_would_die};
use cantrip_mechanics::{damage_target, exile_target, targets};
use cantrip_oracle as oracle;

/// Suplex deals 3 damage to target creature. If that creature would die this turn, exile it instead. The replacement is the
/// spell's own effect on that creature (CR 611.2c, 614.1a), made whether or not the damage is dealt, and before it.
fn damage_creature(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = targets::object(g, item, 0) {
        exile_if_it_would_die(g, item.object, 0, creature);
        damage_target(g, item, 0, 3);
    }
    Ok(())
}

/// Exile target artifact.
fn exile_artifact(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    exile_target(g, item, 0);
    Ok(())
}

static MODAL: ModalDef = modes(&[
    mode(damage_creature, "Suplex deals 3 damage to target creature. If that creature would die this turn, exile it instead.")
        .targets(&[targets::CREATURE]),
    mode(exile_artifact, "Exile target artifact.")
        .targets(&[targets::ARTIFACT]),
]);

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef::MODAL),
    modal: Some(&MODAL),
    granted_replacements: &[EXILE_INSTEAD_OF_DYING],
    ..oracle::SUPLEX
};
