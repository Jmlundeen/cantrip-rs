use cantrip_api::*;
use cantrip_mechanics::statics::{base_pt, creature_too};
use cantrip_mechanics::{attach, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    statics: &[
        static_ability(
            Layer::Type,
            attach::attached_object,
            creature_too,
            "Enchanted artifact is a creature with base power and toughness 5/5 in addition to its other types.",
        ),
        static_ability(
                Layer::PtSet,
                attach::attached_object,
                base_pt::<5, 5>,
                "Enchanted artifact is a creature with base power and toughness 5/5 in addition to its other types.",
            )
            .continues(),
    ],
    enchant: Some(targets::ARTIFACT),
    ..oracle::ENSOUL_ARTIFACT
};
