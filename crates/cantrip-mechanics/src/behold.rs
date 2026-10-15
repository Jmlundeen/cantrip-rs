//! Behold (CR 701.4a): "choose a [quality] permanent you control or reveal a [quality] card from your hand", used here as
//! an additional cost ("as an additional cost to cast this spell, you may behold a Dragon"). Dispelling Exhale, Piercing
//! Exhale and Osseous Exhale print it, and Caustic Exhale, Kinsbaile Aspirant and Silvergill Mentor print it as a choice
//! with another cost.
//!
//! Choosing the permanent or revealing the card is the core's (`Cost::behold`, paid as the other costs are, CR 601.2h);
//! nothing moves. A card with the optional kind gives `optional(filter)` as its `optional_cost`, and "if a Dragon was
//! beheld" is `beheld` of the resolving spell: the cost is its only way to behold, so it was paid or not (CR 601.2b).

use cantrip_api::*;

/// The additional cost "behold a [quality]" as a part of a cost.
pub const fn cost(filter: ObjectFilter) -> Cost {
    Cost { behold: Some((1, filter)), ..Cost::NONE }
}

/// "As an additional cost to cast this spell, you may behold a [quality]."
pub const fn optional(filter: ObjectFilter) -> OptionalCost {
    OptionalCost { cost: cost(filter) }
}

/// "If a [quality] was beheld": the spell's optional additional cost was paid. A copy of the spell has it paid when the
/// original did (CR 707.10).
pub fn beheld(item: &StackItem) -> bool {
    item.paid > 0
}

/// A Dragon, read through the words of the card whose text says it (CR 612): a permanent or a card in a hand.
pub fn dragon(q: &dyn Query, asker: &Asker, id: ObjectId) -> bool {
    q.chars(id).has_subtype(q.words(asker.source).subtype(Subtype::Dragon))
}
