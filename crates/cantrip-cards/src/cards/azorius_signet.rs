use cantrip_api::*;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[mana_ability(cost("{1}, {T}"), add_each(&[Mana::W, Mana::U]), "{1}, {T}: Add {W}{U}.")],
    ..oracle::AZORIUS_SIGNET
};
