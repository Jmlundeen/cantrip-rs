use cantrip_api::*;
use cantrip_mechanics::delayed;
use cantrip_mechanics::targets;
use cantrip_mechanics::triggers;
use cantrip_oracle as oracle;

/// Exile it, then schedule the return at the next end step (CR 603.7), carrying the exiled cards - each card of a merged
/// permanent (CR 730.3c) - as the delayed ability's objects. The delayed ability exists whether or not anything was exiled
/// (CR 603.7d); a token that was exiled ceased to exist and is carried by nothing (CR 111.7).
fn exile(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let mut cards = Vec::new();
    if let Some(permanent) = targets::object(g, item, 0) {
        cards = g.exile_all(&[permanent]).into_iter().filter(|&c| !g.obj(c).token && g.obj(c).zone.is(ZoneKind::Exile)).collect();
    }
    delayed::at_next_end_step_about(g, item, 0, &cards);
    Ok(())
}

/// Return that card under its owner's control, together with the rest of the cards it carries; if it entered under the ability's
/// controller, Phelia - still the permanent she was (CR 400.7) - gets one +1/+1 counter: the counter goes on Phelia, not on what
/// returned, so it is not repeated for each card, and "if it entered under your control" is one check of the cards found (CR 730.3c).
fn give_back(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let cards = cantrip_mechanics::library::still_exiled(g, g.event_objects(item));
    let entering: Vec<(ObjectId, PlayerId)> = cards.into_iter().map(|c| (c, g.obj(c).owner)).collect();
    let entered = g.put_all_onto_battlefield(&entering)?;
    if entered.iter().any(|&e| g.obj(e).zone.is(ZoneKind::Battlefield) && g.controller(e) == item.controller) && targets::permanent(g, item.object) {
        g.add_counters(item.object, CounterKind::PlusOne, 1);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(triggers::THIS_ATTACKS, exile, "Whenever Phelia attacks, exile up to one other target nonland permanent. At the beginning of the next end step, return that card to the battlefield under its owner's control. If it entered under your control, put a +1/+1 counter on Phelia.")
        .targets(&[up_to_one(targets::other_nonland_permanent_filter)])],
    delayed: &[delayed_trigger(give_back, "Return that card to the battlefield under its owner's control. If it entered under your control, put a +1/+1 counter on Phelia.")],
    ..oracle::PHELIA_EXUBERANT_SHEPHERD
};
