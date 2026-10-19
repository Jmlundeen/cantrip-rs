use cantrip_api::*;
use cantrip_mechanics::{damage_target, targets};
use cantrip_oracle as oracle;

/// "{T}, Sacrifice this creature: It deals 1 damage to any target." The sacrifice is a cost (CR 602.2b), so by the time the
/// ability resolves the Firebrand is gone and deals the damage as it last existed (CR 113.7a, 608.2h): it is still the source
/// of the damage, with its characteristics. A target that is no longer legal gets nothing (CR 608.2b).
fn ping(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    damage_target(g, item, 0, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(TAP.and(SACRIFICE_THIS), ping, "{T}, Sacrifice this creature: It deals 1 damage to any target.").targets(&[targets::ANY])],
    ..oracle::FANATICAL_FIREBRAND
};
