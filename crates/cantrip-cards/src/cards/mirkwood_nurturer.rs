use cantrip_api::*;
use cantrip_mechanics::effects::return_first_target_to_hand_then_plus_one_counter_on_this;
use cantrip_mechanics::{targets, triggers};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::THIS_ENTERS,
        return_first_target_to_hand_then_plus_one_counter_on_this,
        "When this creature enters, return up to one other target permanent you control to its owner's hand. If you do, put a +1/+1 counter on this creature.",
    )
    .targets(&[up_to_one(targets::other_permanent_you_control_filter)])],
    ..oracle::MIRKWOOD_NURTURER
};
