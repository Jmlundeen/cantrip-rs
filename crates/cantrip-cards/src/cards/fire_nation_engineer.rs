use cantrip_api::*;
use cantrip_mechanics::{effects, raid::you_attacked_this_turn, targets, triggers::YOUR_END_STEP};
use cantrip_oracle as oracle;

/// "Another target creature or Vehicle you control" (CR 115.1, 205.3g): not the Engineer itself.
fn another_creature_or_vehicle_you_control(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    (targets::CREATURE_OR_VEHICLE_YOU_CONTROL.filter)(q, asker, t) && t != Target::Object(asker.source)
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        YOUR_END_STEP,
        effects::plus_one_counter_on_target,
        "Raid \u{2014} At the beginning of your end step, if you attacked this turn, put a +1/+1 counter on another target creature or Vehicle you control.",
    )
    .targets(&[target(another_creature_or_vehicle_you_control)])
    .condition(you_attacked_this_turn)],
    ..oracle::FIRE_NATION_ENGINEER
};
