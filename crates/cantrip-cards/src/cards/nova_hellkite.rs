use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_mechanics::{damage_target, targets, warp};
use cantrip_oracle as oracle;

/// "When this creature enters, it deals 1 damage to target creature an opponent controls": the Hellkite is the source, if
/// the target is still legal (CR 608.2b); the damage is dealt by it as it was last known if it has already left (CR 608.2h).
fn ping(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    damage_target(g, item, 0, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_ENTERS, ping, "When this creature enters, it deals 1 damage to target creature an opponent controls.")
        .targets(&[targets::CREATURE_AN_OPPONENT_CONTROLS])],
    ways: &[warp::way(mana("{2}{R}"), "Warp {2}{R}")],
    end_step_riders: warp::RIDERS,
    delayed: &[warp::EXILE],
    ..oracle::NOVA_HELLKITE
};
