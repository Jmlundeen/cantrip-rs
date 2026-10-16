use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_mechanics::triggers::OPPONENT_ACTIVATES_ABILITY_OF_ARTIFACT_CREATURE_OR_LAND;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        OPPONENT_ACTIVATES_ABILITY_OF_ARTIFACT_CREATURE_OR_LAND,
        |g, item| effects::damage_that_player(g, item, 2),
        "Whenever an opponent activates an ability of an artifact, creature, or land on the battlefield, if it isn't a mana ability, this creature deals 2 damage to that player.",
    )],
    ..oracle::HARSH_MENTOR
};
