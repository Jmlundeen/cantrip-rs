use cantrip_api::*;
use cantrip_mechanics::{statics, warp};
use cantrip_oracle as oracle;

/// "During your turn, other creatures you control get +1/+0" (CR 604.1, 613.4c): the other creatures its controller controls,
/// only while that player is the active player, asked again whenever characteristics are computed.
fn other_creatures_you_control_during_your_turn(q: &dyn LayerQuery, source: ObjectId, affected: ObjectId, so_far: &Chars) -> bool {
    q.state().turn.active == q.state().obj(source).controller && statics::other_creatures_you_control(q, source, affected, so_far)
}

pub static CARD: CardDef = CardDef {
    statics: &[static_ability(
        Layer::PtModify,
        other_creatures_you_control_during_your_turn,
        statics::plus_pt::<1, 0>,
        "During your turn, other creatures you control get +1/+0.",
    )],
    ways: &[warp::way(mana("{1}{R}"), "Warp {1}{R}")],
    end_step_riders: warp::RIDERS,
    delayed: &[warp::EXILE],
    ..oracle::MEMORIAL_TEAM_LEADER
};
