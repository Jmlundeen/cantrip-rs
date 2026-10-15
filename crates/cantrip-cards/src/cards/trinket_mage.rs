use cantrip_api::*;
use cantrip_mechanics::effects::ask_may;
use cantrip_mechanics::library::{Find, search_to_hand};
use cantrip_mechanics::triggers::this_enters;
use cantrip_oracle as oracle;

/// You may search your library for an artifact card with mana value 1 or less, reveal it, put it into your hand, then
/// shuffle (CR 701.23). The "may" is asked first; declining skips the search and the shuffle.
fn search(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let player = item.controller;
    if !ask_may(g, player, item.object, Purpose::Optional)? {
        return Ok(());
    }
    search_to_hand(g, player, item.object, Find::ONE.revealed(), |q, c| {
        let chars = q.chars(c);
        chars.is(CardType::Artifact) && q.mana_value(c) <= 1
    })?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[AbilityDef::Triggered(TriggeredDef {
        text: "When Trinket Mage enters, you may search your library for an artifact card with mana value 1 or less, reveal that card, put it into your hand, then shuffle.",
        listens: EventMask::ZONE_CHANGE,
        matches: this_enters,
        effect: search,
        ..TriggeredDef::BLANK
    })],
    ..oracle::TRINKET_MAGE
};
