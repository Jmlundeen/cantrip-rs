use cantrip_api::*;
use cantrip_mechanics::statics::{have_haste, plus_pt};
use cantrip_mechanics::{attach, escape, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    statics: &[
        static_ability(Layer::Ability, attach::attached_object, have_haste, "Enchanted creature gets +1/+0 and has haste."),
        static_ability(Layer::PtModify, attach::attached_object, plus_pt::<1, 0>, "Enchanted creature gets +1/+0 and has haste.").continues(),
    ],
    enchant: Some(targets::CREATURE),
    ways: &[escape::way(mana("{1}{R}"), 2, "Escape\u{2014}{1}{R}, Exile two other cards from your graveyard.")],
    ..oracle::ESCAPE_VELOCITY
};
