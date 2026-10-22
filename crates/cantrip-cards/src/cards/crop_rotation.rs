use cantrip_api::*;
use cantrip_mechanics::{library, select};
use cantrip_oracle as oracle;

/// Search your library for a land card, put that card onto the battlefield, then shuffle.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    library::find_to_battlefield(g, item.controller, item.object, select::LAND_CARD, false)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], effect),
    // As an additional cost to cast this spell, sacrifice a land.
    additional_cost: Some(Cost::sacrifice(1, cantrip_mechanics::targets::filters::land)),
    ..oracle::CROP_ROTATION
};
