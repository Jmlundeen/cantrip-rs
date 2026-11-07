use cantrip_api::*;
use cantrip_oracle as oracle;

/// "Direct Hit deals 5 damage to target creature." (CR 115.1, 120.3e).
pub static CARD: CardDef = CardDef {
    spell: spell(
        &[cantrip_mechanics::targets::CREATURE],
        cantrip_mechanics::effects::damage_first_target::<5>,
    ),
    ..oracle::DIRECT_HIT
};
