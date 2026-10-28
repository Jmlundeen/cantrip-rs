use cantrip_api::*;
use cantrip_mechanics::{targets, your_turn};
use cantrip_oracle as oracle;

/// "an artifact source": the types of the source a copied ability may come from.
const ARTIFACT_SOURCE: u16 = TypeSet::of(&[CardType::Artifact]).0;

/// "Copy target activated or triggered ability ... You may choose new targets for the copy." (CR 707.10, 707.10c)
fn copy_it(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.copy_ability(item.target(0), item.controller, true)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(life(2), copy_it, "Pay 2 life: Copy target activated or triggered ability you control from an artifact source. You may choose new targets for the copy. Activate only during your turn and only once each turn.")
        .targets(&[target(targets::your_ability_from_a_source_of_types::<ARTIFACT_SOURCE>)])
        .once_each_turn()
        .condition(your_turn)],
    ..oracle::SCIENTIST_SUPREME_OF_A_I_M
};
