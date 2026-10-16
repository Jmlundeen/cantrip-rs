use cantrip_api::*;
use cantrip_mechanics::{storied, targets, triggers};
use cantrip_oracle as oracle;

/// "Whenever Bifur enters or attacks" (CR 603.6a, 508.3a).
fn enters_or_attacks(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    triggers::this_enters(q, source, event).or_else(|| triggers::this_attacks(q, source, event))
}

/// "Put a +1/+1 counter on target creature."
fn counter(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = targets::object(g, item, 0) {
        g.add_counters(creature, CounterKind::PlusOne, 1);
    }
    Ok(())
}

/// "As long as you have an enduring story, if a triggered ability of a Dwarf you control triggers, that ability triggers an
/// additional time" (CR 603.2d, 702.195b): a permanent's ability, its source a Dwarf under your control as it triggers - as
/// it last existed for one that looks back. The designation is had before the abilities an event triggers are looked for
/// (CR 702.195c), so Bifur entering as the third storied permanent doubles its own enters ability (the rulings). "Dwarf"
/// is a word of its text (CR 612).
fn dwarf_triggers_again(q: &dyn Query, asker: &Asker, _: &Event, triggering: &Triggering) -> bool {
    storied::has_an_enduring_story(q.state(), asker.controller)
        && triggering.of_a_permanent_controlled_by(asker.controller)
        && q.chars(triggering.source).has_subtype(q.words(asker.source).subtype(Subtype::Dwarf))
}

pub static CARD: CardDef = CardDef {
    storied: true,
    abilities: &[triggered(when(EventMask(EventMask::ZONE_CHANGE.0 | EventMask::ATTACKS.0), enters_or_attacks), counter, "Whenever Bifur enters or attacks, put a +1/+1 counter on target creature.")
        .targets(&[targets::CREATURE])],
    additional_triggers: &[dwarf_triggers_again],
    ..oracle::BIFUR_MELODIC_RIDER
};
