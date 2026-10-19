use cantrip_api::*;
use cantrip_mechanics::{destroy_target, targets, triggers};
use cantrip_oracle as oracle;

/// "When this creature enters, destroy up to one target artifact or enchantment." The target is chosen as the ability
/// triggers; an illegal target on resolution is ignored (CR 603.3, 608.2b).
fn destroy(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    destroy_target(g, item, 0);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(triggers::THIS_ENTERS, destroy, "When this creature enters, destroy up to one target artifact or enchantment.")
        .targets(&[up_to_one(targets::ARTIFACT_OR_ENCHANTMENT.filter)])],
    ..oracle::CHOMPING_CHANGELING
};
