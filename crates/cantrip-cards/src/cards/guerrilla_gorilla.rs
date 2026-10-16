use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

fn destroy(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    cantrip_mechanics::destroy_target(g, item, 0);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(SACRIFICE_THIS, destroy, "Sacrifice Guerrilla Gorilla: Destroy target noncreature artifact or noncreature enchantment. Activate only as a sorcery.")
        .sorcery_speed()
        .targets(&[targets::NONCREATURE_ARTIFACT_OR_ENCHANTMENT])],
    ..oracle::GUERRILLA_GORILLA
};
