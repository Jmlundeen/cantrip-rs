use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// The Adventure (CR 715): "Destroy target artifact or enchantment. (Then exile this card. You may cast the creature later from
/// exile.)" The exile is the core's (CR 715.4), done only if the spell resolves; with its target illegal it does not and the card goes
/// to the graveyard.
pub static BEAR_DOWN: CardDef = CardDef {
    spell: spell(&[targets::ARTIFACT_OR_ENCHANTMENT], effects::destroy_first_target),
    ..oracle::BEAR_DOWN
};

pub static CARD: CardDef = CardDef {
    // "This creature can't be blocked by creatures with power 2 or less" (CR 509.1b): checked as blockers are declared, with the
    // power the blocker has then, and no matter what happens to it afterwards (the ruling).
    keywords: Keywords::of(&[Keyword::CantBeBlockedByPowerTwoOrLess]),
    adventure: Some(&BEAR_DOWN),
    ..oracle::STORMKELD_VANGUARD
};

pub static EXTRA: &[&CardDef] = &[&BEAR_DOWN];
