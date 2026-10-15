use cantrip_api::*;
use cantrip_mechanics::{discard_at_random, targets};
use cantrip_oracle as oracle;

/// Target player discards X cards at random.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(player) = item.targets[0].player() else { return Ok(()) };
    discard_at_random(g, player, item.x as usize);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef { targets: &[targets::PLAYER], effect }),
    ..oracle::MIND_TWIST
};
