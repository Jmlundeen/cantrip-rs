use cantrip_api::*;
use cantrip_mechanics::{counter_unless_pays, targets};
use cantrip_mechanics::triggers::cycling;
use cantrip_oracle as oracle;

/// Counter target spell unless its controller pays {2}: the targeted spell's controller, whoever that is.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    counter_unless_pays(g, item, 0, mana(2, 0, 0, 0, 0, 0))?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef { targets: &[targets::SPELL], effect }),
    abilities: &[cycling(mana(2, 0, 0, 0, 0, 0), "Cycling {2} ({2}, Discard this card: Draw a card.)")],
    ..oracle::MISCALCULATION
};
