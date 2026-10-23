use cantrip_api::*;
use cantrip_mechanics::{counter_target, targets};
use cantrip_oracle as oracle;

/// "Counter target noncreature spell." (CR 701.6) "This spell can't be countered" is the core keyword.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    counter_target(g, item, 0);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::NONCREATURE_SPELL], effect),
    ..oracle::DOVINS_VETO
};
