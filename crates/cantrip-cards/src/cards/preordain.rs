use cantrip_api::*;
use cantrip_mechanics::scry;
use cantrip_oracle as oracle;

/// Scry 2, then draw a card.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    scry(g, item.controller, item.object, 2)?;
    g.draw(item.controller, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: Some(SpellDef { targets: &[], effect }), ..oracle::PREORDAIN };
