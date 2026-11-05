use cantrip_api::*;
use cantrip_mechanics::{harmonize, lands};
use cantrip_oracle as oracle;

/// "Search your library for a basic land card, put it onto the battlefield tapped, then shuffle" (CR 701.23): the player finds
/// one or none.
fn search(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    lands::search_basic_land_to_battlefield(g, item.controller, item.object, true)
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], search),
    ways: &[harmonize::way(mana("{4}{G}"), "Harmonize {4}{G}")],
    ..oracle::ROAMERS_ROUTINE
};
