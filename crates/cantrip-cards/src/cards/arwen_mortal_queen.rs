use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Another target creature": a creature on the battlefield other than the ability's source.
fn another_creature(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if id != asker.source) && (targets::CREATURE.filter)(q, asker, t)
}

/// "Arwen enters with an indestructible counter on her" (CR 122.1b, 614.1c).
fn enters(_: &mut dyn Game, entering: &mut Entering) -> Result<(), Suspend> {
    entering.counters[CounterKind::Indestructible as usize] += 1;
    Ok(())
}

/// The target gains indestructible until end of turn and gets a +1/+1 counter and a lifelink counter; Arwen gets
/// the same two counters. Nothing happens if the target is no longer legal (CR 608.2b).
fn spend(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    g.add_effect(
        item.object,
        creature,
        EffectKind::AddKeywords(Keywords::of(&[Keyword::Indestructible])),
        Duration::EndOfTurn,
    );
    g.add_counters(creature, CounterKind::PlusOne, 1);
    g.add_counters(creature, CounterKind::Lifelink, 1);
    g.add_counters(item.object, CounterKind::PlusOne, 1);
    g.add_counters(item.object, CounterKind::Lifelink, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    enters: Some(enters),
    abilities: &[ability(cost("{1}").and(remove_counters(CounterKind::Indestructible, 1)), spend, "{1}, Remove an indestructible counter from Arwen: Another target creature gains indestructible until end of turn. Put a +1/+1 counter and a lifelink counter on that creature and a +1/+1 counter and a lifelink counter on Arwen.")
        .targets(&[target(another_creature)])],
    ..oracle::ARWEN_MORTAL_QUEEN
};
