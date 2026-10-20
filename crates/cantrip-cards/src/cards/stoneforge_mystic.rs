use cantrip_api::*;
use cantrip_mechanics::choose;
use cantrip_mechanics::library::{ask_may_search, Find, search_to_hand};
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

fn equipment(q: &dyn Query, id: ObjectId) -> bool {
    q.chars(id).has_subtype(Subtype::Equipment)
}

const ASK: Question = Question::new("SearchLibrary", "search your library for an Equipment card?", "Reveal it and put it into your hand, then shuffle", "Search", "Don't search");

/// "You may search your library for an Equipment card, reveal it, put it into your hand, then shuffle" (CR 701.23): asked
/// first; declining skips the search and the shuffle.
fn search(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if !ask_may_search(g, item.controller, item.object, &ASK)? {
        return Ok(());
    }
    search_to_hand(g, item.controller, item.object, Find::ONE.revealed(), equipment)?;
    Ok(())
}

/// "You may put an Equipment card from your hand onto the battlefield": up to one card; nothing is asked when the hand
/// has none.
fn put_equipment_in(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let player = item.controller;
    let hand = g.state().zone(Zone::hand(player)).to_vec();
    let eligible: Vec<ObjectId> = hand.iter().copied().filter(|&c| equipment(g, c)).collect();
    if let Some(&card) = choose::cards_from(g, player, item.object, Purpose::PutOntoBattlefield, hand, eligible, 1, true)?.first() {
        g.put_onto_battlefield(card, player, false)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, search, "When Stoneforge Mystic enters, you may search your library for an Equipment card, reveal it, put it into your hand, then shuffle."),
        ability(cost("{1}{W}, {T}"), put_equipment_in, "{1}{W}, {T}: You may put an Equipment card from your hand onto the battlefield."),
    ],
    ..oracle::STONEFORGE_MYSTIC
};
