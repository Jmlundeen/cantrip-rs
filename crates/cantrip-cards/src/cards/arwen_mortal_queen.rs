use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Arwen enters with an indestructible counter on her" (CR 122.1b, 614.1c).
fn enters(_: &mut dyn Game, entering: &mut Entering) -> Result<(), Suspend> {
    entering.counters[CounterKind::Indestructible as usize] += 1;
    Ok(())
}

/// The target gains indestructible until end of turn and gets a +1/+1 counter and a lifelink counter; Arwen gets
/// the same two counters. Nothing happens if the target is no longer legal (CR 608.2b).
fn spend(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    cantrip_mechanics::effects::gain_indestructible_until_end_of_turn(g, item.object, creature);
    g.add_counters(creature, CounterKind::PlusOne, 1);
    g.add_counters(creature, CounterKind::Lifelink, 1);
    g.add_counters(item.object, CounterKind::PlusOne, 1);
    g.add_counters(item.object, CounterKind::Lifelink, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    enters: Some(enters),
    abilities: &[ability(cost("{1}").and(remove_counters(CounterKind::Indestructible, 1)), spend, "{1}, Remove an indestructible counter from Arwen: Another target creature gains indestructible until end of turn. Put a +1/+1 counter and a lifelink counter on that creature and a +1/+1 counter and a lifelink counter on Arwen.")
        .targets(&[targets::OTHER_CREATURE])],
    ..oracle::ARWEN_MORTAL_QUEEN
};
