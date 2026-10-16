use cantrip_api::*;
use cantrip_mechanics::attach::{attached_object, equip};
use cantrip_oracle as oracle;

/// "Equipped creature has deathtouch and lifelink" (CR 301.5, 613.1f).
fn deathtouch_and_lifelink(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.keywords = chars.keywords.with(Keyword::Deathtouch).with(Keyword::Lifelink);
}

pub static CARD: CardDef = CardDef {
    abilities: &[equip(mana("{2}"), "Equip {2}")],
    statics: &[static_ability(Layer::Ability, attached_object, deathtouch_and_lifelink, "Equipped creature has deathtouch and lifelink.")],
    ..oracle::BASILISK_COLLAR
};
