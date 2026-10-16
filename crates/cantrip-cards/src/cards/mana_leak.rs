use cantrip_api::*;
use cantrip_oracle as oracle;
use cantrip_mechanics::{counter_unless_pays, targets};

/// Counter target spell unless its controller pays {3} (CR 118.12): the spell's controller is asked, and may make
/// mana for it while asked.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    counter_unless_pays(g, item, 0, mana("{3}"))?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::SPELL], effect),
    ..oracle::MANA_LEAK
};
