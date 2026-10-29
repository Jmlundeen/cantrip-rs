use cantrip_api::*;
use cantrip_mechanics::{attach::attached_object, effects::enchanted_creature_gains_first_strike, statics::plus_pt, targets, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

/// Flash is a core keyword.
pub static CARD: CardDef = CardDef {
    enchant: Some(targets::CREATURE),
    abilities: &[triggered(
        THIS_ENTERS,
        enchanted_creature_gains_first_strike,
        "When this Aura enters, enchanted creature gains first strike until end of turn.",
    )],
    statics: &[static_ability(
        Layer::PtModify,
        attached_object,
        plus_pt::<2, 0>,
        "Enchanted creature gets +2/+0.",
    )],
    ..oracle::CAPTAINS_TUTELAGE
};
