use cantrip_api::*;
use cantrip_mechanics::{
    lands, regenerate,
    statics::{itself_while_you_control_a, plus_pt},
};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(cost("{B}"), regenerate::this_effect, "{B}: Regenerate this creature."),
    ],
    statics: &[
        static_ability(
            Layer::PtModify,
            itself_while_you_control_a::<{ lands::sub(Subtype::Swamp) }>,
            plus_pt::<1, 1>,
            "This creature gets +1/+1 as long as you control a Swamp.",
        ),
    ],
    ..oracle::SEDGE_TROLL
};
