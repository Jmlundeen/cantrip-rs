use cantrip_api::*;
use cantrip_mechanics::{effects, triggers::THIS_ENTERS, warp};
use cantrip_oracle as oracle;

/// Flying is a core keyword. "When this creature enters, surveil 2" (CR 701.25); Warp {1}{U} (CR 702.185a).
pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        THIS_ENTERS,
        |g, item| effects::surveil(g, item, 2),
        "When this creature enters, surveil 2. (Look at the top two cards of your library, then put any number of them into your graveyard and the rest on top of your library in any order.)",
    )],
    ways: &[warp::way(mana("{1}{U}"), "Warp {1}{U}")],
    end_step_riders: warp::RIDERS,
    delayed: &[warp::EXILE],
    ..oracle::STARBREACH_WHALE
};
