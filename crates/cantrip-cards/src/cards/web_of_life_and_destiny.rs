use cantrip_api::*;
use cantrip_mechanics::{library, select, triggers::BEGINNING_OF_YOUR_COMBAT};
use cantrip_oracle as oracle;

/// "Look at the top five cards of your library. You may put a creature card from among them onto the battlefield. Put the rest on
/// the bottom of your library in a random order." (CR 608.2c, 401.4): the five cards are looked at (a smaller library shows what
/// it has, CR 609.3), one creature card of any mana value may be chosen to enter, and the others still in the library go to the bottom.
fn look_for_a_creature(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    library::look_at_top_onto_battlefield_select(g, item.controller, item.object, 5, select::CREATURE_CARD, false, library::Rest::BottomRandom)?;
    Ok(())
}

// Convoke is the core's, read from the card's JSON.
pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        BEGINNING_OF_YOUR_COMBAT,
        look_for_a_creature,
        "At the beginning of combat on your turn, look at the top five cards of your library. You may put a creature card from among them onto the battlefield. Put the rest on the bottom of your library in a random order.",
    )],
    ..oracle::WEB_OF_LIFE_AND_DESTINY
};
