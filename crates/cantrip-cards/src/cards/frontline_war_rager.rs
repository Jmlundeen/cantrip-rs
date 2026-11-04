use cantrip_api::*;
use cantrip_mechanics::{effects::plus_one_counter_on_this, triggers::{YOUR_END_STEP, you_control_two_or_more_tapped_creatures}};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        YOUR_END_STEP,
        plus_one_counter_on_this,
        "At the beginning of your end step, if you control two or more tapped creatures, put a +1/+1 counter on this creature.",
    )
    .condition(you_control_two_or_more_tapped_creatures)],
    ..oracle::FRONTLINE_WAR_RAGER
};
