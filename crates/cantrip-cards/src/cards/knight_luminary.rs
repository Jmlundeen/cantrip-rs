use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_mechanics::{tokens, warp};
use cantrip_oracle as oracle;

/// "Create a 1/1 white Human Soldier creature token" (CR 111.1, 701.7).
fn create_a_human_soldier(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &tokens::HUMAN_SOLDIER, 1)?;
    Ok(())
}

/// Warp is a keyword: `warp::way`, with its end-step exile and recast (CR 702.185).
pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_ENTERS, create_a_human_soldier, "When this creature enters, create a 1/1 white Human Soldier creature token.")],
    ways: &[warp::way(mana("{1}{W}"), "Warp {1}{W}")],
    end_step_riders: warp::RIDERS,
    delayed: &[warp::EXILE],
    ..oracle::KNIGHT_LUMINARY
};

pub static EXTRA: &[&CardDef] = &[&tokens::HUMAN_SOLDIER];
