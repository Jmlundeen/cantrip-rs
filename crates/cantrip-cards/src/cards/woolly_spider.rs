use cantrip_api::*;
use cantrip_mechanics::{effects, triggers};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            triggers::THIS_BLOCKS_A_CREATURE_WITH_FLYING,
            effects::this_gets::<0, 2>,
            "Whenever this creature blocks a creature with flying, this creature gets +0/+2 until end of turn.",
        ),
    ],
    ..oracle::WOOLLY_SPIDER
};
