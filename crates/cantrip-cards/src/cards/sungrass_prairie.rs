use cantrip_api::*;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[mana_ability(cost("{1}, {T}"), add_each(&[Mana::G, Mana::W]), "{1}, {T}: Add {G}{W}.")],
    ..oracle::SUNGRASS_PRAIRIE
};
