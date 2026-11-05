use cantrip_api::*;
use cantrip_mechanics::{damage_target, targets, triggers};
use cantrip_oracle as oracle;

/// "When this creature enters, it deals 4 damage to target tapped creature an opponent controls." The damage is dealt by
/// this creature (CR 120.2); a target that is no longer tapped, or no longer an opponent's, is illegal by then and
/// nothing is dealt (CR 608.2b).
fn damage_a_tapped_creature(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    damage_target(g, item, 0, 4);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::THIS_ENTERS,
        damage_a_tapped_creature,
        "When this creature enters, it deals 4 damage to target tapped creature an opponent controls.",
    )
    .targets(&[targets::TAPPED_CREATURE_AN_OPPONENT_CONTROLS])],
    ..oracle::SHUTTLE_CREW
};
