use cantrip_api::*;
use cantrip_mechanics::{effects, targets, triggers};
use cantrip_oracle as oracle;

/// "At the beginning of combat on your turn, target creature you control gets +1/+0 and gains first strike until end of turn"
/// (CR 507.1, 603.2, 611.2a): the target may be the Modifier itself, and is chosen as the trigger goes on the stack.
pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::BEGINNING_OF_YOUR_COMBAT,
        effects::target_creature_gets_and_gains_first_strike::<1, 0>,
        "At the beginning of combat on your turn, target creature you control gets +1/+0 and gains first strike until end of turn.",
    )
    .targets(&[targets::CREATURE_YOU_CONTROL])],
    ..oracle::MOLECULAR_MODIFIER
};
