use cantrip_api::*;
use cantrip_mechanics::statics;
use cantrip_mechanics::{
    attach::{attached_object, equip},
    statics::{have_double_strike_and_trample, plus_pt},
};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    opening_hand: Some(statics::BEGIN_ON_THE_BATTLEFIELD),
    abilities: &[equip(mana("{3}"), "Equip {3}")],
    statics: &[
        static_ability(Layer::PtModify, attached_object, plus_pt::<1, 1>, "Equipped creature gets +1/+1."),
        static_ability(Layer::Ability, attached_object, have_double_strike_and_trample, "Equipped creature has double strike and trample."),
    ],
    ..oracle::LEYLINE_AXE
};
