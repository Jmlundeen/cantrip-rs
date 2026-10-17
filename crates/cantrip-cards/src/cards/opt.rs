use cantrip_api::*;
use cantrip_mechanics::scry;
use cantrip_oracle as oracle;

/// "Scry 1. Draw a card." (CR 701.22): the card is drawn once the scry is complete.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    scry(g, item.controller, item.object, 1)?;
    g.draw(item.controller, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[], effect), ..oracle::OPT };
