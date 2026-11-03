use cantrip_api::*;
use cantrip_mechanics::statics;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    statics: &[
        static_ability(
            Layer::PtModify,
            statics::creatures_of_color::<{ Color::White as u8 }>,
            statics::plus_pt::<1, 1>,
            "White creatures get +1/+1.",
        ),
    ],
    ..oracle::CRUSADE
};
