use cantrip_api::*;
use cantrip_mechanics::{damage_target, targets};
use cantrip_oracle as oracle;

/// "Fiery Temper deals 3 damage to any target." (Madness {R} is the `madness` field, CR 702.35.)
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    damage_target(g, item, 0, 3);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::ANY], effect),
    madness: Some(mana("{R}")),
    ..oracle::FIERY_TEMPER
};
