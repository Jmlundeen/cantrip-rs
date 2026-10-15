use cantrip_api::*;
use cantrip_oracle as oracle;
use cantrip_mechanics::mana;

pub static CARD: CardDef = CardDef {
    abilities: &[mana::tap_for_n(mana::add_cc, Mana::C, 2, "{T}: Add {C}{C}.")],
    ..oracle::SOL_RING
};
