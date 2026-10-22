use cantrip_api::*;
use cantrip_mechanics::{choose, top_of_library};
use cantrip_oracle as oracle;

/// "Reveal the top five cards of your library. An opponent separates those cards into two piles. Put one pile into your
/// hand and the other into your graveyard." Reveal to every player (CR 701.20a); form piles without moving the cards,
/// either possibly empty (CR 700.3a, 700.3c-d). Follow the instructions in order and choose the opponent and kept pile
/// during resolution (CR 608.2c-d), without targeting the opponent (CR 115.10a). With no opponent, everything opens in the
/// second pile; a library with fewer than five cards shows what it has (CR 609.3).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let revealed = top_of_library(g, you, 5);
    if revealed.is_empty() {
        return Ok(());
    }
    g.reveal(you, &revealed);
    let opponents: Vec<PlayerId> = g.state().opponents(you).collect();
    let separator = choose::player(g, you, item.object, opponents, "separate the revealed cards into two piles", 5)?;
    let (first, second) = choose::separate(g, separator, item.object, revealed)?;
    let (keep, rest) = choose::pile(g, you, item.object, first, second)?;
    g.put_into_hand_and_graveyard(&keep, &rest);
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[], effect), ..oracle::FACT_OR_FICTION };
