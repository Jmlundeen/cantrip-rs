use cantrip_api::*;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[mana_ability(cost("{1}, {T}"), add_each(&[Mana::B, Mana::G]), "{1}, {T}: Add {B}{G}.")],
    ..oracle::VIRIDESCENT_BOG
};
