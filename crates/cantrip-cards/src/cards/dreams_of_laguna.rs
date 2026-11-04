use cantrip_api::*;
use cantrip_mechanics::{flashback, surveil};
use cantrip_oracle as oracle;

/// "Surveil 1, then draw a card." (CR 701.25a)
fn surveil_and_draw(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    surveil(g, item.controller, item.object, 1)?;
    g.draw(item.controller, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], surveil_and_draw),
    ways: &[flashback::way(cost("{3}{U}"), "Flashback {3}{U}")],
    ..oracle::DREAMS_OF_LAGUNA
};
