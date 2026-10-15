use cantrip_api::*;
use cantrip_mechanics::{replicate, targets};
use cantrip_oracle as oracle;

/// Counter target spell unless its controller pays {2} (CR 118.12).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    cantrip_mechanics::counter_unless_pays(g, item, 0, mana(2, 0, 0, 0, 0, 0))?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef { targets: &[targets::SPELL], effect }),
    kicker: Some(Kicker { cost: mana(0, 0, 1, 0, 0, 0), multi: true, waterbend: false }),
    abilities: &[replicate::trigger("Replicate {U} (When you cast this spell, copy it for each time you paid its replicate cost. You may choose new targets for the copies.)")],
    ..oracle::LOSE_FOCUS
};
