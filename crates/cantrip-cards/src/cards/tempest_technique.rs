use cantrip_api::*;
use cantrip_mechanics::statics::plus_one_plus_one_per_enchantment_you_control;
use cantrip_mechanics::storm;
use cantrip_mechanics::attach::attached_object;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    enchant: Some(targets::CREATURE_YOU_CONTROL),
    abilities: &[storm::trigger("Storm (When you cast this spell, copy it for each spell cast before it this turn. You may choose new targets for the copies. Copies become tokens.)")],
    statics: &[static_ability(Layer::PtModify, attached_object, plus_one_plus_one_per_enchantment_you_control, "Enchanted creature gets +1/+1 for each enchantment you control.")],
    ..oracle::TEMPEST_TECHNIQUE
};
