//! Playing the card a hideaway permanent exiled (CR 702.75a, 608.2g).

use cantrip_api::*;

/// "You may play the exiled card without paying its mana cost" as the ability of `source` resolves (CR 118.9, 305.1, 608.2g): the
/// card found by the exile link off `source` as it was. A land is played then or not at all; a spell is cast with its timing
/// waived. The controller is asked first, and nothing is asked when the card could not be played at all. Shelldock Isle,
/// Collector's Cage and Windbrisk Heights print it.
pub fn play_exiled_card(g: &mut dyn Game, source: ObjectId, you: PlayerId) -> Result<(), Suspend> {
    let Some(&card) = g.exiled_with(source).first() else { return Ok(()) };
    if g.could_play_land_during_resolution(card) {
        if g.ask(you, RequestKind::May { source: card, purpose: Purpose::Optional })? == Answer::Yes {
            g.play_land_during_resolution(card)?;
        }
    } else if g.could_cast_during_resolution(card)
        && g.ask(you, RequestKind::May { source: card, purpose: Purpose::CastFree })? == Answer::Yes
    {
        g.cast_during_resolution(card, Some(ManaCost::ZERO))?;
    }
    Ok(())
}
