use cantrip_api::*;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[mana_ability(cost("{1}, {T}"), add_each(&[Mana::U, Mana::B]), "{1}, {T}: Add {U}{B}.")],
    ..oracle::DIMIR_SIGNET
};
