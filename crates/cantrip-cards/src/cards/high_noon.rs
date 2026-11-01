use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "Each player can't cast more than one spell each turn." (CR 101.2.)
pub static CARD: CardDef = CardDef {
    cast_limit: 1,
    abilities: &[ability(
        cost("{4}{R}").and(SACRIFICE_THIS),
        effects::damage_first_target::<5>,
        "{4}{R}, Sacrifice High Noon: It deals 5 damage to any target.",
    )
    .targets(&[targets::ANY])],
    ..oracle::HIGH_NOON
};
