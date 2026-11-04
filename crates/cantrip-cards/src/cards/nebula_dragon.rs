use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "When this creature enters, it deals 3 damage to any target" (CR 603.6a, 120.3): the target is chosen as the trigger goes on
/// the stack and may be a player or a creature of either side - the Dragon's own controller and creatures included.
pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_ENTERS, effects::damage_first_target::<3>, "When this creature enters, it deals 3 damage to any target.")
        .targets(&[targets::ANY])],
    ..oracle::NEBULA_DRAGON
};
