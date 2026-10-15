use cantrip_api::*;
use cantrip_mechanics::storm;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// Target player mills three cards (CR 701.17).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if g.target_legal(item, 0)
        && let Target::Player(player) = item.targets[0]
    {
        g.mill(player, 3);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef { targets: &[targets::PLAYER], effect }),
    abilities: &[storm::trigger("Storm (When you cast this spell, copy it for each spell cast before it this turn. You may choose new targets for the copies.)")],
    ..oracle::BRAIN_FREEZE
};
