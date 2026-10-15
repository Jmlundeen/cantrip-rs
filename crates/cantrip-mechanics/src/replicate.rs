//! Replicate [cost] (CR 702.56a): "As an additional cost to cast this spell, you may pay [cost] any number of times.
//! When you cast this spell, if a replicate cost was paid for it, copy it for each time its replicate cost was paid. If
//! the spell has targets, you may choose new targets for any of the copies." Lose Focus prints it.
//!
//! A card with replicate gives the cost as a multi `kicker` and lists `trigger(text)` among its `abilities`.

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
    AbilityDef::Triggered(TriggeredDef {
        text,
        listens: EventMask::SPELL_CAST,
        matches: replicated,
        effect: copy_per_payment,
        anywhere: true,
        ..TriggeredDef::BLANK
    })
}
