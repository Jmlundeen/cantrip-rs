use cantrip_api::*;
use cantrip_mechanics::{effects, targets, triggers};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(triggers::CREATURE_YOU_CONTROL_ENTERS, effects::may_destroy_first_target, "Whenever a creature you control enters, you may destroy target artifact or enchantment.")
        .targets(&[targets::ARTIFACT_OR_ENCHANTMENT])],
    ..oracle::AURA_SHARDS
};
