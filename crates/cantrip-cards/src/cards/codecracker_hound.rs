use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_mechanics::{library, warp};
use cantrip_oracle as oracle;

/// "Look at the top two cards of your library. Put one into your hand and the other into your graveyard" (CR 701.20a): with fewer than two
/// cards the one there goes to the hand. Warp {2}{U} is the shared mechanic (CR 702.185).
pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        THIS_ENTERS,
        |g, item| library::look_at_top_take_one_rest_to_graveyard(g, item.controller, item.object, 2),
        "When this creature enters, look at the top two cards of your library. Put one into your hand and the other into your graveyard.",
    )],
    ways: &[warp::way(mana("{2}{U}"), "Warp {2}{U}")],
    end_step_riders: warp::RIDERS,
    delayed: &[warp::EXILE],
    ..oracle::CODECRACKER_HOUND
};
