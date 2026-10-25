//! Playing the card a hideaway permanent exiled (CR 702.75a, 608.2g).

use cantrip_api::*;

/// "You may play the exiled card without paying its mana cost" as the ability `item` resolves (CR 118.9, 305.1, 608.2g): the
/// card found by the exile link off the ability's source as it was, by its text (CR 607.2a). A land is played then or not at all; a spell is cast with its timing
/// waived. The controller is asked first, and nothing is asked when the card could not be played at all. Shelldock Isle,
/// Collector's Cage and Windbrisk Heights print it.
pub fn play_exiled_card(g: &mut dyn Game, item: &StackItem, you: PlayerId) -> Result<(), Suspend> {
    let Some(&card) = g.exiled_with(g.linked(item)).first() else { return Ok(()) };
    if g.could_play_land_during_resolution(card) {
        if g.ask(you, RequestKind::May { source: card, purpose: Purpose::Ask(&PLAY_EXILED_LAND) })? == Answer::Yes {
            g.play_land_during_resolution(card)?;
        }
    } else {
        crate::effects::may_cast_without_paying(g, you, card)?;
    }
    Ok(())
}

/// What playing an exiled land asks: a land played this way is the turn's land play (the hideaway rulings).
const PLAY_EXILED_LAND: Question = Question::new(
    "PlayLandDuringResolution",
    "Play {source}?",
    "It uses your land play this turn. If you don't, it stays in exile",
    "Play it",
    "Don't play",
);
