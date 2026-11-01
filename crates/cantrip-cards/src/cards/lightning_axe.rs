use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE], effects::damage_first_target::<5>),
    // "As an additional cost to cast this spell, discard a card or pay {5}" (CR 601.2b).
    additional_choice: &[
        ExtraCost { text: "Discard a card", cost: discard_cards(1) },
        ExtraCost { text: "Pay {5}", cost: Cost::mana(mana("{5}")) },
    ],
    ..oracle::LIGHTNING_AXE
};
