use cantrip_api::*;
use cantrip_mechanics::{targets, triggers, typecycling};
use cantrip_oracle as oracle;

/// "When this creature enters, put three +1/+1 counters on target creature." One event of three counters (CR 122.6),
/// on the creature as it is when the ability resolves, its controller's choice of any creature (CR 115.1).
fn three_counters(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = targets::object(g, item, 0) {
        g.add_counters(creature, CounterKind::PlusOne, 3);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            triggers::THIS_ENTERS,
            three_counters,
            "When this creature enters, put three +1/+1 counters on target creature.",
        )
        .targets(&[targets::CREATURE]),
        typecycling::basic_landcycling(mana("{2}"), "Basic landcycling {2}"),
    ],
    ..oracle::VINELASHER_ADEPT
};
