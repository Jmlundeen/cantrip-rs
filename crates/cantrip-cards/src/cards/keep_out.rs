use cantrip_api::*;
use cantrip_mechanics::{destroy_target, effects, targets};
use cantrip_oracle as oracle;

/// "Destroy target enchantment."
fn destroy_enchantment(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    destroy_target(g, item, 0);
    Ok(())
}

static MODAL: ModalDef = modes(&[
    mode(effects::damage_first_target::<4>, "Keep Out deals 4 damage to target tapped creature.").targets(&[targets::TAPPED_CREATURE]),
    mode(destroy_enchantment, "Destroy target enchantment.").targets(&[targets::ENCHANTMENT]),
]);

pub static CARD: CardDef = CardDef { spell: Some(SpellDef::MODAL), modal: Some(&MODAL), ..oracle::KEEP_OUT };
