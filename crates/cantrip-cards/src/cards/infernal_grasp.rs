use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// Destroy target creature. You lose 2 life. Nothing happens at all if the creature is no longer a legal target
/// (CR 608.2b); an indestructible one stays, and the life is still lost.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    cantrip_mechanics::destroy_target(g, item, 0);
    g.lose_life(item.controller, 2);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE], effect),
    ..oracle::INFERNAL_GRASP
};
