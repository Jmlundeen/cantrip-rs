use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[ability(SACRIFICE_THIS, cantrip_mechanics::effects::destroy_first_target, "Sacrifice Guerrilla Gorilla: Destroy target noncreature artifact or noncreature enchantment. Activate only as a sorcery.")
        .sorcery_speed()
        .targets(&[targets::NONCREATURE_ARTIFACT_OR_ENCHANTMENT])],
    ..oracle::GUERRILLA_GORILLA
};
