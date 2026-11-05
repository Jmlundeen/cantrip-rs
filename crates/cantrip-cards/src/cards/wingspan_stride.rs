use cantrip_api::*;
use cantrip_mechanics::statics::{have_flying, plus_pt};
use cantrip_mechanics::{attach, effects, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[ability(cost("{2}{U}"), effects::return_this_to_hand, "{2}{U}: Return this Aura to its owner's hand.")],
    statics: &[
        static_ability(Layer::Ability, attach::attached_object, have_flying, "Enchanted creature gets +1/+1 and has flying."),
        static_ability(Layer::PtModify, attach::attached_object, plus_pt::<1, 1>, "Enchanted creature gets +1/+1 and has flying.").continues(),
    ],
    enchant: Some(targets::CREATURE),
    ..oracle::WINGSPAN_STRIDE
};
