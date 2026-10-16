use cantrip_api::*;
use cantrip_mechanics::attach::{attached_object, equip};
use cantrip_oracle as oracle;

fn haste_and_shroud(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.keywords = chars.keywords.with(Keyword::Haste).with(Keyword::Shroud);
}

pub static CARD: CardDef = CardDef {
    abilities: &[equip(ManaCost::ZERO, "Equip {0}")],
    statics: &[static_ability(Layer::Ability, attached_object, haste_and_shroud, "Equipped creature has haste and shroud.")],
    ..oracle::LIGHTNING_GREAVES
};
