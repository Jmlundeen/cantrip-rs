use cantrip_api::*;
use cantrip_mechanics::{library, select};
use cantrip_oracle as oracle;

/// "Search your library for up to two basic land cards, reveal those cards, put one onto the battlefield tapped and the other
/// into your hand, then shuffle." (CR 701.23, 701.20, 400.7).
fn search_library_up_two_basic(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    library::search_two_one_onto_battlefield_tapped_other_into_hand(g, item, select::basic_land_cards(select::up_to(2)))
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], search_library_up_two_basic),
    ..oracle::CULTIVATE
};
