use cantrip_api::*;
use cantrip_mechanics::{statics::permanent_entering_triggers_again, warp};
use cantrip_oracle as oracle;

/// "If a permanent entering the battlefield causes a triggered ability of a permanent you control to trigger, that ability
/// triggers an additional time" (CR 603.2d): the permanent's own enters abilities too, whoever controls it (the rulings).
/// Warp {1}{U} (CR 702.185a).
pub static CARD: CardDef = CardDef {
    additional_triggers: &[permanent_entering_triggers_again],
    ways: &[warp::way(mana("{1}{U}"), "Warp {1}{U}")],
    end_step_riders: warp::RIDERS,
    delayed: &[warp::EXILE],
    ..oracle::STARFIELD_VOCALIST
};
