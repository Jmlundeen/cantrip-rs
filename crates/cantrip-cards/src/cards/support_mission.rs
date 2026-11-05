use cantrip_api::*;
use cantrip_mechanics::triggers::CREATURE_YOU_CONTROL_ENTERS;
use cantrip_oracle as oracle;

/// "Whenever a creature you control enters, put a quest counter on this enchantment. Put a +1/+1 counter on that creature.
/// If this enchantment has five or more quest counters on it, instead put two +1/+1 counters on that creature." (CR
/// 603.6a, 122.1): the quest counter is put first, so the fifth counters' own trigger already sees five (CR 608.2c); the
/// "instead" replaces the one counter with two (CR 614.1a by way of its wording), never adding both. "That creature" is
/// the entering creature, still on the battlefield (CR 400.7), its counters put as the ability resolves.
fn support(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.add_counters(item.object, CounterKind::Quest, 1);
    let Target::Object(creature) = item.subject else { return Ok(()) };
    let n = if g.state().obj(item.object).counters(CounterKind::Quest) >= 5 { 2 } else { 1 };
    g.add_counters(creature, CounterKind::PlusOne, n);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        CREATURE_YOU_CONTROL_ENTERS,
        support,
        "Whenever a creature you control enters, put a quest counter on this enchantment. Put a +1/+1 counter on that creature. If this enchantment has five or more quest counters on it, instead put two +1/+1 counters on that creature.",
    )],
    ..oracle::SUPPORT_MISSION
};
