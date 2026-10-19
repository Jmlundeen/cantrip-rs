//! Persist (CR 702.79a): "When this permanent is put into a graveyard from the battlefield, if it had no -1/-1 counters
//! on it, return it to the battlefield under its owner's control with a -1/-1 counter on it." Woodfall Primus prints it.
//!
//! A card with persist lists `trigger(text)` among its `abilities`.

use crate::triggers::THIS_DIES;
use cantrip_api::*;

/// "If it had no -1/-1 counters on it" (CR 603.4): the permanent as it last was on the battlefield.
fn had_no_minus_counters(q: &dyn Query, asker: &Asker) -> bool {
    q.obj(asker.source).counters(CounterKind::MinusOne) == 0
}

/// The return: the card the permanent became in the graveyard, if it is still there (CR 400.7).
fn returns(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(card) = g.found_after_move(item.object) else { return Ok(()) };
    let owner = g.obj(card).owner;
    g.put_onto_battlefield_with_counters(card, owner, false, &[(CounterKind::MinusOne, 1)])?;
    Ok(())
}

pub const fn trigger(text: &'static str) -> AbilityDef {
    triggered(THIS_DIES, returns, text)
        .condition(had_no_minus_counters)
        .look_back()
}
