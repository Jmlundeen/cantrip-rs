use cantrip_api::*;
use cantrip_mechanics::{blight, effects, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::SPELL], effects::counter_first_target),
    // "As an additional cost to cast this spell, blight 2 or pay {1}" (CR 601.2b, 701.68a).
    additional_choice: &[
        ExtraCost { text: "Blight 2", cost: blight::cost(2) },
        ExtraCost { text: "Pay {1}", cost: Cost::mana(mana("{1}")) },
    ],
    ..oracle::WILD_UNRAVELING
};
