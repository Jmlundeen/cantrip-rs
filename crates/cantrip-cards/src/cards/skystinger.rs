use cantrip_api::*;
use cantrip_mechanics::{effects, triggers};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::THIS_BLOCKS_A_CREATURE_WITH_FLYING,
        effects::this_gets::<5, 0>,
        "Whenever this creature blocks a creature with flying, this creature gets +5/+0 until end of turn.",
    )],
    ..oracle::SKYSTINGER
};
