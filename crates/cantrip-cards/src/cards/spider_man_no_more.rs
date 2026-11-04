use cantrip_api::*;
use cantrip_mechanics::statics::{base_pt, creature_type_alone, have_defender};
use cantrip_mechanics::{attach::attached_object, lands, targets};
use cantrip_oracle as oracle;

// "Enchanted creature is a Citizen with base power and toughness 1/1. It has defender and loses all other abilities.": static
// abilities of the Aura, in layers 4, 6 and 7b, all at the time it became attached (CR 613.7e). The loss comes before the grant in
// the list, so defender - granted later in layer 6 - is not lost with the rest (CR 613.7).

pub static CARD: CardDef = CardDef {
    enchant: Some(targets::CREATURE),
    statics: &[
        static_ability(Layer::Type, attached_object, creature_type_alone::<{ lands::sub(Subtype::Citizen) }>, "Enchanted creature is a Citizen."),
        static_ability(Layer::PtSet, attached_object, base_pt::<1, 1>, "Enchanted creature has base power and toughness 1/1."),
        loses_all_abilities(attached_object, "Enchanted creature loses all other abilities."),
        static_ability(Layer::Ability, attached_object, have_defender, "Enchanted creature has defender."),
    ],
    ..oracle::SPIDER_MAN_NO_MORE
};
