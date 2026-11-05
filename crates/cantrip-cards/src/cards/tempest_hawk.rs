use cantrip_api::*;
use cantrip_mechanics::select::{self, up_to};
use cantrip_mechanics::{library, triggers};
use cantrip_oracle as oracle;

const ASK: Question = Question::new(
    "SearchLibrary",
    "search your library for a card named Tempest Hawk?",
    "Reveal it and put it into your hand, then shuffle",
    "Search",
    "Don't search",
);

/// "You may search your library for a card named Tempest Hawk, reveal it, put it into your hand, then shuffle" (CR 701.23):
/// asked first; declining skips the search and the shuffle. "A deck can have any number of cards named Tempest Hawk" is a
/// deck-construction rule (CR 100.2a), not an ability: it is in no game and no line of code here.
fn search(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let player = item.controller;
    if !library::ask_may_search(g, player, item.object, &ASK)? {
        return Ok(());
    }
    let rule = select::cards(up_to(1)).with(select::named(g.card_id(&CARD)));
    library::find_to_hand(g, player, item.object, rule, true)?;
    Ok(())
}

// Flying is the core's, read from the card's JSON.
pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::DEALS_COMBAT_DAMAGE_TO_PLAYER,
        search,
        "Whenever this creature deals combat damage to a player, you may search your library for a card named Tempest Hawk, reveal it, put it into your hand, then shuffle.",
    )],
    ..oracle::TEMPEST_HAWK
};
