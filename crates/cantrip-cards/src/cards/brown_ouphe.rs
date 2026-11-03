use cantrip_api::*;
use cantrip_mechanics::{counter_target_ability, targets};
use cantrip_oracle as oracle;

fn counter_ability(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    counter_target_ability(g, item, 0);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(
        cost("{1}{G}").and(TAP),
        counter_ability,
        "{1}{G}, {T}: Counter target activated ability from an artifact source.",
    ).targets(&[target(targets::activated_ability_from_an_artifact_source)])],
    ..oracle::BROWN_OUPHE
};
