use cantrip_api::*;
use cantrip_mechanics::{discard_chosen, targets, effects};
use cantrip_oracle as oracle;

/// Target player discards two cards, their choice. Then that player may copy this spell and choose a new target for
/// the copy (CR 707.10, 707.10c).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(player) = item.targets[0].player() else { return Ok(()) };
    discard_chosen(g, player, item.object, 2)?;
    if effects::ask_may(g, player, item.object, Purpose::Optional)? {
        g.copy_spell(item.object, player, true)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef { targets: &[targets::PLAYER], effect }),
    ..oracle::CHAIN_OF_SMOG
};
