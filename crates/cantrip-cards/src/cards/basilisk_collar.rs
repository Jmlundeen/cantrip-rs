use cantrip_api::*;
use cantrip_mechanics::attach::{attached_object, equip};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[equip(mana("{2}"), "Equip {2}")],
    statics: &[static_ability(Layer::Ability, attached_object, cantrip_mechanics::statics::have_deathtouch_and_lifelink, "Equipped creature has deathtouch and lifelink.")],
    ..oracle::BASILISK_COLLAR
};
