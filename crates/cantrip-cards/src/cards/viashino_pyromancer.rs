use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_mechanics::{damage_target, targets};
use cantrip_oracle as oracle;

/// "It deals 2 damage to target player or planeswalker": the source is the Pyromancer; nothing happens if the target is
/// no longer legal (CR 608.2b).
fn two_damage(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    damage_target(g, item, 0, 2);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_ENTERS, two_damage, "When this creature enters, it deals 2 damage to target player or planeswalker.")
        .targets(&[targets::PLAYER_OR_PLANESWALKER])],
    ..oracle::VIASHINO_PYROMANCER
};
