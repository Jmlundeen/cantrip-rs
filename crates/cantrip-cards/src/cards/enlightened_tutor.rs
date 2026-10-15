use cantrip_api::*;
use cantrip_mechanics::library::{search_to_top, Find};
use cantrip_oracle as oracle;

/// Search your library for an artifact or enchantment card, reveal it, then shuffle and put that card on top. The card is
/// revealed while it is still in the library; it never changes zones. The whole library is shuffled and the card is
/// then placed on top, which leaves the rest in the same distribution as shuffling it without the card.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    search_to_top(g, item.controller, item.object, Find::ONE.revealed(), |q, c| {
        let chars = q.chars(c);
        chars.is(CardType::Artifact) || chars.is(CardType::Enchantment)
    })?;
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: Some(SpellDef { targets: &[], effect }), ..oracle::ENLIGHTENED_TUTOR };
