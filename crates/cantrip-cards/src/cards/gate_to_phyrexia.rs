use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(
                sacrifice(1, targets::filters::creature),
                effects::destroy_first_target,
                "Sacrifice a creature: Destroy target artifact. Activate only during your upkeep and only once each turn.",
            )
            .targets(&[targets::ARTIFACT])
            .once_each_turn()
            .condition(cantrip_mechanics::your_upkeep),
    ],
    ..oracle::GATE_TO_PHYREXIA
};
