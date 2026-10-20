use cantrip_api::*;
use cantrip_oracle as oracle;

/// "{T}: Add {R}." (CR 712.12: the back face of a modal double-faced card, played as a land.)
pub static BACK: CardDef = CardDef {
    abilities: &[mana_ability(TAP, add(Mana::R), "{T}: Add {R}.")],
    ..oracle::LAVAGLIDE_PATHWAY
};

/// "{T}: Add {U}."
pub static CARD: CardDef = CardDef {
    abilities: &[mana_ability(TAP, add(Mana::U), "{T}: Add {U}.")],
    back: Some(&BACK),
    ..oracle::RIVERGLIDE_PATHWAY
};
