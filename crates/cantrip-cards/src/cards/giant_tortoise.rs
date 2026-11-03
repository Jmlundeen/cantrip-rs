use cantrip_api::*;
use cantrip_oracle as oracle;
use cantrip_mechanics::statics::{itself_while_untapped, plus_pt};

pub static CARD: CardDef = CardDef {
    statics: &[
        static_ability(
            Layer::PtModify,
            itself_while_untapped,
            plus_pt::<0, 3>,
            "This creature gets +0/+3 as long as it's untapped.",
        ),
    ],
    ..oracle::GIANT_TORTOISE
};
