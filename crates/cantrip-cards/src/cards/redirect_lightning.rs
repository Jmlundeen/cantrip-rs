use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "Change the target of target spell or ability with a single target" (CR 115.7a). The new target is chosen as this
/// resolves, by Redirect Lightning's controller, and must be legal for the spell or ability's controller.
pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::SPELL_OR_ABILITY_WITH_A_SINGLE_TARGET], effects::change_first_target),
    // "As an additional cost to cast this spell, pay 5 life or pay {2}" (CR 601.2b).
    additional_choice: &[
        ExtraCost { text: "Pay 5 life", cost: Cost { life: 5, ..Cost::NONE } },
        ExtraCost { text: "Pay {2}", cost: Cost::mana(mana("{2}")) },
    ],
    ..oracle::REDIRECT_LIGHTNING
};
