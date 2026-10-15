//! Spree (CR 702.172): "Choose one or more additional costs: each mode has its own cost, which is paid on top of the
//! spell's mana cost, and you choose the modes by paying them." Requisition Raid, Three Steps Ahead and Trash the Town
//! print it.
//!
//! A card with spree gives `ModeDef`s whose `cost` is the mode's added cost and makes its `modal` from `modal`.

use cantrip_api::*;

/// The modal spell of a spree card: at least one of its modes, any number up to all of them, each adding its own cost.
pub const fn modal(modes: &'static [ModeDef]) -> ModalDef {
    ModalDef { modes, min: 1, max: modes.len() as u8, escalate: None, repeat: false , allowed: None}
}
