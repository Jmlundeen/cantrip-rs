use cantrip_api::*;
use cantrip_mechanics::{double, targets, triggers};
use cantrip_oracle as oracle;

/// Landfall: put a +1/+1 counter on target creature.
fn counter(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = targets::object(g, item, 0) {
        g.add_counters(creature, CounterKind::PlusOne, 1);
    }
    Ok(())
}

/// Double the number of +1/+1 counters on each creature you control (CR 701.10e): each gets as many more as it has.
fn double(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    double::counters_on_creatures_you_control(g, item.controller, CounterKind::PlusOne);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(triggers::LANDFALL, counter, "Landfall - Whenever a land you control enters, put a +1/+1 counter on target creature.")
            .targets(&[targets::CREATURE]),
        ability(cost("{3}{G}{G}"), double, "{3}{G}{G}: Double the number of +1/+1 counters on each creature you control."),
    ],
    ..oracle::BRISTLY_BILL_SPINE_SOWER
};
