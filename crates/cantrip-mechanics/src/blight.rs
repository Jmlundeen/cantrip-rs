//! Blight (CR 701.68a): "to blight N means to put N -1/-1 counters on a creature you control", used here as an additional
//! cost ("as an additional cost to cast this spell, you may blight 1"). Requiting Hex, Cinder Strike, Burning Curiosity
//! and Pyrrhic Strike print it as an optional cost; Gristle Glutton and Sting-Slinger as part of an activation.
//!
//! Choosing the creature and putting the counters is the core's (`Cost::blight`, CR 601.2h); a player who controls no
//! creature can't choose to blight (CR 701.68b). A card with the optional kind gives `optional(n)` as its
//! `optional_cost`, and "if this spell's additional cost was paid" is `blighted` of the resolving spell.

use cantrip_api::*;

/// The additional cost "blight N" as a part of a cost.
pub const fn cost(n: u8) -> Cost {
    Cost { blight: n, ..Cost::NONE }
}

/// "As an additional cost to cast this spell, you may blight N."
pub const fn optional(n: u8) -> OptionalCost {
    OptionalCost { cost: cost(n) }
}

/// "If this spell's additional cost was paid": it was cast with its optional cost paid. A copy of the spell has it paid
/// when the original did (CR 707.10).
pub fn blighted(item: &StackItem) -> bool {
    item.paid > 0
}
