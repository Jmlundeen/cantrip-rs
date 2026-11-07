use cantrip_api::*;
use cantrip_mechanics::{attach, statics, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    statics: &[static_ability(
        Layer::Ability,
        attach::attached_object,
        statics::have_indestructible,
        "Enchanted land has indestructible.",
    )],
    enchant: Some(targets::LAND),
    targeting_rules: &[attach::ENCHANTED_NO_OTHER_AURAS],
    ..oracle::CONSECRATE_LAND
};
