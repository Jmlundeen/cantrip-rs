use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_mechanics::triggers::DEALS_COMBAT_DAMAGE_TO_PLAYER;
use cantrip_oracle as oracle;

/// Deathtouch and lifelink are core keywords read from its characteristics.
pub static CARD: CardDef = CardDef {
    abilities: &[triggered(DEALS_COMBAT_DAMAGE_TO_PLAYER, effects::loot, "Whenever Locke Cole deals combat damage to a player, draw a card, then discard a card.")],
    ..oracle::LOCKE_COLE
};
