use cantrip_api::*;
use cantrip_mechanics::statics;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    statics: &[
        static_ability(
            Layer::PtModify,
            statics::creatures_of_color::<{ Color::Black as u8 }>,
            statics::plus_pt::<1, 1>,
            "Black creatures get +1/+1.",
        ),
    ],
    ..oracle::BAD_MOON
};
