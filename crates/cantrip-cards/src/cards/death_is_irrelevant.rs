use cantrip_api::*;
use cantrip_mechanics::{assimilate, statics, targets, triggers};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::YOUR_END_STEP,
        assimilate::assimilate_the_first_target,
        "At the beginning of your end step, assimilate target creature card from a graveyard.",
    )
    .targets(&[targets::CREATURE_CARD_IN_A_GRAVEYARD])],
    statics: &[static_ability(
        Layer::Ability,
        statics::your_artifact_creatures,
        statics::have_indestructible,
        "Artifact creatures you control have indestructible.",
    )],
    ..oracle::DEATH_IS_IRRELEVANT
};
