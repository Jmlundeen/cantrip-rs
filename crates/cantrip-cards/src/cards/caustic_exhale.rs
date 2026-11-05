use cantrip_api::*;
use cantrip_mechanics::{behold, effects, targets};
use cantrip_oracle as oracle;

/// "As an additional cost to cast this spell, behold a Dragon or pay {1}" (CR 601.2b, 701.4a): one of the two, chosen before
/// the target, which is chosen after it.
pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE], effects::target_creature_gets::<-3, -3>),
    late_targets: true,
    additional_choice: &[
        ExtraCost { text: "Behold a Dragon", cost: behold::cost(behold::dragon, "Dragon") },
        ExtraCost { text: "Pay {1}", cost: Cost { mana: mana("{1}"), ..Cost::NONE } },
    ],
    ..oracle::CAUSTIC_EXHALE
};
