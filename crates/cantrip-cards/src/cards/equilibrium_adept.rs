use cantrip_api::*;
use cantrip_mechanics::library::exile_top_card_play_until_end_of_next_turn;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_mechanics::{effects, flurry};
use cantrip_oracle as oracle;

/// The enter trigger is Gundabad Opportunist's and Crimson Operative's; flurry (an ability word, CR 207.2c) gives this
/// creature double strike (CR 702.4) until end of turn.
pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, exile_top_card_play_until_end_of_next_turn, "When this creature enters, exile the top card of your library. Until the end of your next turn, you may play that card."),
        flurry::trigger(
            "Flurry \u{2014} Whenever you cast your second spell each turn, this creature gains double strike until end of turn.",
            |g, item| effects::this_gains_until_end_of_turn(g, item, Keyword::DoubleStrike),
        ),
    ],
    ..oracle::EQUILIBRIUM_ADEPT
};
