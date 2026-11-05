use cantrip_api::*;
use cantrip_mechanics::{lands, library, permanents_with_subtype_you_control, select, triggers};
use cantrip_oracle as oracle;

/// What the search asks: the shared top-of-library fetch's question, with the Dragon branch worded too.
const FETCH_BASIC_TO_TOP_OR_BATTLEFIELD: Question = Question::new(
    "SearchLibrary",
    "search your library for a basic land card?",
    "Reveal it, then shuffle and put that card on top of your library, or onto the battlefield tapped instead if you control a Dragon",
    "Search",
    "Don't search",
);

/// "You may search your library for a basic land card, reveal it, then shuffle and put that card on top. If you control a Dragon,
/// put that card onto the battlefield tapped instead." (CR 701.23, 701.20, 608.2c): asked first, and declining skips the search and the
/// shuffle. Whether the controller has a Dragon is read as the ability resolves ("Dragon" a word of this card's text, CR 612),
/// the Sentinel counted if it is one itself; the land is found, revealed and the library shuffled either way, only where the
/// card goes differs. The Dragon branch puts it onto the battlefield tapped, so it is a land entering, not a card played (CR 305.4).
fn search(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let (you, source) = (item.controller, item.object);
    if !library::ask_may_search(g, you, source, &FETCH_BASIC_TO_TOP_OR_BATTLEFIELD)? {
        return Ok(());
    }
    let dragon = g.words(source).subtype(Subtype::Dragon);
    if permanents_with_subtype_you_control(g, you, dragon) > 0 {
        let found = library::search(g, you, you, source, SearchDestination::BattlefieldTapped, select::BASIC_LAND_CARD)?;
        g.reveal(you, &found);
        g.put_all_onto_battlefield_as(&found.into_iter().map(|card| (card, you, true)).collect::<Vec<_>>())?;
        g.shuffle(you);
    } else {
        lands::search_basic_land_to_top(g, you, source)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::THIS_ENTERS,
        search,
        "When this creature enters, you may search your library for a basic land card, reveal it, then shuffle and put that card on top. If you control a Dragon, put that card onto the battlefield tapped instead.",
    )],
    ..oracle::EMBERMOUTH_SENTINEL
};
