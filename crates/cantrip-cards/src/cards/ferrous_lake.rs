use cantrip_api::*;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[mana_ability(cost("{1}, {T}"), add_each(&[Mana::U, Mana::R]), "{1}, {T}: Add {U}{R}.")],
    ..oracle::FERROUS_LAKE
};
