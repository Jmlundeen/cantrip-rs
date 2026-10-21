use cantrip_api::*;
use cantrip_mechanics::{discard_at_random, targets};
use cantrip_oracle as oracle;

/// Target player discards two cards at random.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(player) = item.target(0).player() else { return Ok(()) };
    discard_at_random(g, player, 2);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::PLAYER], effect),
    ..oracle::HYMN_TO_TOURACH
};
