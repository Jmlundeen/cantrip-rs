use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "{4}, {T}, Sacrifice this artifact: It deals 5 damage to target creature."
pub static CARD: CardDef = CardDef {
    abilities: &[ability(
        cost("{4}, {T}").and(SACRIFICE_THIS),
        effects::damage_first_target::<5>,
        "{4}, {T}, Sacrifice this artifact: It deals 5 damage to target creature.",
    )
    .targets(&[targets::CREATURE])],
    ..oracle::DISRUPTOR_PISTOL
};
