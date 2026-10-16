use cantrip_api::*;
use cantrip_mechanics::effects::SACRIFICE_TO_DESTROY_ARTIFACT_OR_ENCHANTMENT;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[SACRIFICE_TO_DESTROY_ARTIFACT_OR_ENCHANTMENT],
    ..oracle::CATHAR_COMMANDO
};
