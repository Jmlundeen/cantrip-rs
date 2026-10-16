use cantrip_api::*;
use cantrip_mechanics::statics::{have_haste, plus_pt};
use cantrip_mechanics::storied::itself_with_an_enduring_story;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;


pub static CARD: CardDef = CardDef {
    storied: true,
    statics: &[
        static_ability(Layer::PtModify, itself_with_an_enduring_story, plus_pt::<1, 0>, "As long as you have an enduring story, Óin gets +1/+0 and has haste."),
        static_ability(Layer::Ability, itself_with_an_enduring_story, have_haste, "As long as you have an enduring story, Óin gets +1/+0 and has haste."),
    ],
    abilities: &[ability(cost("{1}, {T}").and(discard(1, targets::filters::any)), |g, item| effects::draw(g, item, 1), "{1}, {T}, Discard a card: Draw a card.")],
    ..oracle::IN_THE_BRAVE
};
