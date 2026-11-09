use cantrip_api::*;
use cantrip_mechanics::{regenerate, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[ability(cost("{G}"), regenerate::enchanted_creature, "{G}: Regenerate enchanted creature.")],
    enchant: Some(targets::CREATURE),
    ..oracle::REGENERATION
};
