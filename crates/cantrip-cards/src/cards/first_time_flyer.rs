use cantrip_api::*;
use cantrip_mechanics::statics::{itself_with_a_lesson_in_your_graveyard, plus_pt};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    statics: &[static_ability(
        Layer::PtModify,
        itself_with_a_lesson_in_your_graveyard,
        plus_pt::<1, 1>,
        "This creature gets +1/+1 as long as there's a Lesson card in your graveyard.",
    )],
    ..oracle::FIRST_TIME_FLYER
};
