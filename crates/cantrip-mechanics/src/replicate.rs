//! Replicate [cost] (CR 702.56a): "As an additional cost to cast this spell, you may pay [cost] any number of times.
//! When you cast this spell, if a replicate cost was paid for it, copy it for each time its replicate cost was paid. If
//! the spell has targets, you may choose new targets for any of the copies." Lose Focus prints it.
//!
//! A card with replicate gives the cost as a multi `kicker` and lists `trigger(text)` among its `abilities`.
//!
//! Replicate another object gives a spell (Hugh, First of Many's "artifact spells you cast have replicate {2}{U}") is
//! `granted(cost, text)`: the giving card lists it among its `delayed` abilities and grants it with
//! `cantrip_mechanics::statics::grant_delayed_ability`. The cost is an optional additional cost paid any number of times
//! (`OptionalCost::repeated`), which the core offers beside the spell's own and counts in the spell's `paid` (CR 601.2b,
//! 702.56a); each payment copies the spell as it resolves (CR 702.56b).

use cantrip_api::*;

/// "When you cast this spell, if a replicate cost was paid for it" (CR 603.4): it functions on the stack, and an unpaid
/// cast triggers nothing. The count is fixed as it triggers, from the cast itself.
fn replicated(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::SpellCast { spell, .. } if *spell == source && q.obj(source).kicked > 0 => {
            Some(Fired { subject: Target::None, data: [q.obj(source).kicked as u32, 0] })
        }
        _ => None,
    }
}

/// Copies the spell once for each time the replicate cost was paid, each copy offered new targets (CR 707.10c).
fn copy_per_payment(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    for _ in 0..item.data[0] {
        g.copy_spell(item.object, item.controller, true)?;
    }
    Ok(())
}

pub const fn trigger(text: &'static str) -> AbilityDef {
    triggered(when_this_spell_is_cast(replicated), copy_per_payment, text)
}

/// Replicate another object gives a spell (CR 702.56a-b): "It has replicate [cost]". The cost is an optional additional
/// cost paid any number of times (`OptionalCost::repeated`), and the spell is copied once for every payment, the count the
/// cast recorded in its `paid`. Hugh, First of Many prints it.
pub const fn granted(cost: ManaCost, text: &'static str) -> AbilityDef {
    paid_as_it_is_cast(OptionalCost::repeated(cost), copy_per_granted_payment, text)
}

/// "Copy it for each time its replicate cost was paid. If the spell has any targets, you may choose new targets for the
/// copies" (CR 702.56b, 707.10c): a copy of a permanent spell becomes a token as it resolves (CR 707.10f).
fn copy_per_granted_payment(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let paid = g.state().obj(item.object).paid;
    for _ in 0..paid {
        g.copy_spell(item.object, item.controller, true)?;
    }
    Ok(())
}
