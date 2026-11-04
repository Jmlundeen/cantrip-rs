use cantrip_api::*;
use cantrip_mechanics::{firebending, triggers};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggers::PROWESS,
        firebending::trigger_its_power("Firebending X, where X is this creature's power. (Whenever this creature attacks, add X {R}. This mana lasts until end of combat.)"),
    ],
    ..oracle::FIREBENDING_STUDENT
};
