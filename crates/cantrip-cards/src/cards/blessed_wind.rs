use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Target player's life total becomes 20" (CR 119.5): they gain or lose the difference, which counts as gaining or losing
/// life (the ruling), so a player who can't gain life and has less than 20 keeps their total (CR 119.7).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(player) = item.targets[0].player() else { return Ok(()) };
    g.set_life_total(player, 20);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::PLAYER], effect),
    ..oracle::BLESSED_WIND
};
