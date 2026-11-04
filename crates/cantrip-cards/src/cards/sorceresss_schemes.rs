use cantrip_api::*;
use cantrip_mechanics::{flashback, targets};
use cantrip_oracle as oracle;

/// "Target instant or sorcery card from your graveyard or exiled card with flashback you own" (CR 115.1): one target, from
/// either zone.
fn instant_or_sorcery_in_your_graveyard_or_exiled_with_flashback(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    (targets::INSTANT_OR_SORCERY_IN_YOUR_GRAVEYARD.filter)(q, asker, t) || (targets::EXILED_CARD_WITH_FLASHBACK_YOU_OWN.filter)(q, asker, t)
}

/// "Return target instant or sorcery card from your graveyard or exiled card with flashback you own to your hand. Add {R}."
/// A spell whose only target is illegal does not resolve (CR 608.2b), so no {R} is added then.
fn return_it_and_add_red(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(card) = targets::object(g, item, 0) else { return Ok(()) };
    g.return_to_hand(&[card]);
    g.add_mana(item.controller, Mana::R, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[target(instant_or_sorcery_in_your_graveyard_or_exiled_with_flashback)], return_it_and_add_red),
    ways: &[flashback::way(cost("{4}{R}"), "Flashback {4}{R}")],
    ..oracle::SORCERESSS_SCHEMES
};
