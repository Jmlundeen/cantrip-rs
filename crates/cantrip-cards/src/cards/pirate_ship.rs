use cantrip_api::*;
use cantrip_mechanics::{effects, lands, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(
                TAP,
                effects::damage_first_target::<1>,
                "{T}: This creature deals 1 damage to any target.",
            )
            .targets(&[targets::ANY]),
        state_triggered(
            lands::you_control_no_islands,
            effects::sacrifice_self,
            "When you control no Islands, sacrifice this creature.",
        ),
    ],
    attack_rules: &[lands::attack_rule_defender_controls::<{ lands::sub(Subtype::Island) }>],
    ..oracle::PIRATE_SHIP
};
