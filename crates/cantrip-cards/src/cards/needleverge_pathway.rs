use cantrip_api::*;
use cantrip_oracle as oracle;

/// "{T}: Add {W}." (CR 712.12: the back face of a modal double-faced card, played as a land.)
pub static BACK: CardDef = CardDef {
    abilities: &[mana_ability(TAP, add(Mana::W), "{T}: Add {W}.")],
    ..oracle::PILLARVERGE_PATHWAY
};

/// "{T}: Add {R}."
pub static CARD: CardDef = CardDef {
    abilities: &[mana_ability(TAP, add(Mana::R), "{T}: Add {R}.")],
    back: Some(&BACK),
    ..oracle::NEEDLEVERGE_PATHWAY
};
