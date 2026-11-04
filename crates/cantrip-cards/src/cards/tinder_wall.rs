use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(
            SACRIFICE_THIS,
            add_n(Mana::R, 2),
            "Sacrifice this creature: Add {R}{R}.",
        ),
        ability(
                cost("{R}").and(SACRIFICE_THIS),
                effects::damage_first_target::<2>,
                "{R}, Sacrifice this creature: It deals 2 damage to target creature it's blocking.",
            )
            .targets(&[targets::CREATURE_IT_IS_BLOCKING]),
    ],
    ..oracle::TINDER_WALL
};
