use cantrip_api::*;
use cantrip_mechanics::{damage_target, targets};
use cantrip_oracle as oracle;

/// Shrapnel Blast deals 5 damage to any target.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    damage_target(g, item, 0, 5);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::ANY], effect),
    // As an additional cost to cast this spell, sacrifice an artifact (CR 118.8).
    additional_cost: Some(Cost::sacrifice(1, targets::filters::artifact)),
    ..oracle::SHRAPNEL_BLAST
};
