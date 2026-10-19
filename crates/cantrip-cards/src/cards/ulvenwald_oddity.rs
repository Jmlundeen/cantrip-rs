use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

fn trample_and_haste(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.keywords = chars.keywords.with(Keyword::Trample).with(Keyword::Haste);
}

pub static BACK: CardDef = CardDef {
    statics: &[
        static_ability(Layer::PtModify, cantrip_mechanics::statics::other_creatures_you_control, cantrip_mechanics::statics::plus_pt::<1, 1>, "Other creatures you control get +1/+1."),
        static_ability(Layer::Ability, cantrip_mechanics::statics::other_creatures_you_control, trample_and_haste, "Other creatures you control have trample and haste."),
    ],
    ..oracle::ULVENWALD_BEHEMOTH
};

pub static CARD: CardDef = CardDef {
    abilities: &[ability(cost("{5}{G}{G}"), effects::transform_self, "{5}{G}{G}: Transform this creature.")],
    back: Some(&BACK),
    ..oracle::ULVENWALD_ODDITY
};
