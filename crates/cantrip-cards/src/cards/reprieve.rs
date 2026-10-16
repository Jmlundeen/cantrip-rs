use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// Return target spell to its owner's hand (not a counter, so a spell that can't be countered goes too), then draw a
/// card. A spell whose target is gone does neither (CR 608.2b).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(spell) = targets::object(g, item, 0) {
        g.remove_spell(spell, CounterTo::Hand);
        g.draw(item.controller, 1);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[targets::SPELL], effect), ..oracle::REPRIEVE };
