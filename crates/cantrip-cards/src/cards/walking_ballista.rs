use cantrip_api::*;
use cantrip_mechanics::{damage_target, targets};
use cantrip_oracle as oracle;

/// {4}: Put a +1/+1 counter on this creature.
fn grow(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if is_live(g, item.object) && g.is_on_battlefield(item.object) {
        g.add_counters(item.object, CounterKind::PlusOne, 1);
    }
    Ok(())
}

/// It deals 1 damage to any target (the counter is already gone: the cost).
fn ping(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    damage_target(g, item, 0, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    enters: Some(cantrip_mechanics::effects::enters_with_x_counters),
    abilities: &[
        ability(cost("{4}"), grow, "{4}: Put a +1/+1 counter on this creature."),
        ability(remove_counters(CounterKind::PlusOne, 1), ping, "Remove a +1/+1 counter from this creature: It deals 1 damage to any target.")
            .targets(&[targets::ANY]),
    ],
    ..oracle::WALKING_BALLISTA
};
