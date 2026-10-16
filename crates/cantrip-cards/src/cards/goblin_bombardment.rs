use cantrip_api::*;
use cantrip_mechanics::{damage_target, targets};
use cantrip_oracle as oracle;

fn ping(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    damage_target(g, item, 0, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(sacrifice(1, targets::filters::creature), ping, "Sacrifice a creature: Goblin Bombardment deals 1 damage to any target.")
        .targets(&[targets::ANY])],
    ..oracle::GOBLIN_BOMBARDMENT
};
