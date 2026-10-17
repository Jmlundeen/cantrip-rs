use cantrip_api::*;
use cantrip_mechanics::effects::{destroy_first_target, gain_life};
use cantrip_mechanics::{targets, triggers};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(triggers::THIS_DIES, |g, item| gain_life(g, item, 3), "When this creature dies, you gain 3 life.").look_back(),
        ability(cost("{2}").and(SACRIFICE_THIS), destroy_first_target, "{2}, Sacrifice this creature: Destroy up to one target artifact or enchantment.")
            .targets(&[targets::ARTIFACT_OR_ENCHANTMENT.between(0, 1)]),
    ],
    ..oracle::CURIOUS_FARM_ANIMALS
};
