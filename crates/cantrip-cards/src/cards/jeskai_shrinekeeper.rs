use cantrip_api::*;
use cantrip_mechanics::{effects, triggers::DEALS_COMBAT_DAMAGE_TO_PLAYER_SUBJECT};
use cantrip_oracle as oracle;

/// Flying and haste are core keywords read from its characteristics.
pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        DEALS_COMBAT_DAMAGE_TO_PLAYER_SUBJECT,
        effects::gain_life_then_draw::<1, 1>,
        "Whenever this creature deals combat damage to a player, you gain 1 life and draw a card.",
    )],
    ..oracle::JESKAI_SHRINEKEEPER
};
