use cantrip_api::*;
use cantrip_mechanics::{attach, statics, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    enchant: Some(targets::CREATURE),
    statics: &[
        static_ability(Layer::Ability, attach::attached_object, statics::have_reach, "Enchanted creature gets +0/+2 and has reach."),
        static_ability(Layer::PtModify, attach::attached_object, statics::plus_pt::<0, 2>, "Enchanted creature gets +0/+2 and has reach.").continues(),
    ],
    ..oracle::WEB
};
