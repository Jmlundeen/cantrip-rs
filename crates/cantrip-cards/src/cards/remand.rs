use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// Counter target spell; if it is countered this way, it goes to its owner's hand instead of the graveyard. Then draw
/// a card. A spell whose target is gone does neither (CR 608.2b).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(spell) = targets::object(g, item, 0) {
        g.counter_to(spell, CounterTo::Hand);
        g.draw(item.controller, 1);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[targets::SPELL], effect), ..oracle::REMAND };
