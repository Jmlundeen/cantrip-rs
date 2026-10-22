use cantrip_api::*;
use cantrip_mechanics::triggers::{THIS_DIES, THIS_ENTERS};
use cantrip_mechanics::{effects, lands};
use cantrip_oracle as oracle;

/// "When this creature enters, you may search your library for a basic land card, put that card onto the battlefield
/// tapped, then shuffle" (CR 701.23): asked first whether to, and declining skips the search and the shuffle.
fn fetch_a_basic(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    lands::may_fetch_basic(g, item.controller, item.object, true)
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            THIS_ENTERS,
            fetch_a_basic,
            "When this creature enters, you may search your library for a basic land card, put that card onto the battlefield tapped, then shuffle.",
        ),
        // CR 700.4: "dies" is put into a graveyard from the battlefield; the ability looks back at the creature (CR 603.10a).
        triggered(THIS_DIES, |g, item| effects::draw(g, item, 1), "When this creature dies, draw a card."),
    ],
    ..oracle::SIMULACRUM_SHAPER
};
