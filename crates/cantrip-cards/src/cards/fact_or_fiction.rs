use cantrip_api::*;
use cantrip_mechanics::{choose, top_of_library};
use cantrip_oracle as oracle;

/// "Reveal the top five cards of your library. An opponent separates those cards into two piles. Put one pile into your
/// hand and the other into your graveyard." Reveal to every player (CR 701.20a); form piles without moving the cards,
/// either possibly empty (CR 700.3a, 700.3c-d). Follow the instructions in order and choose the opponent and kept pile
/// during resolution (CR 608.2c-d), without targeting the opponent (CR 115.10a). A caster always has an opponent here: with
/// every opponent gone the game is over (CR 104.2a), so "no separator" is unreachable, and `choose::separate` merely keeps
/// it from crashing. A library with fewer than five cards shows what it has (CR 609.3). The piles go to hand and graveyard
/// together, each card seen by all as it moves, since the reveal lasts through the instruction about them (CR 701.20a).
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
    // One instruction about the revealed cards: both piles move at once, still revealed as they move (CR 701.20a, 603.10a).
    let moves: Vec<(ObjectId, Zone)> = keep.iter().map(|&c| (c, Zone::hand(g.obj(c).owner)))
        .chain(rest.iter().map(|&c| (c, Zone::graveyard(g.obj(c).owner))))
        .collect();
    g.move_together_revealed(&moves, ZoneReason::Other);
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[], effect), ..oracle::FACT_OR_FICTION };
