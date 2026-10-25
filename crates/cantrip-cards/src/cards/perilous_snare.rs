use cantrip_api::*;
use cantrip_mechanics::max_speed::max_speed;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_mechanics::{exile_target_until_leaves_effect, targets};
use cantrip_oracle as oracle;

/// "Put a +1/+1 counter on target creature or Vehicle you control" (CR 122.1a): nothing if the target is gone or no
/// longer a creature or Vehicle you control (CR 608.2b).
fn counter_on_creature_or_vehicle(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    cantrip_mechanics::effects::plus_one_counter_on_target(g, item)
}

pub static CARD: CardDef = CardDef {
    start_your_engines: true,
    abilities: &[triggered(THIS_ENTERS, exile_target_until_leaves_effect, "When this artifact enters, exile target nonland permanent an opponent controls until this artifact leaves the battlefield.")
        .targets(&[targets::NONLAND_PERMANENT_AN_OPPONENT_CONTROLS])],
    delayed: &[ability(TAP, counter_on_creature_or_vehicle, "{T}: Put a +1/+1 counter on target creature or Vehicle you control. Activate only as a sorcery.")
        .targets(&[targets::CREATURE_OR_VEHICLE_YOU_CONTROL])
        .sorcery_speed()],
    statics: &[max_speed("Max speed \u{2014} {T}: Put a +1/+1 counter on target creature or Vehicle you control. Activate only as a sorcery.")],
    ..oracle::PERILOUS_SNARE
};
