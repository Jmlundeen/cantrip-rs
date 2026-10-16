use cantrip_api::*;
use cantrip_mechanics::attach::{attached_object, equip};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[equip(mana("{1}"), "Equip {1}")],
    delayed: &[mana_ability(TAP, add_any_color(1), "{T}: Add one mana of any color.")],
    statics: &[static_ability(Layer::Ability, attached_object, cantrip_mechanics::statics::grant_delayed_ability, "Equipped creature has \"{T}: Add one mana of any color.\"")],
    ..oracle::PARADISE_MANTLE
};
