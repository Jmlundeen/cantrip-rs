use cantrip_api::*;
use cantrip_mechanics::{effects, targets, triggers::ENCHANTED_PERMANENTS_CONTROLLERS_UPKEEP};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    enchant: Some(targets::ARTIFACT),
    abilities: &[triggered(
        ENCHANTED_PERMANENTS_CONTROLLERS_UPKEEP,
        |g, item| effects::damage_that_player(g, item, 1),
        "At the beginning of the upkeep of enchanted artifact's controller, this Aura deals 1 damage to that player.",
    )],
    ..oracle::WARP_ARTIFACT
};
