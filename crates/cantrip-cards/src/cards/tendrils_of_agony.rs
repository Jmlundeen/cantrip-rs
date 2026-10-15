use cantrip_api::*;
use cantrip_mechanics::storm;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// Target player loses 2 life and you gain 2 life. A copy is controlled by whoever made it, so the gain is theirs. A
/// target that is no longer legal makes the whole spell fizzle (CR 608.2b).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Target::Player(player) = item.targets[0] {
        g.lose_life(player, 2);
    }
    g.gain_life(item.controller, 2);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef { targets: &[targets::PLAYER], effect }),
    abilities: &[storm::trigger("Storm (When you cast this spell, copy it for each spell cast before it this turn. You may choose new targets for the copies.)")],
    ..oracle::TENDRILS_OF_AGONY
};
