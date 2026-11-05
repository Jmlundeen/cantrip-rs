use cantrip_api::*;
use cantrip_mechanics::{effects, triggers::LANDFALL};
use cantrip_oracle as oracle;

/// "Landfall — Whenever a land you control enters, put a +1/+1 counter on this creature" (CR 122.1, 603.6a): the counter
/// goes on the Surveyors only while they are still on the battlefield (CR 400.7).
pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        LANDFALL,
        effects::plus_one_counter_on_this,
        "Landfall \u{2014} Whenever a land you control enters, put a +1/+1 counter on this creature.",
    )],
    ..oracle::XINDI_SURVEYORS
};
