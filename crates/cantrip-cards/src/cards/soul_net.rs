use cantrip_api::*;
use cantrip_mechanics::{effects, triggers::CREATURE_DIES};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            CREATURE_DIES,
            |g, item| effects::may_pay_to_gain_life(g, item, cost("{1}")),
            "Whenever a creature dies, you may pay {1}. If you do, you gain 1 life.",
        ),
    ],
    ..oracle::SOUL_NET
};
