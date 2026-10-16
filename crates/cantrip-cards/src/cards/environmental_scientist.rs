use cantrip_api::*;
use cantrip_mechanics::lands::may_search_basic_land_to_hand;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

/// "You may search your library for a basic land card, reveal it, put it into your hand, then shuffle" (CR 701.23).
fn search(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    may_search_basic_land_to_hand(g, item.controller, item.object)
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_ENTERS, search, "When this creature enters, you may search your library for a basic land card, reveal it, put it into your hand, then shuffle.")],
    ..oracle::ENVIRONMENTAL_SCIENTIST
};
