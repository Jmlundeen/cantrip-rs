use cantrip_api::*;
use cantrip_mechanics::{flashback, targets};
use cantrip_oracle as oracle;

/// "Target creature you own": owned, not controlled (CR 108.3) - an opponent's Control Magic on it does not make it
/// illegal, and a creature you control but do not own is.
fn creature_you_own(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    (targets::CREATURE.filter)(q, asker, t) && matches!(t, Target::Object(id) if q.obj(id).owner == asker.controller)
}

/// "Exile target creature you own, then return that card to the battlefield under your control with a +1/+1 counter
/// on it." (CR 400.7: a new object; CR 122.6: it enters with the counter; CR 111.7: a token exiled this way ceases to
/// exist and does not return.) Nothing happens if the target is no longer legal (CR 608.2b).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    let token = g.obj(creature).token;
    let exiled = g.exile(creature);
    if !token && g.obj(exiled).zone.is(ZoneKind::Exile) {
        g.put_onto_battlefield_with_counters(exiled, item.controller, false, &[(CounterKind::PlusOne, 1)])?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[target(creature_you_own)], effect),
    ways: &[flashback::way(cost("{2}{W}"), "Flashback {2}{W}")],
    ..oracle::DAYDREAM
};
