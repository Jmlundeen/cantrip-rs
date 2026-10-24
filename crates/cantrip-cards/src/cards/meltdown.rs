use cantrip_api::*;
use cantrip_mechanics::effects::destroy_artifacts_with_mana_value_at_most;
use cantrip_oracle as oracle;

/// "Destroy each artifact with mana value X or less." Every such artifact goes at once.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    destroy_artifacts_with_mana_value_at_most(g, item.x as u32);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], effect),
    ..oracle::MELTDOWN
};
