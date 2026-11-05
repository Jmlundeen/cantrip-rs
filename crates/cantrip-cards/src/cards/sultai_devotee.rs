use cantrip_api::*;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[mana_ability(cost("{1}"), add_one_of(&[Color::Black, Color::Green, Color::Blue]), "{1}: Add {B}, {G}, or {U}. Activate only once each turn.")
        .once_each_turn()],
    ..oracle::SULTAI_DEVOTEE
};
