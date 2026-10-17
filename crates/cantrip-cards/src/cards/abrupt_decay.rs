use cantrip_api::*;
use cantrip_mechanics::{destroy_target, targets};
use cantrip_oracle as oracle;

/// Destroy target nonland permanent with mana value 3 or less. (It can't be countered: the card's keyword.)
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    destroy_target(g, item, 0);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[target(targets::nonland_permanent_with_mana_value_at_most::<3>)], effect),
    ..oracle::ABRUPT_DECAY
};
