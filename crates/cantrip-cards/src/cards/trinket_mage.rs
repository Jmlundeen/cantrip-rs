use cantrip_api::*;
use cantrip_mechanics::library::{ask_may_search, find_to_hand};
use cantrip_mechanics::select::{self, up_to};
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

const ASK: Question = Question::new("SearchLibrary", "search your library for an artifact card with mana value 1 or less?", "Reveal it and put it into your hand, then shuffle", "Search", "Don't search");

/// You may search your library for an artifact card with mana value 1 or less, reveal it, put it into your hand, then
/// shuffle (CR 701.23). The "may" is asked first; declining skips the search and the shuffle.
fn search(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let player = item.controller;
    if !ask_may_search(g, player, item.object, &ASK)? {
        return Ok(());
    }
    find_to_hand(g, player, item.object, select::artifact_cards(up_to(1)).with(select::mana_value_at_most(1)), true)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_ENTERS, search, "When Trinket Mage enters, you may search your library for an artifact card with mana value 1 or less, reveal that card, put it into your hand, then shuffle.")],
    ..oracle::TRINKET_MAGE
};
