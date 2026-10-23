use cantrip_api::*;
use cantrip_oracle as oracle;

/// "{T}: Add {B}." (CR 712.12: the back face of a modal double-faced card, played as a land.)
pub static BACK: CardDef = CardDef {
    abilities: &[mana_ability(TAP, add(Mana::B), "{T}: Add {B}.")],
    ..oracle::GRIMCLIMB_PATHWAY
};

/// "{T}: Add {W}."
pub static CARD: CardDef = CardDef {
    abilities: &[mana_ability(TAP, add(Mana::W), "{T}: Add {W}.")],
    back: Some(&BACK),
    ..oracle::BRIGHTCLIMB_PATHWAY
};
