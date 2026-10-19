use cantrip_api::*;
use cantrip_mechanics::history::entered_this_turn;
use cantrip_mechanics::statics::have_haste;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Each other creature you control with a +1/+1 counter on it has haste" (CR 702.10a, layer 6): a creature its controller
/// controls other than this one that has any +1/+1 counter on it as the layers apply (CR 122.1).
fn other_creature_with_a_counter(q: &dyn LayerQuery, source: ObjectId, affected: ObjectId, so_far: &Chars) -> bool {
    let state = q.state();
    so_far.is(CardType::Creature)
        && affected != source
        && state.obj(affected).controller == state.obj(source).controller
        && state.obj(affected).counters(CounterKind::PlusOne) > 0
}

/// "Target creature that entered this turn" (CR 115.1, 400.7), asked of the event history.
fn entered_this_turn_creature(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    (targets::CREATURE.filter)(q, asker, t) && matches!(t, Target::Object(id) if entered_this_turn(q.history(), id))
}

/// "Put a +1/+1 counter on target creature that entered this turn."
fn counter(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    g.add_counters(creature, CounterKind::PlusOne, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(
        cost("{1}{R}").and(TAP),
        counter,
        "{1}{R}, {T}: Put a +1/+1 counter on target creature that entered this turn.",
    )
    .targets(&[target(entered_this_turn_creature)])],
    statics: &[static_ability(
        Layer::Ability,
        other_creature_with_a_counter,
        have_haste,
        "Each other creature you control with a +1/+1 counter on it has haste.",
    )],
    ..oracle::GALLIA_THE_MERRYMAKER
};
