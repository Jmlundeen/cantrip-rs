use cantrip_api::*;
use cantrip_mechanics::{attach, regenerate, statics, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(
            cost("{R}{R}{R}"),
            regenerate::enchanted_creature,
            "{R}{R}{R}: Regenerate enchanted creature.",
        ),
    ],
    statics: &[
        static_ability(
            Layer::PtModify,
            attach::attached_object,
            statics::plus_pt::<1, 0>,
            "Enchanted creature gets +1/+0.",
        ),
    ],
    enchant: Some(targets::CREATURE),
    ..oracle::THE_BRUTE
};
