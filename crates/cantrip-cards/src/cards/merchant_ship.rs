use cantrip_api::*;
use cantrip_mechanics::{effects, lands, triggers};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(when_asked(EventMask::UNBLOCKED, triggers::this_attacks_and_isnt_blocked), |g, item| effects::gain_life(g, item, 2),
            "Whenever this creature attacks and isn't blocked, you gain 2 life."),
        state_triggered(lands::you_control_no_islands, effects::sacrifice_self, "When you control no Islands, sacrifice this creature."),
    ],
    attack_rules: &[lands::attack_rule_defender_controls::<{ lands::sub(Subtype::Island) }>],
    ..oracle::MERCHANT_SHIP
};
