use cantrip_api::*;
use cantrip_mechanics::effects::target_player_loses_1_life_you_gain_1;
use cantrip_mechanics::targets;
use cantrip_mechanics::triggers::CREATURE_DIES;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    // "Whenever this creature or another creature dies" (CR 700.4, 603.10a): any creature, Blood Artist itself among them, looked
    // back on, so Blood Artist dying with others triggers for each of them (the ruling). "... target player loses 1 life and
    // you gain 1 life" (CR 119.3): with its target gone the ability doesn't resolve (CR 608.2b) and nothing is gained.
    abilities: &[triggered(CREATURE_DIES, target_player_loses_1_life_you_gain_1, "Whenever this creature or another creature dies, target player loses 1 life and you gain 1 life.")
        .targets(&[targets::PLAYER])
        .look_back()],
    ..oracle::BLOOD_ARTIST
};
