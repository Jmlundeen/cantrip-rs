use cantrip_api::*;
use cantrip_mechanics::{exile_target, targets};
use cantrip_oracle as oracle;

fn exile(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    exile_target(g, item, 0);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        cantrip_mechanics::effects::dies_gain_life::<2>( "When Haywire Mite dies, you gain 2 life."),
        ability(cost("{G}").and(SACRIFICE_THIS), exile, "{G}, Sacrifice Haywire Mite: Exile target noncreature artifact or noncreature enchantment.")
            .targets(&[targets::NONCREATURE_ARTIFACT_OR_ENCHANTMENT]),
    ],
    ..oracle::HAYWIRE_MITE
};
