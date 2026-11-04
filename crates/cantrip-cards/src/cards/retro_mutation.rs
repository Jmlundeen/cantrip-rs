use cantrip_api::*;
use cantrip_mechanics::{attach::{attached_object, enchanted_creature_cant_attack}, lands, statics::{base_pt, creature_type_alone}, targets};
use cantrip_oracle as oracle;

// "Enchanted creature is a Turtle with base power and toughness 0/1. It can't attack and loses all abilities.": static
// abilities of the Aura, in layers 4, 6 and 7b, all at the time it became attached (CR 613.7e), and a rule of attacking.

pub static CARD: CardDef = CardDef {
    enchant: Some(targets::CREATURE),
    statics: &[
        static_ability(Layer::Type, attached_object, creature_type_alone::<{ lands::sub(Subtype::Turtle) }>, "Enchanted creature is a Turtle."),
        static_ability(Layer::PtSet, attached_object, base_pt::<0, 1>, "Enchanted creature has base power and toughness 0/1."),
        loses_all_abilities(attached_object, "Enchanted creature loses all abilities."),
    ],
    attack_rules: &[enchanted_creature_cant_attack],
    ..oracle::RETRO_MUTATION
};
