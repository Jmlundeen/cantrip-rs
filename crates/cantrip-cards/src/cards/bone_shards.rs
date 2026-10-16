use cantrip_api::*;
use cantrip_mechanics::{destroy_target, targets};
use cantrip_oracle as oracle;

/// Destroy target creature or planeswalker.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    destroy_target(g, item, 0);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE_OR_PLANESWALKER], effect),
    // As an additional cost to cast this spell, sacrifice a creature or discard a card (CR 601.2b).
    additional_choice: &[
        ExtraCost { text: "Sacrifice a creature", cost: Cost::sacrifice(1, targets::filters::creature) },
        ExtraCost { text: "Discard a card", cost: Cost { discard: Some((1, targets::filters::any)), ..Cost::NONE } },
    ],
    ..oracle::BONE_SHARDS
};
