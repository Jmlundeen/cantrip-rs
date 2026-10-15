use cantrip_api::*;
use cantrip_mechanics::attach::{attached_object, equip};
use cantrip_oracle as oracle;

fn plus_three_power(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.power = chars.power.map(|v| v + 3);
}

fn trample_and_lifelink(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.keywords = chars.keywords.with(Keyword::Trample).with(Keyword::Lifelink);
}

pub static CARD: CardDef = CardDef {
    abilities: &[equip(mana(3, 0, 0, 0, 0, 0), "Equip {3}")],
    statics: &[
        StaticDef {
            text: "Equipped creature gets +3/+0.",
            layer: Layer::PtModify,
            affects: attached_object,
            apply: plus_three_power,
            ..StaticDef::BLANK
        },
        StaticDef {
            text: "Equipped creature has trample and lifelink.",
            layer: Layer::Ability,
            affects: attached_object,
            apply: trample_and_lifelink,
            ..StaticDef::BLANK
        },
    ],
    ..oracle::LOXODON_WARHAMMER
};
