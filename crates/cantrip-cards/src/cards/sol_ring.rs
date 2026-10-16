use cantrip_api::*;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[mana_ability(TAP, add_n(Mana::C, 2), "{T}: Add {C}{C}.")],
    ..oracle::SOL_RING
};
