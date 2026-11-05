use cantrip_api::*;
use cantrip_mechanics::{effects, targets, triggers::THIS_ENTERS_OR_DIES};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        THIS_ENTERS_OR_DIES,
        effects::plus_one_counter_on_target,
        "When this creature enters or dies, put a +1/+1 counter on target creature you control.",
    )
    .targets(&[targets::CREATURE_YOU_CONTROL])],
    ..oracle::REPUTABLE_MERCHANT
};
