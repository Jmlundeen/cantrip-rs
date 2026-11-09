use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(
                cost("{1}").and(TAP),
                effects::tap_targets,
                "{1}, {T}: Tap target artifact, creature, or land.",
            )
            .targets(&[targets::ARTIFACT_CREATURE_OR_LAND]),
    ],
    ..oracle::ICY_MANIPULATOR
};
