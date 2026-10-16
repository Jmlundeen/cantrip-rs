use cantrip_api::*;
use cantrip_mechanics::{regenerate, targets};
use cantrip_oracle as oracle;

fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    regenerate::target(g, item, 0);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(SACRIFICE_THIS, effect, "Sacrifice this artifact: Regenerate target artifact.")
        .targets(&[targets::ARTIFACT])],
    ..oracle::WELDING_JAR
};
