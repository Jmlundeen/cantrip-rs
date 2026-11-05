use cantrip_api::*;
use cantrip_mechanics::{blight, effects, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE], effects::exile_first_target),
    // "As an additional cost to cast this spell, blight 1 or pay {3}" (CR 601.2b, 701.68a).
    additional_choice: &[
        ExtraCost { text: "Blight 1", cost: blight::cost(1) },
        ExtraCost { text: "Pay {3}", cost: Cost::mana(mana("{3}")) },
    ],
    ..oracle::BOGSLITHERS_EMBRACE
};
