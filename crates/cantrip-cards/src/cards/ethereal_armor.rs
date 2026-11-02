use cantrip_api::*;
use cantrip_mechanics::statics::{have_first_strike, plus_one_plus_one_per_enchantment_you_control};
use cantrip_mechanics::{attach, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    statics: &[
        static_ability(
            Layer::PtModify,
            attach::attached_object,
            plus_one_plus_one_per_enchantment_you_control,
            "Enchanted creature gets +1/+1 for each enchantment you control and has first strike.",
        ),
        static_ability(
            Layer::Ability,
            attach::attached_object,
            have_first_strike,
            "Enchanted creature gets +1/+1 for each enchantment you control and has first strike.",
        ),
    ],
    enchant: Some(targets::CREATURE),
    ..oracle::ETHEREAL_ARMOR
};
