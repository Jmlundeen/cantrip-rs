use cantrip_api::*;
use cantrip_oracle as oracle;
/// Enters prepared (CR 722); the identical Seed Suture spell is shared with Blossom-Blessed Angel.
pub static CARD: CardDef = CardDef {
 enters: Some(cantrip_mechanics::prepare::enters_prepared),
 prepare: Some(&super::blossom_blessed_angel::SEED_SUTURE),
 ..oracle::EMERGENCY_PHYTOMEDIC
};
