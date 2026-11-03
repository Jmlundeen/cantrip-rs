use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_mechanics::triggers::AN_OPPONENT_DRAWS_A_CARD;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            AN_OPPONENT_DRAWS_A_CARD,
            |g, item| effects::damage_that_player(g, item, 1),
            "Whenever an opponent draws a card, this enchantment deals 1 damage to that player.",
        ),
    ],
    ..oracle::UNDERWORLD_DREAMS
};
