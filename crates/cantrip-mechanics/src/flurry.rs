//! Flurry (CR 702.170a): "Whenever you cast your second spell each turn, [effect]." Cori-Steel Cutter and Cosmogrand Zenith print it.
//!
//! A card with flurry lists `trigger(text, effect)` among its `abilities`, or `modal_trigger(text, modes)` when the
//! effect is a choice of modes (CR 700.2b).

use cantrip_api::*;

fn your_second_spell(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::SpellCast { player, nth: 2, .. } if *player == q.controller(source) => Fired::PLAIN,
        _ => None,
    }
}

pub const fn trigger(text: &'static str, effect: fn(&mut dyn Game, &StackItem) -> Result<(), Suspend>) -> AbilityDef {
    triggered(when(EventMask::SPELL_CAST, your_second_spell), effect, text)
}

/// The same trigger with modes, chosen as it is put on the stack (CR 603.3c, 700.2b).
pub const fn modal_trigger(modes: &'static ModalDef, text: &'static str) -> AbilityDef {
    modal_triggered(when(EventMask::SPELL_CAST, your_second_spell), modes, text)
}
