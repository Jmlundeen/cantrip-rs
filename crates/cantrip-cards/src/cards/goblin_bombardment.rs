use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[ability(sacrifice(1, targets::filters::creature), effects::damage_first_target::<1>, "Sacrifice a creature: Goblin Bombardment deals 1 damage to any target.")
        .targets(&[targets::ANY])],
    ..oracle::GOBLIN_BOMBARDMENT
};
