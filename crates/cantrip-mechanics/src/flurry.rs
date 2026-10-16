//! Flurry (CR 702.170a): "Whenever you cast your second spell each turn, [effect]." Cori-Steel Cutter prints it.
//!
//! A card with flurry lists `trigger(text, effect)` among its `abilities`.

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
