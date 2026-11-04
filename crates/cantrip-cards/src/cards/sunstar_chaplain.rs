use cantrip_api::*;
use cantrip_mechanics::triggers::{YOUR_END_STEP, you_control_two_or_more_tapped_creatures};
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            YOUR_END_STEP,
            effects::plus_one_counter_on_target,
            "At the beginning of your end step, if you control two or more tapped creatures, put a +1/+1 counter on target creature you control.",
        )
        .condition(you_control_two_or_more_tapped_creatures)
        .targets(&[targets::CREATURE_YOU_CONTROL]),
        ability(
            // Only +1/+1 counters pay it, from any creature you control (CR 118.3, 122.1).
            cost("{2}").and(remove_kind_counters_among(CounterKind::PlusOne, 1, targets::filters::creature)),
            effects::tap_targets,
            "{2}, Remove a +1/+1 counter from a creature you control: Tap target artifact or creature.",
        )
        .targets(&[targets::ARTIFACT_OR_CREATURE]),
    ],
    ..oracle::SUNSTAR_CHAPLAIN
};
