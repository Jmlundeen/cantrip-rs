use cantrip_api::*;
use cantrip_mechanics::{attach, regenerate, statics, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[ability(SACRIFICE_THIS, regenerate::enchanted_creature, "Sacrifice this Aura: Regenerate enchanted creature.")],
    statics: &[static_ability(
        Layer::PtModify,
        attach::attached_object,
        statics::plus_pt::<1, 1>,
        "Enchanted creature gets +1/+1.",
    )],
    enchant: Some(targets::CREATURE),
    ..oracle::THRULL_RETAINER
};
