use cantrip_api::*;
use cantrip_mechanics::{replicate, targets};
use cantrip_oracle as oracle;

/// Counter target spell unless its controller pays {2} (CR 118.12).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    cantrip_mechanics::counter_unless_pays(g, item, 0, mana("{2}"))?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::SPELL], effect),
    kicker: multikicker(mana("{U}")),
    abilities: &[replicate::trigger("Replicate {U} (When you cast this spell, copy it for each time you paid its replicate cost. You may choose new targets for the copies.)")],
    ..oracle::LOSE_FOCUS
};
