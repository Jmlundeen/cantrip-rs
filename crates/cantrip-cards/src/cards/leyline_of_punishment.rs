use cantrip_api::*;
use cantrip_mechanics::statics;
use cantrip_mechanics::statics::{damage_cant_be_prevented, players_cant_gain_life};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    cant_gain_life: &[players_cant_gain_life],
    unpreventable_damage: &[damage_cant_be_prevented],
    opening_hand: Some(statics::BEGIN_ON_THE_BATTLEFIELD),
    ..oracle::LEYLINE_OF_PUNISHMENT
};
