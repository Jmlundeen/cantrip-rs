use cantrip_api::*;
use cantrip_mechanics::{fight, targets};
use cantrip_oracle as oracle;

/// "One or two target creatures you control each deal damage equal to their power to target creature an opponent
/// controls." The powers are read as this resolves and the damage is dealt together (CR 120.2, 120.3, 120.4b, 608.2b).
fn deal_damage(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    fight::targets_each_bite_last(g, item);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(
        &[
            targets::CREATURE_YOU_CONTROL.between(1, 2),
            targets::CREATURE_AN_OPPONENT_CONTROLS,
        ],
        deal_damage,
    ),
    ..oracle::COMMON_GOAL
};
