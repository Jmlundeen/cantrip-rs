use cantrip_api::*;
use cantrip_mechanics::{attach::attached_object, targets};
use cantrip_oracle as oracle;

/// Enchant creature (CR 303.4a, 702.5a); the Aura's controller controls the enchanted creature in layer 2
/// (CR 613.1b) for as long as it remains attached.
pub static CARD: CardDef = CardDef {
    enchant: Some(targets::CREATURE),
    statics: &[control_changing(attached_object, "You control enchanted creature.")],
    ..oracle::CONTROL_MAGIC
};
