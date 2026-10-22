use cantrip_api::*;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[mana_ability(cost("{1}, {T}"), add_each(&[Mana::R, Mana::W]), "{1}, {T}: Add {R}{W}.")],
    ..oracle::SUNSCORCHED_DIVIDE
};
