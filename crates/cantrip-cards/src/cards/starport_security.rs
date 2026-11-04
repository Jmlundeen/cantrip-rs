use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "This ability costs {2} less to activate if you control a creature with a +1/+1 counter on it" (CR 601.2f, 602.2b): generic
/// mana only (CR 118.7a). Starport Security itself counts when it has one.
fn less_with_a_plus_one_counter_creature(q: &dyn Query, asker: &Asker, _: Costed, _: &[Target]) -> CostDelta {
    let has_counter = q.permanents_of(asker.controller).into_iter().any(|id| q.chars(id).is(CardType::Creature) && q.obj(id).counters(CounterKind::PlusOne) > 0);
    if has_counter { CostDelta::less(2) } else { CostDelta::NONE }
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(
        cost("{3}{W}").and(TAP),
        effects::tap_targets,
        "{3}{W}, {T}: Tap another target creature. This ability costs {2} less to activate if you control a creature with a +1/+1 counter on it.",
    )
    .targets(&[targets::OTHER_CREATURE])
    .cost_change(less_with_a_plus_one_counter_creature)],
    ..oracle::STARPORT_SECURITY
};
