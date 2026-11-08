use cantrip_api::*;
use cantrip_mechanics::mana;
use cantrip_oracle as oracle;

/// "{T}: Add {C}." (CR 605.1a) and "{T}: Add {W} or {B}. This artifact deals 1 damage to you." (CR 605.1a, 605.3b):
/// the Caves of Koilos shape, dealing the damage with the mana as it resolves.
pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        mana_ability(TAP, add_one_of(&[Color::White, Color::Black]).by(mana::add_chosen_and_hurt), "{T}: Add {W} or {B}. This artifact deals 1 damage to you."),
    ],
    ..oracle::TALISMAN_OF_HIERARCHY
};
