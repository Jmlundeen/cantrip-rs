use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Target non-Human creature that entered this turn" (CR 115.1, 400.7), asked of the event history.
fn non_human_that_entered_this_turn(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    (targets::CREATURE.filter)(q, asker, t)
        && matches!(t, Target::Object(id) if !q.chars(id).has_subtype(q.words(asker.source).subtype(Subtype::Human)) && cantrip_mechanics::history::entered_this_turn(q, id))
}

/// Put two +1/+1 counters on the target.
fn counters(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = targets::object(g, item, 0) {
        g.add_counters(creature, CounterKind::PlusOne, 2);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        ability(cost("{2}, {T}"), counters, "{2}, {T}: Put two +1/+1 counters on target non-Human creature that entered this turn.")
            .targets(&[TargetSpec { min: 1, max: 1, filter: non_human_that_entered_this_turn }]),
    ],
    ..oracle::DRANNITH_RUINS
};
