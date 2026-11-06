use cantrip_api::*;
use cantrip_mechanics::{effects, triggers};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(triggers::THIS_ENTERS, effects::tap_each_creature_opponents_control, "When Ultimecia enters, tap all creatures your opponents control."),
        triggered(when_asked(EventMask::DAMAGE, triggers::creature_you_control_deals_combat_damage_to_player), |g, item| effects::draw(g, item, 1), "Whenever a creature you control deals combat damage to a player, draw a card."),
    ],
    ..oracle::ULTIMECIA_TEMPORAL_THREAT
};
