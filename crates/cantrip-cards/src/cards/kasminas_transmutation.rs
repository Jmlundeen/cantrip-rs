use cantrip_api::*;
use cantrip_mechanics::{attach::attached_object, statics::base_pt, targets};
use cantrip_oracle as oracle;

// "Enchanted creature loses all abilities and has base power and toughness 1/1": one static ability, an effect in layer 6
// and one in layer 7b, both at the time this Aura became attached (CR 613.7e).

pub static CARD: CardDef = CardDef {
    enchant: Some(targets::CREATURE),
    statics: &[
        attached_loses_all_abilities("Enchanted creature loses all abilities."),
        static_ability(Layer::PtSet, attached_object, base_pt::<1, 1>, "Enchanted creature has base power and toughness 1/1."),
    ],
    ..oracle::KASMINAS_TRANSMUTATION
};
