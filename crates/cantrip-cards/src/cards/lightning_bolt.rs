use cantrip_api::*;
use cantrip_oracle as oracle;
use cantrip_mechanics::{damage_target, targets};

fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    damage_target(g, item, 0, 3);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::ANY], effect),
    ..oracle::LIGHTNING_BOLT
};
