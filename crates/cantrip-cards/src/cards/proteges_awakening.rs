use cantrip_api::*;
use cantrip_mechanics::empower;
use cantrip_oracle as oracle;

/// "Empower Jace 6. Draw a card." The tokens and counters first, then the draw (CR 701.71a).
fn empower_then_draw(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    empower::jace(g, item.controller, item.object, 6)?;
    g.draw(item.controller, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[], empower_then_draw), ..oracle::PROTEGES_AWAKENING };

pub static EXTRA: &[&CardDef] = &[&empower::JACE];
