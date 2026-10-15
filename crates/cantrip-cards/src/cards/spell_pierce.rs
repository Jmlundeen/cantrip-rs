use cantrip_api::*;
use cantrip_mechanics::counter_unless_pays;
use cantrip_oracle as oracle;

/// Counter target noncreature spell unless its controller pays {2} (CR 118.12).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    counter_unless_pays(g, item, 0, mana(2, 0, 0, 0, 0, 0))?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef { targets: &[cantrip_mechanics::targets::NONCREATURE_SPELL], effect }),
    ..oracle::SPELL_PIERCE
};
