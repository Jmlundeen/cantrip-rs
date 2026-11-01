use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[ability(cost("{2}").and(sacrifice(1, cantrip_mechanics::targets::filters::creature)), effects::damage_first_target::<2>, "{2}, Sacrifice a creature: This enchantment deals 2 damage to any target.")
        .targets(&[cantrip_mechanics::targets::ANY])],
    ..oracle::WEAPONIZE_THE_MONSTERS
};
