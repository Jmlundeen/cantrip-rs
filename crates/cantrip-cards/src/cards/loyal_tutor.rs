use cantrip_api::*;
use cantrip_mechanics::library::{search_to_top, Find};
use cantrip_oracle as oracle;

/// "Search your library for a planeswalker card, reveal it, then shuffle and put that card on top." (CR 701.23)
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    search_to_top(g, item.controller, item.object, Find::ONE.revealed(), |q, card| {
        q.chars(card).is(CardType::Planeswalker)
    })?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], effect),
    ..oracle::LOYAL_TUTOR
};
