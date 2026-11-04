use cantrip_api::*;
use cantrip_mechanics::{attach, statics::plus_pt, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    statics: &[static_ability(
        Layer::PtModify,
        attach::attached_object,
        plus_pt::<2, -2>,
        "Enchanted creature gets +2/-2.",
    )],
    enchant: Some(targets::CREATURE),
    ..oracle::IMMOLATION
};
