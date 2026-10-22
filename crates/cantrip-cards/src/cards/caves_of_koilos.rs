use cantrip_api::*;
use cantrip_mechanics::mana;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        mana_ability(TAP, add_one_of(&[Color::White, Color::Black]).by(mana::add_chosen_and_hurt), "{T}: Add {W} or {B}. This land deals 1 damage to you."),
    ],
    ..oracle::CAVES_OF_KOILOS
};
