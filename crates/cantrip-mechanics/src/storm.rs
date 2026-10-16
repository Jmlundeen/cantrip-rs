//! Storm (CR 702.40a): "When you cast this spell, copy it for each spell cast before it this turn. You may choose new
//! targets for the copies." Brain Freeze, Haze of Rage, Mind's Desire, Tempest Technique and Tendrils of Agony print it.
//!
//! A card with storm lists `trigger(text)` among its `abilities`. It triggers from the stack when the spell is cast,
//! with the number of spells every player cast this turn before it, fixed as it triggers, even when that is zero.

use crate::history::spells_cast_this_turn;
use cantrip_api::*;

fn cast_this_spell(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::SpellCast { spell, .. } if *spell == source => {
            let cast = spells_cast_this_turn(q) as u32;
            Some(Fired { subject: Target::None, data: [cast - 1, 0] })
        }
        _ => None,
    }
}

/// Copies the spell once for each spell cast before it, each copy offered new targets (CR 707.10, 707.10c). A copy of a
/// permanent spell becomes a token as it resolves (CR 707.10f).
fn copy_per_spell(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    for _ in 0..item.data[0] {
        g.copy_spell(item.object, item.controller, true)?;
    }
    Ok(())
}

/// The storm ability itself; `text` is the card's own reminder text.
pub const fn trigger(text: &'static str) -> AbilityDef {
    triggered(when(EventMask::SPELL_CAST, cast_this_spell), copy_per_spell, text)
        .anywhere()
}
