use cantrip_api::*;
use cantrip_mechanics::{behold, regenerate, targets};
use cantrip_oracle as oracle;

/// Regenerate target creature (CR 701.19a).
fn regenerate_target_creature(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    regenerate::target(g, item, 0);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(sacrifice(1, behold::goblin), regenerate_target_creature, "Sacrifice a Goblin: Regenerate target creature.")
            .targets(&[targets::CREATURE]),
    ],
    ..oracle::GOBLIN_CHIRURGEON
};
