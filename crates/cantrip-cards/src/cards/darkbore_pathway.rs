use cantrip_api::*;
use cantrip_oracle as oracle;

/// "{T}: Add {G}." (CR 712.12: the back face of a modal double-faced card, played as a land.)
pub static BACK: CardDef = CardDef {
    abilities: &[mana_ability(TAP, add(Mana::G), "{T}: Add {G}.")],
    ..oracle::SLITHERBORE_PATHWAY
};

/// "{T}: Add {B}."
pub static CARD: CardDef = CardDef {
    abilities: &[mana_ability(TAP, add(Mana::B), "{T}: Add {B}.")],
    back: Some(&BACK),
    ..oracle::DARKBORE_PATHWAY
};
