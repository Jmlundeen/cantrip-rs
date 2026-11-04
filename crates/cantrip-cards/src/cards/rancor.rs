use cantrip_api::*;
use cantrip_mechanics::{attach::attached_object, effects, statics::have_trample, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    enchant: Some(targets::CREATURE),
    abilities: &[triggered(when(EventMask::ZONE_CHANGE, cantrip_mechanics::triggers::this_dies_about_the_card), effects::return_subject_from_graveyard_to_hand, "When Rancor is put into a graveyard from the battlefield, return it to its owner's hand.")],
    statics: &[
        static_ability(Layer::PtModify, attached_object, cantrip_mechanics::statics::plus_pt::<2, 0>, "Enchanted creature gets +2/+0."),
        static_ability(Layer::Ability, attached_object, have_trample, "Enchanted creature has trample."),
    ],
    ..oracle::RANCOR
};
