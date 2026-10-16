//! Warp (CR 702.185a): "You may cast this card from your hand for its warp cost. Exile this permanent at the beginning
//! of the next end step, then you may cast it from exile on a later turn." Quantum Riddler and Pinnacle Emissary print it.
//!
//! A card with warp lists `way(cost, text)` as its only alternative cost, `EXILE` among its `delayed` abilities as the
//! first, and `RIDERS` as its `end_step_riders`: the way casts it, and the core creates the delayed ability without
//! the stack as the spell is cast for it (CR 608.3g, 603.7a), about the permanent the spell becomes.

use cantrip_api::*;

/// The alternative cost of warp: cast from the hand for `cost` (CR 118.9).
pub const fn way(cost: ManaCost, text: &'static str) -> WayDef {
    WayDef { text, cost: Cost::mana(cost), from: ZoneKind::Hand, exile_after: false, condition: None, instant_speed: false }
}

/// Warp's delayed ability (CR 702.185a): the way is the card's first and this its first delayed ability.
pub const RIDERS: &[(u8, u8)] = &[(1, 0)];

/// Exile the permanent the warped spell became, if it is still on the battlefield, and let its owner cast it from
/// exile on a later turn, for its mana cost: warp's cost is a way from the hand only, so it is not offered again.
fn exile_for_later(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Target::Object(spell) = item.subject else { return Ok(()) };
    let mut now = spell;
    while let Some(next) = g.obj(now).became {
        now = next;
    }
    if !is_live(g, now) || !g.obj(now).zone.is(ZoneKind::Battlefield) {
        return Ok(());
    }
    let owner = g.obj(now).owner;
    let exiled = g.exile(now);
    if g.obj(exiled).zone.is(ZoneKind::Exile) {
        g.allow_play(exiled, owner, PlayGrant { later_turns: true, ..PlayGrant::CAST });
    }
    Ok(())
}

pub const EXILE: AbilityDef = delayed_trigger(exile_for_later, "Exile this permanent at the beginning of the next end step, then you may cast it from exile on a later turn.");
