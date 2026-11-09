use cantrip_api::*;
use cantrip_mechanics::{
    lands,
    statics::{itself_while_you_control_a, plus_pt},
};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    statics: &[
        static_ability(
            Layer::PtModify,
            itself_while_you_control_a::<{ lands::sub(Subtype::Forest) }>,
            plus_pt::<1, 2>,
            "This creature gets +1/+2 as long as you control a Forest.",
        ),
    ],
    ..oracle::KIRD_APE
};
