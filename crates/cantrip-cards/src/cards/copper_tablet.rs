use cantrip_api::*;
use cantrip_mechanics::{effects, triggers::EACH_PLAYERS_UPKEEP};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        EACH_PLAYERS_UPKEEP,
        |g, item| effects::damage_that_player(g, item, 1),
        "At the beginning of each player's upkeep, this artifact deals 1 damage to that player.",
    )],
    ..oracle::COPPER_TABLET
};
