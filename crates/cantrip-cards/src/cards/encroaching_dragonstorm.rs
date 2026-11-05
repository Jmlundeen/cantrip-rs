use cantrip_api::*;
use cantrip_mechanics::triggers::{DRAGON_YOU_CONTROL_ENTERS, THIS_ENTERS};
use cantrip_mechanics::{effects, library, select};
use cantrip_oracle as oracle;

/// "Search your library for up to two basic land cards, put them onto the battlefield tapped, then shuffle" (CR 701.23):
/// fewer may be found, or none, and the library is shuffled either way.
fn search(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    library::find_to_battlefield(g, item.controller, item.object, select::basic_land_cards(select::up_to(2)), true)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, search, "When this enchantment enters, search your library for up to two basic land cards, put them onto the battlefield tapped, then shuffle."),
        triggered(DRAGON_YOU_CONTROL_ENTERS, effects::return_this_to_hand, "When a Dragon you control enters, return this enchantment to its owner's hand."),
    ],
    ..oracle::ENCROACHING_DRAGONSTORM
};
