use cantrip_api::*;
use cantrip_mechanics::library::{Find, search_to_hand};
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

/// "You may search your library for an instant or sorcery card with mana value 2 or less, reveal it, put it into your
/// hand, then shuffle." Declining the search ("you may") is choosing no card.
fn search(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    search_to_hand(g, item.controller, item.object, Find::ONE.revealed(), |q, c| {
        let chars = q.chars(c);
        (chars.is(CardType::Instant) || chars.is(CardType::Sorcery)) && q.mana_value(c) <= 2
    })?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_ENTERS, search, "When Spellseeker enters, you may search your library for an instant or sorcery card with mana value 2 or less, reveal it, put it into your hand, then shuffle.")],
    ..oracle::SPELLSEEKER
};
