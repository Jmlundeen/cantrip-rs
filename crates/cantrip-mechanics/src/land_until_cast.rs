//! "{N}, Exile this card from your hand: Target land gains "{T}: Add ..." until this card is cast from exile. You may cast
//! this card for as long as it remains exiled." Emrakul, the Exigent Doom prints it, and so do Streets of New Capenna's
//! Glamorous Outlaw, Masked Bandits, Rakish Revelers, Shattered Seraph and Spara's Adjudicators, word for word but for the
//! cost and the mana.
//!
//! A card with it lists `ability::<N>(cost, text)` among its `abilities`, and the mana ability the land gains as its `N`th
//! `delayed` ability.

use crate::targets;
use cantrip_api::*;

/// The land gains the card's `N`th delayed ability until the card the cost exiled is cast from exile (CR 611.2a, 400.7):
/// cast any other way out of exile, or put anywhere else, it is a new object and the land keeps the ability (the cards'
/// rulings). Then the ability's controller may cast that card for as long as it stays in exile (CR 601.3).
fn grant<const N: u8>(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    // The card in exile is what the cost made of this one (CR 400.7). If it has already left exile, it can never be cast
    // from there as this card, and the land has the ability for as long as it stays.
    let exiled = g.obj(item.object).became.unwrap_or(item.object);
    if let Some(land) = targets::object(g, item, 0) {
        let card = g.obj(item.object).card;
        g.add_effect(item.object, land, EffectKind::AddAbility(AbilityRef::delayed(card, N)), Duration::UntilCastFromExile(exiled));
    }
    if let Some(card) = g.found_after_move(item.object) {
        g.allow_play(card, item.controller, PlayGrant::CAST);
    }
    Ok(())
}

/// The ability, activated from the hand (CR 113.6d) for `cost`, which exiles the card along with what else it costs.
pub const fn ability<const N: u8>(cost: Cost, text: &'static str) -> AbilityDef {
    cantrip_api::ability(cost.and(EXILE_THIS), grant::<N>, text).targets(&[targets::LAND]).from(ZoneKind::Hand)
}
