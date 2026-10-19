use cantrip_api::*;
use cantrip_mechanics::library::{search_to_hand, Find};
use cantrip_mechanics::discard_at_random;
use cantrip_oracle as oracle;

/// "When this creature enters, search your library for a card, put it into your hand, shuffle, then discard a card at
/// random." (CR 701.23, 701.24a.) A search for "a card" must take one if the library has any (CR 701.23b).
fn enter_search_and_discard(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    search_to_hand(g, item.controller, item.object, Find::ANY, |_, _| true)?;
    discard_at_random(g, item.controller, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(cantrip_mechanics::triggers::THIS_ENTERS, enter_search_and_discard, "When this creature enters, search your library for a card, put it into your hand, shuffle, then discard a card at random.")],
    ..oracle::CURSE_MARRED_DEMON
};
