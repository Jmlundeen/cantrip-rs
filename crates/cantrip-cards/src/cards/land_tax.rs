use cantrip_api::*;
use cantrip_mechanics::{lands_you_control, library, select, triggers};
use cantrip_oracle as oracle;

const ASK: Question = Question::new("SearchLibrary", "search your library for up to three basic land cards?", "Reveal them and put them into your hand, then shuffle", "Search", "Don't search");

/// The intervening condition is checked at upkeep and again on resolution (CR 603.4).
fn opponent_has_more_lands(q: &dyn Query, asker: &Asker) -> bool {
    let ours = lands_you_control(q, asker.controller);
    q.state().opponents(asker.controller).any(|opponent| lands_you_control(q, opponent) > ours)
}

/// A declined or forbidden search does not shuffle; a search for zero cards does (CR 701.23, 701.24).
fn search(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if library::ask_may_search(g, item.controller, item.object, &ASK)? {
        library::find_to_hand(g, item.controller, item.object, select::basic_land_cards(select::up_to(3)), true)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(triggers::YOUR_UPKEEP, search, "At the beginning of your upkeep, if an opponent controls more lands than you, you may search your library for up to three basic land cards, reveal them, put them into your hand, then shuffle.")
        .condition(opponent_has_more_lands)],
    ..oracle::LAND_TAX
};
