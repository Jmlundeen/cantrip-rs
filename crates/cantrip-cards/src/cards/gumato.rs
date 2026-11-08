use cantrip_api::*;
use cantrip_oracle as oracle;

/// "{T}: Add {G}{G}{G}." (CR 605.1a, 605.3b): a mana ability - it needs no target, could add mana and does not spend a
/// loyalty counter - so it resolves at once and cannot be responded to. The {T} is the only cost (CR 107.5), so the creature
/// is tapped and three green mana are in the pool when it is done. Vigilance (CR 702.20a, from the card's keywords) only
/// keeps it untapped when it attacks, not when its own cost taps it.
pub static CARD: CardDef = CardDef {
    abilities: &[mana_ability(TAP, add_n(Mana::G, 3), "{T}: Add {G}{G}{G}.")],
    ..oracle::GUMATO
};
