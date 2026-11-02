use cantrip_api::*;
use cantrip_mechanics::{exile_target_until_leaves_effect, targets, triggers};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    enchant: Some(targets::BASIC_LAND_YOU_CONTROL),
    abilities: &[triggered(
        triggers::THIS_ENTERS,
        exile_target_until_leaves_effect,
        "When this Aura enters, exile target creature or planeswalker an opponent controls until this Aura leaves the battlefield.",
    )
    .targets(&[targets::CREATURE_OR_PLANESWALKER_AN_OPPONENT_CONTROLS])],
    ..oracle::OSSIFICATION
};
