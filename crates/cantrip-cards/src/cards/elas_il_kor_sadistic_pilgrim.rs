use cantrip_api::*;
use cantrip_mechanics::{effects, triggers};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            when_asked(EventMask::ZONE_CHANGE, triggers::another_creature_you_control_enters),
            |g, item| effects::gain_life(g, item, 1),
            "Whenever another creature you control enters, you gain 1 life.",
        ),
        triggered(
            triggers::ANOTHER_CREATURE_YOU_CONTROL_DIES,
            effects::each_opponent_loses_1_life,
            "Whenever another creature you control dies, each opponent loses 1 life.",
        ),
    ],
    ..oracle::ELAS_IL_KOR_SADISTIC_PILGRIM
};
