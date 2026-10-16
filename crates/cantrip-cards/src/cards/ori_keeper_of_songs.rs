use cantrip_api::*;
use cantrip_mechanics::statics::{have_vigilance, plus_pt};
use cantrip_mechanics::storied::itself_with_an_enduring_story;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    storied: true,
    statics: &[
        static_ability(Layer::PtModify, itself_with_an_enduring_story, plus_pt::<1, 0>, "As long as you have an enduring story, Ori gets +1/+0 and has vigilance."),
        static_ability(Layer::Ability, itself_with_an_enduring_story, have_vigilance, "As long as you have an enduring story, Ori gets +1/+0 and has vigilance."),
    ],
    ..oracle::ORI_KEEPER_OF_SONGS
};
