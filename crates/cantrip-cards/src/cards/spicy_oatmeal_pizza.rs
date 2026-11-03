use cantrip_api::*;
use cantrip_mechanics::effects::{damage_target_and_you, gain_three_life};
use cantrip_mechanics::targets;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, damage_target_and_you::<4, 3>, "When this artifact enters, it deals 4 damage to any target and 3 damage to you.").targets(&[targets::ANY]),
        ability(cost("{2}, {T}").and(SACRIFICE_THIS), gain_three_life, "{2}, {T}, Sacrifice this artifact: You gain 3 life."),
    ],
    ..oracle::SPICY_OATMEAL_PIZZA
};
