use cantrip_api::*;
use cantrip_mechanics::{regenerate, targets};
use cantrip_oracle as oracle;

fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    regenerate::target(g, item, 0);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[AbilityDef::Activated(ActivatedDef {
        text: "Sacrifice this artifact: Regenerate target artifact.",
        cost: Cost { sacrifice_self: true, ..Cost::NONE },
        targets: &[targets::ARTIFACT],
        effect,
        ..ActivatedDef::BLANK
    })],
    ..oracle::WELDING_JAR
};
