use cantrip_api::*;
use cantrip_oracle as oracle;
use cantrip_mechanics::{counter_target, targets};

fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    counter_target(g, item, 0);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef { targets: &[targets::SPELL], effect }),
    ..oracle::COUNTERSPELL
};
