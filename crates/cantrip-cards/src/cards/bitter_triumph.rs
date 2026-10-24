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
    // As an additional cost to cast this spell, discard a card or pay 3 life (CR 601.2b).
    additional_choice: &[
        ExtraCost { text: "Discard a card", cost: discard_cards(1) },
        ExtraCost { text: "Pay 3 life", cost: Cost { life: 3, ..Cost::NONE } },
    ],
    ..oracle::BITTER_TRIUMPH
};
