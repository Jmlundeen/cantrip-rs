use cantrip_api::*;
use cantrip_mechanics::{damage_target, targets};
use cantrip_oracle as oracle;

/// "Lightning Axe deals 5 damage to target creature."
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    damage_target(g, item, 0, 5);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE], effect),
    // "As an additional cost to cast this spell, discard a card or pay {5}" (CR 601.2b).
    additional_choice: &[
        ExtraCost { text: "Discard a card", cost: discard_cards(1) },
        ExtraCost { text: "Pay {5}", cost: Cost::mana(mana("{5}")) },
    ],
    ..oracle::LIGHTNING_AXE
};
