use cantrip_api::*;
use cantrip_mechanics::attach::{attached_object, equip};
use cantrip_oracle as oracle;

fn trample_and_lifelink(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.keywords = chars.keywords.with(Keyword::Trample).with(Keyword::Lifelink);
}

pub static CARD: CardDef = CardDef {
    abilities: &[equip(mana("{3}"), "Equip {3}")],
    statics: &[
        static_ability(Layer::PtModify, attached_object, cantrip_mechanics::statics::plus_pt::<3, 0>, "Equipped creature gets +3/+0."),
        static_ability(Layer::Ability, attached_object, trample_and_lifelink, "Equipped creature has trample and lifelink."),
    ],
    ..oracle::LOXODON_WARHAMMER
};
