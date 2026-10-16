use cantrip_api::*;
use cantrip_mechanics::{targets, destroy_target};
use cantrip_oracle as oracle;

fn destroy_it(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    destroy_target(g, item, 0);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(cost("{1}").and(SACRIFICE_THIS), destroy_it, "{1}, Sacrifice Cathar Commando: Destroy target artifact or enchantment.")
        .targets(&[targets::ARTIFACT_OR_ENCHANTMENT])],
    ..oracle::CATHAR_COMMANDO
};
