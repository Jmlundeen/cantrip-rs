use cantrip_api::*;
use cantrip_mechanics::attach::{attach_to_target, attached_object, equip};
use cantrip_mechanics::statics::plus_pt;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_mechanics::{craft, effects, targets};
use cantrip_oracle as oracle;

pub static BACK: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, attach_to_target, "When this Equipment enters, attach it to target creature you control.")
            .targets(&[targets::CREATURE_YOU_CONTROL]),
        equip(mana("{2}"), "Equip {2}"),
    ],
    statics: &[static_ability(Layer::PtModify, attached_object, plus_pt::<2, 0>, "Equipped creature gets +2/+0.")],
    ..oracle::SOVEREIGNS_MACUAHUITL
};

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, effects::damage_first_target::<2>, "When this artifact enters, it deals 2 damage to any target.").targets(&[targets::ANY]),
        craft::ability(mana("{2}{R}"), &craft::ARTIFACT, "Craft with artifact {2}{R}"),
    ],
    back: Some(&BACK),
    ..oracle::IDOL_OF_THE_DEEP_KING
};
