use cantrip_api::*;
use cantrip_mechanics::{attach::{attached_object, equip}, statics::have_haste, triggers::ward};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[equip(mana("{1}"), "Equip {1}")],
    delayed: &[ward::<1>("Ward {1} (Whenever this creature becomes the target of a spell or ability an opponent controls, counter it unless that player pays {1}.)")],
    statics: &[
        static_ability(Layer::PtModify, attached_object, cantrip_mechanics::statics::plus_pt::<1, 0>, "Equipped creature gets +1/+0."),
        static_ability(Layer::Ability, attached_object, have_haste, "Equipped creature has haste."),
        static_ability(Layer::Ability, attached_object, cantrip_mechanics::statics::grant_delayed_ability, "Equipped creature has ward {1}."),
    ],
    ..oracle::LAVASPUR_BOOTS
};
