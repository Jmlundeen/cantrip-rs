use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_mechanics::{effects, lands};
use cantrip_oracle as oracle;

/// "When Thrór's Map enters, search your library for a basic land card, reveal it, put it into your hand, then shuffle"
/// (CR 701.23): not optional, though the player may find nothing, and the library is shuffled either way.
fn fetch_a_basic(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    lands::search_basic_land_to_hand(g, item.controller, item.object)
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            THIS_ENTERS,
            fetch_a_basic,
            "When Thr\u{f3}r's Map enters, search your library for a basic land card, reveal it, put it into your hand, then shuffle.",
        ),
        ability(cost("{2}, {T}"), effects::loot, "{2}, {T}: Draw a card, then discard a card."),
    ],
    ..oracle::THR_RS_MAP
};
