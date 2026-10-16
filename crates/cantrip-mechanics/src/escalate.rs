//! Escalate [cost] (CR 702.120): "Choose one or more modes. Pay the escalate cost for each mode chosen beyond the
//! first." Collective Brutality prints it (escalate by discarding a card).
//!
//! A card with escalate makes its `modal` from `modal`, its modes' own `cost`s being zero.

use cantrip_api::*;

/// The modal spell of an escalate card: one or more of its modes, `cost` paid for each beyond the first.
pub const fn modal(modes: &'static [ModeDef], cost: Cost) -> ModalDef {
    cantrip_api::modes(modes)
        .choose(1, modes.len() as u8)
        .escalate(cost)
}
