use cantrip_api::*;
use cantrip_oracle as oracle;

/// "Indestructible" is a core keyword (CR 702.12); "{T}: Add {C}." its mana ability (CR 605.1a).
pub static CARD: CardDef = CardDef { abilities: &[mana_ability(TAP, add(Mana::C), "{T}: Add {C}.")], ..oracle::DARKSTEEL_CITADEL };
