use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

fn destroy(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    cantrip_mechanics::destroy_target(g, item, 0);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[AbilityDef::Activated(ActivatedDef {
        text: "Sacrifice Guerrilla Gorilla: Destroy target noncreature artifact or noncreature enchantment. Activate only as a sorcery.",
        cost: Cost { sacrifice_self: true, ..Cost::NONE },
        timing: Timing::Sorcery,
        targets: &[targets::NONCREATURE_ARTIFACT_OR_ENCHANTMENT],
        effect: destroy,
        ..ActivatedDef::BLANK
    })],
    ..oracle::GUERRILLA_GORILLA
};
