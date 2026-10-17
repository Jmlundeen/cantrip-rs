use cantrip_api::*;
use cantrip_mechanics::library::{Find, instant_or_sorcery_card, search_to_top};
use cantrip_oracle as oracle;

/// Search your library for an instant or sorcery card, reveal it, then shuffle and put that card on top. The card is
/// revealed while it is still in the library; it never changes zones. The whole library is shuffled and the card is
/// then placed on top, which leaves the rest in the same distribution as shuffling it without the card.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let player = item.controller;
    search_to_top(g, player, item.object, Find::ONE.revealed(), instant_or_sorcery_card)?;
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[], effect), ..oracle::MYSTICAL_TUTOR };
