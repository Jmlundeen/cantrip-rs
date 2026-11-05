use cantrip_api::*;
use cantrip_mechanics::{damage_target, history, targets};
use cantrip_oracle as oracle;

/// "Planetary Bombardment deals 2 damage to target creature. Planetary Bombardment deals 4 damage to that creature
/// instead if a creature died this turn." The instead is a self-replacement, judged as the spell resolves (CR 614.1a,
/// 700.4): any creature that died this turn, either player's, tokens included.
fn bombard(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let damage = if history::creatures_died_this_turn(g, None) > 0 { 4 } else { 2 };
    damage_target(g, item, 0, damage);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE], bombard),
    ..oracle::PLANETARY_BOMBARDMENT
};
