use cantrip_api::*;
use cantrip_mechanics::{targets, triggers};
use cantrip_oracle as oracle;

/// "When The Doctor enters, put a +1/+1 counter on target creature. Then put a +1/+1 counter on each creature you control
/// with a counter on it." (CR 122.1, 608.2h) The target gets its counter first, so a creature you control that had none
/// qualifies for the second sentence; those counters are put at once (CR 101.4), and creatures without a counter, or
/// controlled by the other player, get none.
fn enter(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    g.add_counters(creature, CounterKind::PlusOne, 1);
    g.end_sentence();
    let puts: Vec<(ObjectId, CounterKind, i32)> = g
        .permanents_of(item.controller)
        .into_iter()
        .filter(|&id| g.chars(id).is(CardType::Creature) && g.obj(id).has_counters())
        .map(|id| (id, CounterKind::PlusOne, 1))
        .collect();
    g.add_counters_together(&puts);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::THIS_ENTERS,
        enter,
        "When The Doctor enters, put a +1/+1 counter on target creature. Then put a +1/+1 counter on each creature you control with a counter on it.",
    )
    .targets(&[targets::CREATURE])],
    ..oracle::THE_DOCTOR_EMH
};
