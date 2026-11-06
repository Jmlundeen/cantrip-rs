use cantrip_api::*;
use cantrip_mechanics::{effects, triggers};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            triggers::THIS_BECOMES_TAPPED,
            effects::each_opponent_loses_1_life_you_gain_1,
            "Whenever Badgey becomes tapped, each opponent loses 1 life and you gain 1 life.",
        ),
    ],
    ..oracle::BADGEY_MALICIOUS_GLITCH
};
