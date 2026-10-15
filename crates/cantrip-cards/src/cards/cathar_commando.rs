use cantrip_api::*;
use cantrip_mechanics::{targets, destroy_target};
use cantrip_oracle as oracle;

fn destroy_it(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    destroy_target(g, item, 0);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[AbilityDef::Activated(ActivatedDef {
        text: "{1}, Sacrifice Cathar Commando: Destroy target artifact or enchantment.",
        cost: Cost { mana: mana(1, 0, 0, 0, 0, 0), sacrifice_self: true, ..Cost::NONE },
        targets: &[targets::ARTIFACT_OR_ENCHANTMENT],
        effect: destroy_it,
        ..ActivatedDef::BLANK
    })],
    ..oracle::CATHAR_COMMANDO
};
