use cantrip_api::*;
use cantrip_mechanics::{effects, targets, triggers};
use cantrip_oracle as oracle;

/// "Whenever a creature you control enters, you gain 1 life." (CR 603.6a, 119.3): one life for each creature that enters,
/// this artifact's controller's own creatures only.
fn gain_one(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::gain_life(g, item, 1)
}

/// "At the beginning of combat on your turn, put X +1/+1 counters on target creature you control, where X is one plus the
/// amount of life you gained this turn." (CR 608.2h, 122.6): counted as the ability resolves, so the life gained earlier
/// this turn - its own trigger's among it - is what X reads.
fn counters_by_life_gained(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    let amount = 1 + g.history().life_gained(item.controller);
    g.add_counters(creature, CounterKind::PlusOne, amount as i32);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(triggers::CREATURE_YOU_CONTROL_ENTERS, gain_one, "Whenever a creature you control enters, you gain 1 life."),
        triggered(
            triggers::BEGINNING_OF_YOUR_COMBAT,
            counters_by_life_gained,
            "At the beginning of combat on your turn, put X +1/+1 counters on target creature you control, where X is one plus the amount of life you gained this turn.",
        )
        .targets(&[targets::CREATURE_YOU_CONTROL]),
    ],
    ..oracle::SICKBAY
};
