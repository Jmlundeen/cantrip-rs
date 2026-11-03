use cantrip_api::*;
use cantrip_mechanics::{attach::attached_object, effects, lands::sub, statics::set_land_type, targets, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    // "Enchant land" (CR 702.5a, 303.4a).
    enchant: Some(targets::LAND),
    // "When this Aura enters, draw a card." (CR 603.6a)
    abilities: &[triggered(THIS_ENTERS, |g, item| effects::draw(g, item, 1), "When this Aura enters, draw a card.")],
    // "Enchanted land is an Island." (CR 303.4m: the land it is attached to.)
    statics: &[sets_land_subtypes(attached_object, set_land_type::<{ sub(Subtype::Island) }>, "Enchanted land is an Island.")],
    ..oracle::SPREADING_SEAS
};
