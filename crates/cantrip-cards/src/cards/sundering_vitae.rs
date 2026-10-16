use cantrip_api::*;
use cantrip_mechanics::{destroy_target, targets};
use cantrip_oracle as oracle;

/// Destroy target artifact or enchantment. (Convoke is a characteristic of the card: `pay_with`.)
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    destroy_target(g, item, 0);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::ARTIFACT_OR_ENCHANTMENT], effect),
    ..oracle::SUNDERING_VITAE
};
