use cantrip_api::*;
use cantrip_mechanics::library::{search_to_hand, Find};
use cantrip_oracle as oracle;

/// Search your library for a land card, reveal it, put it into your hand, then shuffle.
fn search(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    search_to_hand(g, item.controller, item.object, Find::ONE.revealed(), |q, c| q.chars(c).is(CardType::Land))?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(cost("{2}, {T}").and(SACRIFICE_THIS), search, "{2}, {T}, Sacrifice Expedition Map: Search your library for a land card, reveal it, put it into your hand, then shuffle.")],
    ..oracle::EXPEDITION_MAP
};
