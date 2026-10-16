use cantrip_api::*;
use cantrip_mechanics::storied::itself_with_an_enduring_story;
use cantrip_oracle as oracle;

fn plus_one_power(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.power = chars.power.map(|v| v + 1);
}

fn vigilance(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.keywords = chars.keywords.with(Keyword::Vigilance);
}

pub static CARD: CardDef = CardDef {
    storied: true,
    statics: &[
        static_ability(Layer::PtModify, itself_with_an_enduring_story, plus_one_power, "As long as you have an enduring story, Ori gets +1/+0 and has vigilance."),
        static_ability(Layer::Ability, itself_with_an_enduring_story, vigilance, "As long as you have an enduring story, Ori gets +1/+0 and has vigilance."),
    ],
    ..oracle::ORI_KEEPER_OF_SONGS
};
