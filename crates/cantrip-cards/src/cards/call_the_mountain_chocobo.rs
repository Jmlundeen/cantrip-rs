use cantrip_api::*;
use cantrip_mechanics::{flashback, library, select, tokens};
use cantrip_oracle as oracle;

/// "A Mountain card" (CR 205.3i, 305.6): a card with the subtype Mountain, basic or not.
const MOUNTAIN_CARD: select::Selection = select::Selection::of(select::up_to(1), &[select::subtype_card(Subtype::Mountain)]).called("Mountain");

/// "Search your library for a Mountain card, reveal it, put it into your hand, then shuffle. Create a 2/2 green Bird creature token
/// with 'Whenever a land you control enters, this token gets +1/+0 until end of turn.'" (CR 701.23, 701.20a, 701.24a): the search
/// may find nothing, the library is shuffled either way, and the token is made all the same.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    library::find_to_hand(g, item.controller, item.object, MOUNTAIN_CARD, true)?;
    tokens::create_a_landfall_bird(g, item)
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], effect),
    ways: &[flashback::way(cost("{5}{R}"), "Flashback {5}{R}")],
    ..oracle::CALL_THE_MOUNTAIN_CHOCOBO
};

pub static EXTRA: &[&CardDef] = &[&tokens::LANDFALL_BIRD];
