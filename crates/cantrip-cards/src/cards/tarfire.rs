use cantrip_api::*;
use cantrip_mechanics::{damage_target, targets};
use cantrip_oracle as oracle;

fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    damage_target(g, item, 0, 2);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::ANY], effect),
    ..oracle::TARFIRE
};
