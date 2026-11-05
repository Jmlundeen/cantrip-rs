use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::R), "{T}: Add {R}."),
        ability(
            cost("{2}{R}").and(TAP).and(SACRIFICE_THIS),
            effects::damage_first_target::<6>,
            "{2}{R}, {T}, Sacrifice this creature: It deals 6 damage to target creature with flying.",
        )
        .targets(&[targets::CREATURE_WITH_FLYING]),
    ],
    ..oracle::SUNSET_STRIKEMASTER
};
