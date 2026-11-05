//! Seek out (Star Trek): "Seek out — Whenever you draw your second card each turn, ..." A seek out ability triggers once
//! for the second card drawn in a turn, and not again for the third (CR 603.2, 121.2): the trigger is
//! `triggers::YOU_DRAW_YOUR_SECOND_CARD`. Beverly Crusher, Dr. Pulaski and Julian Bashir print it.
//!
//! A card with seek out lists `seek_out(...)` among its `abilities`, giving its own effect, its target slots and its
//! reminder text; the mechanic is the trigger, and no card writes that trigger itself.

use crate::triggers;
use cantrip_api::*;

/// "Seek out — Whenever you draw your second card each turn, `effect`" (CR 121.1, 603.2): one ability, whose targets (CR
/// 115.1) go on the stack as the trigger does. `text` is the ability's whole printed line.
pub const fn seek_out(effect: Effect, targets: &'static [TargetSpec], text: &'static str) -> AbilityDef {
    triggered(triggers::YOU_DRAW_YOUR_SECOND_CARD, effect, text).targets(targets)
}
