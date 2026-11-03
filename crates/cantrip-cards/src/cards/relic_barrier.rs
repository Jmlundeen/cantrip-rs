use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[ability(TAP, effects::tap_targets, "{T}: Tap target artifact.").targets(&[targets::ARTIFACT])],
    ..oracle::RELIC_BARRIER
};
