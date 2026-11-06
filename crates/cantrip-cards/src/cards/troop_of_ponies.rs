use cantrip_api::*;
use cantrip_mechanics::{library, select};
use cantrip_oracle as oracle;

fn fetch_two_basic_lands(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    library::search_two_one_onto_battlefield_tapped_other_into_hand(g, item, select::basic_land_cards(select::up_to(2)))
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(
        cost("{2}, {T}").and(SACRIFICE_THIS),
        fetch_two_basic_lands,
        "{2}, {T}, Sacrifice this creature: Search your library for up to two basic land cards, reveal them, put one onto the battlefield tapped and the other into your hand, then shuffle.",
    )],
    ..oracle::TROOP_OF_PONIES
};
