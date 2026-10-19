use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_mechanics::triggers::{DEALS_COMBAT_DAMAGE_TO_PLAYER, EACH_END_STEP};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(DEALS_COMBAT_DAMAGE_TO_PLAYER, |g, item| effects::draw(g, item, 2), "Whenever this creature deals combat damage to a player, draw two cards."),
        // Every end step, any player's (CR 513.1, 603.2): cast with flash, it survives until the next one (CR 603.2b).
        triggered(EACH_END_STEP, effects::sacrifice_self, "At the beginning of the end step, sacrifice this creature."),
    ],
    ..oracle::FROSTBITE_PYROMENTAL
};
