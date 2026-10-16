//! Tiered (CR 702.183a): "Choose one. As an additional cost to cast this spell, pay the cost associated with that
//! mode." Restoration Magic prints it.
//!
//! A card with tiered gives `ModeDef`s whose `cost` is the mode's cost (it is paid on top of the spell's mana cost, which
//! alone gives its mana value, CR 202.3) and makes its `modal` from `modal`.

use cantrip_api::*;

/// The modal spell of a tiered card: exactly one of its modes, which adds its own cost.
pub const fn modal(modes: &'static [ModeDef]) -> ModalDef {
    cantrip_api::modes(modes).choose(1, 1)
}
