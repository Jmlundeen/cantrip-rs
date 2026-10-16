use cantrip_api::*;
use cantrip_mechanics::delayed;
use cantrip_mechanics::targets;
use cantrip_mechanics::triggers;
use cantrip_oracle as oracle;

/// "Up to one other target nonland permanent."
fn other_nonland_permanent(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    (targets::NONLAND_PERMANENT.filter)(q, asker, t) && t != Target::Object(asker.source)
}

/// Exile it, then schedule the return at the next end step (CR 603.7), carrying the exiled card as the delayed
/// ability's subject. The delayed ability exists whether or not anything was exiled (CR 603.7d).
fn exile(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let mut subject = Target::None;
    if let Some(permanent) = targets::object(g, item, 0) {
        let exiled = g.exile(permanent);
        if !g.obj(exiled).token && g.obj(exiled).zone.is(ZoneKind::Exile) {
            subject = Target::Object(exiled);
        }
    }
    delayed::at_next_end_step(g, item, 0, subject);
    Ok(())
}

/// Return that card under its owner's control; if it entered under the ability's controller, Phelia (if still on the
/// battlefield) gets a +1/+1 counter.
fn give_back(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Target::Object(card) = item.subject else { return Ok(()) };
    if g.obj(card).became.is_some() || !g.obj(card).zone.is(ZoneKind::Exile) {
        return Ok(());
    }
    let owner = g.obj(card).owner;
    let entered = g.put_onto_battlefield(card, owner, false)?;
    if g.obj(entered).zone.is(ZoneKind::Battlefield)
        && g.controller(entered) == item.controller
        && targets::permanent(g, item.object)
    {
        g.add_counters(item.object, CounterKind::PlusOne, 1);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(triggers::THIS_ATTACKS, exile, "Whenever Phelia attacks, exile up to one other target nonland permanent. At the beginning of the next end step, return that card to the battlefield under its owner's control. If it entered under your control, put a +1/+1 counter on Phelia.")
        .targets(&[TargetSpec { min: 0, max: 1, filter: other_nonland_permanent }])],
    delayed: &[delayed_trigger(give_back, "Return that card to the battlefield under its owner's control. If it entered under your control, put a +1/+1 counter on Phelia.")],
    ..oracle::PHELIA_EXUBERANT_SHEPHERD
};
