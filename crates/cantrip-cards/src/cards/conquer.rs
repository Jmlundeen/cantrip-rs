use cantrip_api::*;
use cantrip_mechanics::{attach, targets};
use cantrip_oracle as oracle;

/// "Enchant land" (CR 702.5a, 303.4) and "You control enchanted land." (CR 613.1b, 611.3a).
pub static CARD: CardDef = CardDef {
    statics: &[control_changing(attach::attached_object, "You control enchanted land.")],
    enchant: Some(targets::LAND),
    ..oracle::CONQUER
};
