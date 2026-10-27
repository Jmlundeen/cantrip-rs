use cantrip_api::*;
use cantrip_mechanics::{effects, triggers};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::ANOTHER_CREATURE_DIES,
        effects::plus_one_counter_on_this,
        "Whenever another creature dies, put a +1/+1 counter on this creature. This ability triggers only once each turn.",
    )
    .triggers_each_turn(1)],
    ..oracle::BATTLE_SCARRED_SURVIVALIST
};
