//! Web-slinging [cost] (CR 702.188a): "You may cast this spell by paying [cost] and returning a tapped creature you
//! control to its owner's hand rather than paying its mana cost." Spider-Sense, Spider-Man, Web-Slinger, Amazing
//! Spider-Girl, Spider-Man, Brooklyn Visionary and Spiders-Man, Heroic Horde print it.
//!
//! A card with it lists `way(cost, text)` among its `ways`: an alternative cost (CR 118.9) used from the hand, at the
//! spell's own timing, following CR 601.2b and 601.2f-h. The creature is chosen as the cost is paid, once the mana is
//! (`PermanentCost::tapped`): it must be tapped then, not before casting began, so one tapped for a mana ability while
//! paying the web-slinging cost may be the one returned (the 2025-09-19 ruling).

use cantrip_api::*;

use crate::targets::filters::creature;

/// The alternative cost of web-slinging: `cost` and a tapped creature you control returned to its owner's hand. `text`
/// is the way's text ("Web-slinging {U}"), which a scenario's `Ability:` matches and the client shows.
pub const fn way(cost: ManaCost, text: &'static str) -> WayDef {
    cantrip_api::way(Cost::mana(cost).and(return_tapped_to_hand(1, creature)), ZoneKind::Hand, text).kind(WayKind::WebSlinging)
}

/// Web-slinging a permanent gives to spells (CR 702.188a, 113.6, 604): "each legendary spell you cast that's one or more
/// colors has web-slinging {G}{W}{U}" (Amazing Spider-Man). `applies` says which spells have it; `mana` is its cost, with a
/// tapped creature the caster controls returned to its owner's hand, the cost a card's own `way` asks; and `text` is what the
/// menu calls it, as a card's own is called ("Web-slinging {G}{W}{U}"). Like a card's own web-slinging, which names no zone
/// (CR 118.9, 601.3), it is an alternative cost offered wherever the spell may be cast from: its hand, or a graveyard or exile
/// that a permission (Lurrus of the Dream-Den, plot) lets it be cast from. It is cast at the spell's own timing, and a spell
/// cast by it was cast using web-slinging (`WayKind::WebSlinging`).
pub const fn granted(applies: fn(&dyn Query, &Asker, ObjectId, PlayerId) -> bool, mana: ManaCost, text: &'static str) -> SpellAlternative {
    SpellAlternative {
        text,
        applies,
        cost: InsteadOfManaCost::Cost(Cost::mana(mana).and(return_tapped_to_hand(1, creature))),
        flash: false,
        kind: WayKind::WebSlinging,
    }
}

/// "Was cast using web-slinging" (CR 702.188a): the spell, or the permanent it became, was cast for a web-slinging cost
/// (`WayKind::WebSlinging`, set by `way` and `granted`): the card's own, or the one a permanent gave it (Amazing Spider-Man),
/// from whatever zone. Another way and a permanent that was not cast were not; a copy of a spell cast so copies its
/// alternative cost (CR 707.10) and was too. Spiders-Man, Heroic Horde and Scarlet Spider, Ben Reilly ask it.
pub fn was_cast_using_web_slinging(q: &dyn Query, id: ObjectId) -> bool {
    crate::sneak::cast_through_turn(q, id, WayKind::WebSlinging).is_some()
}
