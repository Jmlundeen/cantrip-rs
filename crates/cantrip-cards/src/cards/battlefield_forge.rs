use cantrip_api::*;
use cantrip_mechanics::mana;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        mana_ability(TAP, add_one_of(&[Color::Red, Color::White]).by(mana::add_chosen_and_hurt), "{T}: Add {R} or {W}. This land deals 1 damage to you."),
    ],
    ..oracle::BATTLEFIELD_FORGE
};
