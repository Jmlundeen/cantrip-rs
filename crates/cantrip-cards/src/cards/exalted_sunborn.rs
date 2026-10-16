use cantrip_api::*;
use cantrip_mechanics::{replacements::DOUBLE_TOKENS, warp};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    replacements: &[DOUBLE_TOKENS],
    ways: &[warp::way(mana("{1}{W}"), "Warp {1}{W}")],
    end_step_riders: warp::RIDERS,
    delayed: &[warp::EXILE],
    ..oracle::EXALTED_SUNBORN
};
