use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        cantrip_mechanics::effects::dies_gain_life::<2>( "When Haywire Mite dies, you gain 2 life."),
        ability(cost("{G}").and(SACRIFICE_THIS), effects::exile_first_target, "{G}, Sacrifice Haywire Mite: Exile target noncreature artifact or noncreature enchantment.")
            .targets(&[targets::NONCREATURE_ARTIFACT_OR_ENCHANTMENT]),
    ],
    ..oracle::HAYWIRE_MITE
};
