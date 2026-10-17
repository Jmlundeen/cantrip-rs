use cantrip_api::*;
use cantrip_mechanics::{damage_target, discard_chosen, targets};
use cantrip_oracle as oracle;

/// "Ozai's Cruelty deals 2 damage to target player. That player discards two cards." The player discards what they
/// choose (CR 701.9), whether or not the damage was dealt (prevented, or a life total that was already gone).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    damage_target(g, item, 0, 2);
    let Some(player) = targets::player(g, item, 0) else { return Ok(()) };
    discard_chosen(g, player, item.object, 2)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::PLAYER], effect),
    ..oracle::OZAIS_CRUELTY
};
