use cantrip_api::*;
use cantrip_mechanics::{destroy_target_no_regen, targets};
use cantrip_oracle as oracle;

/// "Destroy target creature. It can't be regenerated." (CR 701.8, 701.19c): the shared no-regeneration destroy, so a
/// regeneration shield standing ready does not save the creature.
fn destroy_it(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    destroy_target_no_regen(g, item, 0);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE], destroy_it),
    ..oracle::TERMINATE
};
