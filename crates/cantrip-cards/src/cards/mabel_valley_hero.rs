use cantrip_api::*;
use cantrip_mechanics::{history, targets};
use cantrip_oracle as oracle;

/// "Target creature that entered this turn" (CR 603.6a): a creature on the battlefield whose zone change onto it is in
/// this turn's history.
fn entered_this_turn(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    (targets::CREATURE.filter)(q, asker, t) && matches!(t, Target::Object(id) if history::entered_this_turn(q.history(), id))
}

/// The self-enter clause also applies when a copy is not a creature (CR 201.5).
fn mabel_or_another_creature_enters(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { new, to, .. } if to.is(ZoneKind::Battlefield)
            && (*new == me.source || (q.controller(*new) == me.controller && q.chars(*new).is(CardType::Creature))) => Fired::PLAIN,
        _ => None,
    }
}
fn add_counter(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = targets::object(g, item, 0) {
        g.add_counters(creature, CounterKind::PlusOne, 1);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        when_asked(EventMask::ZONE_CHANGE, mabel_or_another_creature_enters),
        add_counter,
        "Whenever Mabel or another creature you control enters, put a +1/+1 counter on target creature that entered this turn.",
    )
    .targets(&[target(entered_this_turn)])],
    ..oracle::MABEL_VALLEY_HERO
};
