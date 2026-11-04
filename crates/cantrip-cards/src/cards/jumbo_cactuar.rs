use cantrip_api::*;
use cantrip_mechanics::{effects, triggers};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::THIS_ATTACKS,
        effects::this_gets::<9999, 0>,
        "10,000 Needles \u{2014} Whenever this creature attacks, it gets +9999/+0 until end of turn.",
    )],
    ..oracle::JUMBO_CACTUAR
};
