use cantrip_api::*;
use cantrip_mechanics::{regenerate, targets};
use cantrip_oracle as oracle;

/// "Regenerate target creature." (CR 701.19a).
fn regenerate_target_creature(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    regenerate::target(g, item, 0);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(
        &[targets::CREATURE],
        regenerate_target_creature,
    ),
    ..oracle::DEATH_WARD
};
