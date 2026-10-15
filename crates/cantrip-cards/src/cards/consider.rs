use cantrip_api::*;
use cantrip_mechanics::surveil;
use cantrip_oracle as oracle;

/// Surveil 1, then draw a card.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    surveil(g, item.controller, item.object, 1)?;
    g.draw(item.controller, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef { targets: &[], effect }),
    ..oracle::CONSIDER
};
