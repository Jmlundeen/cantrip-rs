use cantrip_api::*;
use cantrip_mechanics::{effects, enduring, triggers};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(when_asked(EventMask::DAMAGE, triggers::creature_you_control_deals_combat_damage_to_player), |g, item| effects::draw(g, item, 1), "Whenever a creature you control deals combat damage to a player, draw a card."),
        triggered(when(EventMask::ZONE_CHANGE, enduring::dies_as_a_creature), enduring::return_as_enchantment, "When Enduring Curiosity dies, if it was a creature, return it to the battlefield under its owner's control. It's an enchantment. (It's not a creature.)"),
    ],
    ..oracle::ENDURING_CURIOSITY
};
