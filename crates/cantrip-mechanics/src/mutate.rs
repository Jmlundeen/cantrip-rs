//! Mutate [cost] (CR 702.140): "You may pay [cost] rather than pay this spell's mana cost. If you do, it becomes a mutating
//! creature spell and targets a non-Human creature with the same owner as this spell." Cubwarden, Gemrazer, Nethroi, Apex of
//! Death and the other Ikoria mutate cards print it.
//!
//! A card with mutate lists `way(cost, text)` among its `ways`. The rest is the core's: cast for it, the spell is its card's
//! mutating face (`cantrip_api::mutating_face`), which targets a non-Human creature with the same owner (CR 702.140a); as it
//! resolves with its target illegal it is a creature spell again and enters (CR 702.140b), and with its target legal it merges
//! with it, over or under as its controller chooses (CR 702.140c, 730): the merged permanent has its topmost component's
//! characteristics and every ability of each component (CR 702.140e). "Whenever this creature mutates" is `THIS_MUTATES`
//! (CR 702.140d), and "the number of times it has mutated" `times_mutated`.

use cantrip_api::*;

/// The mutate way: cast for `cost` rather than the mana cost (CR 702.140a, 118.9). It names no zone, so it is open to a card
/// cast from wherever a rule or permission casts it (Mindleecher's ruling: "you may pay alternative costs such as mutate").
pub const fn way(cost: ManaCost, text: &'static str) -> WayDef {
    WayDef { kind: WayKind::Mutate, ..cantrip_api::way(Cost::mana(cost), ZoneKind::Hand, text) }
}

/// "Whenever this creature mutates" (CR 702.140d): a mutating creature spell merged with it as the spell resolved - the
/// ability's own card on top or under, either way (the release notes: "they trigger whether the ability came from the
/// mutating card or from the card it merged with"). Not when a mutating spell ceased to be one and entered (CR 702.140b).
pub fn this_mutates(_: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::Mutated { permanent, .. } if *permanent == source => Fired::PLAIN,
        _ => None,
    }
}

/// `this_mutates` as a trigger.
pub const THIS_MUTATES: Trigger = when(EventMask::MUTATED, this_mutates);

/// How many times `permanent` has mutated (CR 702.140d): every spell that merged with it while it has been this object (CR
/// 730.2c: a merged permanent is the object it was), the one that has just merged included (the release notes: "abilities
/// that count mutations include the mutation that caused the trigger"). Huntmaster Liger's and Illuna's "for each time it
/// has mutated" count it.
pub fn times_mutated(q: &dyn Query, permanent: ObjectId) -> u32 {
    q.history().all().iter().filter(|r| matches!(r.event, Event::Mutated { permanent: p, .. } if p == permanent)).count() as u32
}
