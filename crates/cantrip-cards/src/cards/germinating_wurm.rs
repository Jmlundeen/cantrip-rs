use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_mechanics::{effects, warp};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_ENTERS, |g, item| effects::gain_life(g, item, 2), "When this creature enters, you gain 2 life.")],
    ways: &[warp::way(mana("{1}{G}"), "Warp {1}{G}")],
    end_step_riders: warp::RIDERS,
    delayed: &[warp::EXILE],
    ..oracle::GERMINATING_WURM
};
