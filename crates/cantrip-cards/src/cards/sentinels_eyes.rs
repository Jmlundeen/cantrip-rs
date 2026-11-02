use cantrip_api::*;
use cantrip_mechanics::statics::{have_vigilance, plus_pt};
use cantrip_mechanics::{attach, escape, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    statics: &[
        static_ability(Layer::PtModify, attach::attached_object, plus_pt::<1, 1>, "Enchanted creature gets +1/+1 and has vigilance."),
        static_ability(Layer::Ability, attach::attached_object, have_vigilance, "Enchanted creature gets +1/+1 and has vigilance."),
    ],
    enchant: Some(targets::CREATURE),
    ways: &[escape::way(mana("{W}"), 2, "Escape\u{2014}{W}, Exile two other cards from your graveyard.")],
    ..oracle::SENTINELS_EYES
};
