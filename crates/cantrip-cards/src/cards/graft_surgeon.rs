use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_mechanics::triggers::THIS_DIES;
use cantrip_oracle as oracle;

/// "This creature enters with a +1/+1 counter on it" (CR 614.1c, 122.6).
fn enter_with_counter(_: &mut dyn Game, entering: &mut Entering) -> Result<(), Suspend> {
    entering.counters[CounterKind::PlusOne as usize] += 1;
    Ok(())
}

/// "When this creature dies, put its counters on up to one target creature you control" Use last-known
/// counters; put them on the target rather than moving them (CR 608.2h).
fn pass_counters(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    let counters = g.obj(item.object).counter_array();
    // Every kind together, one event (CR 608.2, 122.6).
    let puts: Vec<(ObjectId, CounterKind, i16)> = CounterKind::ALL.into_iter().map(|kind| (creature, kind, counters[kind as usize])).collect();
    g.add_counters_together(&puts);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    enters: Some(enter_with_counter),
    abilities: &[triggered(
        THIS_DIES,
        pass_counters,
        "When this creature dies, put its counters on up to one target creature you control.",
    )
    .targets(&[targets::CREATURE_YOU_CONTROL.optional()])],
    ..oracle::GRAFT_SURGEON
};
