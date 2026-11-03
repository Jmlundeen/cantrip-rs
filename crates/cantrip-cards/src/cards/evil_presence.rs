use cantrip_api::*;
use cantrip_mechanics::{attach::attached_object, lands::sub, statics::set_land_type, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    enchant: Some(targets::LAND),
    statics: &[sets_land_subtypes(attached_object, set_land_type::<{ sub(Subtype::Swamp) }>, "Enchanted land is a Swamp.")],
    ..oracle::EVIL_PRESENCE
};
