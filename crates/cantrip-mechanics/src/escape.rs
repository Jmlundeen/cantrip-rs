//! Escape--[cost], Exile N other cards from your graveyard (CR 702.138a): "You may cast this card from your graveyard
//! by paying [cost] rather than paying its mana cost" (CR 118.9). Bloodbraid Challenger, From the Catacombs, Phlage,
//! Titan of Fire's Fury and Uro, Titan of Nature's Wrath print it; Underworld Breach grants it to every nonland card in
//! its controller's graveyard (CR 113.6).
//!
//! A card with escape lists `way(cost, exile, text)` as its first way; "unless it escaped" reads the way the permanent
//! was cast for (CR 702.138b), which `sacrifice_unless_escaped` does for Phlage and Uro.

use crate::triggers::THIS_ENTERS;
use cantrip_api::*;

/// The escape cost: `cost` and the exiling of `exile` other cards from the graveyard (CR 702.138a).
pub const fn cost(cost: ManaCost, exile: u8) -> Cost {
    Cost { mana: cost, exile_from_graveyard: exile, ..Cost::NONE }
}

/// The alternative cost of escape: cast from the graveyard for `cost`, exiling `exile` other cards from it. `text` is the
/// way's full text.
pub const fn way(mana: ManaCost, exile: u8, text: &'static str) -> WayDef {
    WayDef { text, cost: cost(mana, exile), from: ZoneKind::Graveyard, exile_after: false, condition: None, instant_speed: false }
}

/// Escape whose exile is "any number of other cards from your graveyard with `types` or more card types among them"
/// (CR 702.138a): the cards exiled are chosen as the cost is paid. `text` is the way's full text. Nethergoyf prints it.
pub const fn way_by_types(mana: ManaCost, types: u8, text: &'static str) -> WayDef {
    WayDef {
        text,
        cost: Cost { mana, variable: Some(VariableCost::ExileFromGraveyardWithTypes(types)), ..Cost::NONE },
        from: ZoneKind::Graveyard,
        exile_after: false,
        condition: None,
        instant_speed: false,
    }
}

/// "Sacrifice it unless it escaped": always triggers, and is judged as it resolves (CR 702.138b, 701.21a).
fn sacrifice_unless_escaped_effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if g.obj(item.object).cast_way != 1
        && g.is_on_battlefield(item.object)
        && g.obj(item.object).became.is_none()
        && g.controller(item.object) == item.controller
    {
        g.sacrifice(&[item.object]);
    }
    Ok(())
}

/// "When this enters, sacrifice it unless it escaped": `text` names the card.
pub const fn sacrifice_unless_escaped(text: &'static str) -> AbilityDef {
    triggered(THIS_ENTERS, sacrifice_unless_escaped_effect, text)
}
