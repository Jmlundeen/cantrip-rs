use cantrip_api::*;
use cantrip_mechanics::{library, select};
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// Search your library for an artifact card, put that card onto the battlefield, then shuffle.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    library::find_to_battlefield(g, item.controller, item.object, select::ARTIFACT_CARD, false)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], effect),
    // As an additional cost to cast this spell, sacrifice an artifact (CR 118.8).
    additional_cost: Some(Cost::sacrifice(1, targets::filters::artifact)),
    ..oracle::TINKER
};
