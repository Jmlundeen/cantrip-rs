use cantrip_api::*;
use cantrip_mechanics::{attach, statics};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[attach::equip(mana("{2}"), "Equip {2}")],
    statics: &[static_ability(
        Layer::Ability,
        attach::attached_object,
        statics::have_indestructible,
        "Equipped creature has indestructible.",
    )],
    ..oracle::DARKSTEEL_PLATE
};
