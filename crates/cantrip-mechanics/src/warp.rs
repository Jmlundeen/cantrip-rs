//! Warp (CR 702.185a): "You may cast this card from your hand for its warp cost. Exile this permanent at the beginning
//! of the next end step, then you may cast it from exile on a later turn." Quantum Riddler, Pinnacle Emissary and Timeline Culler print it.
//!
//! A card with warp lists `way(cost, text)` as its only alternative cost, `EXILE` among its `delayed` abilities as the
//! first, and `RIDERS` as its `end_step_riders`: the way casts it, and the core creates the delayed ability without
//! the stack as the spell is cast for it (CR 608.3g, 603.7a), about the permanent the spell becomes. A permanent that
//! gives warp lists `granted(cost)` among its granted ways and `EXILE` as its first `delayed` ability (Tannuk, Steadfast
//! Second).

use cantrip_api::*;

/// The alternative cost of warp: cast from the hand for `cost` (CR 118.9), and only from the hand: "You may cast this card
/// from your hand for its warp cost" (CR 702.185a), so a permission to cast it from exile or a graveyard does not open it.
pub const fn way(cost: ManaCost, text: &'static str) -> WayDef {
    way_with_cost(Cost::mana(cost), text)
}

/// `way` for a warp cost that is more than mana ("Warp - {B}, Pay 2 life", CR 118.8): Timeline Culler.
pub const fn way_with_cost(cost: Cost, text: &'static str) -> WayDef {
    cantrip_api::way(cost, ZoneKind::Hand, text).kind(WayKind::Warp).only_from()
}

/// Warp's delayed ability (CR 702.185a): the way is the card's first and this its first delayed ability.
pub const RIDERS: &[(u8, u8)] = &[(1, 0)];

/// "Was cast for its warp cost" (CR 702.185a): the permanent (or spell) was cast through a way marked as warp
/// (`WayKind::Warp`), the card's own (`way`) or one a permanent gave it (`granted`: Tannuk, Steadfast Second). Another
/// keyword's way is not warp's, and a permanent that was never a spell cast (a token, one put onto the battlefield) was not
/// cast for it; a copy of a spell copies the alternative cost the original was cast for (CR 707.10). Read from the spell's
/// `Cast` record (`sneak::cast_through_turn`). Full Bore asks it.
pub fn was_cast_for_warp(q: &dyn Query, id: ObjectId) -> bool {
    crate::sneak::cast_through_turn(q, id, WayKind::Warp).is_some()
}

/// "A spell was warped this turn" (CR 702.185c): a spell, any player's, was cast for its warp cost this turn, its own or a
/// granted one. Elegy Acolyte's void condition asks it.
pub fn spell_was_warped_this_turn(q: &dyn Query) -> bool {
    q.history().casts().any(|c| crate::history::way_kind_of(q, *c) == Some(WayKind::Warp))
}

/// Warp a permanent gives cards in its controller's hand (CR 702.185a, 113.6, 604): "artifact cards and red creature cards
/// in your hand have warp {2}{R}" (Tannuk, Steadfast Second). `cost` is the warp cost of each card, or none for one without
/// it. The granting card lists `EXILE` as its first `delayed` ability: the core sets it up as the spell is cast by this
/// way (`GrantedWay::end_step_rider`), about the permanent the spell becomes, as it does for a card's own warp.
pub const fn granted(cost: GrantedCost) -> GrantedWay {
    GrantedWay { text: "Warp", cost, from: ZoneKind::Hand, kind: WayKind::Warp, end_step_rider: Some(0), ..GrantedWay::CAST }
}

/// "A warped card in exile" (CR 702.185b): one that was exiled by the delayed triggered ability a warp ability created, asked
/// of the history - the move that put this object into exile (CR 400.7: a card that left exile and came back is a new
/// object, not warped). That ability is the delayed one a warp way rides with (`RIDERS`), of the card that was cast for it,
/// so a card a merged permanent became is warped with the rest (CR 730.3c), and a card exiled any other way is not. Close
/// Encounter asks it.
pub fn warped_card_in_exile(q: &dyn Query, id: ObjectId) -> bool {
    if !q.obj(id).zone.is(ZoneKind::Exile) {
        return false;
    }
    let Some(entry) = q.history().all().iter().rev().find(|r| matches!(r.event, Event::ZoneChange { new, .. } if new == id)) else { return false };
    let Cause::Ability { ability, .. } = entry.cause else { return false };
    if !ability.is_delayed() {
        return false;
    }
    let card = q.card(ability.card);
    let delayed = ability.index - AbilityRef::DELAYED;
    // Its own warp's delayed ability, or the one a warp it grants names (Tannuk, Steadfast Second).
    card.end_step_riders.iter().any(|&(way, d)| d == delayed && card.ways.get((way as usize).wrapping_sub(1)).is_some_and(|w| w.kind == WayKind::Warp))
        || card.granted_ways.iter().any(|w| w.kind == WayKind::Warp && w.end_step_rider.is_some_and(|d| d == delayed))
}

/// Exile the permanent the warped spell became, if it is still on the battlefield, and let its owner cast it from
/// exile on a later turn, for its mana cost: warp's cost is a way from the hand only, so it is not offered again.
fn exile_for_later(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Target::Object(spell) = item.subject else { return Ok(()) };
    // Only the permanent the spell became, one move on: once that permanent leaves, what it becomes is a new object
    // (CR 400.7) the ability no longer finds, even back on the battlefield.
    let Some(now) = g.obj(spell).became else { return Ok(()) };
    if !is_live(g, now) || !g.obj(now).zone.is(ZoneKind::Battlefield) {
        return Ok(());
    }
    // Each card a merged permanent became may be cast (CR 730.3c: the same actions are taken upon each).
    for exiled in g.exile_all(&[now]) {
        if g.obj(exiled).zone.is(ZoneKind::Exile) {
            let owner = g.obj(exiled).owner;
            g.allow_play(exiled, owner, PlayGrant { later_turns: true, ..PlayGrant::CAST });
        }
    }
    Ok(())
}

pub const EXILE: AbilityDef = delayed_trigger(exile_for_later, "Exile this permanent at the beginning of the next end step, then you may cast it from exile on a later turn.");
