//! Evoke [cost] (CR 702.74a): "You may cast this spell for its evoke cost. If you do, it's sacrificed when it enters."
//! Endurance, Fury, Grief, Solitude, Subtlety, Vibrance and Wistfulness print it.
//!
//! A card with evoke lists `way(cost, text)` (or `way_exiling_card`, for the pitch cost) as its only alternative cost
//! and `sacrifice_trigger(text)` among its `abilities`; the way is the card's first, which is what `evoked` reads.

use crate::triggers::THIS_ENTERS;
use cantrip_api::*;

/// The alternative cost of evoke: cast from the hand for `cost` (CR 118.9). It is the card's first way.
pub const fn way(cost: Cost, text: &'static str) -> WayDef {
    cantrip_api::way(cost, ZoneKind::Hand, text)
}

/// The evoke cost "exile a card of the kind `filter` accepts from your hand".
pub const fn exile_from_hand(filter: ObjectFilter) -> Cost {
    Cost { exile_from_hand: Some((1, filter)), ..Cost::NONE }
}

/// "If its evoke cost was paid": the way the permanent was cast.
pub fn evoked(q: &dyn Query, asker: &Asker) -> bool {
    q.obj(asker.source).cast_way == 1
}

/// "Its controller sacrifices it" (CR 702.74a): whoever controls the permanent as the ability resolves, not necessarily the
/// ability's controller.
fn sacrifice_it(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if g.obj(item.object).became.is_none() {
        g.sacrifice(g.controller(item.object), &[item.object]);
    }
    Ok(())
}

/// "When this enters, if its evoke cost was paid, its controller sacrifices it" (CR 702.74a): `text` names the card.
pub const fn sacrifice_trigger(text: &'static str) -> AbilityDef {
    triggered(THIS_ENTERS, sacrifice_it, text)
        .condition(evoked)
}
