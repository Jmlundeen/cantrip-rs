use cantrip_api::*;
use cantrip_mechanics::{attach::attached_object, targets};
use cantrip_oracle as oracle;

/// "Enchant creature" (CR 303.4a, 702.5) and "You control enchanted creature." (CR 613.1b, layer 2): the Aura's controller
/// controls the creature it is attached to, at the Aura's timestamp (CR 613.7e), for as long as it is attached. Its
/// Auras and Equipment do not come with it (the card's ruling).
pub static CARD: CardDef = CardDef {
    enchant: Some(targets::CREATURE),
    statics: &[control_changing(attached_object, "You control enchanted creature.")],
    ..oracle::MIND_CONTROL
};
