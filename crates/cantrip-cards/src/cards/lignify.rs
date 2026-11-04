use cantrip_api::*;
use cantrip_mechanics::{attach::attached_object, lands, statics::{base_pt, creature_type_alone}, targets};
use cantrip_oracle as oracle;

// "Enchanted creature is a Treefolk with base power and toughness 0/4 and loses all abilities.": one static ability, an
// effect in layers 4, 6 and 7b, all at the time this Aura became attached (CR 613.7e).

pub static CARD: CardDef = CardDef {
    enchant: Some(targets::CREATURE),
    statics: &[
        static_ability(Layer::Type, attached_object, creature_type_alone::<{ lands::sub(Subtype::Treefolk) }>, "Enchanted creature is a Treefolk."),
        static_ability(Layer::PtSet, attached_object, base_pt::<0, 4>, "Enchanted creature has base power and toughness 0/4."),
        loses_all_abilities(attached_object, "Enchanted creature loses all abilities."),
    ],
    ..oracle::LIGNIFY
};
