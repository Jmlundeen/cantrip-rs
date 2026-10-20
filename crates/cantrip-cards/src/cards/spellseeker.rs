use cantrip_api::*;
use cantrip_mechanics::library::{ask_may_search, Find, search_to_hand};
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

const ASK: Question = Question::new("SearchLibrary", "search your library for an instant or sorcery card with mana value 2 or less?", "Reveal it and put it into your hand, then shuffle", "Search", "Don't search");

/// "You may search your library for an instant or sorcery card with mana value 2 or less, reveal it, put it into your
/// hand, then shuffle" (CR 701.23): asked first; declining skips the search and the shuffle.
fn search(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if !ask_may_search(g, item.controller, item.object, &ASK)? {
        return Ok(());
    }
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
