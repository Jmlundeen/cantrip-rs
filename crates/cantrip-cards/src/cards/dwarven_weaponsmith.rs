use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(
                TAP.and(sacrifice(1, targets::filters::artifact)),
                effects::plus_one_counter_on_target,
                "{T}, Sacrifice an artifact: Put a +1/+1 counter on target creature. Activate only during your upkeep.",
            )
            .targets(&[targets::CREATURE])
            .condition(cantrip_mechanics::your_upkeep),
    ],
    ..oracle::DWARVEN_WEAPONSMITH
};
