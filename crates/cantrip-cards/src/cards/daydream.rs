use cantrip_api::*;
use cantrip_mechanics::{flashback, targets};
use cantrip_oracle as oracle;

/// "Exile target creature you control, then return that card to the battlefield under its owner's control with a
/// +1/+1 counter on it." (CR 400.7: a new object; CR 122.6: it enters with the counter; CR 111.7: a token exiled this
/// way ceases to exist and does not return.) Nothing happens if the target is no longer legal (CR 608.2b).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    let token = g.obj(creature).token;
    let exiled = g.exile(creature);
    if !token && g.obj(exiled).zone.is(ZoneKind::Exile) {
        let owner = g.obj(exiled).owner;
        g.put_onto_battlefield_with_counters(exiled, owner, false, &[(CounterKind::PlusOne, 1)])?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE_YOU_CONTROL], effect),
    ways: &[flashback::way(cost("{2}{W}"), "Flashback {2}{W}")],
    ..oracle::DAYDREAM
};
