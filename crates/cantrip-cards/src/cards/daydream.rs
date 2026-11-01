use cantrip_api::*;
use cantrip_mechanics::{flashback, targets};
use cantrip_oracle as oracle;

/// "Exile target creature you control, then return that card to the battlefield under its owner's control with a
/// +1/+1 counter on it." (CR 400.7: a new object; CR 122.6: it enters with the counter; CR 111.7: a token exiled this
/// way ceases to exist and does not return.) Nothing happens if the target is no longer legal (CR 608.2b). Each card of a merged
/// permanent returns, together, each with its counter (CR 730.3c).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    let returning: Vec<(ObjectId, PlayerId, bool)> = g
        .exile_all(&[creature])
        .into_iter()
        .filter(|&c| !g.obj(c).token && g.obj(c).zone.is(ZoneKind::Exile))
        .map(|c| (c, g.obj(c).owner, false))
        .collect();
    if !returning.is_empty() {
        g.put_all_onto_battlefield_with_counters(&returning, &[(CounterKind::PlusOne, 1)])?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE_YOU_CONTROL], effect),
    ways: &[flashback::way(cost("{2}{W}"), "Flashback {2}{W}")],
    ..oracle::DAYDREAM
};
