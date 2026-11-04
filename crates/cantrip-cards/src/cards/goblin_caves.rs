use cantrip_api::*;
use cantrip_mechanics::{statics, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    statics: &[
        static_ability(
            Layer::PtModify,
            statics::goblin_creatures_while_enchanted_land_is_basic_mountain,
            statics::plus_pt::<0, 2>,
            "As long as enchanted land is a basic Mountain, Goblin creatures get +0/+2.",
        ),
    ],
    enchant: Some(targets::LAND),
    ..oracle::GOBLIN_CAVES
};
