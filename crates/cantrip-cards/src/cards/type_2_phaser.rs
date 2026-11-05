use cantrip_api::*;
use cantrip_mechanics::attach::{attached_object, equip};
use cantrip_mechanics::effects;
use cantrip_mechanics::statics::plus_pt;
use cantrip_mechanics::targets;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            THIS_ENTERS,
            effects::tap_and_stun::<1>,
            "When this Equipment enters, tap target creature an opponent controls and put a stun counter on it.",
        )
        .targets(&[targets::CREATURE_AN_OPPONENT_CONTROLS]),
        equip(mana("{3}"), "Equip {3}"),
    ],
    statics: &[static_ability(Layer::PtModify, attached_object, plus_pt::<3, 0>, "Equipped creature gets +3/+0.")],
    ..oracle::TYPE_2_PHASER
};
